use super::BackEndGenerator;

use super::super::ast::{
    AbstractSyntaxTree,
    ASTNode,
    Literal,
    BinaryOperation,
    UnaryOperation
};
use super::builtin_functions::BARRACUDA_BUILT_IN_FUNCTIONS;

use barracuda_common::{
    ProgramCode,
    BarracudaInstructions as INSTRUCTION,
    FixedBarracudaOperators as OP,
};

use std::collections::HashMap;
use crate::compiler::ast::datatype::DataType;
use crate::compiler::ast::{
    ScopeId,
    ScopeTracker,
    symbol_table::SymbolType
};
use crate::compiler::backend::analysis::stack_estimator::StackEstimator;
use crate::compiler::backend::program_code_builder::BarracudaProgramCodeBuilder;
use crate::compiler::semantic_analyser::function_tracker::{FunctionTracker, FunctionImplementation};

use crate::compiler::PrimitiveDataType;
use crate::compiler::Qualifier;

/// BarracudaByteCodeGenerator is a Backend for Barracuda
/// It generates program code from an Abstract Syntax Tree
///
/// # Implementation Details
///   + Uses bottom of stack for two registers, Frame Pointer and Return Store.
///     + Frame Pointer, FP, keeps track of the current frame and points to the previous frame pointer
///       when within a function context. Otherwise points to itself.
///     + Return Store, RS, holds the return result of a function.
///
///   + Local Variables: Stored relative to a frame pointer, FP. Given a localvar_id the stack address
///     of a local variable can be calculated from FP+(index+1).
///
pub struct BarracudaByteCodeGenerator {
    builder: BarracudaProgramCodeBuilder,
    symbol_tracker: ScopeTracker,

    function_labels: HashMap<String, Vec<u64>>,
    functions: HashMap<String, FunctionTracker>,

    // Max analysis branching depth
    // used for estimating the stack depth of a program
    max_analysis_branch_depth: usize
}

impl BackEndGenerator for BarracudaByteCodeGenerator {
    /// Creates a default configuration of BarracudaByteCodeGenerator
    fn default() -> Self {
        Self {
            builder: BarracudaProgramCodeBuilder::new(),
            symbol_tracker: ScopeTracker::default(),
            function_labels: HashMap::default(),
            functions: HashMap::default(),
            max_analysis_branch_depth: 512,
        }
    }

    /// Generates ProgramCode from an Abstract Syntax Tree
    fn generate(mut self, tree: AbstractSyntaxTree) -> ProgramCode {
        // Create symbol tracker
        self.symbol_tracker = ScopeTracker::new(tree.get_symbol_table());
        self.functions = tree.get_functions();

        // Generate built-in functions
        self.generate_builtin_functions();

        // Generate program
        let tree_root_node = tree.into_root();
        self.builder.comment(String::from("PROGRAM START"));
        self.generate_node( &tree_root_node);

        // Finalise and attach variable header
        let header: Vec<f64> = vec![
            // Return Store Register
            0.0,

            // Frame pointer
            // must point to local_var:0 - 1
            f64::from_be_bytes((Self::static_register_count() - 1).to_be_bytes()),
        ];

        // Generate code
        let mut code = self.builder.finalize_with_header(header);

        // Estimate stack size
        let (stacksize, max_depth_reached) = StackEstimator::estimate_max_stacksize(&code, self.max_analysis_branch_depth);
        code.max_stack_size = if max_depth_reached {
            stacksize + Self::default_max_stacksize()
        } else {
            stacksize
        };


        return code;
    }

    fn add_environment_variable(&mut self) {
        self.builder.add_environment_variable();
    }

    fn set_precision(&mut self, precision: usize) {
        self.builder.set_precision(precision);
    }
}

/// # Description
///     + This implementation block holds functions related to managing the registers of this backend
///       as well as utilities for accessing addresses for parameters and local variables. The internal
///       implementation of these functions is kept opaque to allow for replacement with VM instructions
///       in the future as an optimisation.
///
/// # Stack Frame Structure
/// SP ->   ANONYMOUS VALUE N
///         ...
///         ANONYMOUS VALUE 1
///         ANONYMOUS VALUE 0
///         LOCAL VAR N
///         ...
///         LOCAL VAR 1
///         LOCAL VAR 0
/// FP ->   PREV FRAME PTR
///         RETURN ADDRESS
///         FUNC PARAMETER 0
///         FUNC PARAMETER 1
///         ...
///         FUNC PARAMETER N
///
/// # Key
///     + ANONYMOUS VALUE: Temporary computation values for instance for the sequence
///       push 4, push 5, add. The 4 and 5 would be added to the stack as anonymous values.
///     + LOCAL VAR: Are local variables within the current scope.
///     + PREV FRAME PTR: The previous frame pointer is stored so that the previous frame can
///       be restored after returning from a function context.
///     + RETURN ADDRESS: Stores the instruction address of the caller of a function. This allows
///       execution to return to the origin regardless of where a function is called.
///     + FUNC PARAMETER: Are the set function parameters to a function they can be called and
///       modified the same as variables they are just stored separately to simplify the function
///       call procedure.
impl BarracudaByteCodeGenerator {

    /// CONST FUNCTIONS
    const fn frame_ptr_address() -> usize { 1 }
    const fn return_store_address() -> usize { 0 }
    const fn static_register_count() -> usize { 2 }
    const fn default_max_stacksize() -> usize { 128 }

    // Generate code to push frame pointer on the top of the stack
    fn generate_get_frame_ptr(&mut self) {
        self.builder.emit_value(f64::from_be_bytes(Self::frame_ptr_address().to_be_bytes()));
        self.builder.emit_op(OP::STK_READ);
    }

    // Generate code to set stack pointer to the value on top of the stack.
    // Must add one as VM RCSTK_PTR is off by one.
    fn generate_set_stack_ptr(&mut self) {
        self.builder.emit_value(f64::from_be_bytes(1_u64.to_be_bytes()));
        self.builder.emit_op(OP::ADD_PTR);
        self.builder.emit_op(OP::RCSTK_PTR);
    }

