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
mod parser;

use clap::Parser;
use emulator::ThreadContext;
use emulator::ops::MathStackOperators::*;
use emulator::instructions::MathStackInstructions::*;
use crate::parser::text_parser::TextParser;
use std::io;
use std::cell::RefCell;
use std::rc::Rc;
use parser::ProgramParser;
use std::fs::File;
use anyhow::{Context, Result};


/// Command Line interface struct
/// Describes possible arguments using the clap library
#[derive(Parser)]
struct Cli {
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,

    #[clap(short, long, default_value_t = 256)]
    stack_size: usize,

    #[clap(short, long)]
    debug: bool
}

fn main()  -> Result<()> {
    let args = Cli::parse();

    let path = args.path.as_path();

    let file = File::open(path).with_context(|| format!("Could not open file {:?}", &path))?;
    let code = TextParser::new().parse(file)
                            .with_context(|| format!("Could not parse file into program code {:?}", &path))?;

    let mut context = emulator::ThreadContext::from_code(args.stack_size, code, Rc::new(RefCell::new(io::stdout())));

    if args.debug {
        let mut visualiser = visualiser::MathStackVisualiser::new(context);
        visualiser.run().with_context(|| "Visualiser failed to run")?;
    } else {
        context.run_till_halt().with_context(|| "An unrecoverable error occured while running the program")?;
    }

    Ok(())
}
