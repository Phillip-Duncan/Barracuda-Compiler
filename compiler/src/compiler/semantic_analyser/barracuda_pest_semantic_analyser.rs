use std::process::id;

use crate::compiler::PrimitiveDataType;
use crate::compiler::ast::{Literal, datatype, UnaryOperation, BinaryOperation};
use crate::compiler::ast::datatype::DataType;

use super::{SemanticAnalyser, EnvironmentSymbolContext};
use super::super::ast::{
    AbstractSyntaxTree,
    ASTNode,
};



/// BarracudaSemanticAnalyser is a concrete SemanticAnalyser.
pub struct BarracudaSemanticAnalyser;

impl BarracudaSemanticAnalyser {

    /// Parses all pest pair tokens into a valid ASTNode
    fn analyse_node(self, node: &ASTNode) -> ASTNode {
        match node {
            ASTNode::IDENTIFIER(identifier_name) => {
                self.analyse_identifier(identifier_name) 
            }
            ASTNode::REFERENECE(identifier_name) => {
                self.analyse_reference(identifier_name)
            }
            ASTNode::DATATYPE(_) => {
                panic!("Malformed AST! Datatypes should not be directly analysed during type checking.");
            }
            ASTNode::LITERAL(literal) => {
                self.analyse_literal(literal)
            }
            ASTNode::ARRAY(items) => {
                self.analyse_array(items)
            }
            ASTNode::UNARY_OP { op, expression } => {
                self.analyse_unary_op(op, expression)
            }
            ASTNode::BINARY_OP { op, lhs, rhs } => {
                self.analyse_binary_op(op, lhs, rhs)
            }
            ASTNode::ARRAY_INDEX { index, expression } => {
                self.analyse_array_index(index, expression)
            }
            ASTNode::CONSTRUCT { identifier, datatype, expression } => {
                self.analyse_construct_statement(identifier, datatype, expression)
            }
            ASTNode::EXTERN { identifier } => {
                self.analyse_extern_statement(identifier)
            }
            ASTNode::ASSIGNMENT { identifier, array_index, expression } => {
                self.analyse_assignment_statement(identifier, array_index, expression)
            }
            ASTNode::PRINT { expression } => node.clone(),
            ASTNode::RETURN { expression } => {
                self.analyse_return_statement(expression)
            }
            ASTNode::BRANCH { condition, if_branch, else_branch } => {
                self.analyse_branch_statement(condition, if_branch, else_branch)
            }
            ASTNode::WHILE_LOOP { condition, body } => {
                self.analyse_while_statement(condition, body)
            }
            ASTNode::FOR_LOOP { initialization, condition, advancement, body } => {
                self.analyse_for_loop(initialization, condition, advancement, body)
            }
            ASTNode::PARAMETER { identifier, datatype } => {
                self.analyse_parameter(identifier, datatype)
            }
            ASTNode::FUNCTION { identifier, parameters, return_type, body } => {
                self.analyse_function_definition(identifier, parameters, return_type, body)
            }
            ASTNode::FUNC_CALL { identifier, arguments } => {
                self.analyse_function_call(identifier, arguments)
            }
            ASTNode::NAKED_FUNC_CALL { func_call } => {
                self.analyse_naked_function_call(func_call)
            }
            ASTNode::STATEMENT_LIST(statement_list) => {
                self.analyse_statement_list(statement_list)
            }
            ASTNode::SCOPE_BLOCK { inner, scope } => {
                self.analyse_scope_block(inner, scope);
            }
            ASTNode::TYPED_NODE { .. } => {
                panic!("Typed nodes are not implemened yet!");
            }
        };
    }

    fn mark_identifier_type(&mut self, name: &String, datatype: DataType) {
        panic!("Still need to do this");
    }

    fn type_from_identifier(&mut self, name: &String) -> DataType {
        panic!("Still need to do this");
    }

    fn type_from_node(&mut self, name: &ASTNode) -> DataType {
        panic!("Still need to do this");
    }

    fn analyse_identifier(&mut self, name: &String) -> ASTNode {
        let datatype = self.type_from_identifier(name);
        ASTNode::TYPED_NODE { datatype, inner: Box::new(ASTNode::IDENTIFIER(*name)) }
    }

