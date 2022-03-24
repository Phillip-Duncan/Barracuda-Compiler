extern crate core;
extern crate statrs;
extern crate float_next_after;
#[macro_use]
extern crate approx;



mod emulator;
mod test;

use clap::Parser;


/// Command Line interface struct
/// Describes possible arguments using the clap library
#[derive(Parser)]
struct Cli {
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main()  {
    let _args = Cli::parse();
}
