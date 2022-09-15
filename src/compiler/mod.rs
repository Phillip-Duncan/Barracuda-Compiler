mod ast;
pub mod backend;
pub mod parser;
pub mod program_code;

use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::error::Error;

// Interface Definitions
use self::parser::AstParser;
use self::backend::BackEndGenerator;
use self::program_code::ProgramCode;

// Concrete Definitions Re-Export
pub use self::backend::BarracudaByteCodeGenerator;
pub use self::parser::PestBarracudaParser;



pub struct Compiler<P: AstParser, G: BackEndGenerator> {
    parser: P,
    generator: G
}

#[allow(dead_code)] // Many of the functions on compiler act as a library interface and are not used
impl<P: AstParser, G: BackEndGenerator> Compiler<P, G> {
    pub fn default() -> Self {
        Compiler {
            parser: P::default(),
            generator: G::default()
        }
    }

    pub fn new(parser: P, generator: G) -> Self {
        Compiler {
            parser,
            generator
        }
    }

    pub fn compile_str(self, source: &str) -> ProgramCode {
        let ast = self.parser.parse(source);
        let program_code = self.generator.generate(ast);

        return program_code
    }

    pub fn compile(self, source_filename: &Path) -> Result<ProgramCode, Box<dyn Error>> {
        let source_str = fs::read_to_string(source_filename)?;
        Ok(self.compile_str(source_str.as_str()))
    }

    pub fn compile_and_save(self, source_filename: &Path, dest_filename: &Path) -> Result<(), Box<dyn Error>> {
        let compiled_program = self.compile(source_filename)?;
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

