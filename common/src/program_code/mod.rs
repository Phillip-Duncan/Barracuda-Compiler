pub mod instructions;
pub mod ops;

use std::collections::HashMap;
pub use self::ops::{
    FixedBarracudaOperators,
    BarracudaOperators
};
pub use self::instructions::BarracudaInstructions;
use std::fmt;

/// Program Code Decorations holds all non functional data related to a compile program code
/// presently this is just line comments related to instructions
#[derive(Debug)]
pub struct ProgramCodeDecorations {
    line_comments: HashMap<usize, Vec<String>>
}

impl ProgramCodeDecorations {
    fn new() -> Self {
        Self {
            line_comments: Default::default()
        }
    }

    /// Add a comment to program code at an instruction line
    /// multiple comments can be added to the same line.
    fn add_comment(&mut self, line: usize, comment: String) {
        if let Some(existing_comments) = self.line_comments.get_mut(&line) {
            existing_comments.push(comment);
        } else {
            self.line_comments.insert(line, vec![comment]);
        }
    }

    /// Get all comments on a line.
    fn get_comments(&self, line: usize) -> Option<&Vec<String>> {
        self.line_comments.get(&line)
    }
}

/// ProgramCode describes the tables required to run barracuda code in the VM.
#[derive(Debug)]
pub struct ProgramCode {
    /// Value lists loaded with instruction VALUE. This list is padded to align with instructions
    pub values: Vec<f64>,

    /// Operation list loaded with instruction OP. This list is padded to align with instructions
    pub operations: Vec<BarracudaOperators>,

    /// Instruction list denotes the execution of the program from bottom to top
    pub instructions: Vec<BarracudaInstructions>,

    /// Estimate given for max stack size needed for execution of the program code
    pub max_stack_size: usize,

    /// Render decorations is used when formatting to determine if to include decorations.
    render_decorations: bool,

    /// Non functional meta data
    decorations: ProgramCodeDecorations
}

#[allow(dead_code)]
impl ProgramCode {

    /// Generates an empty ProgramCode. Useful when using the builder functions.
    pub fn default() -> ProgramCode {
        ProgramCode {
            values: vec![],
            operations: vec![],
            instructions: vec![],
            max_stack_size: 0,
            render_decorations: false,
            decorations: ProgramCodeDecorations::new()
        }
    }

    /// When creating program code all three lists will be padded to be the same size as instructions.
    pub fn new(values: Vec<f64>, operations: Vec<BarracudaOperators>, instructions: Vec<BarracudaInstructions>) -> ProgramCode {
        ProgramCode {
            values: Self::pad_list_to_size_of_instructions(BarracudaInstructions::VALUE, &instructions, &values, 0.0),
            operations: Self::pad_list_to_size_of_instructions(BarracudaInstructions::OP, &instructions, &operations, BarracudaOperators::FIXED(FixedBarracudaOperators::NULL)),
            instructions,
            max_stack_size: 0,
            render_decorations: false,
            decorations: ProgramCodeDecorations::new()
        }
    }

    /// Replaces self with a decorated version of program code
    pub fn decorated(mut self) -> Self {
        self.render_decorations = true;
        self
    }

    /// Builder function adds value to program code while keeping other arrays padded
    pub fn push_value(&mut self, value: f64) {
        self.values.push(value);
        self.operations.push(BarracudaOperators::FIXED(FixedBarracudaOperators::NULL));
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
        self.operations.push(BarracudaOperators::FIXED(FixedBarracudaOperators::NULL));
        self.instructions.push(instruction);
    }

    /// Builder function adds comment to program code decorations at current line
    pub fn push_comment(&mut self, comment: String) {
        self.decorations.add_comment(self.instructions.len(), comment);
    }

