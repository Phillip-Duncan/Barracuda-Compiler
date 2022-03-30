pub(crate) mod ops;
pub(crate) mod instructions;
mod test;
mod emulator_heap;

use crate::emulator::ops::MathStackOperators;
use crate::emulator::instructions::MathStackInstructions;
use std::io::{Error, ErrorKind, Write};
use crate::emulator::ops::MathStackOperators::LDA;
use std::cell::RefCell;
use std::rc::Rc;
use crate::emulator::emulator_heap::EmulatorHeap;
use std::fmt;


/// Environment var count as given by the operations. This needs to be updated manually if adding
/// more env var load instructions.
pub const ENV_VAR_COUNT: usize = 56;

/// Loop tracker holds the current value of iteration of a loop and the max value of the loop
/// The max value is exclusive
#[derive(Getters)]
pub(crate) struct LoopTracker {
    current: i64,
    max: i64,
    loop_start: usize
}

impl LoopTracker {
    pub(crate) fn new(start: i64, end: i64, loop_start: usize) -> Self {
        LoopTracker {
            current: start,
            max: end,
            loop_start
        }
    }
}

/// Represents possible values for StackValue
/// In the VM this will be done by transmuting the bytes for the emulator however
/// it is more useful and more safe to directly store the current representation of the value
#[derive(Copy, Clone)]
pub enum StackValue {
    REAL(f64),
    UINT(u64),
    INT(i64)
}

impl StackValue {
    pub(crate) fn into_u64(self) -> u64 {
        match self {
            StackValue::REAL(value) => {
                value as u64
            },
            StackValue::UINT(value) => {
                value as u64
            },
            StackValue::INT(value) => {
                value as u64
            }
        }
    }

    pub(crate) fn into_f64(self) -> f64 {
        match self {
            StackValue::REAL(value) => {
                value as f64
            },
            StackValue::UINT(value) => {
                value as f64
            },
            StackValue::INT(value) => {
                value as f64
            }
        }
    }

    pub(crate) fn into_i64(self) -> i64 {
        match self {
            StackValue::REAL(value) => {
                value as i64
            },
            StackValue::UINT(value) => {
                value as i64
            },
            StackValue::INT(value) => {
                value as i64
            }
        }
    }
}

impl fmt::Debug for StackValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackValue::REAL(value) => {write!(f, "{:<10} F64", *value)}
            StackValue::UINT(value) => {write!(f, "{:<10} U64", *value)}
            StackValue::INT(value)  => {write!(f, "{:<10} I64", *value)}
        }

    }
}


/// Thread context is a struct that represents all the information an individual thread would
/// have access to. It also includes functions with step() and run_till_halt() to emulate the
/// execution of the program.
pub struct ThreadContext {
    /// thread_id is an emulated variable. It does not specify multithreaded execution.
    thread_id: u64,

    /// Output handle is used to redirect output of operations such as PRINTFF, PRINTC
    output_handle: Rc<RefCell<dyn Write>>,

    /// Stack pointer points to the next instruction to execute in instructions list.
    /// Since instructions are loaded top to bottom this
    /// points to instructions(instruction.len()-1-stack_pointer)
    stack_pointer: usize,

    /// stack_maxsize is an emulated variable. It does not specify the actual stack size in the
    /// emulator but is used to enforce a set max size.
    stack_maxsize: usize,

    /// Environment variable can be loaded in using specific instructions such as LDA..LDZ0 and set
    /// with RCA..RCZ
    env_vars: [f64; ENV_VAR_COUNT],

    /// Value lists loaded with instruction VALUE. This list is padded to align with instructions
    values: Vec<f64>,

    /// Operation list loaded with instruction OP. This list is padded to align with instructions
    operations: Vec<MathStackOperators>,

    /// Instruction list denotes the execution of the program from top to bottom
    instructions: Vec<MathStackInstructions>,

    /// Computation stack. Initializes as empty on construction.
    stack: Vec<StackValue>,

    /// Memory heap
    heap: EmulatorHeap,

    /// Loop tracker stack used for tracking nested loops
    loop_counters: Vec<LoopTracker>
}

impl ThreadContext {
    /// Creates a new thread context using vectors to describe the program. The stack is initalized
    /// as empty. The env vars have to be set after creation.
    /// @stack_size: Sets the max size the stack can reach
    /// @values: Vector of values to load into the value list of ThreadContext.
    ///          These values are loaded in from top to bottom.
    /// @operations: Vector of operations that are used in the instruction list.
    ///              These operations are loaded in from top to bottom
    /// @instructions: Vector of instructions that are executed from top to bottom.
    /// @output_stream: Object that implements std::io::Write. This is used for output operations
    ///                 such as PRINTFF, PRINTC.
    pub(crate) fn new(stack_size: usize, values: Vec<f64>, operations: Vec<MathStackOperators>, instructions: Vec<MathStackInstructions>, output_stream: Rc<RefCell<dyn Write>>) -> ThreadContext {
        ThreadContext {
            thread_id: 0,
            output_handle: output_stream,
            stack_pointer: 0,
            stack_maxsize: stack_size,
            env_vars: [0.0; ENV_VAR_COUNT],
            values: Self::pad_list_to_size_of_instructions(MathStackInstructions::VALUE, &instructions, &values, 0.0),
            operations: Self::pad_list_to_size_of_instructions(MathStackInstructions::OP, &instructions, &operations, MathStackOperators::NULL),
            instructions,
            stack: Vec::new(),
            heap: EmulatorHeap::new(),
            loop_counters: Vec::new()
        }
    }

