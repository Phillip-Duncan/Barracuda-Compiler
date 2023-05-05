mod ast;
pub mod backend;
pub mod parser;
use barracuda_common;

use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::error::Error;

// Interface Definitions
use self::parser::AstParser;
use self::backend::BackEndGenerator;
use barracuda_common::ProgramCode;

// Concrete Definitions Re-Export
pub use self::backend::BarracudaByteCodeGenerator;
pub use self::parser::PestBarracudaParser;
pub use self::ast::EnvironmentSymbolContext;
pub use self::ast::datatype::PrimitiveDataType;


/// Compiler is a simple class that holds the configuration of a compilation configuration.
/// Compiler takes two typed parameters defining the AstParser being used as well as the
/// BackEndGenerator.
///
/// # Compilation Diagram
/// barracuda_code -> AstParser -> AbstractSyntaxTree -> BackEndGenerator -> ProgramCode
pub struct Compiler<P: AstParser, G: BackEndGenerator> {
    parser: P,
    generator: G,
    env_vars: EnvironmentSymbolContext
}

#[allow(dead_code)] // Many of the functions on compiler act as a library interface and are not used
impl<P: AstParser, G: BackEndGenerator> Compiler<P, G> {

    /// Default generates a default compiler configuration. Default configuration is determined by
    /// the default methods of the parser and generator.
    pub fn default() -> Self {
        Compiler {
            parser: P::default(),
            generator: G::default(),
            env_vars: EnvironmentSymbolContext::new()
        }
    }

    /// Create new compiler using a preconfigured parser and generator.
    pub fn new(parser: P, generator: G, env_vars: EnvironmentSymbolContext) -> Self {
        Compiler {
            parser,
            generator,
            env_vars
        }
    }

    pub fn set_environment_variables(mut self, env_vars: EnvironmentSymbolContext) -> Self {
        self.env_vars = env_vars;
        return self
    }

    /// Compiles a string representing an interpretable language by the parser into program code.
    pub fn compile_str(self, source: &str) -> ProgramCode {
        let ast = self.parser.parse(source, self.env_vars.clone());
        let program_code = self.generator.generate(ast);

        return program_code
    }

    /// Compiles a program file containing an interpretable language by the parser into program code.
    /// @return: ProgramCode if Ok. Otherwise IO Error from a failed read.
    pub fn compile(self, source_filename: &Path) -> Result<ProgramCode, Box<dyn Error>> {
        let source_str = fs::read_to_string(source_filename)?;

        Ok(self.compile_str(source_str.as_str()))
    }

    /// Compiles a program file and writes program code encoded as string into the destination file
    /// path.
    /// @return: ProgramCode if Ok. Otherwise IO Error from a failed read/write.
    pub fn compile_and_save(self, source_filename: &Path, dest_filename: &Path, decorated: bool) -> Result<(), Box<dyn Error>> {
        let mut compiled_program = self.compile(source_filename)?;
        if decorated {
            compiled_program = compiled_program.decorated();
        }

        let program_str = format!("{}", compiled_program);

        let display_dest = dest_filename.display();

        let mut file = match File::create(&dest_filename) {
            Err(why) => panic!("Couldn't create {}: {}", display_dest, why),
            Ok(file) => file,
        };

        match file.write_all(program_str.as_bytes()) {
            Err(why) => panic!("Couldn't write to {}: {}", display_dest, why),
            Ok(_) => println!("Successfully wrote to {}", display_dest),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    
    use barracuda_common:: {
        ProgramCode,
        BarracudaOperators::*,
        BarracudaOperators,
        FixedBarracudaOperators::*,
        VariableBarracudaOperators::*,
        BarracudaInstructions::*,
    };
    use super::Compiler;

    type PARSER = super::PestBarracudaParser;
    type GENERATOR = super::BarracudaByteCodeGenerator;
    
    // Type to represent values and instructions on one stack for easier testing.
    #[derive(Debug, PartialEq, Clone)]
    enum MergedInstructions {
        Val(f64),
        Op(BarracudaOperators),
    }
    use MergedInstructions::*;

    fn ptr(int: u64) -> f64 {
        f64::from_ne_bytes(int.to_ne_bytes())
    }

    fn compile_and_merge(text: &str) -> Vec<MergedInstructions> {
        let compiler: Compiler<PARSER, GENERATOR> = Compiler::default();
        let code = compiler.compile_str(text);
        // iterates through values, operations, and instructions at once
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
                _ => assert!(false)
            }
        }
        assert_eq!([Val(0.0), Val(ptr(1))], out[..2]);
        out[2..].to_vec()
    }

    #[test]
    fn test_literals() {
        let literals = vec![
            // Integers
            ("0", 0.0),
            ("1", 1.0),
            ("3545", 3545.0),
            ("9007199254740991", 9007199254740991.0), // Maximum safe integer    
            // Floats
            ("0.0", 0.0),
            ("1.0", 1.0),
            ("3545.0", 3545.0),
            ("1000000000000000000000000000000000000000000000000.0", 1000000000000000000000000000000000000000000000000.0),
            ("1.0e1", 10.0),
            ("1.0e+1", 10.0),
            ("1.0e-1", 0.1),
            ("1.0e3", 1000.0),
            ("1.0e+3", 1000.0),
            ("1.0e-3", 0.001),
            ("1.0e0", 1.0),
            ("1.0e+0", 1.0),
            ("1.0e-0", 1.0),
            ("1.7976931348623157e308", f64::MAX), // Maximum float
            // Booleans
            ("false", 0.0),
            ("true", 1.0),
        ];
        for (text, value) in &literals {
            let stack = compile_and_merge(&format!("{};", text));
            assert_eq!(vec![Val(*value)], stack);
        }
    }

    #[test]
    fn test_binary_operators() {
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

    #[test]
    fn test_unary_operators() {
        let unary_operators = vec![
            ("!", NOT),
            ("-", NEGATE),     
        ];
        for (text, op) in &unary_operators {
            let stack = compile_and_merge(&format!("{}4;", text));
            assert_eq!(vec![Val(4.0), Op(FIXED(*op))], stack);
        }
    }


}