    fn analyse_reference(&mut self, name: &String) -> ASTNode {
        let datatype = self.type_from_identifier(name);
        ASTNode::TYPED_NODE { 
            datatype: DataType::POINTER(Box::new(datatype)), 
            inner: Box::new(ASTNode::REFERENECE(*name)) 
        }
    }

    fn analyse_literal(&mut self, literal: &Literal) -> ASTNode {
        let datatype = match *literal {
            Literal::FLOAT(value) => DataType::CONST(PrimitiveDataType::F64),
            Literal::INTEGER(value) => DataType::CONST(PrimitiveDataType::I64),
            Literal::BOOL(value) => DataType::CONST(PrimitiveDataType::Bool)
        };

        ASTNode::TYPED_NODE { 
            datatype, 
            inner: Box::new(ASTNode::LITERAL(*literal))
        }
    }

    fn analyse_array(&mut self, items: &Vec<ASTNode>) -> ASTNode {
        if items.len() == 0 {
            panic!("Cannot create an array of length 0!")
        }
        let mut typed_items: Vec<ASTNode> = vec![];
        for item in items {
            typed_items.push(self.analyse_node(item))
        }
        let datatype = self.type_from_node(&typed_items[0]);
        for item in typed_items.iter().skip(1) {
            let datatype_2 = self.type_from_node(item); 
            if datatype != datatype_2 {
                panic!("Cannot create array with mismatched types!")
            }
        }
        ASTNode::TYPED_NODE { 
            datatype: DataType::ARRAY(Box::new(datatype), items.len()), 
            inner: Box::new(ASTNode::ARRAY(typed_items))
        }
    }

    fn analyse_unary_op(&mut self, op: &UnaryOperation, expression: &Box<ASTNode>) -> ASTNode {
        let expression = self.analyse_node(expression);
        let datatype = self.type_from_node(&expression);
        let datatype = match op {
            UnaryOperation::NOT | UnaryOperation::NEGATE => { 
                match datatype {
                    DataType::CONST(_) | DataType::MUTABLE(_) => datatype,
                    _ => panic!("Cannot use operation {:?} on type {:?}", op, datatype)
                }
            }
            UnaryOperation::PTR_DEREF => { 
                match datatype {
                    DataType::POINTER(inner_datatype) => *inner_datatype.as_ref(),
                    _ => panic!("Cannot use operation {:?} on type {:?}", op, datatype)
                }
            }
        };
        ASTNode::TYPED_NODE { 
            datatype,
            inner: Box::new(ASTNode::UNARY_OP { 
                op: op.clone(), 
                expression: Box::new(expression) 
            })
        }
    }

    fn analyse_binary_op(&mut self, op: &BinaryOperation, lhs: &Box<ASTNode>, rhs: &Box<ASTNode>) -> ASTNode {
        let lhs = self.analyse_node(lhs);
        let rhs = self.analyse_node(rhs);
        let lhs_datatype = self.type_from_node(&lhs);
        let rhs_datatype = self.type_from_node(&rhs);
        if lhs_datatype != rhs_datatype {
            panic!("Cannot perform operation {:?} with mismatched types!", op)
        }
        let datatype = lhs_datatype;
        
        let datatype = match op {
            BinaryOperation::ADD | BinaryOperation::SUB | BinaryOperation::DIV 
          | BinaryOperation::MUL | BinaryOperation::MOD | BinaryOperation::POW => { 
                match datatype {
                    DataType::CONST(_) | DataType::MUTABLE(_) => datatype,
                    _ => panic!("Cannot use operation {:?} on type {:?}", op, datatype)
                }
            }
            BinaryOperation::GREATER_THAN | BinaryOperation::LESS_THAN 
          | BinaryOperation::GREATER_EQUAL | BinaryOperation::LESS_EQUAL => { 
                match datatype {
                    DataType::CONST(_) | DataType::MUTABLE(_) => DataType::CONST(PrimitiveDataType::Bool),
                    _ => panic!("Cannot use operation {:?} on type {:?}", op, datatype)
                }
            }
            BinaryOperation::EQUAL | BinaryOperation::NOT_EQUAL => { 
                DataType::CONST(PrimitiveDataType::Bool)
            }
        };
        ASTNode::TYPED_NODE { 
            datatype,
            inner: Box::new(ASTNode::BINARY_OP { 
                op: op.clone(), 
                lhs: Box::new(lhs),
                rhs: Box::new(rhs)
            })
        }
    }