    // Generate code to place stack pointer on top of the stack.
    // Must remove one as VM LDSTK_PTR is off by one.
    fn generate_get_stack_ptr(&mut self) {
        self.builder.emit_op(OP::LDSTK_PTR);
        self.builder.emit_value(f64::from_be_bytes(1_u64.to_be_bytes()));
        self.builder.emit_op(OP::SUB_PTR);
    }

    /// Generate code to push return store on the top of the stack
    fn generate_get_return_store(&mut self) {
        self.builder.emit_value(f64::from_be_bytes(Self::return_store_address().to_be_bytes()));
        self.builder.emit_op(OP::STK_READ);
    }

    /// Generate code to set return store with the result of an expression
    fn generate_set_return_store(&mut self, expression: &ASTNode) {
        self.builder.emit_value(f64::from_be_bytes(Self::return_store_address().to_be_bytes()));
        self.generate_node(expression);
        self.builder.emit_op(OP::STK_WRITE);
    }

    /// Generate code to push local variable stack address onto the top of the stack
    fn generate_local_var_address(&mut self, localvar_index: usize) {
        self.builder.emit_value(f64::from_be_bytes((localvar_index + 1).to_be_bytes())); // id
        self.generate_get_frame_ptr();
        self.builder.emit_op(OP::ADD_PTR);  // FRAME_PTR + (id + 1)
    }

    /// Generate code to push parameter stack address onto the top of the stack
    fn generate_parameter_address(&mut self, parameter_index: usize) {
        self.generate_get_frame_ptr();
        self.builder.emit_value(f64::from_be_bytes((parameter_index + 2).to_be_bytes())); // id
        self.builder.emit_op(OP::SUB_PTR);  // FRAME_PTR - (id + 1)
    }

    /// Add a symbol to symbol tracker to declare as existing in the backend context.
    /// This is done to ensure that only backend processed symbols are considered in scope.
    fn add_symbol(&mut self, name: String) {
        self.symbol_tracker.add_symbol(name.clone());
    }
}


/// # Description:
///     + This implementation block holds the business logic of generating ProgramCode using the
///       builder from ASTNodes. The general overview is unknown nodes / expression nodes are passed
///       into generate_node and are then unpacked and sent to more specific generator functions.
///
/// # Implementation Notes:
///     + This code is proof of concept and makes several assumptions around the structure of the
///       AST that will crash the program if malformed. All identifiers are assumed to exist and
///       the program will panic over symbolic errors here. In future development I would modify
///       the return type of these functions to return a Custom CompilerError struct so that it is
///       easier to inform callers on why programs failed to compile, rather than crashing.
impl BarracudaByteCodeGenerator {
    fn generate_node(&mut self, node: &ASTNode) {
        match node {
            ASTNode::TYPED_NODE { datatype, inner, qualifier } => match inner.as_ref() {
                ASTNode::IDENTIFIER(identifier_name) => {
                    self.generate_identifier(identifier_name)
                }
                ASTNode::REFERENCE(identifier_name) => {
                    self.generate_reference(identifier_name)
                }
                ASTNode::LITERAL(literal) => {
                    self.generate_literal(literal)
                }
                ASTNode::ARRAY { .. } => {
                    panic!("Arrays literals can only be used for direct assignment!");
                }
                ASTNode::UNARY_OP { op, expression } => {
                    self.generate_unary_op(op, expression)
                }
                ASTNode::BINARY_OP { op, lhs, rhs } => {
                    self.generate_binary_op(op, lhs, rhs)
                }
                ASTNode::TERNARY_OP { condition, true_branch, false_branch } => {
                    self.generate_ternary_op(condition, true_branch, false_branch)
                }
                ASTNode::ARRAY_INDEX { index, expression } => {
                    self.generate_array_index(index, expression, datatype, qualifier)
                }
                ASTNode::FUNC_CALL { identifier, arguments } => {
                    self.generate_function_call(identifier, arguments)
                }
                _ => panic!("Malformed AST! Node {:?} should not be inside a typed node.", node)
            }
            ASTNode::CONSTRUCT { identifier, expression, .. } => {
                self.generate_construct_statement(identifier, expression);
            }
            ASTNode::EMPTY_CONSTRUCT { identifier, .. } => {
                self.generate_empty_construct_statement(identifier);
            }
            ASTNode::EXTERN { identifier } => {
                self.generate_extern_statement(identifier);
            }
            ASTNode::ASSIGNMENT { identifier, pointer_level, array_index, expression } => {
                self.generate_assignment_statement(identifier, pointer_level.clone(), array_index, expression)
            }
            ASTNode::PRINT { expression } => {
                self.generate_print_statement(expression)
            }
            ASTNode::RETURN { expression } => {
                self.generate_return_statement(expression)
            }
            ASTNode::BRANCH { condition, if_branch, else_branch } => {
                self.generate_branch_statement(condition, if_branch, else_branch)
            }
            ASTNode::WHILE_LOOP { condition, body } => {
                self.generate_while_statement(condition, body)
            }
            ASTNode::FOR_LOOP { initialization, condition, advancement, body } => {
                self.generate_for_loop(initialization, condition, advancement, body)
            }
            ASTNode::FUNCTION { identifier, .. } => {
                self.generate_function_definition(identifier)
            }
            ASTNode::NAKED_FUNC_CALL { func_call } => {
                self.generate_naked_function_call(func_call)
            }
            ASTNode::STATEMENT_LIST(statement_list) => {
                self.generate_statement_list(statement_list)
            }
            ASTNode::SCOPE_BLOCK { inner, scope } => {
                self.generate_scope_block(inner, scope);
            }
            _ => {
                panic!("Malformed AST! Node {:?} should not be directly generated.", node);
            }
        };
    }

