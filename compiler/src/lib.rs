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
type ANALYSER = compiler::BarracudaSemanticAnalyser;
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

    let compiler: Compiler<PARSER, ANALYSER, GENERATOR> = Compiler::default()
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
    fn compile_and_merge_with_env_vars(text: &str, env_vars: EnvironmentSymbolContext) -> Vec<MergedInstructions> {
        let compiler: Compiler<PARSER, ANALYSER, GENERATOR> = Compiler::default()
            .set_environment_variables(env_vars);
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

    // Compiles a program string without providing environemnt variables.
    fn compile_and_merge(text: &str) -> Vec<MergedInstructions> {
        compile_and_merge_with_env_vars(text, EnvironmentSymbolContext::new())
    }

    fn compile_and_assert_equal(x: &str, y: &str) {
        assert_eq!(compile_and_merge(x), compile_and_merge(y))
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
            let stack = compile_and_merge(&format!("let a = {};", text));
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
            let stack = compile_and_merge(&format!("let a = 4{}5;", text));
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
            let stack = compile_and_merge(&format!("let a = {}4;", text));
            assert_eq!(vec![Val(4.0), Op(FIXED(*op))], stack);
        }
    }

    // Tests that whitespace and comments are ignored as expected.
    // The statement 'let a = true;' has no signigicance in the below tests.
    // It's just there to make sure whitespace and comments are ignored correctly.
    #[test]
    fn whitespace_and_comments_ignored() {
        let test_cases = vec![
            "     let a = true    ;    ",
            "\nlet a = true\n;\n", 
            "\tlet a = true\t;\t", 
            "\rlet a = true\r;\r",
            "//comment\nlet a = true;//comment\n//comment",
            "/*multiline\ncomment*/let a = true;/*multiline comment*//*multiline\ncomment*/",
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
            ("-4-3", vec![Val(4.0), Op(FIXED(NEGATE)), Val(3.0), Op(FIXED(SUB))]),
            ("4--3", vec![Val(4.0), Val(3.0), Op(FIXED(NEGATE)), Op(FIXED(SUB))]),
            ("--4--3", vec![Val(4.0), Op(FIXED(NEGATE)), Op(FIXED(NEGATE)), Val(3.0), Op(FIXED(NEGATE)), Op(FIXED(SUB))]),
            ("-4^3", vec![Val(4.0), Op(FIXED(NEGATE)), Val(3.0), Op(FIXED(POW))]),
            ("4+-3", vec![Val(4.0), Val(3.0), Op(FIXED(NEGATE)), Op(FIXED(ADD))]),
        ];
        for (text, expected_stack) in &unary_operators {
            let stack = compile_and_merge(&format!("let a = {};", text));
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
                let text = &format!("let a = 1{}2{}3;", op_str_1, op_str_2);
                let stack = compile_and_merge(text);
                if precedence_1 >= precedence_2 {
                    assert_eq!(vec![Val(1.0), Val(2.0), Op(FIXED(*operation_1)), Val(3.0), Op(FIXED(*operation_2))], stack);
                } else {
                    assert_eq!(vec![Val(1.0), Val(2.0), Val(3.0), Op(FIXED(*operation_2)), Op(FIXED(*operation_1))], stack);
                }
            }
        }
    }


    #[test]
    fn builtin_functions() {
        let functions = vec![ACOS,ACOSH,ASIN,ASINH,ATAN,ATAN2,ATANH,CBRT,CEIL,CPYSGN,COS,COSH,
        COSPI,BESI0,BESI1,ERF,ERFC,ERFCI,ERFCX,ERFI,EXP,EXP10,EXP2,EXPM1,FABS,FDIM,FLOOR,FMA,FMAX,FMIN,FMOD,FREXP,HYPOT,
        ILOGB,ISFIN,ISINF,ISNAN,BESJ0,BESJ1,BESJN,LDEXP,LGAMMA,LLRINT,LLROUND,LOG,LOG10,LOG1P,LOG2,LOGB,LRINT,LROUND,MAX,MIN,MODF,
        NXTAFT,POW,RCBRT,REM,REMQUO,RHYPOT,RINT,ROUND,
        RSQRT,SCALBLN,SCALBN,SGNBIT,SIN,SINH,SINPI,SQRT,TAN,TANH,TGAMMA,TRUNC,BESY0,BESY1,BESYN];
        for function in &functions {
            let input = vec!["3"; function.consume() as usize].join(",");
            let text = &format!("let a = __{}({});", function.to_string().to_lowercase(), input);
            let stack = compile_and_merge(text);
            let mut output = vec![Val(3.0); function.consume() as usize];
            output.push(Op(FIXED(*function)));
            assert_eq!(output, stack); 
        }
    }

    // Tests that parentheses work with operator precedence.
    #[test]
    fn parentheses_precedence() {
        assert_eq!(vec![Val(1.0), Val(2.0), Op(FIXED(SUB)), Val(3.0), Op(FIXED(ADD))], 
            compile_and_merge("let a = (1-2)+3;"));
        assert_eq!(vec![Val(1.0), Val(2.0), Op(FIXED(SUB)), Val(3.0), Op(FIXED(ADD))], 
            compile_and_merge("let a = (((1-2+3)));"));
        assert_eq!(vec![Val(1.0), Val(2.0), Val(3.0), Op(FIXED(ADD)), Op(FIXED(SUB))], 
            compile_and_merge("let a = 1-(2+3);"));
    }

    // Generates a function call given the current stack position, the location of the start of the function, 
    // and how many parameters the function takes.
    // Returns the generated function call and the new position of the stack.
    fn generate_function_call(position: usize, func_location: usize, parameters: usize) -> (Vec<MergedInstructions>, usize) {
        let mut function_call = vec![Val(ptr(position + 13)), Val(ptr(1)), Op(FIXED(STK_READ)), 
            Op(FIXED(LDSTK_PTR)), Val(ptr(1)), Op(FIXED(SUB_PTR)), Val(ptr(1)), Op(FIXED(SWAP)), 
            Op(FIXED(STK_WRITE)), Val(ptr(func_location)), Instr(GOTO)];
        for _ in 0..parameters {
            function_call.push(Op(FIXED(DROP)));
        }
        function_call.extend(vec![Val(0.0), Op(FIXED(STK_READ))]);

        return (function_call, position + 13 + parameters);
    }

    // Generates a function call given the current stack position and the location of the start of the function.
    // Returns the generated function call and the new position of the stack.
    fn generate_default_function_call(position: usize, func_location: usize) -> (Vec<MergedInstructions>, usize) {
        return generate_function_call(position, func_location, 0);
    }


    // Generates a function definition for an function given the current stack position and the compiled body of the function.
    // Returns the generated function definition, the location of the start of the function, and the new position of the stack.
    fn generate_function_def_precompiled(position: usize, compiled_body: Vec<MergedInstructions>) 
            -> (Vec<MergedInstructions>, usize, usize) {
        let body_length = compiled_body.len();
        let mut function_definition = vec![Val(ptr(position + 13 + body_length)), Instr(GOTO)];
        function_definition.extend(compiled_body);
        function_definition.extend(vec![Val(ptr(1)), Op(FIXED(STK_READ)), Val(ptr(1)), Op(FIXED(ADD_PTR)), 
            Op(FIXED(RCSTK_PTR)), Val(ptr(1)), Op(FIXED(SWAP)), Op(FIXED(STK_WRITE)), Instr(GOTO)]);
        return (function_definition, position + 4, position + 11 + body_length);
    }

    // Generates a function definition for an function given the current stack position and the body of the function.
    // Returns the generated function definition, the location of the start of the function, and the new position of the stack.
    fn generate_function_definition(position: usize, body: &str) -> (Vec<MergedInstructions>, usize, usize) {
        let compiled_body = compile_and_merge(body);
        return generate_function_def_precompiled(position, compiled_body)
    }
    
    // Generates a function definition for an empty function given the current stack position.
    // Returns the generated function definition, the location of the start of the function, and the new position of the stack.
    fn generate_empty_function_definition(position: usize) -> (Vec<MergedInstructions>, usize, usize) {
        return generate_function_definition(position, "")
    }

    // Tests that creating functions works
    #[test]
    fn empty_function() {
        let stack = compile_and_merge(
            "fn test_func() {}");
        assert_eq!(0, stack.len());
    }

    // Tests calling functions works
    #[test]
    fn function_call() {
        let stack = compile_and_merge(
            "fn test_func() {} let a = test_func();");
        let (function_def, test_func_location, position) 
            = generate_empty_function_definition(0);
        assert_eq!(function_def, stack[..position]);
        let (function_call, _) 
            = generate_default_function_call(position, test_func_location);
        assert_eq!(function_call, stack[position..]);
    }

    // Tests calling a function without assigning it to a variable works
    #[test]
    fn naked_function_call() {
        let stack = compile_and_merge(
            "fn test_func() {} test_func();");
        let (function_def, test_func_location, position) 
            = generate_empty_function_definition(0);
        assert_eq!(function_def, stack[..position]);
        let (function_call, position_2) 
            = generate_default_function_call(position, test_func_location);
        assert_eq!(function_call, stack[position..position_2]);
        assert_eq!(vec!(Op(FIXED(DROP))), stack[position_2..]); // Must drop as we don't need to keep the return value
    }

    // Tests calling a function 3 times
    #[test]
    fn function_multiple_call() {
        let stack = compile_and_merge(
            "fn test_func() {} let a = test_func(); let b = test_func(); let c = test_func();");
        let (function_def, test_func_location, position) 
            = generate_empty_function_definition(0);
        assert_eq!(function_def, stack[..position]);
        let (function_call, position_2) 
            = generate_default_function_call(position, test_func_location);
        assert_eq!(function_call, stack[position..position_2]);
        let (function_call, position_3) 
            = generate_default_function_call(position_2, test_func_location);
        assert_eq!(function_call, stack[position_2..position_3]);
        let (function_call, _) 
            = generate_default_function_call(position_3, test_func_location);
        assert_eq!(function_call, stack[position_3..]);
    }

    // Tests defining and calling two functions
    #[test]
    fn double_function() {
        let stack = compile_and_merge(
            "fn test_func() {} fn test_func_2() {} let a = test_func(); let b = test_func_2();");
        let (function_def, test_func_location, position) 
            = generate_empty_function_definition(0);
        assert_eq!(function_def, stack[..position]);
        let (function_def, test_func_2_location, position_2) 
            = generate_empty_function_definition(position);
        assert_eq!(function_def, stack[position..position_2]);
        let (function_call, position_3) 
            = generate_default_function_call(position_2, test_func_location);
        assert_eq!(function_call, stack[position_2..position_3]);
        let (function_call, _) 
            = generate_default_function_call(position_3, test_func_2_location);
        assert_eq!(function_call, stack[position_3..]);
    }

    // Tests defining a function with content
    #[test]
    fn function_with_contents() {
        let function_contents = "let a = 3+4;";
        let stack = compile_and_merge(
            &format!("fn test_func() {{{}}} let a = test_func();", function_contents));
        let (function_def, test_func_location, position) 
            = generate_function_definition(0, function_contents);
        assert_eq!(function_def, stack[..position]);
        let (function_call, position_2) 
            = generate_default_function_call(position, test_func_location);
        assert_eq!(function_call, stack[position..position_2]);
    }

    // Checks calling a parameterized function
    #[test]
    fn function_with_parameter_call() {
        let stack = compile_and_merge("fn test_func(a) {} let a = test_func(4);");
        let (function_def, test_func_location, position) 
            = generate_empty_function_definition(0);
        assert_eq!(function_def, stack[..position]);
        assert_eq!(Val(4.0), stack[position]);
        let position = position + 1;
        let (function_call, _) 
            = generate_function_call(position, test_func_location, 1);
        assert_eq!(function_call, stack[position..]);
    }

    // Checks defining a function with a parameter and then using that parameter
    #[test]
    fn function_with_parameter_used() {
        let stack = compile_and_merge("fn test_func(a) {let b = a;} let a = test_func(4);");
        let (function_def, test_func_location, position) 
            = generate_function_def_precompiled(0, 
                vec![Val(ptr(1)), Op(FIXED(STK_READ)), Val(ptr(2)), Op(FIXED(SUB_PTR)), Op(FIXED(STK_READ))]);
        assert_eq!(function_def, stack[..position]);
        assert_eq!(Val(4.0), stack[position]);
        let position = position + 1;
        let (function_call, _) 
            = generate_function_call(position, test_func_location, 1);
        assert_eq!(function_call, stack[position..]);
    }

    // Checks return works
    #[test]
    fn function_with_return() {
        let stack = compile_and_merge("fn test_func() {return 3;} let a = test_func();");
        let (function_def, test_func_location, position) = generate_function_def_precompiled(0, 
            vec![Val(0.0), Val(3.0), Op(FIXED(STK_WRITE)), // write variable to stack
            Val(ptr(1)), Op(FIXED(STK_READ)), Val(ptr(1)), Op(FIXED(ADD_PTR)), // return from function
            Op(FIXED(RCSTK_PTR)), Val(ptr(1)), Op(FIXED(SWAP)), Op(FIXED(STK_WRITE)), Instr(GOTO)]);
        assert_eq!(function_def, stack[..position]);
        let (function_call, _) 
            = generate_default_function_call(position, test_func_location);
        assert_eq!(function_call, stack[position..]);
    }

    // Checks return types work
    #[test]
    fn function_with_return_used() {
        let stack = compile_and_merge("fn test_func() {return 3;} let a = test_func() * 3;");
        let (function_def, test_func_location, position) = generate_function_def_precompiled(0, 
            vec![Val(0.0), Val(3.0), Op(FIXED(STK_WRITE)), // write variable to stack
            Val(ptr(1)), Op(FIXED(STK_READ)), Val(ptr(1)), Op(FIXED(ADD_PTR)), // return from function
            Op(FIXED(RCSTK_PTR)), Val(ptr(1)), Op(FIXED(SWAP)), Op(FIXED(STK_WRITE)), Instr(GOTO)]);
        assert_eq!(function_def, stack[..position]);
        let (function_call, position_2) 
            = generate_default_function_call(position, test_func_location);
        assert_eq!(function_call, stack[position..position_2]);
        assert_eq!(vec![Val(3.0), Op(FIXED(MUL))], stack[position_2..]);
    }

    // Checks that function parameters can be assigned to
    #[test]
    fn function_with_parameter_assigned() {
        let stack = compile_and_merge("fn test_func(a) {a = 3;} let b = test_func(4);");
        let (function_def, test_func_location, position) 
            = generate_function_def_precompiled(0, 
            vec![Val(ptr(1)), Op(FIXED(STK_READ)), Val(ptr(2)), Op(FIXED(SUB_PTR)), // get pointer to parameter
                    Val(3.0), Op(FIXED(STK_WRITE))]); // read parameter
        assert_eq!(function_def, stack[..position]);
        assert_eq!(Val(4.0), stack[position]);
        let position = position + 1;
        let (function_call, _) 
            = generate_function_call(position, test_func_location, 1);
        assert_eq!(function_call, stack[position..]);
    }

    // Checks calling the same function twice with differently typed parameters results in two seperate function calls
    #[test]
    fn function_with_multiple_dispatch() {
        let stack = compile_and_merge("fn test_func(a) {} let b = test_func(4); let c = test_func(&b);");
        let (function_def, test_func_location, position) 
            = generate_empty_function_definition(0);
        assert_eq!(function_def, stack[..position]);
        let (function_def, test_func_2_location, position_2) 
            = generate_empty_function_definition(position);
            assert_eq!(function_def, stack[position..position_2]);
        assert_eq!(Val(4.0), stack[position_2]);
        let position_3 = position_2 + 1;
        let (function_call, position_4) 
            = generate_function_call(position_3, test_func_location, 1);
        assert_eq!(function_call, stack[position_3..position_4]);
        let position_5 = position_4 + 4;
        assert_eq!(vec![Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR))], stack[position_4..position_5]);
        let (function_call, _) 
            = generate_function_call(position_5, test_func_2_location, 1);
        assert_eq!(function_call, stack[position_5..]);
    }

    // Tests that if and else work
    #[test]
    fn if_and_else() {
        let stack = compile_and_merge("if false {print 3;}");
        assert_eq!(vec![Val(0.0), Val(ptr(9)), Instr(GOTO_IF), Val(3.0), 
            Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC))], stack);

        let stack = compile_and_merge("if false {print 3;} else {print 4;}");
        assert_eq!(vec![Val(0.0), Val(ptr(11)), Instr(GOTO_IF), Val(3.0), Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC)),
        Val(ptr(15)), Instr(GOTO), Val(4.0), Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC))], stack);

        let stack = compile_and_merge("if false {print 3;} else if false {print 4;}");
        assert_eq!(vec![Val(0.0), Val(ptr(11)), Instr(GOTO_IF), Val(3.0), Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC)),
            Val(ptr(18)), Instr(GOTO), Val(0.0), Val(ptr(18)), Instr(GOTO_IF), 
            Val(4.0), Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC))], stack);

        let stack = compile_and_merge("if false {print 3;} else if false {print 4;} else {print 5;}");
        assert_eq!(vec![Val(0.0), Val(ptr(11)), Instr(GOTO_IF), Val(3.0), Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC)),
            Val(ptr(24)), Instr(GOTO), Val(0.0), Val(ptr(20)), Instr(GOTO_IF), Val(4.0), Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC)),
            Val(ptr(24)), Instr(GOTO), Val(5.0), Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC))], stack);
    }

    // Generates a variable call.
    // Takes the position of the variable.
    fn generate_variable_call(position: usize) -> Vec<MergedInstructions> {
        return vec![Val(ptr(position)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR)), Op(FIXED(STK_READ))]
    }

    // Generates a variable assignment.
    // Takes the position of the variable, and the assignment expression.
    fn generate_variable_assign(position: usize, expression: &str) -> Vec<MergedInstructions> {
        let compiled_expression = compile_and_merge(expression);
        let mut variable_assign = vec![Val(ptr(position)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR))];
        variable_assign.extend(compiled_expression);
        variable_assign.push(Op(FIXED(STK_WRITE)));
        return variable_assign
    }

    // Tests construct statements.
    #[test]
    fn construct() {
        let stack = compile_and_merge("let a = 3;");
        assert_eq!(vec![Val(3.0)], stack);
    }

    #[test]
    fn empty_construct() {
        let stack = compile_and_merge("let a: bool;");
        assert_eq!(vec![Val(0.0)], stack);
    }

    // Tests using a variable.
    #[test]
    fn use_variable() {
        let stack = compile_and_merge("let a = 3; let b = a;");
        assert_eq!(Val(3.0), stack[0]);
        assert_eq!(generate_variable_call(1), stack[1..]);
    }

    // Tests using a variable twice.
    #[test]
    fn use_variable_twice() {
        let stack = compile_and_merge("let a = 3; let b = a; let c = a;");
        assert_eq!(Val(3.0), stack[0]);
        assert_eq!(generate_variable_call(1), stack[1..6]);
        assert_eq!(generate_variable_call(1), stack[6..]);
    }

    // Tests using a second variable.
    #[test]
    fn double_construct_with_use() {
        let stack = compile_and_merge("let a = 3; let b = 4; let c = b;");
        assert_eq!(Val(3.0), stack[0]);
        assert_eq!(Val(4.0), stack[1]);
        assert_eq!(generate_variable_call(2), stack[2..]);
    }

    // Tests variable assignment
    #[test]
    fn variable_assignment() {
        let stack = compile_and_merge("let a = 3; a = 4;");
        assert_eq!(Val(3.0), stack[0]);
        assert_eq!(generate_variable_assign(1, "let a = 4;"), stack[1..]);
    }

    // Tests variable assignment for a second variable
    #[test]
    fn second_variable_assignment() {
        let stack = compile_and_merge("let a = 3; let b = 4; b = 5;");
        assert_eq!(Val(3.0), stack[0]);
        assert_eq!(Val(4.0), stack[1]);
        assert_eq!(generate_variable_assign(2, "let a = 5;"), stack[2..]);
    }

    // Tests print statement.
    #[test]
    fn print() {
        let stack = compile_and_merge("print 3;");
        assert_eq!(vec![Val(3.0), Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC))], stack);
    }

    // Tests while loop.
    #[test]
    fn while_loop() {
        let stack = compile_and_merge(
            "while 3 {print 4;}");
        assert_eq!(vec![
            Val(3.0), Val(ptr(11)), Instr(GOTO_IF), // loop exit condition
            Val(4.0), Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC)), // loop body
            Val(ptr(2)), Instr(GOTO) // loop restart
        ], stack);
    }

    // Tests for loop.
    #[test]
    fn for_loop() {
        let stack = compile_and_merge("for (let i = 4; 5; i = 6) {print 7;}");
        assert_eq!(vec![
            Val(4.0), // construction 
            Val(5.0), Val(ptr(18)), Instr(GOTO_IF), // loop exit condition 
            Val(7.0), Op(FIXED(PRINTFF)), Val(10.0), Op(FIXED(PRINTC)), // body
            Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR)), Val(6.0), Op(FIXED(STK_WRITE)), // assignment 
            Val(ptr(3)), Instr(GOTO), // restart loop 
            Op(FIXED(DROP)) // drop loop variable
        ], stack);
    }

    // Tests reading an external variable
    #[test]
    fn external_variable() {
        let mut env_vars = EnvironmentSymbolContext::new();
        env_vars.add_symbol("a".to_string(), 7, PrimitiveDataType::F64, "".to_string());
        let stack = compile_and_merge_with_env_vars("extern a; let b = a;", env_vars);
        assert_eq!(vec![Val(ptr(7)), Op(FIXED(LDNX))], stack);
    }

    // Tests reading an external variable with a single pointer (*) qualifier
    #[test]
    fn external_variable_with_qualifier() {
        let mut env_vars = EnvironmentSymbolContext::new();
        env_vars.add_symbol("a".to_string(), 7, PrimitiveDataType::F64, "*".to_string());
        let stack = compile_and_merge_with_env_vars("extern a; let b = a;", env_vars);
        assert_eq!(vec![Val(ptr(7)), Op(FIXED(LDNX)), Op(FIXED(READ))], stack);
    }

    // Tests reading an external variable with a double pointer (**) qualifier
    #[test]
    fn external_variable_with_double_qualifier() {
        let mut env_vars = EnvironmentSymbolContext::new();
        env_vars.add_symbol("a".to_string(), 7, PrimitiveDataType::F64, "**".to_string());
        let stack = compile_and_merge_with_env_vars("extern a; let b = a;", env_vars);
        assert_eq!(vec![Val(ptr(7)), Op(FIXED(LDNX)), Op(FIXED(PTR_DEREF)), Op(FIXED(READ))], stack);
    }

    // Tests writing to an external variable
    #[test]
    fn external_variable_write() {
        let mut env_vars = EnvironmentSymbolContext::new();
        env_vars.add_symbol("a".to_string(), 7, PrimitiveDataType::F64, "".to_string());
        let stack = compile_and_merge_with_env_vars("extern a; a = 4;", env_vars);
        assert_eq!(vec![Val(4.0), Val(ptr(7)), Op(FIXED(RCNX))], stack);
    }

    // Tests writing to an external variable with a single pointer (*) qualifier
    #[test]
    fn external_variable_write_with_qualifier() {
        let mut env_vars = EnvironmentSymbolContext::new();
        env_vars.add_symbol("a".to_string(), 7, PrimitiveDataType::F64, "*".to_string());
        let stack = compile_and_merge_with_env_vars("extern a; a = 4;", env_vars);
        assert_eq!(vec![Val(4.0), Val(ptr(7)), Op(FIXED(LDNX)), Op(FIXED(SWAP)), Op(FIXED(WRITE))], stack);
    }

    // Tests reading an external variable with a double pointer (**) qualifier
    #[test]
    fn external_variable_write_with_double_qualifier() {
        let mut env_vars = EnvironmentSymbolContext::new();
        env_vars.add_symbol("a".to_string(), 7, PrimitiveDataType::F64, "**".to_string());
        let stack = compile_and_merge_with_env_vars("extern a; a = 4;", env_vars);
        assert_eq!(vec![Val(4.0), Val(ptr(7)), Op(FIXED(LDNX)), Op(FIXED(PTR_DEREF)), Op(FIXED(SWAP)), Op(FIXED(WRITE))], stack);
    }

    // Tests for pointers
    #[test]
    fn reference() {
        let stack = compile_and_merge("let a = 3; let b = &a;");
        assert_eq!(vec![Val(3.0), Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR))], stack);
    }

    #[test]
    fn dereference() {
        let old_stack = compile_and_merge("let a = 3; let b = &a; let c = b;");
        let stack = compile_and_merge("let a = 3; let b = &a; let c = *b;");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(vec![Op(FIXED(STK_READ))], stack[old_stack.len()..]);
    }

    #[test]
    fn double_dereference() {
        let old_stack = compile_and_merge("let a = 3; let b = &a; let c = &b; let d = c;");
        let stack = compile_and_merge("let a = 3; let b = &a; let c = &b; let d = **c;");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(vec![Op(FIXED(STK_READ)), Op(FIXED(STK_READ))], stack[old_stack.len()..]);
    }

    #[test]
    fn pointer_assign() {
        let old_stack = compile_and_merge("let a = 3; let b = &a;");
        let stack = compile_and_merge("let a = 3; let b = &a; *b = 4;");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(generate_variable_call(2), stack[old_stack.len()..old_stack.len()+5]);
        assert_eq!(vec![Val(4.0), Op(FIXED(STK_WRITE))], stack[old_stack.len()+5..]);
    }

    #[test]
    fn triple_pointer_assign() {
        let old_stack = compile_and_merge("let a = 3; let b = &a; let c = &b; let d = &c;");
        let stack = compile_and_merge("let a = 3; let b = &a; let c = &b; let d = &c; ***d = 4;");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(generate_variable_call(4), stack[old_stack.len()..old_stack.len()+5]);
        assert_eq!(vec![Op(FIXED(STK_READ)), Op(FIXED(STK_READ)), Val(4.0), Op(FIXED(STK_WRITE))], stack[old_stack.len()+5..]);
    }

    //Check that parameters can also use pointer assign syntax
    #[test]
    fn parameter_pointer_assign() {
        let stack = compile_and_merge("fn test_func(a: *i64) {*a = 3;} let b = 1; let c = test_func(&b);");
        let (function_def, test_func_location, position) 
            = generate_function_def_precompiled(0, 
            vec![Val(ptr(1)), Op(FIXED(STK_READ)), Val(ptr(2)), Op(FIXED(SUB_PTR)), Op(FIXED(STK_READ)), // get pointer
                    Val(3.0), Op(FIXED(STK_WRITE))]); // write to pointer
        assert_eq!(function_def, stack[..position]);
        assert_eq!(vec![Val(1.0), Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR))], stack[position..position+5]);
        let position = position + 5;
        let (function_call, _) 
            = generate_function_call(position, test_func_location, 1);
        assert_eq!(function_call, stack[position..]);
    }

    // Tests for arrays
    #[test]
    fn create_array() {
        let stack = compile_and_merge("let a = [1];");
        assert_eq!(vec![Val(ptr(0)), Val(ptr(0)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Val(1.0), Op(FIXED(WRITE)), Val(ptr(0))], stack);
    }

    #[test]
    fn create_empty_array() {
        let stack = compile_and_merge("let a: [i64; 1];");
        assert_eq!(vec![Val(ptr(0))], stack);
    }

    #[test]
    fn create_long_array() {
        let stack = compile_and_merge("let a = [1,2,3,4,5,6,7,8,9,10];");
        let stack_length = 6;
        let array_elements = 10;
        for i in 0..array_elements {
            let start = i * stack_length;
            let end = (i+1) * stack_length;
            assert_eq!(vec![Val(ptr(0)), Val(ptr(i)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Val((i+1) as f64), Op(FIXED(WRITE))], stack[start..end]);
        }
        assert_eq!(Val(ptr(0)), stack[stack.len()-1]);
        assert_eq!(stack.len(), stack_length * array_elements + 1);
    }

    #[test]
    fn create_three_arrays() {
        let stack = compile_and_merge("let a = [1]; let b = [2]; let c = [3];");
        let stack_length = 7;
        let array_elements = 3;
        for i in 0..array_elements {
            let start = i * stack_length;
            let end = (i+1) * stack_length;
            assert_eq!(vec![Val(ptr(i)), Val(ptr(0)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Val((i+1) as f64), Op(FIXED(WRITE)), Val(ptr(i))], stack[start..end]);
        }
        assert_eq!(stack.len(), stack_length * array_elements);
    }

    #[test]
    fn create_three_long_arrays() {
        let stack = compile_and_merge("let a = [1,2,3]; let b = [4,5,6]; let c = [7,8,9];");
        let stack_length = 6;
        let array_number = 3;
        let array_length = 3;
        for i in 0..array_number {
            for j in 0..array_length {
                let start = (3*i+j) * stack_length + i;
                let end = (3*i+j+1) * stack_length + i;
                assert_eq!(vec![Val(ptr(3*i)), Val(ptr(j)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Val((3*i+j+1) as f64), Op(FIXED(WRITE))], stack[start..end]);
            }
            assert_eq!(Val(ptr(i*3)), stack[(3*(i+1)) * stack_length + i]);
        }
        assert_eq!(stack.len(), stack_length * array_number * array_length + array_number);
    }

    #[test]
    fn create_2d_array() {
        let stack = compile_and_merge("let a = [[1]];");
        assert_eq!(vec![Val(ptr(0)), Val(ptr(0)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Val(1.0), Op(FIXED(WRITE)), Val(ptr(0))], stack);
    }

    #[test]
    fn create_large_2d_array() {
        let stack = compile_and_merge("let a = [[1,2,3],[4,5,6],[7,8,9]];");
        let stack_length = 6;
        let array_elements = 9;
        for i in 0..array_elements {
            let start = i * stack_length;
            let end = (i+1) * stack_length;
            assert_eq!(vec![Val(ptr(0)), Val(ptr(i)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Val((i+1) as f64), Op(FIXED(WRITE))], stack[start..end]);
        }
        assert_eq!(Val(ptr(0)), stack[stack.len()-1]);
        assert_eq!(stack.len(), stack_length * array_elements + 1);
    }

    #[test]
    fn create_deep_2d_array() {
        let stack = compile_and_merge("let a = [[[[1,2], [3,4]],[[5,6], [7,8]]],[[[9,10], [11,12]],[[13,14], [15,16]]]];");
        let stack_length = 6;
        let array_elements = 16;
        for i in 0..array_elements {
            let start = i * stack_length;
            let end = (i+1) * stack_length;
            assert_eq!(vec![Val(ptr(0)), Val(ptr(i)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Val((i+1) as f64), Op(FIXED(WRITE))], stack[start..end]);
        }
        assert_eq!(Val(ptr(0)), stack[stack.len()-1]);
        assert_eq!(stack.len(), stack_length * array_elements + 1);
    }

    #[test]
    fn array_access() {
        let old_stack = compile_and_merge("let a = [1];");
        let stack = compile_and_merge("let a = [1]; let b = a[0];");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(vec![Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR)), Op(FIXED(STK_READ)), 
            Val(0.0), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Op(FIXED(READ))], stack[old_stack.len()..stack.len()]);
    }

    #[test]
    fn deeper_array_access() {
        let old_stack = compile_and_merge("let a = [1,2]; let b = [3,4,5];");
        let stack = compile_and_merge("let a = [1,2]; let b = [3,4,5]; let c = b[1];");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(vec![Val(ptr(2)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR)), Op(FIXED(STK_READ)), 
        Val(1.0), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Op(FIXED(READ))], stack[old_stack.len()..stack.len()]);
    }

    #[test]
    fn complex_array_access() {
        let old_stack = compile_and_merge("let a = [1];");
        let stack = compile_and_merge("let a = [1]; let b = a[3 - 3];");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(vec![Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR)), Op(FIXED(STK_READ)), 
        Val(3.0), Val(3.0), Op(FIXED(SUB)), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Op(FIXED(READ))], stack[old_stack.len()..stack.len()]);
    }

    #[test]
    fn multidimensional_array_access() {
        let old_stack = compile_and_merge("let a = [[1]];");
        let stack = compile_and_merge("let a = [[1]]; let b = a[0][0];");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(vec![Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR)), Op(FIXED(STK_READ)),
                        Val(0.0), Val(1.0), Op(FIXED(MUL)), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), // Enter first level
                        Val(0.0), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Op(FIXED(READ))], // Enter second level
            stack[old_stack.len()..stack.len()]);
    }

    #[test]
    fn large_multidimensional_array_access() {
        let old_stack = compile_and_merge("let a = [[[[1,2], [3,4]],[[5,6], [7,8]]],[[[9,10], [11,12]],[[13,14], [15,16]]]];");
        let stack = compile_and_merge("
            let a = [[[[1,2], [3,4]],[[5,6], [7,8]]],[[[9,10], [11,12]],[[13,14], [15,16]]]];
            let b = a[1][0][1][0];
        ");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(vec![Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR)), Op(FIXED(STK_READ)),
            Val(1.0), Val(8.0), Op(FIXED(MUL)), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), 
            Val(0.0), Val(4.0), Op(FIXED(MUL)), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), 
            Val(1.0), Val(2.0), Op(FIXED(MUL)), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), 
            Val(0.0), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Op(FIXED(READ))], stack[old_stack.len()..stack.len()]);
    }

    // Tests for arrays
    #[test]
    fn assign_array() {
        let old_stack = compile_and_merge("let a = [1];");
        let stack = compile_and_merge("let a = [1]; a[0] = 2;");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(vec![Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR)), Op(FIXED(STK_READ)),
            Val(ptr(0)), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Val(2.0), Op(FIXED(WRITE))], stack[old_stack.len()..stack.len()]);
    }

    // Tests for arrays
    #[test]
    fn assign_2d_array() {
        let old_stack = compile_and_merge("let a = [[1]];");
        let stack = compile_and_merge("let a = [[1]]; a[0][0] = 2;");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(vec![Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR)), Op(FIXED(STK_READ)),
            Val(ptr(0)), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), 
            Val(ptr(0)), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Val(2.0), Op(FIXED(WRITE))], stack[old_stack.len()..stack.len()]);
    }

    // Tests for arrays
    #[test]
    fn assign_2d_array_half() {
        let old_stack = compile_and_merge("let a = [[1]];");
        let stack = compile_and_merge("let a = [[1]]; a[0] = [2];");

        assert_eq!(old_stack, stack[..old_stack.len()]);
        assert_eq!(vec![Val(ptr(1)), Val(ptr(1)), Op(FIXED(STK_READ)), Op(FIXED(ADD_PTR)), Op(FIXED(STK_READ)), Val(ptr(0)), Op(FIXED(DOUBLETOLONGLONG)), Op(FIXED(ADD_PTR)), 
            Op(FIXED(DUP)), Val(ptr(0)), Op(FIXED(ADD_PTR)), Op(FIXED(LDNXPTR)), Val(2.0), Op(FIXED(WRITE))], stack[old_stack.len()..stack.len()]);
    }

    // Tests for the type system.
    #[test]
    fn literal_types() {
        let datatypes = vec![
            "f128",
            "f64",
            "f32",
            "f16",
            "f8",
            "i128",
            "i64",
            "i32",
            "i16",
            "i8",
            "bool",
        ];
        for datatype in datatypes {
            compile_and_assert_equal(&format!("let a = 0;"), &format!("let a : {} = 0;", datatype));
        }
    }

    #[test]
    fn binary_operator_types() {
        compile_and_assert_equal("let a = 0; let b = 1; let c = a+b;", "let a:i8 = 0; let b:i16 = 1; let c:i32 = a+b;"); //currently, all literal types are equal.
    }

    #[test]
    fn binary_operator_equality_types() {
        compile_and_assert_equal("let a = [1]; let b = [2]; let c = a==b;", "let a: [i64; 1] = [1]; let b: [i64; 1] = [2]; let c: bool = a==b;");
    }

    #[test]
    fn unary_operator_types() {
        compile_and_assert_equal("let a = 0; let b = -a;", "let a = 0; let b: i64 = -a;");
    }

    #[test]
    fn builtin_function_types() {
        let functions = vec![ACOS,ACOSH,ASIN,ASINH,ATAN,ATAN2,ATANH,CBRT,CEIL,CPYSGN,COS,COSH,
        COSPI,BESI0,BESI1,ERF,ERFC,ERFCI,ERFCX,ERFI,EXP,EXP10,EXP2,EXPM1,FABS,FDIM,FLOOR,FMA,FMAX,FMIN,FMOD,FREXP,HYPOT,
        ILOGB,ISFIN,ISINF,ISNAN,BESJ0,BESJ1,BESJN,LDEXP,LGAMMA,LLRINT,LLROUND,LOG,LOG10,LOG1P,LOG2,LOGB,LRINT,LROUND,MAX,MIN,MODF,
        NXTAFT,POW,RCBRT,REM,REMQUO,RHYPOT,RINT,ROUND,
        RSQRT,SCALBLN,SCALBN,SGNBIT,SIN,SINH,SINPI,SQRT,TAN,TANH,TGAMMA,TRUNC,BESY0,BESY1,BESYN];
        for function in &functions {
            let input = vec!["3"; function.consume() as usize].join(",");
            let text = &format!("let a = __{}({});", function.to_string().to_lowercase(), input);
            let typed_text = &format!("let a: f64 = __{}({});", function.to_string().to_lowercase(), input);
            compile_and_assert_equal(text, typed_text);
        }
    }

    #[test]
    fn parentheses_types() {
        compile_and_assert_equal("let a = ((1+2)/(3-4));", "let a: i64 = ((1+2)/(3-4));");
    }

    #[test]
    fn empty_function_type() {
        compile_and_assert_equal("fn func() {} let a = func();", "fn func() {} let a: none = func();");
    }

    #[test]
    fn empty_function_return_type() {
        compile_and_assert_equal("fn func() {} let a = func();", "fn func() -> none {} let a = func();");
    }

    #[test]
    fn type_in_function() {
        compile_and_assert_equal("fn func() {let a = [1,2,3];} func();", "fn func() {let a: [i64; 3] = [1,2,3];} func();");
    }

    #[test]
    fn parameter_type() {
        compile_and_assert_equal("fn func(a) {a = 2;} func(1);", "fn func(a: i64) {a = 2;} func(1);");
    }

    #[test]
    fn return_type() {
        compile_and_assert_equal("fn func() {return 3;} let a = func();", "fn func() -> i64 {return 3;} let a: i64 = func();");
    }

    #[test]
    fn multiple_dispatch_type() {
        compile_and_assert_equal(
            "fn func(a) {
                return a;
            } 
            let a = 3;
            let b = func(a);
            let c = func(&a);", 
            "fn func(a) {
                return a;
            }
            let a = 3;
            let b: i64 = func(a);
            let c: *i64 = func(&a);");
    }

    #[test]
    fn variable_assignment_type() {
        compile_and_assert_equal("let a = 3; a = 4;", "let a: i64 = 3; a = 4;");
    }

    #[test]
    fn while_loop_type() {
        compile_and_assert_equal("let a = 3; while true { a = 4; }", "let a: i64 = 3; while true { a = 4; }");
    }

    #[test]
    fn for_loop_type() {
        compile_and_assert_equal(
            "let a = 3; for (let i = 4; i < 50; i = i + 1) { a = a + 1; }", 
            "let a: i64 = 3; for (let i: i8 = 4; i < 50; i = i + 1) { a = a + 1; }");
    }

    #[test]
    fn external_variable_type() {
        let mut env_vars = EnvironmentSymbolContext::new();
        env_vars.add_symbol("a".to_string(), 7, PrimitiveDataType::F64, "".to_string());
        let stack = compile_and_merge_with_env_vars("extern a; let b = a;", env_vars);
        let mut env_vars = EnvironmentSymbolContext::new();
        env_vars.add_symbol("a".to_string(), 7, PrimitiveDataType::F64, "".to_string());
        let typed_stack = compile_and_merge_with_env_vars("extern a; let b: i64 = a;", env_vars);
        assert_eq!(stack, typed_stack);
    }

    #[test]
    fn external_variable_type_with_qualifier() {
        let mut env_vars = EnvironmentSymbolContext::new();
        env_vars.add_symbol("a".to_string(), 7, PrimitiveDataType::F64, "*".to_string());
        let stack = compile_and_merge_with_env_vars("extern a; let b = a;", env_vars);
        let mut env_vars = EnvironmentSymbolContext::new();
        env_vars.add_symbol("a".to_string(), 7, PrimitiveDataType::F64, "*".to_string());
        let typed_stack = compile_and_merge_with_env_vars("extern a; let b: i64 = a;", env_vars);
        assert_eq!(stack, typed_stack);
    }

    #[test]
    fn reference_type() {
        compile_and_assert_equal("let a = 3; let b = &a;", "let a: i64 = 3; let b: *i64 = &a;");
    }

    #[test]
    fn dereference_type() {
        compile_and_assert_equal("let a = 3; let b = &a; let c = *b;", "let a: i64 = 3; let b: *i64 = &a; let c: i64 = *b;");
    }

    #[test]
    fn double_dereference_type() {
        compile_and_assert_equal(
            "let a = 3; let b = &a; let c = &b; let d = **c;", 
            "let a: i64 = 3; let b: *i64 = &a; let c: **i64 = &b; let d: i64 = **c;");
    }

    #[test]
    fn pointer_assign_type() {
        compile_and_assert_equal("let a = 3; let b = &a; *b = 4;", "let a: i64 = 3; let b: *i64 = &a; *b = 4;");
    }

    #[test]
    fn array_type() {
        compile_and_assert_equal("let a = [1.0, 2.0];", "let a: [f64; 2] = [1.0, 2.0];");
    }

    #[test]
    fn array_type_2d() {
        compile_and_assert_equal("let a = [[1,2,3],[4,5,6],[7,8,9]];", "let a: [[i64; 3]; 3] = [[1,2,3],[4,5,6],[7,8,9]];");
    }

    #[test]
    fn array_access_type() {
        compile_and_assert_equal("let a = [1]; let b = a[0];", "let a: [i64; 1] = [1]; let b: i64 = a[0];");
    }

    #[test]
    fn array_access_type_2d() {
        compile_and_assert_equal("let a = [[1]]; let b = a[0][0];", "let a: [[i64; 1]; 1] = [[1]]; let b: i64 = a[0][0];");
    }

    #[test]
    fn array_assign_type() {
        compile_and_assert_equal("let a = [1]; a[0] = 2;", "let a: [i64; 1] = [1]; a[0] = 2;");
    }

    #[test]
    fn array_assign_type_2d() {
        compile_and_assert_equal("let a = [[1]]; a[0][0] = 2;", "let a: [[i64; 1]; 1] = [[1]]; a[0][0] = 2;");
    }

    #[test]
    fn array_assign_type_2d_half() {
        compile_and_assert_equal("let a = [[1]]; a[0] = [2];", "let a: [[i64; 1]; 1] = [[1]]; a[0] = [2];");
    }

}