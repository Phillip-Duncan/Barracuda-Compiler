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

mod emulator;
mod test;
mod visualiser;
mod parser;

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

    let board_size: f64 = 50.0;
    let bs = board_size;

    let mut context = emulator::ThreadContext::new(200,
                                                   vec![0.0,0.0,0.0,0.0,0.0,0.0,0.0,1.0,0.0,0.0,110.0,0.0,0.0,4.0,0.0,0.0,0.0,0.0,0.0,7.0,0.0,1.0,0.0,0.0,4.0,0.0,(bs-1.0),1.0,0.0,4.0,0.0,0.0,0.0,0.0,4.0,0.0,0.0,1.0,0.0,0.0,0.0,10.0,0.0,0.0,0.0,0.0,32.0,42.0,0.0,0.0,0.0,4.0,0.0,(bs),0.0,0.0,0.0,(bs-2.0),0.0,0.0,1.0,0.0,(bs-2.0)*4.0,0.0,0.0,4.0,0.0,(bs+2.0)*4.0],
                                                   vec![NULL,DROP,DROP,NULL,SWAP,WRITE,AND,NULL,RSHIFT,SWAP,NULL,OVER,SUB_PTR,NULL,OVER,OR,READ, OVER,AND,NULL,LSHIFT,NULL,SWAP,ADD_PTR, NULL,NULL,NULL,NULL,ADD_PTR,NULL,OVER,OR,READ,ADD_PTR,NULL,OVER,LSHIFT,NULL,READ,DUP,PRINTC,NULL,DROP,NULL,PRINTC,TERNARY,NULL,NULL,READ,DUP, ADD_PTR,NULL,NULL,NULL,NULL,DUP,NULL,NULL,NULL,WRITE,NULL,ADD_PTR,NULL,DUP,ADD_PTR, NULL, MALLOC,NULL],
                                                   vec![LOOP_END,OP,OP,LOOP_END,OP,OP,OP,VALUE,OP,OP,VALUE,OP,OP,VALUE,OP,OP,OP,OP,OP,VALUE,OP,VALUE,OP,OP,VALUE,LOOP_ENTRY, VALUE,VALUE,OP,VALUE,OP,OP,OP,OP,VALUE,OP,OP,VALUE,OP,OP,OP,VALUE,OP,LOOP_END,OP,OP,VALUE,VALUE,OP,OP,OP,VALUE,LOOP_ENTRY,VALUE,VALUE,OP,LOOP_ENTRY,VALUE,VALUE,OP,VALUE,OP,VALUE,OP,OP, VALUE, OP,VALUE],
                                                   Rc::new(RefCell::new(io::stdout())));
    let mut visualiser = visualiser::MathStackVisualiser::new(context);
    visualiser.run().unwrap();
}
