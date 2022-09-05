use super::barracuda_def::{
    symbols::SymbolType,
    symbols::BarracudaSymbol,
    scope::BarracudaScope
};
use super::{BackEndGenerator, CodeToken};

use super::super::ast::{
    AbstractSyntaxTree,
    ASTNode,
    Literal,
    BinaryOperation,
    UnaryOperation
};

use super::super::program_code::{
    ProgramCode,
    instructions::BarracudaInstructions as INSTRUCTION,
    ops::BarracudaOperators as OP
};


use std::borrow::{Borrow, BorrowMut};
use std::iter::Map;
use std::collections::HashMap;
use std::cmp::max;


// TODO(Connor): This should be replaced by a more concrete definition
// Like that found in the emulator
struct BarracudaByteCodeToken {
    code: String
}

impl BarracudaByteCodeToken {
    fn from(code: String) -> Self {
        Self {
            code
        }
    }
}

impl CodeToken for BarracudaByteCodeToken {
    fn repr(&self) -> String {
        self.code.clone()
    }
}


enum BarracudaIR {
    Value(f64),
    Instruction(INSTRUCTION),
    Operation(OP),
    Label(u64),
    Reference(u64),
    Comment(String)
}

pub(crate) struct BarracudaBackend {
    scope: BarracudaScope,
    program_out: Vec<BarracudaIR>,
    label_count: u64
}

impl BarracudaBackend {

    pub(crate) fn new() -> Self {
        BarracudaBackend {
            scope: BarracudaScope::new(),
            program_out: Vec::new(),
            label_count: 0
        }
    }

    fn emit(&mut self, token: BarracudaIR) {
        self.program_out.push(token);
    }

    fn comment(&mut self, comment: String) {
        self.program_out.push(BarracudaIR::Comment(comment));
    }

    fn create_label(&mut self) -> u64 {
        let label = self.label_count;
        self.label_count += 1;
        label
    }

    fn set_label(&mut self, label: u64) {
        self.program_out.push(BarracudaIR::Label(label))
    }

    fn reference(&mut self, label: u64) {
        self.program_out.push(BarracudaIR::Reference(label))
    }

    fn generate(&mut self, node: &ASTNode) -> ProgramCode {
        self.generate_node(node);
        self.insert_program_header();
        self.resolve_labels()
    }

    fn insert_program_header(&mut self) {
        // Push zeros onto stack for local variable storage
        // Note(Connor): Can be optimized in the future by directly inserting initial values here
        for _ in 0..self.scope.scope_total_mutable {
            self.program_out.insert(0, BarracudaIR::Value(0.0))
        }
    }

