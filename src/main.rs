// External Modules
extern crate pest;
#[macro_use]
extern crate pest_derive;

// Internal Modules
mod compiler;
use compiler::Compiler;

use std::path::Path;
use std::error::Error;

type PARSER = compiler::PestBarracudaParser;
type GENERATOR = compiler::BarracudaByteCodeGenerator;

fn main() {
    let compiler: Compiler<PARSER, GENERATOR> = Compiler::default();
    let source_path = Path::new("examples/fib.bc");
    let dest_path = Path::new("output.txt");

    match compiler.compile_and_save(source_path, dest_path) {
        Ok(_) => println!("Compile success!"),
        Err(why) => println!("Compile Error: {:?}", why)
    };

}
