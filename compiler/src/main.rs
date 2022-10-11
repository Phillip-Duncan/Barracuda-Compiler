// External Modules
extern crate pest;
extern crate exitcode;
#[macro_use]
extern crate pest_derive;
extern crate safer_ffi;


// Internal Modules
mod compiler;
use compiler::Compiler;
use compiler::EnvironmentSymbolContext;

use barracuda_common::CLIEnvVarDescriptor;

// Standard Imports
use clap::Parser;

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

    // Configuration

    /// Environment variables definitions space separated identifiers
    /// Syntax: identifier(:address)?
    #[clap(long, multiple = true)]
    env: Option<Vec<CLIEnvVarDescriptor>>,

    // Flags

    /// Write compilation result to stdout instead of output
    #[clap(long, action)]
    stdout: bool,

    /// Generates code with debug decorations
    #[clap(long, action)]
    debug: bool
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

        return self;
    }

    /// Generate EnvironmentSymbolContext from CLI arguments.
    /// If addresses were not specified from the args they will be linearly
    /// set. eg 0, 1, 2. Note this does not check for conflict with user specified addresses
    /// for instance '--env a:1 b c' will assign a:1, b:0, c:1
    /// @return EnvironmentSymbolContext created from '--env' args
    fn get_environment_variables(&self) -> EnvironmentSymbolContext {
        let mut context =  EnvironmentSymbolContext::new();

        if let Some(env_vars) = &self.env {
            let mut next_id = 0;
            for env_var_descriptor in env_vars {
                let address = match env_var_descriptor.given_address {
                    Some(id) => id,
                    None => {
                        next_id += 1;
                        next_id - 1
                    }
                };
                let identifier = env_var_descriptor.identifier.clone();

                context.add_symbol(identifier, address);
            }
        }

        return context;
    }
}

fn main() {
    // Parse Command line arguments
    let cli_args = CompilerCLIOptions::parse().derive_defaults();

    let compiler: Compiler<PARSER, GENERATOR> = Compiler::default()
        .set_environment_variables(cli_args.get_environment_variables());
    let source_path = cli_args.path.as_path();

    // Check if output should be to stdout
    let result = if cli_args.stdout {
        match compiler.compile(source_path) {
            Ok(program_code) => {
                if cli_args.debug {
                    print!("{}", program_code.decorated());
                } else {
                    print!("{}", program_code);
                }
                Ok(())
            }
            Err(result) => { Err(result) }
        }
    } else {
        let dest_path = cli_args.output.unwrap(); // Can unwrap as output will always be derived
        let dest_path = dest_path.as_path();
        compiler.compile_and_save(source_path, dest_path, cli_args.debug)
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
