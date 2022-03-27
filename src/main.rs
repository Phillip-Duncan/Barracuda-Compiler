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

mod emulator;
mod test;
mod visualiser;

use clap::Parser;
use emulator::ThreadContext;
use emulator::ops::MathStackOperators::*;
use emulator::instructions::MathStackInstructions::*;
use std::io;
use std::cell::RefCell;
use std::rc::Rc;


/// Command Line interface struct
/// Describes possible arguments using the clap library
#[derive(Parser)]
struct Cli {
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main()  {
    let _args = Cli::parse();

    let mut context = ThreadContext::new(5,
                                         vec![10.0, 0.0],
                                         vec![PRINTC, PRINTC, PRINTC, LDA, LDB, LDC],
                                         vec![LOOP_END, LOOP_END, OP, OP, OP, OP, OP, OP,LOOP_ENTRY, VALUE, VALUE],
                                         Rc::new(RefCell::new(io::stdout())));
    context.set_env_var(0, ('H' as u8) as f64);
    context.set_env_var(1, ('i' as u8) as f64);
    context.set_env_var(2, ('\n' as u8) as f64);
    let mut visualiser = visualiser::MathStackVisualiser::new(context);
    visualiser.run();
}
