use crate::emulator::ThreadContext;
use crate::emulator::ops::MathStackOperators;
use std::io::{Error, ErrorKind};
use crate::emulator::LoopTracker;
use crate::emulator::StackValue::*;
use strum_macros::EnumString;

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumString)]
pub enum MathStackInstructions {
    OP,         // Runs Operation at the same index in the operations table
    VALUE ,     // Load Immediate the same index in the value table
    GOTO,       // Pop address and set stack pointer
    GOTO_IF,    // Pop address and condition set stack pointer if condition==0
    LOOP_ENTRY, // Pop start, end values and create a new loop counter
    LOOP_END,   // Set stack pointer to most recent loop entry
    #[strum(disabled)]
    FUNC_POINTER(FuncPointerContext)
}

impl MathStackInstructions {
    /// Executes the instruction on a thread context. Calling any appropriate operation or loading
    /// of values.
    /// @context: thread context to apply the instruction execution to.
    /// @return: Ok() on Success, otherwise a io::Error if an instruction fails. ErrorKind::NotFound
    ///          if the instruction is unknown or unimplemented
    pub(crate) fn execute(&self, context: &mut ThreadContext) -> Result<(), Error> {
        match self {
            Self::VALUE => {
                let value: f64 = context.get_value()?;
                context.push(REAL(value))?;
                context.step_pc();
                Ok(())
            },
            Self::OP => {
                let op: MathStackOperators = context.get_operation()?;
                op.execute(context)?;
                context.step_pc();
                Ok(())
            },
            Self::GOTO => {
                let address = context.pop()?.into_u64();
                context.set_pc(address as usize)?;
                Ok(())
            },
            Self::GOTO_IF => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_i64();
                if a == 0 {
                    context.set_pc(b as usize)?;
                } else {
                    context.step_pc();
                }
                Ok(())
            },
            Self::LOOP_ENTRY => {
                let b = context.pop()?.into_i64();
                let a = context.pop()?.into_i64();
                context.loop_counters.push(LoopTracker::new(a,b, context.program_counter));
                context.step_pc();
                Ok(())
            },
            Self::LOOP_END => {
                context.iterate_loop()?;
                context.step_pc();
                Ok(())
            }
            _ => {Err(Error::new(ErrorKind::NotFound, format!("Unknown or unimplemented instruction used {:?}", self))) }
        }
    }
}