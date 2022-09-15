pub mod instructions;
pub mod ops;

use self::ops::BarracudaOperators;
use self::instructions::BarracudaInstructions;
use std::fmt;


/// ProgramCode describes the tables required to run barracuda code
#[derive(Debug)]
pub struct ProgramCode {
    /// Value lists loaded with instruction VALUE. This list is padded to align with instructions
    pub values: Vec<f64>,

    /// Operation list loaded with instruction OP. This list is padded to align with instructions
    pub operations: Vec<BarracudaOperators>,

    /// Instruction list denotes the execution of the program from top to bottom
    pub instructions: Vec<BarracudaInstructions>,
}

#[allow(dead_code)]
impl ProgramCode {

    pub fn default() -> ProgramCode {
        ProgramCode {
            values: vec![],
            operations: vec![],
            instructions: vec![]
        }
    }

    /// When creating program code all three lists will be padded to be the same size as instructions.
    pub fn new(values: Vec<f64>, operations: Vec<BarracudaOperators>, instructions: Vec<BarracudaInstructions>) -> ProgramCode {
        ProgramCode {
            values: Self::pad_list_to_size_of_instructions(BarracudaInstructions::VALUE, &instructions, &values, 0.0),
            operations: Self::pad_list_to_size_of_instructions(BarracudaInstructions::OP, &instructions, &operations, BarracudaOperators::NULL),
            instructions
        }
    }

    /// Builder function adds value to program code while keeping other arrays padded
    pub fn push_value(&mut self, value: f64) {
        self.values.push(value);
        self.operations.push(BarracudaOperators::NULL);
        self.instructions.push(BarracudaInstructions::VALUE);
    }

    /// Builder function adds operation to program code while keeping other arrays padded
    pub fn push_operation(&mut self, operation: BarracudaOperators) {
        self.values.push(0.0);
        self.operations.push(operation);
        self.instructions.push(BarracudaInstructions::OP);
    }

    /// Builder function adds instruction to program code while keeping other arrays padded
    pub fn push_instruction(&mut self, instruction: BarracudaInstructions) {
        self.values.push(0.0);
        self.operations.push(BarracudaOperators::NULL);
        self.instructions.push(instruction);
    }

    /// Generic padding function for the values/operations list to be padded to align with the instruction list
    /// This will create a new aligned list where each value is found where the alignment_instr is found
    /// This allows for the program counter to be used for all lists without misalignment.
    /// @alignment_instr: Expected either OP or VALUE
    /// @instructions: The list of instructions for the program
    /// @unaligned_list: Either the inputted values or operations list
    /// @null_value: what the padded spaces should be filled with
    /// @return: unaligned_list padded to size of instructions.len()
    pub fn pad_list_to_size_of_instructions<T: std::clone::Clone>(alignment_instr: BarracudaInstructions, instructions: &Vec<BarracudaInstructions>, unaligned_list: &Vec<T>, null_value: T) -> Vec<T> {
        // Check if the lists are already the same size.
        // Note: It is impossible to verify if the alignment is correct as the null_value can
        // be used for padding as well as a unaligned value. Best is to just verify length.
        if instructions.len() == unaligned_list.len() {
            return unaligned_list.clone();
        }

        // Pad list with null_value where instructions matches alignment_instr the unaligned_list
        // value is substituted.
        let mut aligned_list: Vec<T> = Vec::new();
        let mut unaligned_index: usize = 0;

        for i in 0..instructions.len() {
            let instr = instructions[i];
            if instr == alignment_instr {
                aligned_list.push(unaligned_list[unaligned_index].clone());
                unaligned_index += 1;
            } else {
                aligned_list.push(null_value.clone());
            }
        };

        aligned_list
    }
}

impl PartialEq for ProgramCode {
    fn eq(&self, other: &Self) -> bool {
        self.instructions == other.instructions &&
            self.operations == other.operations &&
            self.values == other.values
    }
}
impl Eq for ProgramCode {}

impl fmt::Display for ProgramCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        for i in 0..self.instructions.len() {
            let instr = self.instructions[i];
            match instr {
                BarracudaInstructions::VALUE => {
                    let value = self.values[i];
                    writeln!(f, "{}", value)?;
                },
                BarracudaInstructions::OP => {
                    let operation = self.operations[i];
                    writeln!(f, "{:?}", operation)?;
                },
                _ => {
                    writeln!(f, "{:?}", instr)?;
                }
            }
        };

        Ok(())
    }
}