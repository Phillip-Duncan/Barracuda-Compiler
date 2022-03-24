pub(crate) mod ops;
pub(crate) mod instructions;

use crate::emulator::ops::MathStackOperators;
use crate::emulator::instructions::MathStackInstructions;
use std::io::{Error, ErrorKind};
use std::io::{self};

/// Environment var count as given by the operations. This needs to be updated manually if adding
/// more env var load instructions.
pub const ENV_VAR_COUNT: usize = 56;


/// Thread context is a struct that represents all the information an individual thread would
/// have access to. It also includes functions with step() and run_till_halt() to emulate the
/// execution of the program.
pub struct ThreadContext {
    /// thread_id is an emulated variable. It does not specify multithreaded execution.
    thread_id: u64,

    /// Output handle is used to redirect output of operations such as PRINTFF, PRINTC
    output_handle: Box<dyn io::Write>,

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

    /// Value lists loaded with instruction VALUE
    values: Vec<f64>,

    /// Operation list loaded with instruction OP
    operations: Vec<MathStackOperators>,

    /// Instruction list denotes the execution of the program from top to bottom
    instructions: Vec<MathStackInstructions>,

    /// Computation stack. Initializes as empty on construction.
    stack: Vec<f64>
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
    pub(crate) fn new(stack_size: usize, values: Vec<f64>, operations: Vec<MathStackOperators>, instructions: Vec<MathStackInstructions>, output_stream: Box<dyn io::Write>) -> ThreadContext {
        ThreadContext {
            thread_id: 0,
            output_handle: Box::new(io::BufWriter::new(output_stream)),
            stack_pointer: 0,
            stack_maxsize: stack_size,
            env_vars: [0.0; ENV_VAR_COUNT],
            values,
            operations,
            instructions,
            stack: Vec::new()
        }
    }

    /// Pushes a value onto the execution stack.
    /// @return Ok if successful, otherwise if the push exceeds the stack size ErrorKind::OutOfMemory
    fn push(&mut self, value: f64) -> Result<(), Error> {
        if self.stack.len() < self.stack_maxsize {
            Ok(self.stack.push(value))
        } else {
            Err(Error::new(ErrorKind::OutOfMemory, "Tried to push a value to a full stack"))
        }
    }

    /// Pops a value off of the execution stack.
    /// @return value: f64 is successful, otherwise if the stack is empty ErrorKind::AddrNotAvailable
    fn pop(&mut self) -> Result<f64, Error> {
        match self.stack.pop() {
            Some(value) => Ok(value),
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to pop value off stack. Reached end of stack"))
        }
    }

    /// Returns next value in value list
    /// @return: value: f64 is successful, otherwise if the value_pointer is at the end of the
    ///          value list ErrorKind::AddrNotAvailable
    fn next_value(&mut self) -> Result<f64, Error> {
        match self.values.pop() {
            Some(value) => Ok(value),
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to read next value. Reached end of value list"))
        }
    }

    /// Returns next operation in operation list
    /// @return: MathStackOperator if successful, otherwise if the stack_pointer is at the end of
    ///          operation list ErrorKind::AddrNotAvailable
    fn next_operation(&mut self) -> Result<MathStackOperators, Error> {
        match self.operations.pop() {
            Some(value) => Ok(value),
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to read next operation. Reached end of operation list"))
        }
    }

    /// Returns next instruction in instruction list
    /// @return: MathStackInstruction if successful, otherwise if the stack_pointer is at the
    ///          end of program list ErrorKind::AddrNotAvailable
    fn next_instruction(&mut self) -> Result<MathStackInstructions, Error> {
        match self.instructions.get(self.instructions.len() - self.stack_pointer - 1) {
            Some(value) => {
                self.stack_pointer += 1;
                Ok(*value)
            },
            None => Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to read next instruction. Reached end of program list"))
        }
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
    pub(crate) fn get_env_var(&mut self, var_id: usize) -> Result<f64, Error>  {
        if var_id < ENV_VAR_COUNT {
            Ok(self.env_vars[var_id])
        } else {
            Err(Error::new(ErrorKind::AddrNotAvailable, "Tried to get env var that does not exist."))
        }
    }

    /// Returns a clone of the current stack state
    pub(crate) fn get_stack(&self) -> Vec<f64> {
        self.stack.clone()
    }

    /// Returns if the stack pointer has reached the end of the instruction list
    pub(crate) fn is_execution_finished(&self) -> bool {
        self.stack_pointer == self.instructions.len()
    }


    ///    Will step a single instruction of the program loaded.
    ///    @return: nothing if successful, an io error if an unrecoverable error was encountered during
    ///           execution
    pub(crate) fn step(&mut self) -> Result<(), Error> {
        let instruction = self.next_instruction()?;
        instruction.execute(self)
    }

    ///    Will run the program loaded until the execution is finished.
    ///    Execution is finished when the program counter reaches the end of the instruction stack.
    ///    @return nothing if successful an io error if an unrecoverable error was encountered during
    ///            execution.
    pub(crate) fn run_till_halt(&mut self) -> Result<(), Error> {
        while !self.is_execution_finished() {
            self.step()?;
        }

        Ok(())
    }
}