// External Modules
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate safer_ffi;
extern crate core;
extern crate barracuda_common;

use safer_ffi::prelude::*;

use compiler::{Compiler, EnvironmentSymbolContext, PrimitiveDataType};

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
    values_list: repr_c::Vec<f64>,

    /// Recommended stack size is an auto generated estimate for the stack size required
    /// to execute the program code. This will give the exact min required size if analysis
    /// goes okay otherwise it will use a default large size.
    recommended_stack_size: usize
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
    ptr_offset: usize,

    datatype: char_p::Box,

    qualifier: char_p::Box,
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
        let datatype = PrimitiveDataType::parse(String::from(env_var.datatype.to_str())).unwrap();
        let qualifier = String::from(env_var.qualifier.to_str());

        context.add_symbol(identifier, address, datatype, qualifier);
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
    let values: Vec<f64> = program_code.values.into_iter().rev()
                                    .map(|value| value as f64).collect();

    CompilerResponse {
        code_text: compiled_text.try_into().unwrap(),
        instructions_list: repr_c::Vec::try_from(instructions).unwrap(),
        operations_list: repr_c::Vec::try_from(operations).unwrap(),
        values_list: repr_c::Vec::try_from(values).unwrap(),
        recommended_stack_size: program_code.max_stack_size
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

// A large number of unit tests for the compiler are below.
#[cfg(test)]
mod tests {
    use barracuda_common::BarracudaInstructions;
    use barracuda_common::BarracudaOperators;
    use barracuda_common::BarracudaInstructions::*;
    use barracuda_common::BarracudaOperators::*;
    use barracuda_common::FixedBarracudaOperators::*;

    use super::*;
    
    // Type to represent values and instructions on one stack for easier testing.
    #[derive(Debug, PartialEq, Clone)]
    enum MergedInstructions {
        Val(f64),
        Op(BarracudaOperators),
        Instr(BarracudaInstructions),
    }
    use MergedInstructions::*;

    fn ptr(int: usize) -> f64 {
        f64::from_ne_bytes(int.to_ne_bytes())
    }

    // Compiles a program string and converts the result to a vector of merged instructions.
    // Because the instruction stack tells Barracuda whether to read from the operation stack or value stack,
    //  they can be merged into one stack.
    // This function also strips the first two elements as they are the same for every program.
    // It also validates everything that doesn't end up in the final stack.
    fn compile_and_merge(text: &str) -> Vec<MergedInstructions> {
        let compiler: Compiler<PARSER, GENERATOR> = Compiler::default();
        let code = compiler.compile_str(text);
        assert!(code.values.len() == code.operations.len() && code.values.len() == code.instructions.len());
        let mut out: Vec<MergedInstructions> = vec![];
        for i in 0..code.values.len() {
            let value = code.values[i];
            let operation = code.operations[i];
            let instruction = code.instructions[i];
            match instruction {
                VALUE => {
                    assert_eq!(FIXED(NULL), operation);
                    out.push(Val(value));
                },
                OP => {
                    assert_eq!(0.0, value);
                    out.push(Op(operation));
                },
                _ => {
                    assert_eq!(0.0, value);
                    assert_eq!(FIXED(NULL), operation);
                    out.push(Instr(instruction));
                }
            }
        }
        assert_eq!([Val(0.0), Val(ptr(1))], out[..2]);
        out[2..].to_vec()
    }

    // Tests an empty program.
    #[test]
    fn empty() {
        let stack = compile_and_merge("");
        assert_eq!(0, stack.len());
    }

    // Tests that all literal values compile properly.
    // Currently test integers, floats, and booleans.
    #[test]
    fn literals() {
        let literals = vec![
            // Integers
            ("0", 0.0),
            ("1", 1.0),
            ("3545", 3545.0),
            ("9007199254740991", 9007199254740991.0), // Maximum safe integer    
            // Floats
            ("0.0", 0.0),
            ("1.0", 1.0),
            ("1.", 1.0),
            ("3545.0", 3545.0),
            ("1000000000000000000000000000000000000000000000000.0", 1000000000000000000000000000000000000000000000000.0),
            ("1.0e3", 1000.0),
            ("1.0e+3", 1000.0),
            ("1.0e-3", 0.001),
            ("1.0e0", 1.0),
            ("1.0e+0", 1.0),
            ("1.0e-0", 1.0),
            ("1.7976931348623157e308", f64::MAX), // Maximum float
            ("2.2250738585072014e-308", f64::MIN_POSITIVE), // Minimum positive float
            // Booleans
            ("false", 0.0),
            ("true", 1.0),
        ];
        for (text, value) in &literals {
            let stack = compile_and_merge(&format!("{};", text));
            assert_eq!(vec![Val(*value)], stack);
        }
    }

    // Tests that all binary operators compile properly.
    // These are operators in the form a OP b.
    #[test]
    fn binary_operators() {
        let binary_operators = vec![
            ("+", ADD),
            ("-", SUB),
            ("*", MUL),
            ("/", DIV),
            ("%", FMOD),
            ("^", POW),
            ("==", EQ),
            ("!=", NEQ),
            (">=", GTEQ),
            ("<=", LTEQ),
            (">", GT),
            ("<", LT),       
        ];
        for (text, op) in &binary_operators {
            let stack = compile_and_merge(&format!("4{}5;", text));
            assert_eq!(vec![Val(4.0), Val(5.0), Op(FIXED(*op))], stack);
        }
    }

    // Tests that all unary operators compile properly.
    // These are operators in the form OP a.
    #[test]
    fn unary_operators() {
        let unary_operators = vec![
            ("!", NOT),
            ("-", NEGATE),     
        ];
        for (text, op) in &unary_operators {
            let stack = compile_and_merge(&format!("{}4;", text));
            assert_eq!(vec![Val(4.0), Op(FIXED(*op))], stack);
        }
    }

    // Tests that whitespace and comments are ignored as expected.
    // The statement 'true;' has no signigicance in the below tests.
    // It's just there to make sure whitespace and comments are ignored correctly.
    #[test]
    fn whitespace_and_comments_ignored() {
        let test_cases = vec![
            "     true    ;    ",
            "\ntrue\n;\n", 
            "\ttrue\t;\t", 
            "\rtrue\r;\r",
            "//comment\ntrue;//comment\n//comment",
            "/*multiline\ncomment*/true;/*multiline comment*//*multiline\ncomment*/",
        ];

        for test_case in &test_cases {
            let stack = compile_and_merge(test_case);
            assert_eq!(vec![Val(1.0)], stack);
        }
    }

    // Tests to make sure unary operations work with operator precedence.
    #[test]
    fn unary_operator_precedence() {
        let unary_operators = vec![
            ("-4-3;", vec![Val(4.0), Op(FIXED(NEGATE)), Val(3.0), Op(FIXED(SUB))]),
            ("4--3;", vec![Val(4.0), Val(3.0), Op(FIXED(NEGATE)), Op(FIXED(SUB))]),
            ("--4--3;", vec![Val(4.0), Op(FIXED(NEGATE)), Op(FIXED(NEGATE)), Val(3.0), Op(FIXED(NEGATE)), Op(FIXED(SUB))]),
            ("-4^3;", vec![Val(4.0), Op(FIXED(NEGATE)), Val(3.0), Op(FIXED(POW))]),
            ("4+-3;", vec![Val(4.0), Val(3.0), Op(FIXED(NEGATE)), Op(FIXED(ADD))]),
        ];
        for (text, expected_stack) in &unary_operators {
            let stack = compile_and_merge(text);
            assert_eq!(*expected_stack, stack);
        }
    }

    // Tests to make sure binary operations work with operator precedence.
    // Tests every pair of binary operations.
    #[test]
    fn binary_operator_precedence() {
        let operators = vec![
            ("+", 3, ADD),
            ("-", 3, SUB),
            ("/", 4, DIV),
            ("%", 4, FMOD),
            ("*", 4, MUL),
            ("^", 5, POW),
            ("==", 1, EQ),
            ("!=", 1, NEQ),
            (">", 2, GT),
            ("<", 2, LT),
            (">=", 2, GTEQ),
            ("<=", 2, LTEQ),
        ];
        for (op_str_1, precedence_1, operation_1) in &operators {
            for (op_str_2, precedence_2, operation_2) in &operators {
                let text = &format!("1{}2{}3;", op_str_1, op_str_2);
                let stack = compile_and_merge(text);
                if precedence_1 >= precedence_2 {
                    assert_eq!(vec![Val(1.0), Val(2.0), Op(FIXED(*operation_1)), Val(3.0), Op(FIXED(*operation_2))], stack);
                } else {
                    assert_eq!(vec![Val(1.0), Val(2.0), Val(3.0), Op(FIXED(*operation_2)), Op(FIXED(*operation_1))], stack);
                }
            }
        }
    }

    // Tests that parentheses work with operator precedence.
    #[test]
    fn parentheses_precedence() {
        assert_eq!(vec![Val(1.0), Val(2.0), Op(FIXED(SUB)), Val(3.0), Op(FIXED(ADD))], 
            compile_and_merge("(1-2)+3;"));
        assert_eq!(vec![Val(1.0), Val(2.0), Op(FIXED(SUB)), Val(3.0), Op(FIXED(ADD))], 
            compile_and_merge("(((1-2+3)));"));
        assert_eq!(vec![Val(1.0), Val(2.0), Val(3.0), Op(FIXED(ADD)), Op(FIXED(SUB))], 
            compile_and_merge("1-(2+3);"));
    }

    // Generates a function call given the current stack position and the location of the start of the function.
    // Returns the generated function call and the new position of the stack.
    fn generate_function_call(position: usize, func_location: usize) -> (Vec<MergedInstructions>, usize) {
        return (vec![Val(ptr(position + 13)), Val(ptr(1)), Op(FIXED(STK_READ)), 
        Op(FIXED(LDSTK_PTR)), Val(ptr(1)), Op(FIXED(SUB_PTR)), Val(ptr(1)), Op(FIXED(SWAP)), 
        Op(FIXED(STK_WRITE)), Val(ptr(func_location)), Instr(GOTO), Val(0.0), Op(FIXED(STK_READ))], position + 13);
    }

    // Generates a function definition for an empty function given the current stack position.
    // Returns the generated function definition, the location of the start of the function, and the new position of the stack.
    fn generate_empty_function_definition(position: usize) -> (Vec<MergedInstructions>, usize, usize) {
        return (vec![Val(ptr(position + 13)), Instr(GOTO), Val(ptr(1)), Op(FIXED(STK_READ)), Val(ptr(1)), 
            Op(FIXED(ADD_PTR)), Op(FIXED(RCSTK_PTR)), Val(ptr(1)), Op(FIXED(SWAP)), Op(FIXED(STK_WRITE)), Instr(GOTO)]
            , position + 4, position + 11);
    }
    
    // Tests to make sure functions work.
    #[test]
    fn empty_functions() {
        // Checks unused function still exists
        let stack = compile_and_merge(
            "fn test_func() {}");
        let (function_def, _, _) = generate_empty_function_definition(0);
        assert_eq!(function_def, stack);
        // Checks calling that function works
        let stack = compile_and_merge(
            "fn test_func() {} test_func();");
        let (function_def, test_func_location, position) 
            = generate_empty_function_definition(0);
        assert_eq!(function_def, stack[..position]);
        let (function_call, _) 
            = generate_function_call(position, test_func_location);
        assert_eq!(function_call, stack[position..]);
        // Checks calling a function 3 times
        let stack = compile_and_merge(
            "fn test_func() {} test_func(); test_func(); test_func();");
        let (function_def, test_func_location, position) 
            = generate_empty_function_definition(0);
        assert_eq!(function_def, stack[..position]);
        let (function_call, position_2) 
            = generate_function_call(position, test_func_location);
        assert_eq!(function_call, stack[position..position_2]);
        let (function_call, position_3) 
            = generate_function_call(position_2, test_func_location);
        assert_eq!(function_call, stack[position_2..position_3]);
        let (function_call, _) 
            = generate_function_call(position_3, test_func_location);
        assert_eq!(function_call, stack[position_3..]);

    }
    
    // TODO: func_statement, if_statement, for_statement, while_statement, construct_statement, return_statement, assign_statement, print_statement, external_statement 
}