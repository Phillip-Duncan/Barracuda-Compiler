mod text_parser;
mod test;

use crate::emulator::ProgramCode;
use std::fs::File;
use std::io::Read;
use std::fmt;

pub(crate) trait ProgramParser {
    // Parse string
    fn parse_str(&self, data : &str) -> Result<ProgramCode, std::io::Error>;

    fn parse(&self, mut file: File) -> Result<ProgramCode, std::io::Error> {
        let mut file_data = String::new();
        file.read_to_string(&mut file_data)?;
        self.parse_str(file_data.as_str())
    }
}