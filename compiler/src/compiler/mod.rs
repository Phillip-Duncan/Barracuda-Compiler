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