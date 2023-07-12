use super::BackEndGenerator;

use super::super::ast::{
    AbstractSyntaxTree,
    ASTNode,
    Literal,
    BinaryOperation,
    UnaryOperation
};

use barracuda_common::{
    ProgramCode,
    BarracudaInstructions as INSTRUCTION,
    FixedBarracudaOperators as OP,
};

use std::collections::HashMap;
use crate::compiler::ast::operators::LEGAL_POINTER_OPERATIONS;
use crate::compiler::ast::{
    ScopeId,
    ScopeTracker,
    symbol_table::{
        SymbolType
    }
};
use crate::compiler::backend::analysis::stack_estimator::StackEstimator;
use crate::compiler::backend::program_code_builder::BarracudaProgramCodeBuilder;

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

    function_labels: HashMap<String, u64>,

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
            max_analysis_branch_depth: 512,
        }
    }

    /// Generates ProgramCode from an Abstract Syntax Tree
    fn generate(mut self, tree: AbstractSyntaxTree) -> ProgramCode {
        // Create symbol tracker
        self.symbol_tracker = ScopeTracker::new(tree.get_symbol_table());

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
            f64::from_ne_bytes((Self::static_register_count() - 1).to_ne_bytes()),
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
        self.builder.emit_value(f64::from_ne_bytes(Self::frame_ptr_address().to_ne_bytes()));
        self.builder.emit_op(OP::STK_READ);
    }

    // Generate code to set stack pointer to the value on top of the stack.
    // Must add one as VM RCSTK_PTR is off by one.
    fn generate_set_stack_ptr(&mut self) {
        self.builder.emit_value(f64::from_ne_bytes(1_u64.to_ne_bytes()));
        self.builder.emit_op(OP::ADD_PTR);
        self.builder.emit_op(OP::RCSTK_PTR);
    }

    // Generate code to place stack pointer on top of the stack.
    // Must remove one as VM LDSTK_PTR is off by one.
    fn generate_get_stack_ptr(&mut self) {
        self.builder.emit_op(OP::LDSTK_PTR);
        self.builder.emit_value(f64::from_ne_bytes(1_u64.to_ne_bytes()));
        self.builder.emit_op(OP::SUB_PTR);
    }

    /// Generate code to push return store on the top of the stack
    fn generate_get_return_store(&mut self) {
        self.builder.emit_value(f64::from_ne_bytes(Self::return_store_address().to_ne_bytes()));
        self.builder.emit_op(OP::STK_READ);
    }

    /// Generate code to set return store with the result of an expression
    fn generate_set_return_store(&mut self, expression: &ASTNode) {
        self.builder.emit_value(f64::from_ne_bytes(Self::return_store_address().to_ne_bytes()));
        self.generate_node(expression);
        self.builder.emit_op(OP::STK_WRITE);
    }

    /// Generate code to push local variable stack address onto the top of the stack
    fn generate_local_var_address(&mut self, localvar_index: usize) {
        self.builder.emit_value(f64::from_ne_bytes((localvar_index + 1).to_ne_bytes())); // id
        self.generate_get_frame_ptr();
        self.builder.emit_op(OP::ADD_PTR);  // FRAME_PTR + (id + 1)
    }

    /// Generate code to push parameter stack address onto the top of the stack
    fn generate_parameter_address(&mut self, parameter_index: usize) {
        self.generate_get_frame_ptr();
        self.builder.emit_value(f64::from_ne_bytes((parameter_index + 2).to_ne_bytes())); // id
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
            ASTNode::IDENTIFIER(identifier_name) => {
                self.generate_identifier(identifier_name)
            }
            ASTNode::REFERENECE(identifier_name) => {
                self.generate_reference(identifier_name)
            }
            ASTNode::VARIABLE { references, identifier } => {
                self.generate_variable(references, identifier)
            }
            ASTNode::LITERAL(literal) => {
                self.generate_literal(literal)
            }
            ASTNode::UNARY_OP { op, expression } => {
                self.generate_unary_op(op, expression)
            }
            ASTNode::BINARY_OP { op, lhs, rhs } => {
                self.generate_binary_op(op, lhs, rhs)
            }
            ASTNode::CONSTRUCT { identifier, datatype, expression } => {
                self.generate_construct_statement(identifier, datatype, expression);
            }
            ASTNode::EXTERN { identifier } => {
                self.generate_extern_statement(identifier);
            }
            ASTNode::ASSIGNMENT { identifier, expression } => {
                self.generate_assignment_statement(identifier, expression)
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
            ASTNode::PARAMETER { identifier, datatype } => {
                self.generate_parameter(identifier, datatype)
            }
            ASTNode::FUNCTION { identifier, parameters, return_type, body } => {
                self.generate_function_definition(identifier, parameters, return_type, body)
            }
            ASTNode::FUNC_CALL { identifier, arguments } => {
                self.generate_function_call(identifier, arguments)
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
        };
    }

    fn generate_identifier(&mut self, name: &String) {
        let symbol_result = self.symbol_tracker.find_symbol(name).unwrap();

        match symbol_result.symbol_type() {
            SymbolType::Variable(_datatype, _) => {
                let localvar_id = self.symbol_tracker.get_local_id(name).unwrap();

                self.generate_local_var_address(localvar_id);
                self.builder.emit_op(OP::STK_READ);
            }
            SymbolType::EnvironmentVariable(global_id, _, qualifier) => {
                let ptr_depth = qualifier.matches("*").count();
                self.builder.emit_value(f64::from_be_bytes(global_id.to_be_bytes()));
                self.builder.emit_op(OP::LDNX);
                for _n in 0..ptr_depth {
                    if _n == ptr_depth - 1 {
                        self.builder.emit_op(OP::READ);
                    }
                    else {
                        self.builder.emit_op(OP::PTR_DEREF);
                    }
                }
            }
            SymbolType::Parameter(_datatype, _) => {
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
            SymbolType::Variable(_datatype, _) => {
                let localvar_id = self.symbol_tracker.get_local_id(name).unwrap();
                self.generate_local_var_address(localvar_id);
            }
            SymbolType::Parameter(_datatype, _) => {
                let param_id = self.symbol_tracker.get_param_id(name).unwrap();
                self.generate_parameter_address(param_id);
            }
            _ => {panic!("Symbol type does not contain meaning when referenced")}
        }
    }

    fn generate_variable(&mut self, references: &usize, identifier: &String) {
        self.generate_identifier(identifier);
        for _ in 0..*references {
            self.builder.emit_op(OP::STK_READ);
        }
    }

    fn generate_literal(&mut self, literal: &Literal) {
        let literal_value = match *literal {
            Literal::FLOAT(value) => { value }
            Literal::INTEGER(value) => { value as f64 }
            //Literal::STRING(_) => {
            //    unimplemented!()
            //    This would likely involve allocation on the heap
            //}
            Literal::BOOL(value) => { value as i64 as f64 }
        };

        self.builder.emit_value(literal_value);
    }

    fn generate_unary_op(&mut self, op: &UnaryOperation, expression: &Box<ASTNode>) {
        let pointer_level = self.get_pointer_level(&expression);
        if pointer_level != 0 {
            panic!("Pointers cannot be used with the operation '{:?}' !", op);
        }
        self.generate_node(expression);
        match op {
            UnaryOperation::NOT => { self.builder.emit_op(OP::NOT) }
            UnaryOperation::NEGATE => { self.builder.emit_op(OP::NEGATE) }
        };
    }

    fn generate_binary_op(&mut self, op: &BinaryOperation, lhs: &Box<ASTNode>, rhs: &Box<ASTNode>) {
        let lhs_pointer_level = self.get_pointer_level(&lhs);
        let rhs_pointer_level = self.get_pointer_level(&rhs);
        if lhs_pointer_level != 0 || rhs_pointer_level != 0 {
            if !LEGAL_POINTER_OPERATIONS.contains(&op) {
                panic!("Operation {:?} cannot be used with pointers!", op);
            }
        }
        if lhs_pointer_level != rhs_pointer_level {
            panic!("Pointer levels cannot be different in a binary operation! ({} vs {})", lhs_pointer_level, rhs_pointer_level);
        }
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
        };
    }

    fn generate_construct_statement(&mut self, identifier: &Box<ASTNode>, _datatype: &Box<Option<ASTNode>>, expression: &Box<ASTNode>) {
        let (identifier_pointer_level, identifier_name) = identifier.get_variable().unwrap();
        
        let expression_pointer_level = self.get_pointer_level(&expression);
        if identifier_pointer_level != expression_pointer_level {
            panic!("Pointer level of '{}' is different from pointer level of expression! ({} vs {})", identifier_name, identifier_pointer_level, expression_pointer_level);
        }
        // Leave result of expression at top of stack as this is the allocated
        // region for the local variable
        self.generate_node(expression);
        self.add_symbol(identifier_name.clone());

        // Comment local var id
        let local_var_id = self.symbol_tracker.get_local_id(&identifier_name).unwrap();
        self.builder.comment(format!("CONSTRUCT {}:{}", &identifier_name, local_var_id));
    }

    fn generate_extern_statement(&mut self, identifier: &Box<ASTNode>) {
        let identifier_name = identifier.identifier_name().unwrap();
        self.add_symbol(identifier_name.clone())
    }

    fn generate_assignment_statement(&mut self, identifier: &Box<ASTNode>, expression: &Box<ASTNode>) {
        let (references, identifier_name) = identifier.get_variable().unwrap();
        let lhs_pointer_level = self.get_pointer_level(&identifier);
        let rhs_pointer_level = self.get_pointer_level(&expression);
        if lhs_pointer_level != rhs_pointer_level {
            panic!("Pointer levels cannot be different in an assignment statement! Assigning to {} ({} vs {})", identifier_name, lhs_pointer_level, rhs_pointer_level);
        }
        if let Some(symbol) = self.symbol_tracker.find_symbol(&identifier_name) {
            match symbol.symbol_type() {
                SymbolType::Variable(_, _) => {
                    let local_var_id = self.symbol_tracker.get_local_id(&identifier_name).unwrap();

                    self.builder.comment(format!("ASSIGNMENT {}:{}", &identifier_name, local_var_id));
                    self.generate_local_var_address(local_var_id);
                    for _ in 0..references {
                        self.builder.emit_op(OP::STK_READ);
                    }
                    self.generate_node(expression);
                    self.builder.emit_op(OP::STK_WRITE);
                }
                SymbolType::EnvironmentVariable(global_id, _, qualifier) => {
                    self.builder.comment(format!("ASSIGNMENT {}:G{}", &identifier_name, global_id));
                    self.generate_node(expression);
                    if qualifier.contains("*") {
                        self.builder.emit_value(f64::from_be_bytes(global_id.to_be_bytes()));
                        self.builder.emit_op(OP::LDNX);
                        let ptr_depth = qualifier.matches("*").count();
                        for _n in 0..ptr_depth {
                            if _n == ptr_depth - 1 {
                                //self.builder.emit_op(OP::READ);
                                continue;
                            }
                            else {
                                self.builder.emit_op(OP::PTR_DEREF);
                            }
                        }
                        self.builder.emit_op(OP::SWAP);
                        self.builder.emit_op(OP::WRITE);
                    }
                    else {
                        self.builder.emit_value(f64::from_be_bytes(global_id.to_be_bytes()));
                        self.builder.emit_op(OP::RCNX);
                    }
                }
                SymbolType::Parameter(_, _) => {
                    let local_param_id = self.symbol_tracker.get_param_id(&identifier_name).unwrap();

                    self.builder.comment(format!("ASSIGNMENT {}:P{}", &identifier_name, local_param_id));
                    self.generate_parameter_address(local_param_id);
                    for _ in 0..references {
                        self.builder.emit_op(OP::STK_READ);
                    }
                    self.generate_node(expression);
                    self.builder.emit_op(OP::STK_WRITE);
                }
                SymbolType::Function { .. } => {
                    panic!("Cannot reassign a value to function '{}'", identifier_name);
                }
            }
        } else {
            panic!("Assignment identifier '{}' not recognised", identifier_name);
        }

    }

    fn generate_print_statement(&mut self, expression: &Box<ASTNode>) {
        self.builder.comment(format!("PRINT"));
        self.generate_node(expression);
        self.builder.emit_op(OP::PRINTFF);

        // New Line character
        self.builder.emit_value(10.0);
        self.builder.emit_op(OP::PRINTC);
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
        self.builder.emit_value(f64::from_ne_bytes(Self::frame_ptr_address().to_ne_bytes()));
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


                self.symbol_tracker.exit_scope();

                self.builder.emit_op(OP::DROP);
            }
            _ => panic!("Malformed for loop node!")
        };
    }

    fn generate_function_definition(&mut self, identifier: &Box<ASTNode>, parameters: &Vec<ASTNode>, _return_type: &Box<ASTNode>, body: &Box<ASTNode>) {

        let identifier_name = identifier.identifier_name().unwrap();

        // Create labels and assign them
        let function_def_start = self.builder.create_label();
        let function_def_end = self.builder.create_label();

        // Jump over function definition approaching from the top
        self.builder.reference(function_def_end);
        self.builder.emit_instruction(INSTRUCTION::GOTO);

        self.builder.comment(format!("FN {} START", &identifier_name));
        self.builder.set_label(function_def_start);

        // Generate body
        match body.as_ref() {
            ASTNode::SCOPE_BLOCK { inner, scope } => {
                self.symbol_tracker.enter_scope(scope.clone());

                // Process parameters into scope
                for parameter in parameters {
                    self.generate_node(parameter);
                }

                // Generate function body
                self.generate_node(inner);

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
        self.function_labels.insert(identifier_name.clone(), function_def_start);

    }

    fn generate_parameter(&mut self, identifier: &Box<ASTNode>, _datatype: &Box<Option<ASTNode>>) {
        println!("happy param! {:?}", identifier);
        let (_references, identifier_name) = identifier.get_variable().unwrap();
        self.add_symbol(identifier_name);
    }

    fn generate_function_call(&mut self, identifier: &Box<ASTNode>, arguments: &Vec<ASTNode>) {
        let identifier_name = identifier.identifier_name().unwrap();
        let function_def_label = self.function_labels.get(&identifier_name).unwrap().clone();
        let function_call_end = self.builder.create_label();

        // Generate Call Stack
        self.builder.comment(format!("FN CALL {} START", &identifier_name));
        // Push arguments onto the stack in reverse order
        for (i, arg) in arguments.iter().enumerate().rev() {
            self.builder.comment(format!("FN ARG {}", i));
            self.generate_node(arg)
        }

        // Push return address
        self.builder.reference(function_call_end);

        // Push previous frame pointer
        self.generate_get_frame_ptr();

        // Update frame pointer
        self.generate_get_stack_ptr();
        self.builder.emit_value(f64::from_ne_bytes(Self::frame_ptr_address().to_ne_bytes()));
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

    fn get_pointer_level(&mut self, node: &Box<ASTNode>) -> usize {
        match node.as_ref() {
            ASTNode::VARIABLE{references, identifier} => {
                match self.symbol_tracker.find_symbol(identifier).unwrap().symbol_type() {
                    SymbolType::Variable (_, ptr_level) | SymbolType::Parameter (_, ptr_level) => {
                        if references.clone() > ptr_level {
                            panic!("Can't dereference a non-pointer!")
                        }
                        ptr_level - (references.clone())
                    },
                    _ => 0
                }
            },
            ASTNode::REFERENECE(identifier) => {
                match self.symbol_tracker.find_symbol(identifier).unwrap().symbol_type() {
                    SymbolType::Variable (_, ptr_level) | SymbolType::Parameter (_, ptr_level) => ptr_level + 1,
                    _ => 0
                }
            },
            _ => 0
        }
    }

}