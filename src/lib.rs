// External Modules
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate safer_ffi;

use std::error::Error;
use std::path::Path;

use safer_ffi::prelude::*;

use compiler::Compiler;
use compiler::program_code::instructions::BarracudaInstructions;
use compiler::program_code::ops::BarracudaOperators;

// Internal Modules
mod compiler;

type PARSER = compiler::PestBarracudaParser;
type GENERATOR = compiler::BarracudaByteCodeGenerator;


#[derive_ReprC]
#[repr(C)]
pub struct CompilerResponse {
    code_text: char_p::Box,      // C Repr: char *
    instructions_list: repr_c::Vec<u32>,
    operations_list: repr_c::Vec<u64>,
    values_list: repr_c::Vec<f32>
}

#[derive_ReprC]
#[repr(C)]
pub struct CompilerRequest {
    code_text: char_p::Box       // C repr: char *
}

/// Public Definitions
#[ffi_export]
pub fn compile(request: &CompilerRequest) -> CompilerResponse {
    let compiler: Compiler<PARSER, GENERATOR> = Compiler::default();
    let program_code = compiler.compile_str(request.code_text.to_str());
    let compiled_text = program_code.to_string();

    // Convert program code components into primitives
    let instructions: Vec<u32> = program_code.instructions.into_iter().rev()
                                    .map(|instr| instr.as_u32()).collect();
    let operations: Vec<u64> = program_code.operations.into_iter().rev()
                                    .map(|op| op.as_u32() as u64).collect();
    let values: Vec<f32> = program_code.values.into_iter().rev()
                                    .map(|value| value as f32).collect();

    CompilerResponse {
        code_text: compiled_text.try_into().unwrap(),
        instructions_list: repr_c::Vec::try_from(instructions).unwrap(),
        operations_list: repr_c::Vec::try_from(operations).unwrap(),
        values_list: repr_c::Vec::try_from(values).unwrap()
    }
}

#[ffi_export]
pub fn free_compile_response(response: CompilerResponse) {
    drop(response.code_text);
    drop(response.instructions_list);
    drop(response.operations_list);
    drop(response.values_list);
}


// Header generator
// To generate call:
// $ cargo test --features c-headers -- generate_headers
#[::safer_ffi::cfg_headers]
#[test]
fn generate_headers() -> ::std::io::Result<()> {
    ::safer_ffi::headers::builder()
        .to_file("include/barracuda_compiler.h")?
        .generate()
}
