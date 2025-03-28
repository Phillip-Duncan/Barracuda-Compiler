mod ast;
pub mod backend;
pub mod parser;
pub mod semantic_analyser;
pub mod utils;
use barracuda_common;

use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::error::Error;

// Interface Definitions
use self::parser::AstParser;
use self::backend::BackEndGenerator;
use self::semantic_analyser::SemanticAnalyser;
use barracuda_common::ProgramCode;

// Concrete Definitions Re-Export
pub use self::backend::BarracudaByteCodeGenerator;
pub use self::parser::PestBarracudaParser;
pub use self::semantic_analyser::BarracudaSemanticAnalyser;
pub use self::ast::EnvironmentSymbolContext;
pub use self::ast::datatype::PrimitiveDataType;
pub use self::ast::qualifiers::Qualifier;


/// Compiler is a simple class that holds the configuration of a compilation configuration.
/// Compiler takes three typed parameters defining:
///  the AstParser being used,
///  the SemanticAnalyser being used,
///  and the BackEndGenerator being used.
///
/// # Compilation Diagram
/// barracuda_code -> AstParser -> AbstractSyntaxTree -> SemanticAnalyser -> Annotated AbstractSyntaxTree -> BackEndGenerator -> ProgramCode
pub struct Compiler<P: AstParser, A: SemanticAnalyser, G: BackEndGenerator> {
    parser: P,
    semantic_analyser: A,
    generator: G,
    env_vars: EnvironmentSymbolContext,
    precision: usize
}

#[allow(dead_code)] // Many of the functions on compiler act as a library interface and are not used
impl<P: AstParser, A: SemanticAnalyser, G: BackEndGenerator> Compiler<P, A, G> {

    /// Default generates a default compiler configuration. Default configuration is determined by
    /// the default methods of the parser and generator.
    pub fn default() -> Self {
        Compiler {
            parser: P::default(),
            semantic_analyser: A::default(),
            generator: G::default(),
            env_vars: EnvironmentSymbolContext::new(),
            precision: 32
        }
    }

    /// Create new compiler using a preconfigured parser and generator.
    pub fn new(parser: P, semantic_analyser: A, generator: G, env_vars: EnvironmentSymbolContext, precision: usize) -> Self {
        Compiler {
            parser,
            semantic_analyser,
            generator,
            env_vars,
            precision,
        }
    }

    pub fn set_environment_variables(mut self, env_vars: EnvironmentSymbolContext) -> Self {
        self.env_vars = env_vars;
        return self
    }

    pub fn set_environment_variable_count(mut self, env_var_count: usize) -> Self {
        for _ in 0..env_var_count {
            self.generator.add_environment_variable();
        }
        return self
    }

    pub fn set_precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self.generator.set_precision(precision);
        return self
    }

    /// Compiles a string representing an interpretable language by the parser into program code.
    pub fn compile_str(self, source: &str) -> ProgramCode {
        let ast = self.parser.parse(source, self.precision);
        let annotated_ast = self.semantic_analyser.analyse(ast, self.env_vars);
        let program_code = self.generator.generate(annotated_ast);

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