    fn generate_identifier_id(&mut self, name: &String) {
        let localvar_id = self.symbol_tracker.get_local_id(name).unwrap();
        self.generate_local_var_address(localvar_id);
        self.builder.emit_op(OP::STK_READ);
    }

    fn generate_identifier(&mut self, name: &String) {
        let symbol_result = self.symbol_tracker.find_symbol(name).unwrap();

        match symbol_result.symbol_type() {
            SymbolType::Variable(_,_) => {
                self.generate_identifier_id(name)
            }
            SymbolType::EnvironmentVariable( global_id, datatype, qualifier, ptr_levels) => {
                let ptr_depth = ptr_levels.matches("*").count();
                self.builder.emit_value(f64::from_be_bytes(global_id.to_be_bytes()));
                self.builder.emit_op(OP::LDNX);
                for _n in 0..ptr_depth {
                    if _n == ptr_depth - 1 {
                        match datatype {
                            DataType::PRIMITIVE(primitive) | DataType::ENVIRONMENTVARIABLE(primitive) => {
                                match primitive {
                                    PrimitiveDataType::F128 => panic!("F128 not currently supported in environment variables"),
                                    PrimitiveDataType::F64 => self.builder.emit_op(OP::READ_F64),
                                    PrimitiveDataType::F32 => self.builder.emit_op(OP::READ_F32),
                                    PrimitiveDataType::F16 => panic!("F16 not currently supported in environment variables"),
                                    PrimitiveDataType::F8 => panic!("F8 not currently supported in environment variables"),
                                    PrimitiveDataType::I128 => panic!("I128 not currently supported in environment variables"),
                                    PrimitiveDataType::I64 => self.builder.emit_op(OP::READ_I64),
                                    PrimitiveDataType::I32 => self.builder.emit_op(OP::READ_I32),
                                    PrimitiveDataType::I16 => panic!("I16 not currently supported in environment variables"),
                                    PrimitiveDataType::I8 => panic!("I8 not currently supported in environment variables"),
                                    PrimitiveDataType::Bool => self.builder.emit_op(OP::READ_F64),
                                    PrimitiveDataType::String => self.builder.emit_op(OP::READ_F64),
                                }
                            }
                            _ => panic!("Datatype {:?} must be a primitive!", datatype)
                        }
                    }
                    else {
                        self.builder.emit_op(OP::PTR_DEREF);
                    }
                }
            }
            SymbolType::Parameter(_datatype,_qualifier) => {
                let param_id = self.symbol_tracker.get_param_id(name).unwrap();
                self.generate_parameter_address(param_id);
                self.builder.emit_op(OP::STK_READ);
            }
            _ => {panic!("Symbol type does not contain meaning in expressions")}
        }
    }

    fn generate_reference(&mut self, name: &String) {
        let symbol_result = self.symbol_tracker.find_symbol(name).unwrap();

        match symbol_result.symbol_type() {
            SymbolType::Variable(_datatype,_qualifier) => {
                let localvar_id = self.symbol_tracker.get_local_id(name).unwrap();
                self.generate_local_var_address(localvar_id);
            }
            SymbolType::Parameter(_datatype,_qualifier) => {
                let param_id = self.symbol_tracker.get_param_id(name).unwrap();
                self.generate_parameter_address(param_id);
            }
            _ => {panic!("Symbol type does not contain meaning when referenced")}
        }
    }

    fn generate_literal(&mut self, literal: &Literal) {
        let literal_value = match *literal {
            Literal::FLOAT(value) => { value }
            Literal::INTEGER(value) => { value as f64 }
            Literal::BOOL(value) => { value as i64 as f64 }
            Literal::PACKEDSTRING(value) => { value }
        };

        self.builder.emit_value(literal_value);
    }

    fn generate_preallocated_array(&mut self, qualifier: &Box<ASTNode>, values: Vec<f64>, address: usize) {
        let qualifier = match qualifier.as_ref() {
            ASTNode::QUALIFIER(qualifier) => qualifier,
            _ => panic!("Expected a qualifier! Found {:?}", qualifier)
        };

        for (_, value) in values.iter().enumerate() {
            self.builder.emit_userspace(*value, qualifier.to_str().to_owned());
        }
        self.builder.emit_array(address, values.len(), qualifier.to_str().to_owned());
    }

    fn generate_array(&mut self, items: &Vec<ASTNode>, qualifier: &Box<ASTNode>, identifier: &String) {
        let address = self.symbol_tracker.get_array_id(identifier).unwrap();
        self.generate_subarray(items, qualifier, address, 0);

        let qualifier = match qualifier.as_ref() {
            ASTNode::QUALIFIER(qualifier) => qualifier,
            _ => panic!("Expected a qualifier! Found {:?}", qualifier)
        };

        self.builder.emit_array(address, items.len(), qualifier.to_str().to_owned());
    }

    fn generate_subarray(&mut self, items: &Vec<ASTNode>, qualifier: &Box<ASTNode>, address: usize, mut position: usize) -> usize {
        for item in items {
            match item {
                ASTNode::TYPED_NODE { inner, .. } => match inner.as_ref() {
                    ASTNode::ARRAY{items, qualifier} => position = self.generate_subarray(&items, &qualifier, address, position),
                    _ => position = {
                        let qual = match qualifier.as_ref() {
                            ASTNode::QUALIFIER(qualifier) => qualifier,
                            _ => panic!("Expected a qualifier! Found {:?}", qualifier)
                        };
                        self.builder.emit_array(address, 1, qual.to_str().to_owned());
                        self.generate_array_item(&item, position)
                    },
                }
                _ => position = {
                    let qual = match qualifier.as_ref() {
                        ASTNode::QUALIFIER(qualifier) => qualifier,
                        _ => panic!("Expected a qualifier! Found {:?}", qualifier)
                    };

                    self.builder.emit_array(address, 1, qual.to_str().to_owned());
                    self.generate_array_item(&item, position)
                },
            }
        }
        position
    }