    /// Generic padding function for the values/operations list to be padded to align with the instruction list
    /// This will create a new aligned list where each value is found where the alignment_instr is found
    /// This allows for the program counter to be used for all lists without misalignment.
    /// @alignment_instr: Expected either OP or VALUE
    /// @instructions: The list of instructions for the program
    /// @unaligned_list: Either the inputted values or operations list
    /// @null_value: what the padded spaces should be filled with
    /// @return: unaligned_list padded to size of instructions.len()
    pub fn pad_list_to_size_of_instructions<T: Clone>(alignment_instr: BarracudaInstructions, instructions: &Vec<BarracudaInstructions>, unaligned_list: &Vec<T>, null_value: T) -> Vec<T> {
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


// Implement Equality traits for comparing program code
impl PartialEq for ProgramCode {
    fn eq(&self, other: &Self) -> bool {
        self.instructions == other.instructions &&
            self.operations == other.operations &&
            self.values == other.values
    }
}
impl Eq for ProgramCode {}


impl fmt::Display for ProgramCode {
    /// This allows for program code to be converted into a string.
    /// For files this format is stored with the extension .bct.
    /// This format is supported by the barracuda_emulator and can be loaded in for debugging.
    ///
    /// # Format
    /// Each line represents an instruction for the barracuda Virtual Machine. For each instruction
    /// the enum name is displayed unless the instruction is OP or VALUE. For OP the operation enum
    /// name is displayed instead. For VALUE the value is directly written to the line.
    /// Lines that start with # are comments that are ignored.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        // Write recommended stack size at top of program as a comment
        writeln!(f, "# RECOMMENDED_STACKSIZE {}", self.max_stack_size)?;

        for i in 0..self.instructions.len() {
            // Write comments
            if self.render_decorations {
                if let Some(comments) = self.decorations.get_comments(i) {
                    for comment in comments {
                        writeln!(f, "# {}", comment)?;
                    }
                }
            }

            // Write instruction
            let instr = self.instructions[i];
            match instr {
                BarracudaInstructions::VALUE => {
                    let value = self.values[i];
                    writeln!(f, "{}", value)?;
                },
                BarracudaInstructions::OP => {
                    let operation = self.operations[i];
                    writeln!(f, "{}", operation)?;
                },
                _ => {
                    writeln!(f, "{:?}", instr)?;
                }
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ProgramCode;
    use super::BarracudaInstructions::*;
    use super::FixedBarracudaOperators::*;

    #[test]
    fn test_instruction_values_padding() {
        let instructions = vec![VALUE, OP, VALUE, OP, VALUE, OP];
        let unpadded_values = vec![1.0, 2.0, 3.0];
        let padded_values = ProgramCode::pad_list_to_size_of_instructions(VALUE, &instructions, &unpadded_values, 0.0);

        assert_eq!(padded_values.len(), instructions.len());
        assert_eq!(padded_values, vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0]);
    }

    #[test]
    fn test_instruction_operations_padding() {
        let instructions = vec![VALUE, OP, VALUE, OP, VALUE, OP];
        let unpadded_operations = vec![ADD, SUB, MUL];
        let padded_operations = ProgramCode::pad_list_to_size_of_instructions(OP, &instructions, &unpadded_operations, NULL);

        assert_eq!(padded_operations.len(), instructions.len());
        assert_eq!(padded_operations, vec![NULL, ADD, NULL, SUB, NULL, MUL]);
    }

    #[test]
    fn test_instruction_values_prepadded() {
        let instructions = vec![VALUE, OP, VALUE, OP, VALUE, OP];
        let unpadded_values = vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0];
        let padded_values = ProgramCode::pad_list_to_size_of_instructions(VALUE, &instructions, &unpadded_values, 0.0);

        assert_eq!(padded_values.len(), instructions.len());
        assert_eq!(padded_values, vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0]);
    }

    #[test]
    fn test_instruction_operations_prepadded() {
        let instructions = vec![VALUE, OP, VALUE, OP, VALUE, OP];
        let unpadded_operations = vec![NULL, ADD, NULL, SUB, NULL, MUL];
        let padded_operations = ProgramCode::pad_list_to_size_of_instructions(OP, &instructions, &unpadded_operations, NULL);

        assert_eq!(padded_operations.len(), instructions.len());
        assert_eq!(padded_operations, vec![NULL, ADD, NULL, SUB, NULL, MUL]);
    }
}