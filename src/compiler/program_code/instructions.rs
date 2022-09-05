use super::ops::BarracudaOperators;
use std::io::{Error, ErrorKind};
use strum_macros::EnumString;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use safer_ffi::prelude::*;

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

/// BarracudaInstruction is an enum of program instructions that are valid for the Barracuda VM.
/// Each instruction has .execute(context: ThreadContext) function to run the instruction on the
/// current thread context.
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, ToPrimitive, EnumString)]
#[repr(u32)]
pub enum BarracudaInstructions {
    OP,         // Runs Operation at the same index in the operations table
    VALUE ,     // Load Immediate the same index in the value table
    GOTO,       // Pop address and set stack pointer
    GOTO_IF,    // Pop address and condition set stack pointer if condition==0
    LOOP_ENTRY, // Pop start, end values and create a new loop counter
    LOOP_END   // Set stack pointer to most recent loop entry
    // #[strum(disabled)]
    // FUNC_POINTER(FuncPointerContext)
}

impl BarracudaInstructions {
    pub(crate) fn as_u32(&self) -> u32 {
        // Safe to unwrap here as enum should always map to an integer.
        self.to_u32().unwrap()
    }
}