    fn generate_array_item(&mut self, item: &ASTNode, position: usize) -> usize {
        self.builder.emit_value(f64::from_be_bytes(position.to_be_bytes()));
        self.builder.emit_op(OP::ADD_PTR);
        self.generate_node(item);
        self.builder.emit_op(OP::SWAP);
        self.builder.emit_op(OP::RCNX);
        position + 1
    }

    fn generate_unary_op(&mut self, op: &UnaryOperation, expression: &Box<ASTNode>) {
        self.generate_node(expression);
        match op {
            UnaryOperation::NOT => { self.builder.emit_op(OP::NOT) }
            UnaryOperation::NEGATE => { self.builder.emit_op(OP::NEGATE) }
            UnaryOperation::PTR_DEREF => { self.builder.emit_op(OP::STK_READ) }
        };
    }

    fn generate_binary_op(&mut self, op: &BinaryOperation, lhs: &Box<ASTNode>, rhs: &Box<ASTNode>) {
        self.generate_node(lhs);
        self.generate_node(rhs);
        match op {
            BinaryOperation::ADD   => { self.builder.emit_op(OP::ADD); }
            BinaryOperation::SUB   => { self.builder.emit_op(OP::SUB); }
            BinaryOperation::DIV   => { self.builder.emit_op(OP::DIV); }
            BinaryOperation::MUL   => { self.builder.emit_op(OP::MUL); }
            BinaryOperation::MOD   => { self.builder.emit_op(OP::FMOD); }
            BinaryOperation::POW   => { self.builder.emit_op(OP::POW); }
            BinaryOperation::EQUAL => { self.builder.emit_op(OP::EQ); }
            BinaryOperation::NOT_EQUAL => { self.builder.emit_op(OP::NEQ); }
            BinaryOperation::GREATER_THAN  => { self.builder.emit_op(OP::GT); }
            BinaryOperation::LESS_THAN     => { self.builder.emit_op(OP::LT); }
            BinaryOperation::GREATER_EQUAL => { self.builder.emit_op(OP::GTEQ); }
            BinaryOperation::LESS_EQUAL    => { self.builder.emit_op(OP::LTEQ); }
            BinaryOperation::AND => { self.builder.emit_op(OP::AND); }
            BinaryOperation::OR  => { self.builder.emit_op(OP::OR); }
            BinaryOperation::LSHIFT => { self.builder.emit_op(OP::LSHIFT); }
            BinaryOperation::RSHIFT => { self.builder.emit_op(OP::RSHIFT); }
        };
    }

    fn generate_ternary_op(&mut self, condition: &Box<ASTNode>, true_branch: &Box<ASTNode>, false_branch: &Box<ASTNode>) {
        self.generate_node(condition);
        self.generate_node(true_branch);
        self.generate_node(false_branch);
        self.builder.emit_op(OP::TERNARY);
    }

    fn generate_array_index(&mut self, index: &Box<ASTNode>, expression: &Box<ASTNode>, datatype: &DataType, _qualifier: &Qualifier) {   
        // Generate code to determine the index.
        self.generate_node(expression);
        self.generate_node(index);
    
        // Then, perform pointer arithmetic based on the datatype.
        match datatype {
            DataType::ARRAY(_, _) => {
                let array_length = DataType::get_array_length(&datatype);
                self.builder.emit_value(array_length as f64);
                self.builder.emit_op(OP::MUL_PTR);
                self.builder.emit_op(OP::DOUBLETOLONGLONG);
                self.builder.emit_op(OP::ADD_PTR);
            },
            _ => {
                self.builder.emit_op(OP::DOUBLETOLONGLONG);
                self.builder.emit_op(OP::ADD_PTR);
                let array_qualifier = expression.get_qualifier();
                match array_qualifier {
                    Qualifier::CONSTANT => {
                        self.builder.emit_op(OP::LDCUX);
                    },
                    Qualifier::MUTABLE => {
                        self.builder.emit_op(OP::LDNXPTR);
                        self.builder.emit_op(OP::READ_F64);
                    }
                }
            }
        }
    }
    

    fn generate_construct_statement(&mut self, identifier: &Box<ASTNode>, expression: &Box<ASTNode>) {
        let identifier_name = identifier.identifier_name().unwrap();
        self.add_symbol(identifier_name.clone());
    
        let datatype = identifier.get_type();
        match datatype {
            DataType::ARRAY(_, _) => {
                match expression.as_ref() {
                    ASTNode::TYPED_NODE { inner, .. } => match inner.as_ref() {
                        ASTNode::ARRAY {items, qualifier} => {
                            if self.is_static_array(items) {
                                // Preallocate with known values
                                let address = self.symbol_tracker.get_array_id(&identifier_name).unwrap();
                                let precomputed_values = self.get_array_values(items);
                                self.generate_preallocated_array(qualifier, precomputed_values, address);
                            } else {
                                self.generate_array(items, qualifier, &identifier_name);
                            }
                        }
                        _ => self.generate_node(expression)
                    },
                    _ => self.generate_node(expression)
                }
            },
            _ => {
                self.generate_node(expression);
            }
        }
    }

    fn generate_empty_construct_statement(&mut self, identifier: &Box<ASTNode>) {
        let identifier_name = identifier.identifier_name().unwrap();
        self.add_symbol(identifier_name.clone());
    
        let datatype = identifier.get_type();
        match datatype {
            DataType::ARRAY(_, _) => {
                let array_size = DataType::get_array_length(&datatype); // Fetch array size
                let address = self.symbol_tracker.get_array_id(&identifier_name).unwrap();
                
                // Emit code to allocate memory for the array
                let qualifier = identifier.get_qualifier();
                self.generate_preallocated_array(&Box::new(ASTNode::QUALIFIER(qualifier)), vec![0.0; array_size], address);
            },
            _ => {
                self.builder.emit_value(0.0);
            }
        }
    }

