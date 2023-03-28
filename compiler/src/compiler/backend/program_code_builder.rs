use barracuda_common::{
    ProgramCode,
    BarracudaInstructions as INSTRUCTION,
    BarracudaOperators as OP,
    FixedBarracudaOperators as FIXED_OP,
    VariableBarracudaOperators as VAR_OP
};

/// BarracudaIR is linear item format for describing ProgramCode.
enum BarracudaIR {
    /// Value is substituted directly when generating ProgramCode
    Value(f64),

    /// Instruction is substituted directly when generating ProgramCode
    Instruction(INSTRUCTION),

    /// Operation is substituted directly when generating ProgramCode
    Operation(OP),

    /// Label uses a unique id to bookmark an instruction
    Label(u64),

    /// Reference uses a label unique id to reference a bookmark. This reference is replaced
    /// with the instruction index that the label is stored at. This makes it easier to generate
    /// jumps within code without knowing the exact generated size until finished.
    Reference(u64),

    /// Comments are purely decorative and allow for instructions to be annotated these are stored
    /// with ProgramCodeDecorations after finalisation
    Comment(String)
}

/// BarracudaProgramCodeBuilder is a Builder utility class that builds ProgramCode linearly.
/// This is useful for backend generators to implement business logic of generation without
/// worrying about the representation of ProgramCode. In particular the builder offers label
/// generation usage and referencing which greatly reduces the complexity of jump addresses.
pub struct BarracudaProgramCodeBuilder {
    program_out: Vec<BarracudaIR>,
    label_count: u64
}

impl BarracudaProgramCodeBuilder {
    pub fn new() -> Self {
        Self {
            program_out: vec![],
            label_count: 0
        }
    }

    /// Emit value pushes a value to be loaded as the next instruction
    pub fn emit_value(&mut self, value: f64) {
        self.program_out.push(BarracudaIR::Value(value));
    }

    /// Emit instruction pushes the next instruction
    pub fn emit_instruction(&mut self, instruction: INSTRUCTION) {
        self.program_out.push(BarracudaIR::Instruction(instruction));
    }

    /// Emit operation pushes an op to be loaded as the next instruction
    pub fn emit_op(&mut self, operation: FIXED_OP) {
        self.program_out.push(BarracudaIR::Operation(OP::FIXED(operation)));
    }

    /// Emit operation pushes an op to be loaded as the next instruction
    pub fn emit_var_op(&mut self, operation: VAR_OP) {
        self.program_out.push(BarracudaIR::Operation(OP::VARIABLE(operation)));
    }

    /// Comment decorates the next instruction with a string
    /// Multiple comments can be pushed and they will be stored separately for the next instruction.
    /// Comments have no functional usage and purely decorative to help identify sections within
    /// the byte code.
    pub fn comment(&mut self, comment: String) {
        self.program_out.push(BarracudaIR::Comment(comment));
    }

    /// Create label generates a new unique label to address specific instruction indices in the code.
    /// This is useful for jump statements where you do not know the generated code size yet.
    /// ## Typical Usage
    /// let my_label = builder.create_label()       // Creates new unique label id
    /// builder.reference(my_label)                 // On finalise will insert label instruction index
    /// builder.emit_instruction(Instruction::GOTO)
    /// ... // Generate instructions
    /// builder.set_label(my_label)                 // Sets the label instruction location
    pub fn create_label(&mut self) -> u64 {
        let label = self.label_count;
        self.label_count += 1;
        label
    }

    /// Sets a label location within the code.
    /// On finalisation the label will be skipped. However BarracudaIR::References will be replaced
    /// with the instruction index of the set label.
    /// @see create_label for more details
    pub fn set_label(&mut self, label: u64) {
        self.program_out.push(BarracudaIR::Label(label))
    }

    /// References the location of a unique label.
    /// On finalisation the reference is replaced with the value of the label instruction index.
    /// @see create_label for more details
    pub fn reference(&mut self, label: u64) {
        self.program_out.push(BarracudaIR::Reference(label))
    }

    /// Resolves all BarracudaIR items into ProgramCode, consumes self in the process.
    pub fn finalize(self) -> ProgramCode {
        self.resolve_labels()
    }

    /// This method allows for the builder to generate program code with a proceeding
    /// header of values. These values are inserted at the beginning of the program.
    ///
    /// For some generators it may be necessary to allocate space at the start of
    /// the program and this space may not be known until the rest of the program has
    /// been generated.
    pub fn finalize_with_header(mut self, header: Vec<f64>) -> ProgramCode {
        self.insert_program_header(header);
        self.finalize()
    }

    /// Inserts a vector of values at the start of the program
    fn insert_program_header(&mut self, header: Vec<f64>) {
        // Reversed to preserve order after inserting all values
        for value in header.into_iter().rev() {
            self.program_out.insert(0, BarracudaIR::Value(value));
        }
    }

    /// Resolves label locations into specific instruction indices.
    /// Returns vector of location indices.
    fn get_label_locations(&self) -> Vec<usize> {
        let mut locations = vec![0; self.label_count as usize];
        let mut current_line = 0;

        // Label locations have to be determined linearly as comments and labels should take up zero
        // space once generated
        for code_token in &self.program_out {
            match code_token {
                BarracudaIR::Label(id) => {
                    locations[*id as usize] = current_line;
                }
                BarracudaIR::Comment(_) => {}

                // Everything else should take up a instruction slot
                _ => {
                    current_line += 1
                }
            }
        }

        return locations
    }

    /// Resolves all labels and generates program code
    fn resolve_labels(self) -> ProgramCode {
        // First pass finding labels
        let locations = self.get_label_locations();

        // Second pass replacing tokens
        let mut output_program = ProgramCode::default();
        for code_token in &self.program_out {
            match code_token {
                BarracudaIR::Instruction(instruction) => {
                    output_program.push_instruction(instruction.clone());
                }
                BarracudaIR::Operation(operation) => {
                    output_program.push_operation(operation.clone());
                }
                BarracudaIR::Value(value) => {
                    output_program.push_value(value.clone());
                }
                BarracudaIR::Reference(id) => {
                    output_program.push_value(f64::from_ne_bytes(locations[*id as usize].clone().to_ne_bytes()));
                }
                BarracudaIR::Label(_) => {} // Skip labels
                BarracudaIR::Comment(comment) => {
                    output_program.push_comment(comment.clone());
                }
            };
        }

        output_program
    }
}