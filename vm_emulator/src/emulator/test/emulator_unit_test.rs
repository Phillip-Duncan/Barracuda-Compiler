use crate::emulator::{self};
use crate::emulator::MathStackInstructions::*;
use crate::emulator::ops::MathStackOperators::*;

#[test]
fn test_instruction_values_padding() {
    let instructions = vec![VALUE, OP, VALUE, OP, VALUE, OP];
    let unpadded_values = vec![1.0, 2.0, 3.0];
    let padded_values = emulator::ProgramCode::pad_list_to_size_of_instructions(VALUE, &instructions, &unpadded_values, 0.0);

    assert_eq!(padded_values.len(), instructions.len());
    assert_eq!(padded_values, vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0]);
}

#[test]
fn test_instruction_operations_padding() {
    let instructions = vec![VALUE, OP, VALUE, OP, VALUE, OP];
    let unpadded_operations = vec![ADD, SUB, MUL];
    let padded_operations = emulator::ProgramCode::pad_list_to_size_of_instructions(OP, &instructions, &unpadded_operations, NULL);

    assert_eq!(padded_operations.len(), instructions.len());
    assert_eq!(padded_operations, vec![NULL, ADD, NULL, SUB, NULL, MUL]);
}

#[test]
fn test_instruction_values_prepadded() {
    let instructions = vec![VALUE, OP, VALUE, OP, VALUE, OP];
    let unpadded_values = vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0];
    let padded_values = emulator::ProgramCode::pad_list_to_size_of_instructions(VALUE, &instructions, &unpadded_values, 0.0);

    assert_eq!(padded_values.len(), instructions.len());
    assert_eq!(padded_values, vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0]);
}

#[test]
fn test_instruction_operations_prepadded() {
    let instructions = vec![VALUE, OP, VALUE, OP, VALUE, OP];
    let unpadded_operations = vec![NULL, ADD, NULL, SUB, NULL, MUL];
    let padded_operations = emulator::ProgramCode::pad_list_to_size_of_instructions(OP, &instructions, &unpadded_operations, NULL);

    assert_eq!(padded_operations.len(), instructions.len());
    assert_eq!(padded_operations, vec![NULL, ADD, NULL, SUB, NULL, MUL]);
}