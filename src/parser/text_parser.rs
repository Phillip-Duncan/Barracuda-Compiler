use crate::parser::ProgramParser;
use std::io::Error;
use crate::emulator::ProgramCode;
use crate::emulator::ops::MathStackOperators;
use crate::emulator::instructions::MathStackInstructions;
use std::str::FromStr;

pub(crate) struct TextParser {
    delimiter: String
}

impl TextParser {
    const COMMENT_TOKEN: &'static str = "#";

    pub(crate) fn new() -> TextParser {
        TextParser {
            delimiter: String::from('\n')
        }
    }

    pub(crate) fn using_delimiter(delimiter: String) -> TextParser {
        TextParser {
            delimiter
        }
    }

    fn parse_token_as_value(token: &str) -> Option<f64> {
        match token.parse() {
            Ok(value) => Some(value),
            Err(_) => None
        }
    }

    fn parse_token_as_instruction(token: &str) -> Option<MathStackInstructions> {
        match MathStackInstructions::from_str(token) {
            Ok(instruction) => Some(instruction),
            Err(_) => None
        }
    }

    fn parse_token_as_operation(token: &str) -> Option<MathStackOperators> {
        match MathStackOperators::from_str(token) {
            Ok(operation) => Some(operation),
            Err(_) => None
        }
    }
}

impl ProgramParser for TextParser {
    fn parse_str(&self, data: &str) -> Result<ProgramCode, Error> {
        let code_tokens: Vec<&str> = data.split(self.delimiter.as_str()).collect();

        let mut values: Vec<f64> = Vec::new();
        let mut operations: Vec<MathStackOperators> = Vec::new();
        let mut instructions: Vec<MathStackInstructions> = Vec::new();

        for token in code_tokens {
            let token = token.trim();

            if let Some(operation) = Self::parse_token_as_operation(token) {
                instructions.push(MathStackInstructions::OP);
                operations.push(operation);
            } else if let Some(value) = Self::parse_token_as_value(token) {
                instructions.push(MathStackInstructions::VALUE);
                values.push(value);
            } else if let Some(instruction) = Self::parse_token_as_instruction(token) {
                instructions.push(instruction);
            } else if token.len()==0 || token.starts_with(Self::COMMENT_TOKEN ) {
                continue;
            } else {
                return Err(Error::new(std::io::ErrorKind::InvalidInput,
                               format!("Unknown code token found while parsing '{}'.", token)));
            }
        }

        values.reverse();
        operations.reverse();
        instructions.reverse();

        Ok(ProgramCode::new(
            values,
            operations,
            instructions
        ))
    }
}