    /// Generic padding function for the values/operations list to be padded to align with the instruction list
    /// This will create a new aligned list where each value is found where the alignment_instr is found
    /// This allows for the stack pointer to be used for all lists without misalignment.
    /// @alignment_instr: Expected either OP or VALUE
    /// @instructions: The list of instructions for the program
    /// @unaligned_list: Either the inputted values or operations list
    /// @null_value: what the padded spaces should be filled with
    /// @return: unaligned_list padded to size of instructions.len()
    fn pad_list_to_size_of_instructions<T: std::clone::Clone>(alignment_instr: MathStackInstructions, instructions: &Vec<MathStackInstructions>, unaligned_list: &Vec<T>, null_value: T) -> Vec<T> {
        // Check if the lists are already the same size.
        // Note: It is impossible to verify if the alignment is correct as the null_value can
        // be used for padding as well as a unaligned value. Best is to just verify length.
        if instructions.len() == unaligned_list.len() {
            return unaligned_list.clone();
        }

        // Pad list with null_value where instructions matches alignment_instr the unaligned_list
        // value is substituted.
        let mut aligned_list: Vec<T> = Vec::new();
        let mut unaligned_index: usize = 0;

        for i in 0..instructions.len() {
            let instr = instructions[i];
            if instr == alignment_instr {
                aligned_list.push(unaligned_list[unaligned_index].clone());
                unaligned_index += 1;
            } else {
                aligned_list.push(null_value.clone());
            }
        };

        aligned_list
    }

    /// Pushes a value onto the execution stack.
    /// @return Ok if successful, otherwise if the push exceeds the stack size ErrorKind::OutOfMemory
    fn push(&mut self, value: StackValue) -> Result<(), Error> {
        if self.stack.len() < self.stack_maxsize {
            Ok(self.stack.push(value))
        } else {
            Err(Error::new(ErrorKind::OutOfMemory, "Tried to push a value to a full stack"))
        }
    }

    /// Pops a value off of the execution stack.
    /// @return value: f64 is successful, otherwise if the stack is empty ErrorKind::AddrNotAvailable
    fn pop(&mut self) -> Result<StackValue, Error> {
        match self.stack.pop() {
            Some(value) => Ok(value),
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to pop value off stack. Reached end of stack"))
        }
    }

    /// Returns current value at pc in value list
    /// @return: value: f64 is successful, otherwise if the value_pointer is at the end of the
    ///          value list ErrorKind::AddrNotAvailable
    fn get_value(&mut self) -> Result<f64, Error> {
        match self.values.get(self.values.len() - self.stack_pointer - 1) {
            Some(value) => Ok(*value),
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to read next value. Reached end of value list"))
        }
    }

    /// Returns current operation at pc in operation list
    /// @return: MathStackOperator if successful, otherwise if the stack_pointer is at the end of
    ///          operation list ErrorKind::AddrNotAvailable
    fn get_operation(&mut self) -> Result<MathStackOperators, Error> {
        match self.operations.get(self.operations.len() - self.stack_pointer - 1) {
            Some(value) => Ok(*value),
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to read next operation. Reached end of operation list"))
        }
    }

    /// Returns current instruction at pc in instruction list
    /// @return: MathStackInstruction if successful, otherwise if the stack_pointer is at the
    ///          end of program list ErrorKind::AddrNotAvailable
    fn get_instruction(&mut self) -> Result<MathStackInstructions, Error> {
        match self.instructions.get(self.instructions.len() - self.stack_pointer - 1) {
            Some(value) => {
                Ok(*value)
            },
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to read next instruction. Reached end of program list"))
        }
    }

    /// Gets the current program counter
    /// @return: Current program counter.
    pub(crate) fn get_pc(&self) -> usize {
        self.stack_pointer
    }

