use super::ProgramCodeParser;
use crate::{
    ProgramCode,
    BarracudaInstructions,
    BarracudaOperators};

use std::io::Error;
use std::str::FromStr;


/// TextParser for basic text of Barracuda program identified by the extension .bct.
/// Each instruction is loaded in by each line (By default).
/// If the instruction name matches an operation. That operation and an OP instruction is added to the
/// program. If the instruction name matches a number. The value is put on the values list and a VALUE
/// instruction is added to the program instructions. If the instruction is an 'instruction' it is added
/// to the instruction list.
/// Empty lines are ignored as well as comments starting with # (excl whitespace)
pub struct BarracudaCodeTextParser {
    delimiter: String
}

impl BarracudaCodeTextParser {
    const COMMENT_TOKEN: &'static str = "#";

    /// Creates new TextParser with the default delimiter '\n'
    pub fn new() -> Self {
        Self {
            delimiter: String::from('\n')
        }
    }

    /// Creates new TextParser with custom delimiter
    #[allow(dead_code)]
    pub(crate) fn using_delimiter(mut self, delimiter: String) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Tries to parse token string as a value.
    /// @token: string possibly representing a f64.
    /// @return: f64 value if Ok, Otherwise None if token cannot be parsed.
    fn parse_token_as_value(token: &str) -> Option<f64> {
        match token.parse() {
            Ok(value) => Some(value),
            Err(_) => None
        }
    }

    /// Tries to parse token string as an instruction.
    /// @token: string possibly representing an instruction. These must match the enum name as text.
    /// @return: Instruction variant if Ok, Otherwise None if token cannot be parsed.
    fn parse_token_as_instruction(token: &str) -> Option<BarracudaInstructions> {
        match BarracudaInstructions::from_str(token) {
            Ok(instruction) => Some(instruction),
            Err(_) => None
        }
    }

    /// Tries to parse token string as an operation.
    /// @token: string possibly representing an operation. These must match the enum name as text.
    /// @return: Operation variant if Ok, Otherwise None if token cannot be parsed.
    fn parse_token_as_operation(token: &str) -> Option<BarracudaOperators> {
        match BarracudaOperators::from_str(token) {
            Ok(operation) => Some(operation),
            Err(_) => None
        }
    }
}

impl ProgramCodeParser for BarracudaCodeTextParser  {
    fn parse_str(&self, data: &str) -> Result<ProgramCode, Error> {
        let code_tokens: Vec<&str> = data.split(self.delimiter.as_str()).collect();

        let mut values: Vec<f64> = Vec::new();
        let mut operations: Vec<BarracudaOperators> = Vec::new();
        let mut instructions: Vec<BarracudaInstructions> = Vec::new();

        for token in code_tokens {
            let token = token.trim();

            if let Some(operation) = Self::parse_token_as_operation(token) {
                instructions.push(BarracudaInstructions::OP);
                operations.push(operation);
            } else if let Some(value) = Self::parse_token_as_value(token) {
                instructions.push(BarracudaInstructions::VALUE);
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

#[cfg(test)]
mod tests {
    use crate::{
        ProgramCode,
        BarracudaOperators::{
            FIXED,
        },
        FixedBarracudaOperators::*,
        BarracudaInstructions::*,
        ProgramCodeParser,
        BarracudaCodeTextParser
    };

    #[test]
    fn test_text_parser_basic() {
        let expected_program_code = ProgramCode::new(
            vec![0.0, 8.0, 4.5],
            vec![FIXED(PRINTFF), FIXED(ADD)],
            vec![GOTO, VALUE, OP, OP, VALUE, VALUE]
        );

        let text = "4.5\n8\nADD\nPRINTFF\n0\nGOTO\n";
        let code = BarracudaCodeTextParser::new().parse_str(text).unwrap();
        assert_eq!(expected_program_code, code)
    }

    #[test]
    fn test_text_parser_basic_with_variable_ops() {
        let expected_program_code = ProgramCode::new(
            vec![0.0, f64::from_be_bytes((32_i64).to_be_bytes()), 8.0, 4.5],
            vec![FIXED(RCNX), FIXED(ADD)],
            vec![GOTO, VALUE, OP, VALUE, OP, VALUE, VALUE]
        );

        let text = "4.5\n8\nADD\n1.6e-322\nRCNX\n0\nGOTO\n";
        let code = BarracudaCodeTextParser::new().parse_str(text).unwrap();
        assert_eq!(expected_program_code, code)
    }

    #[test]
    fn test_text_parser_invalid_token() {
        let text = "4.5\n8\nADD\nPRINTER\n0\nGOTO\n";
        BarracudaCodeTextParser::new().parse_str(text).expect_err("Testing Parser Error");
    }

    #[test]
    fn test_text_parser_comma_delimited() {
        let expected_program_code = ProgramCode::new(
            vec![0.0, 8.0, 4.5],
            vec![FIXED(PRINTFF), FIXED(ADD)],
            vec![GOTO, VALUE, OP, OP, VALUE, VALUE]
        );

        let text = "4.5,8,ADD,PRINTFF,0,GOTO";
        let code = BarracudaCodeTextParser::new().using_delimiter(String::from(",")).parse_str(text).unwrap();
        assert_eq!(expected_program_code, code)
    }

    #[test]
    fn test_text_parser_with_whitespace() {
        let expected_program_code = ProgramCode::new(
            vec![0.0, 8.0, 4.5],
            vec![FIXED(PRINTFF), FIXED(ADD)],
            vec![GOTO, VALUE, OP, OP, VALUE, VALUE]
        );

        let text = "    4.5   \n   8  \n  ADD   \n\n\n\n     PRINTFF   \n  0   \n     GOTO\n";
        let code = BarracudaCodeTextParser::new().parse_str(text).unwrap();
        assert_eq!(expected_program_code, code)
    }

    #[test]
    fn test_text_parser_with_comments() {
        let expected_program_code = ProgramCode::new(
            vec![0.0, 8.0, 4.5],
            vec![FIXED(PRINTFF), FIXED(ADD)],
            vec![GOTO, VALUE, OP, OP, VALUE, VALUE]
        );

        let text = "# My wonderful program\n    4.5   \n   8  \n  ADD   \n\n\n\n     PRINTFF   \n  0   \n     GOTO\n";
        let code = BarracudaCodeTextParser::new().parse_str(text).unwrap();
        assert_eq!(expected_program_code, code)
    }

    #[test]
    fn test_text_value_parsing_int() {
        let expected_program_code = ProgramCode::new(
            vec![4.0],
            vec![],
            vec![VALUE]
        );

        let text = "4";
        let code = BarracudaCodeTextParser::new().parse_str(text).unwrap();
        assert_eq!(expected_program_code, code)
    }

    #[test]
    fn test_text_value_parsing_float() {
        let expected_program_code = ProgramCode::new(
            vec![4.5],
            vec![],
            vec![VALUE]
        );

        let text = "4.5";
        let code = BarracudaCodeTextParser::new().parse_str(text).unwrap();
        assert_eq!(expected_program_code, code)
    }
}