// External Modules
extern crate pest;
extern crate exitcode;
#[macro_use]
extern crate pest_derive;


// Internal Modules
mod compiler;
use compiler::Compiler;

// Standard Imports
use std::path::Path;
use clap::Parser;
use std::error::Error;

// Basic Compiler Configuration
type PARSER = compiler::PestBarracudaParser;
type GENERATOR = compiler::BarracudaByteCodeGenerator;


/// Command Line interface struct
/// Describes possible arguments using the clap library
#[derive(Parser)]
struct CompilerCLIOptions {
    /// Path of file to compile. Barracuda source files end in .bc
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,

    /// Path to output file, default is <path_filename>.bct
    #[clap(short, long, parse(from_os_str))]
    output: Option<std::path::PathBuf>,

    // Flags

    /// Write compilation result to stdout instead of output
    #[clap(long, action)]
    stdout: bool
}

impl CompilerCLIOptions {
    /// Derives default values for empty arguments that cannot be set to constants.
    /// For instance output is derived from the input file path.
    /// @return: Returns CompilerCLIOptions with modified empty arguments
    fn derive_defaults(mut self) -> Self {
        // Derive output file path from input file path if not set
        if self.output.is_none() {
            self.output = Some(self.path.with_extension("bct"))
        }
        return self
    }
}

fn main() {
    // Parse Command line arguments
    let cli_args = CompilerCLIOptions::parse().derive_defaults();

    let compiler: Compiler<PARSER, GENERATOR> = Compiler::default();
    let source_path = cli_args.path.as_path();

    // Check if output should be to stdout
    let result = if cli_args.stdout {
        match compiler.compile(source_path) {
            Ok(program_code) => {
                print!("{}", program_code);
                Ok(())
            }
            Err(result) => { Err(result) }
        }
    } else {
        let dest_path = cli_args.output.unwrap(); // Can unwrap as output will always be derived
        let dest_path = dest_path.as_path();
        compiler.compile_and_save(source_path, dest_path)
    };

    // Check result
    match result {
        Ok(_) => {
            if !cli_args.stdout { // Don't pollute stdout if it has been selected
                println!("Compile success!");
            }
            std::process::exit(exitcode::OK);
        },
        Err(why) => {
            // TODO(Connor): Differentiate between a compilation error and an internal error
            println!("Compile Error: {:?}", why);
            std::process::exit(exitcode::SOFTWARE);
        }
    };

}
