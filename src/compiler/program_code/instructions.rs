use strum_macros::EnumString;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use safer_ffi::prelude::*;


/// BarracudaInstruction is an enum of program instructions that are valid for the Barracuda VM.
/// Each instruction has .execute(context: ThreadContext) function to run the instruction on the
/// current thread context.
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, ToPrimitive, EnumString)]
#[repr(u32)]
pub enum BarracudaInstructions {
    OP          = 0,   // Runs Operation at the same index in the operations table
    VALUE       = 1,   // Load Immediate the same index in the value table
    GOTO        = 2,   // Pop address and set stack pointer
    GOTO_IF     = 3,   // Pop address and condition set stack pointer if condition==0
    LOOP_ENTRY  = 99,  // Pop start, end values and create a new loop counter
    LOOP_END    = 100  // Set stack pointer to most recent loop entry
}

impl BarracudaInstructions {

    /// Converts instruction into value representing the instruction code
    /// @returns: &self's representation as u32. This is not an option as all instructions
    ///           have a valid u32 code.
    #[allow(dead_code)] // Used in library but not the binary
    pub fn as_u32(&self) -> u32 {
        // Safe to unwrap here as enum should always map to an integer.
        self.to_u32().unwrap()
    }
}