    fn generate_extern_statement(&mut self, identifier: &Box<ASTNode>) {
        //self.builder.add_environment_variable();  Phill: This was bad, was only adding env_vars per extern statement and not based on total number available. This caused index issues.
        let identifier_name = identifier.identifier_name().unwrap();
        self.add_symbol(identifier_name.clone())
    }

    fn generate_assignment_statement(&mut self, identifier: &Box<ASTNode>, pointer_level: usize, array_index: &Vec<ASTNode>, expression: &Box<ASTNode>) {
        let identifier_name = identifier.identifier_name().unwrap();

        if let Some(symbol) = self.symbol_tracker.find_symbol(&identifier_name) {
            match symbol.symbol_type() {
                SymbolType::Variable(datatype, _) => {
                    let local_var_id = self.symbol_tracker.get_local_id(&identifier_name).unwrap();
                    self.generate_local_var_address(local_var_id);
                    match datatype {
                        DataType::ARRAY(_,_) => {
                            self.builder.emit_op(OP::STK_READ);
                            self.generate_array_assignment_statement(array_index, expression, datatype)
                        },
                        _ => self.generate_regular_assignment_statement(expression, array_index, datatype, pointer_level)
                    }
                }
                SymbolType::EnvironmentVariable(global_id, datatype, qualifier, ptr_levels) => {
                    self.builder.comment(format!("ASSIGNMENT {}:G{}", &identifier_name, global_id));
                    self.generate_node(expression);
                    if ptr_levels.contains("*") {
                        self.builder.emit_value(f64::from_be_bytes(global_id.to_be_bytes()));
                        self.builder.emit_op(OP::LDNX);
                        let ptr_depth = ptr_levels.matches("*").count();
                        for _n in 0..ptr_depth {
                            if _n == ptr_depth - 1 {
                                continue;
                            }
                            else {
                                self.builder.emit_op(OP::PTR_DEREF);
                            }
                        }
                        for index in array_index {
                            self.generate_node(index);
                            self.builder.emit_op(OP::LDNT);
                            self.builder.emit_op(OP::LONGLONGTODOUBLE);
                            self.builder.emit_op(OP::MUL_PTR);

                            // Need to multiply by N (element bit-width of pointer)
                            //Phillip: I don't think this is correct since when casting for write/read it will automatically convert the integer to the correct pointer position.
                            //match datatype { 
                            //    DataType::CONST(primitive) | DataType::MUTABLE(primitive) | DataType::ENVIRONMENTVARIABLE(primitive) => {
                            //        self.builder.emit_value(primitive.size() as f64);
                            //        
                            //    }
                            //    _ => panic!("Datatype {:?} should be a primitive!", datatype)
                            //}
                            //self.builder.emit_op(OP::MUL_PTR);
                            //self.builder.emit_op(OP::DOUBLETOLONGLONG);
                            self.builder.emit_op(OP::ADD_PTR);
                        }
                        self.builder.emit_op(OP::SWAP);
                        self.builder.emit_op(OP::WRITE);
                    }
                    else {
                        self.builder.emit_value(f64::from_be_bytes(global_id.to_be_bytes()));
                        self.builder.emit_op(OP::RCNX); // TODO: Implement constant memory for environment variables
                    }
                }
                SymbolType::Parameter(datatype, _) => {
                    let local_param_id = self.symbol_tracker.get_param_id(&identifier_name).unwrap();
                    self.generate_parameter_address(local_param_id);
                    self.generate_regular_assignment_statement(expression, array_index, datatype, pointer_level);
                }
                SymbolType::Function { .. } => {
                    panic!("Cannot reassign a value to function '{}'", identifier_name);
                }
            }
        } else {
            panic!("Assignment identifier '{}' not recognised", identifier_name);
        }
    }

    fn generate_array_assignment_statement(&mut self, array_index: &Vec<ASTNode>, expression: &ASTNode, mut datatype: DataType) {
        //we have pointer as usize on the stack
        for index in array_index {
            datatype = match datatype {
                DataType::ARRAY(inner, _) => {
                    self.generate_node(index);
                    let array_length = DataType::get_array_length(&inner);
                    if array_length > 1 {
                        self.builder.emit_value(array_length as f64); // avoid unneccessary double to longlong conversion
                        self.builder.emit_op(OP::MUL_PTR);
                    }
                    self.builder.emit_op(OP::DOUBLETOLONGLONG);
                    self.builder.emit_op(OP::ADD_PTR);
                    *inner
                },
                _ => panic!("Datatype {:?} should be an array!", datatype)
            }
        }

        match datatype {
            DataType::ARRAY(_, _) => match expression {
                ASTNode::TYPED_NODE { inner, .. } => match inner.as_ref() {
                    ASTNode::ARRAY{items, ..} => {self.generate_array_assignment(items, 0);},
                    _ => panic!("Expected an array! Found {:?}", expression)
                },
                _ => panic!("Expected an array! Found {:?}", expression)
            }
            _ => {
                self.generate_node(expression);
                self.builder.emit_op(OP::SWAP);
                self.builder.emit_op(OP::RCNX);
            }
        }
    }

    fn generate_array_assignment(&mut self, items: &Vec<ASTNode>, mut position: usize) -> usize {
        for item in items {
            match item {
                ASTNode::TYPED_NODE { inner, .. } => match inner.as_ref() {
                    ASTNode::ARRAY{items, ..} => position = self.generate_array_assignment(&items, position),
                    _ => {
                        self.builder.emit_op(OP::DUP);
                        position = self.generate_array_item(&item, position);
                    },
                }
                _ => {
                    self.builder.emit_op(OP::DUP);
                    position = self.generate_array_item(&item, position);
                },
            }
        }
        position
    }

