#[macro_use]
extern crate simple_error;

mod program_code;

pub use program_code::{
    ProgramCode,
    BarracudaOperators,
    FixedBarracudaOperators,
    VariableBarracudaOperators,
    BarracudaInstructions
};

mod parser;

pub use parser::{
    ProgramCodeParser,
    bct_parser::BarracudaCodeTextParser
};

mod cli_utility;
pub use cli_utility::{
    CLIEnvVarDescriptor
};