    fn analyse_array_index(&mut self, index: &Box<ASTNode>, expression: &Box<ASTNode>) -> ASTNode {
        let expression = Box::new(self.analyse_node(expression));
        let index = Box::new(self.analyse_node(index));
        let expression_datatype = self.type_from_node(&expression);
        let index_datatype = self.type_from_node(&index);
        // check index is a literal and expression is an array. Return array innards
        if let DataType::ARRAY(inner_type, _size) = index_datatype {
            if let DataType::CONST(_) | DataType::MUTABLE(_)  = expression_datatype {
                ASTNode::TYPED_NODE { 
                    datatype: inner_type.as_ref().clone(), 
                    inner: Box::new(ASTNode::ARRAY_INDEX { index, expression })
                }
            } else {
                panic!("Can only index arrays with literal values!")
            }
        } else {
            panic!("Can't index a non-array!")
        }
    }

    fn analyse_construct_statement(&mut self, identifier: &Box<ASTNode>, datatype: &Box<Option<ASTNode>>, expression: &Box<ASTNode>) -> ASTNode {
        let expression = Box::new(self.analyse_node(expression));
        let expression_datatype = self.type_from_node(&expression);
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            if let Some(datatype) = datatype.as_ref() {
                let datatype = match datatype {
                    ASTNode::DATATYPE(datatype) => datatype,
                    _ => panic!("Malformed AST! Node {:?} should have been a datatype but wasn't!", datatype)
                };
                if datatype != expression_datatype {
                    panic!("Provided data doesn't match given datatype in construct statement! {:?} vs {:?}", datatype, expression_datatype);
                }
                self.mark_identifier_type(name, datatype.clone());
                ASTNode::CONSTRUCT { identifier: identifier.clone(), datatype: Box::new(None), expression: expression.clone() }   
            } else {
                self.mark_identifier_type(name, expression_datatype);
                ASTNode::CONSTRUCT { identifier: identifier.clone(), datatype: Box::new(None), expression: expression.clone() }   
            }
        } else {
            panic!("Malformed AST! Construct statement should always start with an identifier")
        }
    }

    fn analyse_extern_statement(&mut self, identifier: &Box<ASTNode>) -> ASTNode {
        panic!("Still need to do this!");
    }

    fn analyse_assignment_statement(&mut self, identifier: &Box<ASTNode>, array_index: &Box<Option<ASTNode>>, expression: &Box<ASTNode>) -> ASTNode {
        panic!("Still need to do this!");
    }

    fn analyse_return_statement(&mut self, expression: &Box<ASTNode>) -> ASTNode {
        panic!("Still need to do this!");
    }

    fn analyse_branch_statement(&mut self, condition: &Box<ASTNode>, if_branch: &Box<ASTNode>, else_branch: &Box<Option<ASTNode>>) -> ASTNode {
        let condition = Box::new(self.analyse_node(condition));
        let if_branch = Box::new(self.analyse_node(if_branch));
        let else_branch = match else_branch.as_ref() {
            Some(else_branch) => {
                Box::new(Some(self.analyse_node(else_branch)))
            }
            None => Box::new(None)
        };
        let datatype = self.type_from_node(&condition);
        match datatype {
            DataType::CONST(_) | DataType::MUTABLE(_) => {},
            _ => panic!("Literal values must be used for if statement conditions!")
        }

        ASTNode::BRANCH { condition, if_branch, else_branch }
    }

    fn analyse_while_statement(&mut self, condition: &Box<ASTNode>, body: &Box<ASTNode>) -> ASTNode {
        let condition = Box::new(self.analyse_node(condition));
        let body = Box::new(self.analyse_node(body));
        let datatype = self.type_from_node(&condition);
        match datatype {
            DataType::CONST(_) | DataType::MUTABLE(_) => {},
            _ => panic!("Literal values must be used for while statement conditions!")
        }
        ASTNode::WHILE_LOOP { condition, body }
    }

    fn analyse_for_loop(&mut self, initialization: &Box<ASTNode>, condition: &Box<ASTNode>, advancement: &Box<ASTNode>, body: &Box<ASTNode>) {
        let initialization = Box::new(self.analyse_node(initialization));
        let condition = Box::new(self.analyse_node(condition));
        let advancement = Box::new(self.analyse_node(advancement));
        let body = Box::new(self.analyse_node(body));
        let datatype = self.type_from_node(&condition);
        match datatype {
            DataType::CONST(_) | DataType::MUTABLE(_) => {},
            _ => panic!("Literal values must be used for for statement conditions!")
        }
        ASTNode::FOR_LOOP { initialization, condition, advancement, body }
    }

    fn analyse_function_definition(&mut self, identifier: &Box<ASTNode>, parameters: &Vec<ASTNode>, _return_type: &Box<ASTNode>, body: &Box<ASTNode>) {
        panic!("Still need to do this!")
    }

    fn analyse_parameter(&mut self, identifier: &Box<ASTNode>, _datatype: &Box<Option<ASTNode>>) {
        let expression = Box::new(self.analyse_node(expression));
        let expression_datatype = self.type_from_node(&expression);
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            if let Some(datatype) = datatype.as_ref() {
                let datatype = match datatype {
                    ASTNode::DATATYPE(datatype) => datatype,
                    _ => panic!("Malformed AST! Node {:?} should have been a datatype but wasn't!", datatype)
                };
                if datatype != expression_datatype {
                    panic!("Provided data doesn't match given datatype in construct statement! {:?} vs {:?}", datatype, expression_datatype);
                }
                self.mark_identifier_type(name, datatype.clone());
                ASTNode::CONSTRUCT { identifier: identifier.clone(), datatype: Box::new(None), expression: expression.clone() }   
            } else {
                panic!("Malformed AST! Multiple dispatch is not implemented so functions must contain types")
            }
        } else {
            panic!("Malformed AST! Construct statement should always start with an identifier")
        }
    }

    fn analyse_builtin_functions(&mut self)
    {
        for func in BARRACUDA_BUILT_IN_FUNCTIONS {
            self.function_labels.insert(String::from(format!("__{}", func.to_string().to_lowercase())), vec![func.as_u32() as u64, 1]);
        }
    }

    fn analyse_function_call(&mut self, identifier: &Box<ASTNode>, arguments: &Vec<ASTNode>) {
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
                self.analyse_node(arg);
            }
            self.builder.emit_op(op);

            return;
        }

        // analyse Call Stack
        self.builder.comment(format!("FN CALL {} START", &identifier_name));

        // Push arguments onto the stack in reverse order
        for (i, arg) in arguments.iter().enumerate().rev() {
            self.builder.comment(format!("FN ARG {}", i));
            self.analyse_node(arg);
        }

        // Push return address
        self.builder.reference(function_call_end);

        // Push previous frame pointer
        self.analyse_get_frame_ptr();

        // Update frame pointer
        self.analyse_get_stack_ptr();
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
        self.analyse_get_return_store();
    }

    fn analyse_naked_function_call(&mut self, func_call: &Box<ASTNode>) {
        self.analyse_node(func_call);
        self.builder.emit_op(OP::DROP);
    }

    fn analyse_statement_list(&mut self, statements: &Vec<ASTNode>) {
        for statement in statements {
            self.analyse_node(statement);
        }
    }

    fn analyse_scope_block(&mut self, inner: &Box<ASTNode>, scope: &ScopeId) {
        self.symbol_tracker.enter_scope(scope.clone());
        self.analyse_node(inner);

        // Drop all local vars
        let symbols_dropped = self.symbol_tracker.exit_scope();
        for _ in 0..symbols_dropped {
            self.builder.emit_op(OP::DROP);
        }
    }

}

/// AstParser Trait Concrete Implementation 
impl SemanticAnalyser for BarracudaSemanticAnalyser {
    fn default() -> Self {
        Self {}
    }

    /// Parse processes a source string into an abstract syntax tree
    fn analyse(self, root_node: ASTNode, env_vars: EnvironmentSymbolContext) -> AbstractSyntaxTree { 
        AbstractSyntaxTree::new(BarracudaSemanticAnalyser::analyse_node(self, &root_node), env_vars)
    }
}