    fn generate_regular_assignment_statement(&mut self, expression: &ASTNode, array_index: &Vec<ASTNode>, mut datatype: DataType, pointer_level: usize) {
        for _ in 0..pointer_level {
            datatype = match datatype {
                DataType::POINTER(inner) => *inner,
                _ => panic!("Datatype {:?} should be a pointer!", datatype)
            };
            self.builder.emit_op(OP::STK_READ);
        }
        match datatype {
            DataType::ARRAY(_, _) => {
                self.generate_array_assignment_statement(array_index, expression, datatype);
            }
            _ => {
                self.generate_node(expression);
                self.builder.emit_op(OP::STK_WRITE);
            }
        }
    }

    fn generate_print_statement(&mut self, expression: &Box<ASTNode>) {
        self.builder.comment(format!("PRINT"));
        self.generate_node(expression);

        match expression.as_ref() {
            ASTNode::TYPED_NODE { datatype, qualifier, .. } => {
                match datatype {
                    DataType::ARRAY(sub_datatype, size) => {
                        match **sub_datatype {
                            DataType::PRIMITIVE(primitive) => {
                                match primitive {
                                    PrimitiveDataType::F8 | PrimitiveDataType::F16 | PrimitiveDataType::F32 | PrimitiveDataType::F64 | PrimitiveDataType::F128 => {
                                        // Iterate over array and print each element
                                        for i in 0..*size {
                                            self.builder.emit_op(OP::DUP); // Duplicate the position of start of the array.
                                            self.builder.emit_value(f64::from_be_bytes(i.to_be_bytes())); // Load the index
                                            self.builder.emit_op(OP::ADD_PTR); // Add the index to the address
                                            match qualifier {
                                                Qualifier::CONSTANT => {
                                                    self.builder.emit_op(OP::LDCUX); // Load the address of the element
                                                }
                                                Qualifier::MUTABLE => {
                                                    self.builder.emit_op(OP::LDNXPTR); // Load the address of the element
                                                    self.builder.emit_op(OP::READ_F64); // Read the element
                                                }
                                            }
                                            self.builder.emit_op(OP::PRINTFF); // Print the value
                                        }
                                    }
                                    PrimitiveDataType::String => {
                                        // Iterate over array and print each element
                                        for i in 0..*size {
                                            self.builder.emit_op(OP::DUP); // Duplicate the position of start of the array.
                                            self.builder.emit_value(f64::from_be_bytes(i.to_be_bytes())); // Load the index
                                            self.builder.emit_op(OP::ADD_PTR); // Add the index to the address
                                            match qualifier {
                                                Qualifier::CONSTANT => {
                                                    self.builder.emit_op(OP::LDCUX); // Load the address of the element
                                                }
                                                Qualifier::MUTABLE => {
                                                    self.builder.emit_op(OP::LDNXPTR); // Load the address of the element
                                                    self.builder.emit_op(OP::READ_F64); // Read the element
                                                }
                                            }
                                            self.builder.emit_op(OP::PRINTC); // Print the value
                                        }
                                        self.builder.emit_op(OP::DROP); // Remove the duplicated base address.
                                    }
                                    _ => {
                                        panic!("Cannot print array of primitive type {:?}", primitive);
                                    }
                                }
                            }
                            _ => {
                                panic!("Cannot print array of type {:?}", sub_datatype);
                            }

                        }
                    },
                    DataType::PRIMITIVE(primitive) => {
                        // TODO: Print logic in here doesn't work well for const arrays.
                        match primitive {
                            PrimitiveDataType::F8 | PrimitiveDataType::F16 | PrimitiveDataType::F32 | PrimitiveDataType::F64 | PrimitiveDataType::F128 => {
                                self.builder.emit_op(OP::PRINTFF);
                            }
                            PrimitiveDataType::String => {
                                self.builder.emit_op(OP::PRINTC);
                            }
                            _ => {
                                panic!("Cannot print primitive type {:?}", primitive);
                            }
                        }
                    },
                    _ => {
                        panic!("Cannot print type {:?}", datatype);
                    }
                }
            }
            _ => {
                panic!("Expression {:?} is not printable", expression);
            }
        }
    }

    fn generate_return_statement(&mut self, expression: &Box<ASTNode>) {
        // Store return result in register
        self.generate_set_return_store(expression);
        self.generate_return_handler();
    }

    fn generate_return_handler(&mut self) {
        self.builder.comment(String::from("RETURN HANDLER START"));

        // Set stack pointer to frame ptr
        self.generate_get_frame_ptr();
        self.generate_set_stack_ptr();

        // Set frame ptr to old frame ptr
        self.builder.emit_value(f64::from_be_bytes(Self::frame_ptr_address().to_be_bytes()));
        self.builder.emit_op(OP::SWAP);
        self.builder.emit_op(OP::STK_WRITE);

        // GOTO return address
        self.builder.emit_instruction(INSTRUCTION::GOTO);

        self.builder.comment(String::from("RETURN HANDLER END"));
    }

    fn generate_branch_statement(&mut self, condition: &Box<ASTNode>, if_branch: &Box<ASTNode>, else_branch: &Box<Option<ASTNode>>) {
        let if_end = self.builder.create_label();

        // Conditional Jump
        self.builder.comment(String::from("IF CONDITION"));
        self.generate_node(condition);
        self.builder.reference(if_end);
        self.builder.emit_instruction(INSTRUCTION::GOTO_IF);

        // If condition != 0
        // Generate if block
        self.builder.comment(String::from("IF BRANCH"));
        self.generate_node(if_branch);

        // If condition == 0, i.e Else
        match else_branch.as_ref() {
            None => {
                self.builder.set_label(if_end);
            },
            Some(else_branch) => {
                let else_end = self.builder.create_label();

                // Skip else block if encountered after running if block
                self.builder.reference(else_end);
                self.builder.emit_instruction(INSTRUCTION::GOTO);
                self.builder.set_label(if_end);

                // Generate else block
                self.builder.comment(String::from("ELSE BRANCH"));
                self.generate_node(else_branch);
                self.builder.set_label(else_end);
            }
        }
        self.builder.comment(String::from("IF END"));
    }