    /// Sets the program counter to a new value
    /// @new_pc: New program counter value
    /// @return: Ok if successful, otherwise if the @new_pc is out of range ErrorKind::AddrNotAvailable
    fn set_pc(&mut self, new_pc: usize) -> Result<(), Error> {
        if new_pc <= self.instructions.len() {
            self.stack_pointer = new_pc;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::AddrNotAvailable, format!("Tried to set program counter out of range {} > {}.", new_pc, self.instructions.len())))
        }
    }

    /// Steps the program counter + 1
    fn step_pc(&mut self) {
        self.stack_pointer += 1;
    }

    /// Will check if the top loop_counter current has reached max. If it hasn't it will increment
    /// current and set pc to loop start. Otherwise it will do nothing.
    fn iterate_loop(&mut self) -> Result<(), Error> {
        let mut loop_finished = false;

        // Update loop
        match self.loop_counters.last_mut() {
            Some(counter) => {
                counter.current += 1;
                if counter.current >= counter.max {
                    loop_finished = true;
                }
            },
            None => return Err(Error::new(ErrorKind::NotFound, "Tried to iterate on a non-existent loop counter"))
        }

        // Update pc and loop counters
        if loop_finished {
            self.loop_counters.pop();
            Ok(())
        } else {
            // Can assume it is safe here
            self.set_pc(self.loop_counters.last().unwrap().loop_start)
        }
    }

    /// Gets loop counters stack
    /// @returns loop counter stack
    pub(crate) fn get_loop_counter_stack(&self) -> &Vec<LoopTracker> {
        &self.loop_counters
    }

    /// Sets an environment variable of the program denoted by var_id
    /// Currently there are 56 env variables available
    /// @var_id: env variable index inclusive of 0-55
    /// @value: value to set env variable to.
    /// @return: Ok if successful otherwise ErrorKind::AddrNotAvailable
    pub(crate) fn set_env_var(&mut self, var_id: usize, value: f64) -> Result<(), Error>  {
        if var_id < ENV_VAR_COUNT {
            self.env_vars[var_id] = value;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to set env var that does not exist."))
        }
    }

    /// Returns an environment variable of the program denoted by var_id
    /// Currently there are 56 env variables available
    /// @var_id: env variable index inclusive of 0-55
    /// @return: env_vars[var_id] if ok otherwise ErrorKind::AddrNotAvailable
    pub(crate) fn get_env_var(&self, var_id: usize) -> Result<f64, Error>  {
        if var_id < ENV_VAR_COUNT {
            Ok(self.env_vars[var_id])
        } else {
            Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to get env var that does not exist."))
        }
    }

    /// Returns the associated environment variable name from the load opcodes
    /// @var_id: env variable index inclusive of 0-55
    /// @return: name of env_vars[var_id] if ok otherwise ErrorKind::AddrNotAvailable
    pub(crate) fn get_env_var_name(&self, var_id: usize) -> Result<String, Error> {
        if var_id < ENV_VAR_COUNT {
            // Bit of a dirty solution but just taking the name from the load opcodes
            match MathStackOperators::from(LDA as u32 + var_id as u32) {
                Some(load_op) => {
                    let load_op_name = format!("{:?}", load_op);
                    Ok(String::from(&load_op_name[2..]))
                }
                None => Err(Error::new(ErrorKind::NotFound, "Could not find the load env var op name"))
            }
        } else {
            Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to get env var name that does not exist."))
        }
    }

    /// Sets the output stream for instructions like PRINTC.
    /// @output_stream: Object that implements std::io::Write. This is used for output operations
    ///                 such as PRINTFF, PRINTC.
    pub(crate) fn set_output_stream(&mut self, output_stream: Rc<RefCell<dyn Write>>) {
        self.output_handle = output_stream
    }

    /// Returns a clone of the current stack state
    pub(crate) fn get_stack(&self) -> Vec<StackValue> {
        self.stack.clone()
    }

    /// Returns a reference of the instructions
    pub(crate) fn get_instructions(&self) -> &Vec<MathStackInstructions> {
        &self.instructions
    }

    /// Returns a reference of the operations
    pub(crate) fn get_operations(&self) -> &Vec<MathStackOperators> {
        &self.operations
    }

    /// Returns a reference of the values
    pub(crate) fn get_values(&self) -> &Vec<f64> {
        &self.values
    }

    /// Returns reference to heap
    pub(crate) fn get_heap(&self) -> &EmulatorHeap {
        &self.heap
    }

    /// Returns if the stack pointer has reached the end of the instruction list
    pub(crate) fn is_execution_finished(&self) -> bool {
        self.stack_pointer == self.instructions.len()
    }

    ///  Will step a single instruction of the program loaded.
    ///  @return: nothing if successful, an io error if an unrecoverable error was encountered
    ///           during execution
    pub(crate) fn step(&mut self) -> Result<(), Error> {
        if self.stack_pointer < self.instructions.len() {
            let instruction = self.get_instruction()?;
            instruction.execute(self)
        } else {
            Ok(())
        }
    }

    /// Will run the program loaded until the execution is finished.
    /// Execution is finished when the program counter reaches the end of the instruction stack.
    /// @return nothing if successful an io error if an unrecoverable error was encountered during
    ///         execution.
    pub(crate) fn run_till_halt(&mut self) -> Result<(), Error> {
        while !self.is_execution_finished() {
            self.step()?;
        }

        Ok(())
    }
}