    fn resolve_labels(&self) -> ProgramCode {
        // Note(Connor): Split into separate functions for production

        // First pass finding labels
        let mut locations = vec![0; self.label_count as usize];
        let mut current_line = 0;
        for code_token in &self.program_out {
            match code_token {
                BarracudaIR::Label(id) => {
                    locations[*id as usize] = current_line;
                }
                BarracudaIR::Comment(_) => {}
                _ => {
                    current_line += 1
                }
            }
        }

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
                    output_program.push_value(locations[*id as usize].clone() as f64);
                }
                BarracudaIR::Label(_) => {}
                BarracudaIR::Comment(_) => {}
            };
        }

        output_program
    }

    fn generate_node(&mut self, node: &ASTNode) {
        match node {
            ASTNode::IDENTIFIER(identifier_name) => {
                match self.scope.get_symbol(identifier_name) {
                    Some(symbol) => {
                        self.emit(BarracudaIR::Value(symbol.scope_id as f64)); // Store
                        self.emit(BarracudaIR::Operation(OP::STK_READ));
                    },
                    None => {panic!("Identifier '{}' out of scope", identifier_name)}
                }
            }
            ASTNode::LITERAL(value) => {
                self.emit(BarracudaIR::Value(match *value {
                    Literal::FLOAT(value) => { value }
                    Literal::INTEGER(value) => { value as f64 }
                    Literal::STRING(_) => { unimplemented!() }
                    Literal::BOOL(value) => { value as i64 as f64 }
                }));
            }
            ASTNode::UNARY_OP { op, expression } => {
                self.generate_node(expression);
                match op {
                    UnaryOperation::NOT => {self.emit(BarracudaIR::Operation(OP::NOT))}
                    UnaryOperation::NEGATE => {
                        self.emit(BarracudaIR::Value(0.0));
                        self.emit(BarracudaIR::Operation(OP::SUB));
                    }
                };
            }
            ASTNode::BINARY_OP { op, lhs, rhs } => {
                self.generate_node(lhs);
                self.generate_node(rhs);
                match op {
                    BinaryOperation::ADD   => { self.emit(BarracudaIR::Operation(OP::ADD)); }
                    BinaryOperation::SUB   => { self.emit(BarracudaIR::Operation(OP::SUB)); }
                    BinaryOperation::DIV   => { self.emit(BarracudaIR::Operation(OP::DIV)); }
                    BinaryOperation::MUL   => { self.emit(BarracudaIR::Operation(OP::MUL)); }
                    BinaryOperation::MOD   => { self.emit(BarracudaIR::Operation(OP::FMOD)); }
                    BinaryOperation::POW   => { self.emit(BarracudaIR::Operation(OP::POW)); }
                    BinaryOperation::EQUAL => { self.emit(BarracudaIR::Operation(OP::EQ)); }
                    BinaryOperation::NOT_EQUAL => {
                        self.emit(BarracudaIR::Operation(OP::EQ));
                        self.emit(BarracudaIR::Operation(OP::NOT));
                    }
                    BinaryOperation::GREATER_THAN  => { self.emit(BarracudaIR::Operation(OP::GT)) }
                    BinaryOperation::LESS_THAN     => { self.emit(BarracudaIR::Operation(OP::LT)) }
                    BinaryOperation::GREATER_EQUAL => { self.emit(BarracudaIR::Operation(OP::GTEQ)) }
                    BinaryOperation::LESS_EQUAL    => { self.emit(BarracudaIR::Operation(OP::LTEQ)) }
                };
            }
            ASTNode::CONSTRUCT { identifier, datatype, expression } => {
                // Create symbol
                let identifier_name = match identifier.as_ref() {
                    ASTNode::IDENTIFIER(name) => {name},
                    _ => panic!("Identifier missing from construct statement!")
                };
                let symbol = BarracudaSymbol {
                    name: identifier_name.clone(),
                    symbol_type: SymbolType::Void, // TODO(Connor): Add type info
                    mutable: true,
                    scope_id: self.scope.next_mutable_id()
                };

                self.emit(BarracudaIR::Value(symbol.scope_id as f64)); // Store
                self.generate_node(expression); // Expression
                self.emit(BarracudaIR::Operation(OP::STK_WRITE));
                self.scope.add_symbol(symbol); // Add symbol last to avoid recursive statement let a = a
            }
            ASTNode::ASSIGNMENT { identifier, expression } => {
                let identifier_name = match identifier.as_ref() {
                    ASTNode::IDENTIFIER(name) => {name},
                    _ => panic!("Identifier missing from assignment statement!")
                };

                match self.scope.get_symbol(identifier_name) {
                    Some(symbol) => {
                        if !symbol.mutable {
                            panic!("Assignment to  immutable identifier '{}'", identifier_name)
                        }

                        self.emit(BarracudaIR::Value(symbol.scope_id as f64)); // Store
                        self.generate_node(expression); // Expression
                        self.emit(BarracudaIR::Operation(OP::STK_WRITE));
                    },
                    None => {panic!("Identifier '{}' out of scope", identifier_name)}
                }
            },
            ASTNode::PRINT {expression} => {
                self.generate_node(expression);
                self.emit(BarracudaIR::Operation(OP::PRINTFF));

                // New Line character
                self.emit(BarracudaIR::Value(10.0 as f64));
                self.emit(BarracudaIR::Operation(OP::PRINTC));
            }
            ASTNode::BRANCH { condition, if_branch, else_branch } => {
                let if_end = self.create_label();

                self.generate_node(condition);
                self.reference(if_end);
                self.emit(BarracudaIR::Instruction(INSTRUCTION::GOTO_IF));
                self.generate_node(if_branch);

                match else_branch.as_ref() {
                    None => {
                        self.set_label(if_end);
                    },
                    Some(else_branch) => {
                        let else_end = self.create_label();

                        self.reference(else_end);
                        self.emit(BarracudaIR::Instruction(INSTRUCTION::GOTO));
                        self.set_label(if_end);

                        self.generate_node(else_branch);
                        self.set_label(else_end);
                    }
                }

            }
            ASTNode::WHILE_LOOP { condition, body } => {
                let while_start = self.create_label();
                let while_exit = self.create_label();

                self.set_label(while_start);

                self.generate_node(condition);
                self.reference(while_exit);
                self.emit(BarracudaIR::Instruction(INSTRUCTION::GOTO_IF));

                self.generate_node(body);

                self.reference(while_start);
                self.emit(BarracudaIR::Instruction(INSTRUCTION::GOTO));

                self.set_label(while_exit);

            }
            ASTNode::FOR_LOOP { initialization, condition, advancement, body } => {
                let for_start = self.create_label();
                let for_exit = self.create_label();


                self.generate_node(initialization);

                self.set_label(for_start);
                self.generate_node(condition);
                self.reference(for_exit);
                self.emit(BarracudaIR::Instruction(INSTRUCTION::GOTO_IF));

                self.generate_node(body);
                self.generate_node(advancement);

                self.reference(for_start);
                self.emit(BarracudaIR::Instruction(INSTRUCTION::GOTO));

                self.set_label(for_exit);
            }
            ASTNode::STATEMENT_LIST(statements) => {
                self.scope.add_level();
                for statement in statements {
                    self.generate_node(statement);
                }
                self.scope.remove_level();
            }
            ASTNode::FUNCTION { identifier, parameters, return_type, body } => {
                // Get identifier name
                let identifier_name = match identifier.as_ref() {
                    ASTNode::IDENTIFIER(name) => {name},
                    _ => panic!("Identifier missing from function statement!")
                };

                // Add scope level just for parameters
                self.scope.add_level();

                let parameter_scope_start_id = self.scope.next_mutable_id();

                let param_types: Vec<SymbolType> = parameters.iter().map(|param|  {
                    match param {
                        ASTNode::PARAMETER { identifier, datatype } => {
                            // Create symbol
                            let identifier_name = match identifier.as_ref() {
                                ASTNode::IDENTIFIER(name) => {name},
                                _ => panic!("Identifier missing from parameter statement!")
                            };

                            let datatype_name = match datatype.as_ref() {
                                Some(ASTNode::IDENTIFIER(name)) => {name.clone()},
                                None => "F64".to_string(), // TODO(Connor): Add type inferencing
                                _ => panic!("datatype missing from parameter statement!")
                            };

                            let symbol = BarracudaSymbol {
                                name: identifier_name.clone(),
                                symbol_type: SymbolType::from(datatype_name),
                                mutable: true,
                                scope_id: self.scope.next_mutable_id()
                            };

                            let symbol_type = symbol.symbol_type.clone();
                            self.scope.add_symbol(symbol); // Add symbol last to avoid recursive statement let a = a
                            symbol_type
                        }
                        _ => {panic!("Non parameter found in function list")}
                    }
                }).collect();

                let return_type = match return_type.borrow() {
                    Some(datatype) => {
                        let datatype_name = match datatype {
                            ASTNode::IDENTIFIER(name) => {name.clone()},
                            _ => panic!("Identifier missing from function statement!")
                        };
                        SymbolType::from(datatype_name)
                    },
                    None => SymbolType::Void
                };

                // Create Symbol
                let func_label = self.create_label();
                let symbol = BarracudaSymbol {
                    name: identifier_name.clone(),
                    symbol_type: SymbolType::Function{
                        func_label,
                        func_params: param_types,
                        func_return: Box::new(return_type)
                    },
                    mutable: false,
                    scope_id: parameter_scope_start_id
                };

                // Generate code
                let func_end_label = self.create_label();

                // Jump past function code
                self.comment(format!("FN DEF {}", symbol.name));
                self.reference(func_end_label);
                self.emit(BarracudaIR::Instruction(INSTRUCTION::GOTO));

                // Function body
                self.set_label(func_label);
                self.generate_node(body);

                // Return if reaches bottom
                self.emit(BarracudaIR::Instruction(INSTRUCTION::GOTO));
                self.set_label(func_end_label);
                self.comment(format!("FN DEF END"));
                self.scope.remove_level();

                // Added to scope after function disables recursion
                self.scope.add_symbol(symbol)
            }
            ASTNode::RETURN { expression } => {
                self.generate_node(expression); // Calculate expression
                self.emit(BarracudaIR::Operation(OP::SWAP)); // Swap with return address
                self.emit(BarracudaIR::Instruction(INSTRUCTION::GOTO)); // Goto return address
            }
            ASTNode::FUNC_CALL { identifier, arguments } => {
                let function_symbol = match identifier.as_ref() {
                    ASTNode::IDENTIFIER(name) => {
                        self.scope.get_symbol(name)
                    },
                    _ => panic!("Identifier missing from parameter statement!")
                }.unwrap();

                if let SymbolType::Function { func_label, func_params, func_return } = function_symbol.symbol_type {
                    let mut param_id = function_symbol.scope_id;

                    // Fill in parameters
                    if func_params.len() != arguments.len() {
                        panic!("{} takes {} parameters but {} were given",
                               function_symbol.name, func_params.len(), arguments.len());
                    }

                    self.comment(format!("FN CALL {}", function_symbol.name));

                    for (param, arg_expression) in func_params.iter().zip(arguments.iter()) {
                        self.emit(BarracudaIR::Value(param_id as f64));
                        self.generate_node(arg_expression); // Expression
                        self.emit(BarracudaIR::Operation(OP::STK_WRITE));
                        param_id += 1;
                    };

                    // Push stack pointer
                    let end_fncall_label = self.create_label();
                    self.reference(end_fncall_label);
                    self.reference(func_label);
                    self.emit(BarracudaIR::Instruction(INSTRUCTION::GOTO));
                    self.comment(format!("FN CALL END"));
                    self.set_label(end_fncall_label);

                } else {
                    panic!("Identifier cannot be called {}", function_symbol.name);
                }
            }
            _ => {unimplemented!()}
        }
    }
}


pub struct BarracudaByteCodeGenerator {}

impl BackEndGenerator for BarracudaByteCodeGenerator {
    fn default() -> Self {
        Self {}
    }

    fn generate(self, tree: AbstractSyntaxTree) -> ProgramCode {
        let mut generator = BarracudaBackend::new();
        let tree_root_node = tree.into_root();
        generator.generate(&tree_root_node)
    }
}
