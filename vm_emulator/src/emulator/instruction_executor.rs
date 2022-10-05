use barracuda_common::BarracudaInstructions;
use barracuda_common::BarracudaOperators;
use super::operation_executor::BarracudaOperationExecutor;

use super::{
    ThreadContext,
    LoopTracker,
    StackValue::*
};

pub struct BarracudaInstructionExecutor {
    instruction: BarracudaInstructions
}

impl BarracudaInstructionExecutor {
    pub fn new(instruction: BarracudaInstructions) -> Self {
        Self {
            instruction
        }
    }

    /// Executes the instruction on a thread context. Calling any appropriate operation or loading
    /// of values.
    /// @context: thread context to apply the instruction execution to.
    /// @return: Ok() on Success, otherwise a io::Error if an instruction fails. ErrorKind::NotFound
    ///          if the instruction is unknown or unimplemented
    pub fn execute(&self, context: &mut ThreadContext) -> Result<(), std::io::Error> {
        match self.instruction {
            BarracudaInstructions::VALUE => {
                let value: f64 = context.get_value()?;
                context.push(REAL(value))?;
                context.step_pc();
                Ok(())
            },
            BarracudaInstructions::OP => {
                let op = context.get_operation()?;
                let executor = BarracudaOperationExecutor::new(op);
                executor.execute(context)?;
                context.step_pc();
                Ok(())
            },
            BarracudaInstructions::GOTO => {
                let address = context.pop()?.into_u64();
                context.set_pc(address as usize)?;
                Ok(())
            },
            BarracudaInstructions::GOTO_IF => {
                let b = context.pop()?.into_u64();
                let a = context.pop()?.into_i64();
                if a == 0 {
                    context.set_pc(b as usize)?;
                } else {
                    context.step_pc();
                }
                Ok(())
            },
            BarracudaInstructions::LOOP_ENTRY => {
                let b = context.pop()?.into_i64();
                let a = context.pop()?.into_i64();
                context.loop_counters.push(LoopTracker::new(a,b, context.program_counter));
                context.step_pc();
                Ok(())
            },
            BarracudaInstructions::LOOP_END => {
                context.iterate_loop()?;
                context.step_pc();
                Ok(())
            }
            _ => {Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Unknown or unimplemented instruction used {:?}", self.instruction))) }
        }
    }
}