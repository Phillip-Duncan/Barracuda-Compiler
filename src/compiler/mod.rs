mod ast;
pub mod backend;
pub mod parser;

use std::process::Output;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::error::Error;
use std::ops::Deref;

// Interface Definitions
use self::ast::AbstractSyntaxTree;
use self::parser::AstParser;
use self::backend::BackEndGenerator;

// Concrete Definitions Re-Export
pub use self::backend::BarracudaByteCodeGenerator;
pub use self::parser::PestBarracudaParser;


pub(crate) struct Compiler<P: AstParser, G: BackEndGenerator> {
    parser: P,
    generator: G
}

impl<P: AstParser, G: BackEndGenerator> Compiler<P, G> {
    pub(crate) fn default() -> Self {
        Compiler {
            parser: P::default(),
            generator: G::default()
        }
    }

    pub(crate) fn new(parser: P, generator: G) -> Self {
        Compiler {
            parser,
            generator
        }
    }

    pub(crate) fn compile_str(self, source: &str) -> String {
        let ast = self.parser.parse(source);
        let token_list = self.generator.generate(ast);
        let repr_list: Vec<String> = token_list.into_iter()
            .map(|token | token.repr())
            .rev().collect();
        let output_concat_repr = repr_list.join("\n");

        return output_concat_repr
    }

    pub(crate) fn compile(self, source_filename: &Path) -> Result<String, Box<dyn Error>> {
        let source_str = fs::read_to_string(source_filename)?;
        Ok(self.compile_str(source_str.as_str()))
    }

    pub(crate) fn compile_and_save(self, source_filename: &Path, dest_filename: &Path) -> Result<(), Box<dyn Error>> {
        let compiled_str = self.compile(source_filename)?;

        let display = dest_filename.display();

        let mut file = match File::create(&dest_filename) {
            Err(why) => panic!("Couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        match file.write_all(compiled_str.as_bytes()) {
            Err(why) => panic!("Couldn't write to {}: {}", display, why),
            Ok(_) => println!("Successfully wrote to {}", display),
        };

        Ok(())
    }
}