    fn generate_while_statement(&mut self, condition: &Box<ASTNode>, body: &Box<ASTNode>) {
        let while_start = self.builder.create_label();
        let while_exit = self.builder.create_label();

        // Start
        self.builder.set_label(while_start);

        // Generate condition
        self.builder.comment(String::from("WHILE CONDITION"));
        self.generate_node(condition);
        self.builder.reference(while_exit);
        self.builder.emit_instruction(INSTRUCTION::GOTO_IF);

        // Generate body
        self.builder.comment(String::from("WHILE BODY"));
        self.generate_node(body);

        // Loop back to condition after body
        self.builder.reference(while_start);
        self.builder.emit_instruction(INSTRUCTION::GOTO);

        // Exit
        self.builder.set_label(while_exit);
        self.builder.comment(String::from("WHILE END"));

    }

    fn generate_for_loop(&mut self, initialization: &Box<ASTNode>, condition: &Box<ASTNode>, advancement: &Box<ASTNode>, body: &Box<ASTNode>) {
        // Generate body
        match body.as_ref() {
            ASTNode::SCOPE_BLOCK { inner, scope } => {
                self.symbol_tracker.enter_scope(scope.clone());

                let for_start = self.builder.create_label();
                let for_exit = self.builder.create_label();

                // Start
                self.builder.comment(String::from("FOR INIT"));
                self.generate_node(initialization);
                self.builder.set_label(for_start);

                // Condition
                self.builder.comment(String::from("FOR CONDITION"));
                self.generate_node(condition);
                self.builder.reference(for_exit);
                self.builder.emit_instruction(INSTRUCTION::GOTO_IF);

                // Generate Body
                self.builder.comment(String::from("FOR BODY"));
                self.generate_node(inner);

                self.builder.comment(String::from("FOR ADVANCE"));
                self.generate_node(advancement);

                // Loop back to condition after body
                self.builder.reference(for_start);
                self.builder.emit_instruction(INSTRUCTION::GOTO);

                // Exit
                self.builder.set_label(for_exit);
                self.builder.comment(String::from("FOR END"));

                self.builder.emit_op(OP::DROP);

                self.symbol_tracker.exit_scope();
            }
            _ => panic!("Malformed for loop node!")
        };
    }

    fn generate_function_definition(&mut self, identifier: &Box<ASTNode>) {
        let identifier_name = identifier.identifier_name().unwrap();
        let implementations = self.functions.get(&identifier_name).unwrap().get_implementations().clone();
        for implementation in implementations {
            self.generate_function_implementation(implementation)
        }
    }
    
    fn generate_function_implementation(&mut self, implementation: FunctionImplementation) {
        let identifier_name = implementation.get_name();
        // Create labels and assign them
        let function_def_start = self.builder.create_label();
        let function_def_end = self.builder.create_label();

        // Jump over function definition approaching from the top
        self.builder.reference(function_def_end);
        self.builder.emit_instruction(INSTRUCTION::GOTO);

        self.builder.comment(format!("FN {} START", &identifier_name));
        self.builder.set_label(function_def_start);

        let body = implementation.get_body();
        let parameter_names = implementation.get_parameters();
        // Generate body
        match body {
            ASTNode::SCOPE_BLOCK { inner, scope } => {
                self.symbol_tracker.enter_scope(scope.clone());

                // Process parameters into scope
                for identifier in parameter_names {
                    self.generate_parameter(identifier.clone());
                }

                // Generate function body
                self.generate_node(&inner);

                self.symbol_tracker.exit_scope();
            }
            _ => panic!("Malformed function node!")
        };

        // Return if reaches end
        self.generate_return_handler();
        self.builder.set_label(function_def_end);
        self.builder.comment(format!("FN {} END", &identifier_name));

        // Add function symbol (Done after function to disallow recursion as it doesn't work at the moment)
        if self.symbol_tracker.find_symbol(&identifier_name).is_some() {
            panic!("Identifier `{}` can't be assigned to function as it already exists!", identifier_name);
        }
        self.add_symbol(identifier_name.clone());
        self.function_labels.insert(identifier_name.clone(), vec![function_def_start, 0]);

    }

    fn generate_parameter(&mut self, identifier: String) {
        self.add_symbol(identifier);
    }

    fn generate_builtin_functions(&mut self)
    {
        for func in BARRACUDA_BUILT_IN_FUNCTIONS {
            self.function_labels.insert(String::from(format!("__{}", func.to_string().to_lowercase())), vec![func.as_u32() as u64, 1]);
        }
    }

