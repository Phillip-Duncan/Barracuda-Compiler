use barracuda_common::{
    ProgramCode,
    BarracudaOperators,
    FixedBarracudaOperators,
    VariableBarracudaOperators,
    BarracudaInstructions
};

mod test;
mod emulator_heap;
mod instruction_executor;
mod operation_executor;

use self::emulator_heap::EmulatorHeap;
use self::instruction_executor::BarracudaInstructionExecutor;
use std::io::{Error, ErrorKind, Write};
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use std::borrow::BorrowMut;
use std::collections::HashMap;

/// Loop tracker holds the current value of iteration of a loop and the max value of the loop
/// The max value is exclusive
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

    pub fn loop_start(&self) -> usize {
        return self.loop_start;
    }

    pub fn current(&self) -> i64 {
        return self.current;
    }

    pub fn max(&self) -> i64 {
        return self.max;
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

#[derive(Clone)]
pub struct EnvironmentVariable {
    pub name: Option<String>,
    pub address: usize,
    pub value: f64
}

impl EnvironmentVariable {
    pub fn new(name: String, address: usize, value: f64) -> Self {
        Self {
            name: Some(name),
            address,
            value
        }
    }
}

impl Default for EnvironmentVariable {
    fn default() -> Self {
        Self {
            name: None,
            address: 0,
            value: 0.0
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

    /// Program counter points to the next instruction to execute in instructions list.
    /// Since instructions are loaded top to bottom this
    /// points to instructions(instruction.len()-1-program_counter)
    program_counter: usize,

    /// stack_maxsize is an emulated variable. It does not specify the actual stack size in the
    /// emulator but is used to enforce a set max size.
    stack_maxsize: usize,

    /// Environment variable can be loaded in using specific instructions such as LDNX
    env_vars: HashMap<usize, EnvironmentVariable>,

    /// Program code to execute
    program_code: ProgramCode,

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
    pub(crate) fn new(stack_size: usize, values: Vec<f64>, operations: Vec<BarracudaOperators>, instructions: Vec<BarracudaInstructions>, output_stream: Rc<RefCell<dyn Write>>) -> ThreadContext {
        ThreadContext {
            thread_id: 0,
            output_handle: output_stream,
            program_counter: 0,
            stack_maxsize: stack_size,
            env_vars: Default::default(),
            program_code: ProgramCode::new(
                values,
                operations,
                instructions
            ),
            stack: Vec::new(),
            heap: EmulatorHeap::new(),
            loop_counters: Vec::new()
        }
    }

    /// Creates a new thread context using ProgramCode struct to describe the program. The stack is initalized
    /// as empty. The env vars have to be set after creation.
    /// @stack_size: Sets the max size the stack can reach
    /// @program_code: Program code representing the instructions of a program
    /// @output_stream: Object that implements std::io::Write. This is used for output operations
    ///                 such as PRINTFF, PRINTC.
    pub(crate) fn from_code(stack_size: usize, program_code: ProgramCode,  output_stream: Rc<RefCell<dyn Write>>) -> ThreadContext {
        ThreadContext {
            thread_id: 0,
            output_handle: output_stream,
            program_counter: 0,
            stack_maxsize: stack_size,
            env_vars: Default::default(),
            program_code,
            stack: Vec::new(),
            heap: EmulatorHeap::new(),
            loop_counters: Vec::new()
        }
    }

    /// Adds environment variables to thread context
    pub fn with_env_vars(mut self, vars: HashMap<usize, EnvironmentVariable>) -> Self {
        self.env_vars = vars;
        return self;
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

    /// Reads a value from a stack index
    /// @stack_index: Stack index address to value to read
    /// @return StackValue from stack_index address if okay otherwise returns AddrNotAvailable Error if
    ///         stack_index out of range
    fn read_stack(&self, stack_index: usize) -> Result<StackValue, Error> {
        match self.stack.get(stack_index) {
            Some(value) => {
                Ok(value.clone())
            },
            None => Err(Error::new(ErrorKind::AddrNotAvailable,
                                   format!("Invalid read to stack at address {}, StackLength: {}", stack_index, self.stack.len())))
        }
    }

    /// Writes a value to a stack index
    /// @stack_index: Stack index address to value to change
    /// @new_value: Assigns stack[stack_index] = new_value
    /// @return nothing if ok otherwise return AddrNotAvailable Error if stack_index out of range
    fn write_stack(&mut self, stack_index: usize, new_value: StackValue) -> Result<(), Error> {
        match self.stack.get_mut(stack_index) {
            Some(value) => {
                *value = new_value;
                Ok(())
            }
            None =>  Err(Error::new(ErrorKind::AddrNotAvailable,
                                    format!("Invalid write to stack at address {}, StackLength: {}", stack_index, self.stack.len())))
        }
    }

    /// Returns current value at pc in value list
    /// @return: value: f64 is successful, otherwise if the value_pointer is at the end of the
    ///          value list ErrorKind::AddrNotAvailable
    fn get_value(&mut self) -> Result<f64, Error> {
        match self.program_code.values.get(self.program_code.values.len() - self.program_counter - 1) {
            Some(value) => Ok(*value),
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to read next value. Reached end of value list"))
        }
    }

    /// Returns current operation at pc in operation list
    /// @return: BarracudaOperator if successful, otherwise if the stack_pointer is at the end of
    ///          operation list ErrorKind::AddrNotAvailable
    fn get_operation(&mut self) -> Result<BarracudaOperators, Error> {
        match self.program_code.operations.get(self.program_code.operations.len() - self.program_counter - 1) {
            Some(value) => Ok(*value),
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to read next operation. Reached end of operation list"))
        }
    }

    /// Returns current instruction at pc in instruction list
    /// @return: BarracudaInstruction if successful, otherwise if the stack_pointer is at the
    ///          end of program list ErrorKind::AddrNotAvailable
    fn get_instruction(&mut self) -> Result<BarracudaInstructions, Error> {
        match self.program_code.instructions.get(self.program_code.instructions.len() - self.program_counter - 1) {
            Some(value) => {
                Ok(*value)
            },
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to read next instruction. Reached end of program list"))
        }
    }

    /// Gets the current program counter
    /// @return: Current program counter.
    pub(crate) fn get_pc(&self) -> usize {
        self.program_counter
    }

    /// Sets the program counter to a new value
    /// @new_pc: New program counter value
    /// @return: Ok if successful, otherwise if the @new_pc is out of range ErrorKind::AddrNotAvailable
    fn set_pc(&mut self, new_pc: usize) -> Result<(), Error> {
        if new_pc <= self.program_code.instructions.len() {
            self.program_counter = new_pc;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::AddrNotAvailable, format!("Tried to set program counter out of range {} > {}.", new_pc, self.program_code.instructions.len())))
        }
    }

    /// Steps the program counter + 1
    fn step_pc(&mut self) {
        self.program_counter += 1;
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
    /// @var_id: env variable index
    /// @value: value to set env variable to.
    /// @return: Ok if successful otherwise ErrorKind::AddrNotAvailable
    pub(crate) fn set_env_var(&mut self, var_id: usize, value: f64) -> Result<(), Error>  {
        if let Some(env_var) = self.env_vars.get_mut(&var_id) {
            env_var.value = value;
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
        if let Some(env_var) = self.env_vars.get(&var_id) {
            Ok(env_var.value.clone())
        } else {
            Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to get env var that does not exist."))
        }
    }

    /// Returns an immutable map of environment variables the thread context is using
    pub fn get_env_vars(&self) -> &HashMap<usize, EnvironmentVariable> {
        &self.env_vars
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

    /// Returns index of top element of stack
    pub(crate) fn get_stack_pointer(&self) -> Option<usize> {
        return self.stack.len().checked_sub(1);
    }

    /// Changes stack size such that new_ptr points to the top of
    /// the stack
    pub(crate) fn set_stack_pointer(&mut self, new_ptr: usize) {
        self.set_stack_len(new_ptr + 1);
    }

    /// Returns the current length of the stack
    pub(crate) fn get_stack_len(&self) -> usize {
        self.stack.len()
    }

    /// Changes stack size either throwing away values or adding 0s
    /// to reach new size.
    pub(crate) fn set_stack_len(&mut self, new_length: usize) {
        self.stack.resize(new_length, StackValue::UINT(0));
    }

    /// Returns a reference of the instructions
    pub(crate) fn get_instructions(&self) -> &Vec<BarracudaInstructions> {
        &self.program_code.instructions
    }

    /// Returns a reference of the operations
    pub(crate) fn get_operations(&self) -> &Vec<BarracudaOperators> {
        &self.program_code.operations
    }

    /// Returns a reference of the values
    pub(crate) fn get_values(&self) -> &Vec<f64> {
        &self.program_code.values
    }

    /// Returns reference to heap
    pub(crate) fn get_heap(&self) -> &EmulatorHeap {
        &self.heap
    }

    /// Returns if the stack pointer has reached the end of the instruction list
    pub(crate) fn is_execution_finished(&self) -> bool {
        self.program_counter == self.program_code.instructions.len()
    }

    ///  Will step a single instruction of the program loaded.
    ///  @return: nothing if successful, an io error if an unrecoverable error was encountered
    ///           during execution
    pub(crate) fn step(&mut self) -> Result<(), Error> {
        if self.program_counter < self.program_code.instructions.len() {
            let instruction = self.get_instruction()?;
            let executor = BarracudaInstructionExecutor::new(instruction);
            executor.execute(self)
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