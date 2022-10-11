// External Modules
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate safer_ffi;
extern crate core;
extern crate barracuda_common;

use safer_ffi::prelude::*;

use compiler::{Compiler, EnvironmentSymbolContext};

// Internal Modules
mod compiler;

// Compiler types to use
type PARSER = compiler::PestBarracudaParser;
type GENERATOR = compiler::BarracudaByteCodeGenerator;


/// Compiler response describes a successful compilation result
/// It contains the relevant vectors required to run the program
/// code on the barracuda virtual machine.
#[derive_ReprC]
#[repr(C)]
pub struct CompilerResponse {
    /// Code text is a null-terminated string with the textual representation
    /// of the program code. This is not a required field for the VM.
    code_text: char_p::Box,      // C Repr: char *

    /// Instruction list describes each instruction to be run by the VM
    instructions_list: repr_c::Vec<u32>,

    /// Operations list describes the operation to run during a OP instruction.
    operations_list: repr_c::Vec<u64>,

    /// Value list describes the value to load during a VALUE instruction.
    values_list: repr_c::Vec<f32>
}

/// EnvironmentVariable describes an environment variable the program will have access to in the
/// target environment. These variables can be loaded in code using 'extern <identifier>;' statements.
/// If the environment variable is not defined the compiler will throw an error.
#[derive_ReprC]
#[repr(C)]
pub struct EnvironmentVariable {
    /// Identifier is the name of the variable to use in the script.
    identifier: char_p::Box,

    /// ptr offset describes the location of the variable in the host environment.
    ptr_offset: usize
}

/// Compiler request describes the content needed to attempt a compilation.
/// It contains the code text string and compilation options.
#[derive_ReprC]
#[repr(C)]
pub struct CompilerRequest {
    /// Code text is a null-terminated string with the textual representation
    /// of barracuda high-level code.
    code_text: char_p::Box,       // C repr: char *

    /// Environment variables are used to share data between the host environment and user code
    /// they are mutable and defined by their name for use in barracuda code and their offset
    /// for the memory location in the host environment user space.
    env_vars: repr_c::Vec<EnvironmentVariable>
}

// Private
fn generate_environment_context(request: &CompilerRequest) -> EnvironmentSymbolContext {
    let mut context = EnvironmentSymbolContext::new();

    for env_var in request.env_vars.iter() {
        let identifier = String::from(env_var.identifier.to_str());
        let address = env_var.ptr_offset;

        context.add_symbol(identifier, address);
    }

    return context;
}

/// Compile attempts to compile a CompilerRequest into Barracuda VM
/// low level instructions. The memory for the compiler response
/// is allocated on call, it is then the responsibility of the caller to
/// free this memory via free_compile_response.
#[ffi_export]
pub fn compile(request: &CompilerRequest) -> CompilerResponse {
    let env_vars = generate_environment_context(&request);

    let compiler: Compiler<PARSER, GENERATOR> = Compiler::default()
        .set_environment_variables(env_vars);
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


/// Frees a compiler response returned via the API
/// Calling the function is a requirement after using a response.
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
#[safer_ffi::cfg_headers]
#[test]
fn generate_headers() -> std::io::Result<()> {
    safer_ffi::headers::builder()
        .to_file("include/barracuda_compiler.h")?
        .generate()
}