    fn generate_function_call(&mut self, identifier: &Box<ASTNode>, arguments: &Vec<ASTNode>) {
        let identifier_name = identifier.identifier_name().unwrap();
        let function_def_label = self.function_labels.get(&identifier_name).unwrap().clone()[0];
        let function_builtin_label = self.function_labels.get(&identifier_name).unwrap().clone()[1];
        let function_call_end = self.builder.create_label();

        if function_builtin_label == 1 {
            self.builder.comment(format!("BUILT-IN FN CALL {} START", &identifier_name));
            
            let op = OP::from(function_def_label as u32).unwrap();

            // Make sure number of arguments consumed is equal to the number of args supplied.
            if op.consume() != (arguments.len() as i8) {
                panic!("Invalid number of arguments ({}) supplied to builtin function __{} that requires {} arguments", arguments.len().to_string(), op.to_string().to_lowercase(), op.consume().to_string());
            }
            // Push arguments onto the stack in reverse order
            for (i, arg) in arguments.iter().enumerate().rev() {
                self.builder.comment(format!("FN ARG {}", i));
                self.generate_node(arg);
            }
            self.builder.emit_op(op);

            return;
        }

        // Generate Call Stack
        self.builder.comment(format!("FN CALL {} START", &identifier_name));

        // Push arguments onto the stack in reverse order
        for (i, arg) in arguments.iter().enumerate().rev() {
            self.builder.comment(format!("FN ARG {}", i));
            self.generate_node(arg);
        }

        // Push return address
        self.builder.reference(function_call_end);

        // Push previous frame pointer
        self.generate_get_frame_ptr();

        // Update frame pointer
        self.generate_get_stack_ptr();
        self.builder.emit_value(f64::from_be_bytes(Self::frame_ptr_address().to_be_bytes()));
        self.builder.emit_op(OP::SWAP);
        self.builder.emit_op(OP::STK_WRITE);

        // Jump into function definition
        self.builder.comment(format!("GOTO FN DEF"));
        self.builder.reference(function_def_label);
        self.builder.emit_instruction(INSTRUCTION::GOTO);
        self.builder.set_label(function_call_end);


        // Clean up arguments on stack
        self.builder.comment(format!("DROP ARGS"));
        for _ in 0..arguments.len() {
            self.builder.emit_op(OP::DROP);
        }

        self.builder.comment(format!("FN CALL {} END", &identifier_name));

        // Push return onto stack
        self.generate_get_return_store();
    }

    fn generate_naked_function_call(&mut self, func_call: &Box<ASTNode>) {
        self.generate_node(func_call);
        self.builder.emit_op(OP::DROP);
    }

    fn generate_statement_list(&mut self, statements: &Vec<ASTNode>) {
        for statement in statements {
            self.generate_node(statement);
        }
    }

    fn generate_scope_block(&mut self, inner: &Box<ASTNode>, scope: &ScopeId) {
        self.symbol_tracker.enter_scope(scope.clone());
        self.generate_node(inner);

        // Drop all local vars
        let symbols_dropped = self.symbol_tracker.exit_scope();
        for _ in 0..symbols_dropped {
            self.builder.emit_op(OP::DROP);
        }
    }


    fn is_static_array(&self, items: &Vec<ASTNode>) -> bool {
        // Check if all items in the array are compile-time constants (literals)
        for item in items {
            match item {
                ASTNode::TYPED_NODE { inner, .. } => match inner.as_ref() {
                    ASTNode::LITERAL(_) => continue,
                    ASTNode::UNARY_OP { expression, .. } => {
                        if self.is_literal_expression(expression) {
                            continue;
                        } else {
                            return false;
                        }
                    },
                    ASTNode::ARRAY{items, ..} => {
                        // Recursively check if inner arrays are static
                        if !self.is_static_array(items) {
                            return false;
                        }
                    }
                    _ => return false,
                }
                _ => return false,
            }
        }
        true
    }

    fn get_array_values(&self, items: &Vec<ASTNode>) -> Vec<f64> {
        let mut values = Vec::new();
        for item in items {
            match item {
                ASTNode::TYPED_NODE { inner, .. } => match inner.as_ref() {
                    ASTNode::LITERAL(literal) => values.push(self.extract_literal_value(literal)),
                    ASTNode::UNARY_OP { op, expression } => {
                        if let Some(value) = self.evaluate_unary_operation(op, expression) {
                            values.push(value);
                        } else {
                            panic!("Non-literal value in static array!");
                        }
                    },
                    ASTNode::ARRAY{items, ..} => {
                        let mut sub_values = self.get_array_values(items);
                        values.append(&mut sub_values);
                    }
                    _ => panic!("Non-literal value in static array!"),
                }
                _ => panic!("Non-literal value in static array!"),
            }
        }
        values
    }
    
    
    fn extract_literal_value(&self, literal: &Literal) -> f64 {
        match *literal {
            Literal::FLOAT(value) => value,
            Literal::INTEGER(value) => value as f64,
            Literal::BOOL(value) => value as i64 as f64,
            Literal::PACKEDSTRING(value) => value,  // If you have packed strings as floats
        }
    }

    fn is_literal_expression(&self, expression: &Box<ASTNode>) -> bool {
        match expression.as_ref() {
            ASTNode::TYPED_NODE { inner, .. } => match inner.as_ref() {
                ASTNode::LITERAL(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn evaluate_unary_operation(&self, op: &UnaryOperation, expression: &Box<ASTNode>) -> Option<f64> {
        if let ASTNode::TYPED_NODE { inner, .. } = expression.as_ref() {
            if let ASTNode::LITERAL(literal) = inner.as_ref() {
                let value = self.extract_literal_value(literal);
                match op {
                    UnaryOperation::NEGATE => Some(-value),
                    UnaryOperation::NOT => Some((value == 0.0) as i64 as f64),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /*
    fn evaluate_compile_time_value(&self, node: &ASTNode) -> Option<i64> {
        match node {
            // If it's a literal integer, return the value
            ASTNode::LITERAL(Literal::INTEGER(value)) => Some(*value),
    
            // If it's a variable or loop counter known at compile time, retrieve its value
            ASTNode::IDENTIFIER(name) => {
                if let Some(value) = self.symbol_tracker.get_compile_time_value(name) {
                    Some(value as i64)
                } else {
                    None
                }
            }
    
            // If it's a binary operation (e.g., loop advancement), try to evaluate it
            ASTNode::BINARY_OP { op, lhs, rhs } => {
                if let (Some(lhs_value), Some(rhs_value)) = (self.evaluate_compile_time_value(lhs), self.evaluate_compile_time_value(rhs)) {
                    match op {
                        BinaryOperation::ADD => Some(lhs_value + rhs_value),
                        BinaryOperation::SUB => Some(lhs_value - rhs_value),
                        BinaryOperation::MUL => Some(lhs_value * rhs_value),
                        BinaryOperation::DIV => Some(lhs_value / rhs_value),
                        _ => None
                    }
                } else {
                    None
                }
            }
    
            // Other cases may not be evaluable at compile time
            _ => None,
        }
    }*/

}