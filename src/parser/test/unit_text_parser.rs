use crate::parser::text_parser::TextParser;
use crate::parser::ProgramParser;
use crate::emulator::ProgramCode;
use crate::emulator::ops::MathStackOperators::*;
use crate::emulator::instructions::MathStackInstructions::*;


#[test]
fn test_text_parser_basic() {
    let expected_program_code = ProgramCode::new(
        vec![0.0, 8.0, 4.5],
        vec![PRINTFF, ADD],
        vec![GOTO, VALUE, OP, OP, VALUE, VALUE]
    );

    let text = "4.5\n8\nADD\nPRINTFF\n0\nGOTO\n";
    let code = TextParser::new().parse_str(text).unwrap();
    assert_eq!(expected_program_code, code)
}

#[test]
fn test_text_parser_invalid_token() {
    let text = "4.5\n8\nADD\nPRINTER\n0\nGOTO\n";
    TextParser::new().parse_str(text).expect_err("Testing Parser Error");
}

#[test]
fn test_text_parser_comma_delimited() {
    let expected_program_code = ProgramCode::new(
        vec![0.0, 8.0, 4.5],
        vec![PRINTFF, ADD],
        vec![GOTO, VALUE, OP, OP, VALUE, VALUE]
    );

    let text = "4.5,8,ADD,PRINTFF,0,GOTO";
    let code = TextParser::using_delimiter(String::from(",")).parse_str(text).unwrap();
    assert_eq!(expected_program_code, code)
}

#[test]
fn test_text_parser_with_whitespace() {
    let expected_program_code = ProgramCode::new(
        vec![0.0, 8.0, 4.5],
        vec![PRINTFF, ADD],
        vec![GOTO, VALUE, OP, OP, VALUE, VALUE]
    );

    let text = "    4.5   \n   8  \n  ADD   \n\n\n\n     PRINTFF   \n  0   \n     GOTO\n";
    let code = TextParser::new().parse_str(text).unwrap();
    assert_eq!(expected_program_code, code)
}

#[test]
fn test_text_parser_with_comments() {
    let expected_program_code = ProgramCode::new(
        vec![0.0, 8.0, 4.5],
        vec![PRINTFF, ADD],
        vec![GOTO, VALUE, OP, OP, VALUE, VALUE]
    );

    let text = "# My wonderful program\n    4.5   \n   8  \n  ADD   \n\n\n\n     PRINTFF   \n  0   \n     GOTO\n";
    let code = TextParser::new().parse_str(text).unwrap();
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
    let code = TextParser::new().parse_str(text).unwrap();
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
    let code = TextParser::new().parse_str(text).unwrap();
    assert_eq!(expected_program_code, code)
}