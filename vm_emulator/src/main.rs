extern crate core;
extern crate statrs;
extern crate float_next_after;
#[macro_use]
extern crate approx;
extern crate crossterm;
extern crate tui;
extern crate num_traits;

#[macro_use]
extern crate derive_getters;
extern crate scilib;
extern crate endiannezz;
extern crate anyhow;

mod emulator;
mod test;
mod visualiser;

use barracuda_common::{
    ProgramCodeParser,
    BarracudaCodeTextParser,
    CLIEnvVarDescriptor
};

use clap::Parser;
use emulator::ThreadContext;
use std::io;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::fs::File;
use anyhow::{Context, Result};
use crate::emulator::EnvironmentVariable;


/// Command Line interface struct
/// Describes possible arguments using the clap library
#[derive(Parser)]
struct Cli {
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,

    #[clap(short, long, default_value_t = 256)]
    stack_size: usize,

    #[clap(short, long)]
    debug: bool,

    /// Environment variables definitions space separated identifiers
    /// Syntax: identifier(:address)?
    #[clap(long, multiple = true)]
    env: Option<Vec<CLIEnvVarDescriptor>>,
}

impl Cli {
    /// Generate EnvironmentSymbolContext from CLI arguments.
    /// If addresses were not specified from the args they will be linearly
    /// set. eg 0, 1, 2. Note this does not check for conflict with user specified addresses
    /// for instance '--env a:1 b c' will assign a:1, b:0, c:1
    /// @return EnvironmentSymbolContext created from '--env' args
    fn get_environment_variables(&self) -> HashMap<usize, EnvironmentVariable> {
        let mut context = HashMap::new();

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
                let value = env_var_descriptor.given_value.unwrap_or(0.0);

                context.insert(address, EnvironmentVariable::new(
                    identifier,
                    address,
                    value
                ));
            }
        }

        return context;
    }
}

fn main()  -> Result<()> {
    let args = Cli::parse();

    let path = args.path.as_path();
    let parser:Box<dyn ProgramCodeParser> = Box::new(BarracudaCodeTextParser::new());

    let file = File::open(path).with_context(|| format!("Could not open file {:?}", &path))?;
    let code = parser.parse(file)
                            .with_context(|| format!("Could not parse file into program code {:?}", &path))?;

    let mut context = ThreadContext::from_code(args.stack_size, code, Rc::new(RefCell::new(io::stdout())));
    context = context.with_env_vars(args.get_environment_variables());

    if args.debug {
        let mut visualiser = visualiser::BarracudaVisualiser::new(context);
        visualiser.run().with_context(|| "Visualiser failed to run")?;
    } else {
        context.run_till_halt().with_context(|| "An unrecoverable error occured while running the program")?;
    }

    Ok(())
}
