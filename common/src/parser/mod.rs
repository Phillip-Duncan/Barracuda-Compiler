pub(crate) mod bct_parser;

use super::program_code::ProgramCode;
use std::fs::File;
use std::io::Read;

/// Program parser is a trait to implemented by parsers of different Barracuda file formats
/// Implementors of the trait have to implement the parse_str function
pub trait ProgramCodeParser {
    // Parse string
    fn parse_str(&self, data : &str) -> Result<ProgramCode, std::io::Error>;

    fn parse(&self, mut file: File) -> Result<ProgramCode, std::io::Error> {
        let mut file_data = String::new();
        file.read_to_string(&mut file_data)?;
        self.parse_str(file_data.as_str())
    }
}