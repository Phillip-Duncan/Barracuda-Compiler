use crate::emulator::ThreadContext;
use crate::emulator::ops::MathStackOperators;
use std::io::{Error, ErrorKind};

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct FuncPointerContext {
    ptr_offset : u8
}

impl PartialEq for FuncPointerContext {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_offset==other.ptr_offset
    }
}
impl Eq for FuncPointerContext {}

/// MathStackInstruction is an enum of program instructions that are valid for the MathStack VM.
/// Each instruction has .execute(context: ThreadContext) function to run the instruction on the
/// current thread context.
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MathStackInstructions {
    OP,
    VALUE ,
    GOTO,
    GOTO_IF,
    LOOP_ENTRY,
    LOOP_END,
    FUNC_POINTER(FuncPointerContext)
}

impl MathStackInstructions {
    /// Executes the instruction on a thread context. Calling any appropriate operation or loading
    /// of values.
    /// @context: thread context to apply the instruction execution to.
    /// @return: Ok() on Success, otherwise a io::Error if an instruction fails. ErrorKind::NotFound
    ///          if the instruction is unknown or unimplemented
    pub(crate) fn execute(&self, context: &mut ThreadContext) -> Result<(), Error> {
        println!("INSTR {:?}", self);
        match self {
            Self::VALUE => {
                let value: f64 = context.get_value()?;
                context.push(value)?;
                context.step_pc();
                Ok(())
            },
            Self::OP => {
                let op: MathStackOperators = context.get_operation()?;
                op.execute(context)?;
                context.step_pc();
                Ok(())
            },
            _ => {Err(Error::new(ErrorKind::NotFound, format!("Unknown or unimplemented instruction used {:?}", self))) }
        }
    }
}