use std::collections::HashMap;

use crate::compiler::PrimitiveDataType;
use crate::compiler::ast::qualifiers::Qualifier;
use crate::compiler::ast::scope::ScopeIdGenerator;
use crate::compiler::ast::symbol_table::SymbolType;
use crate::compiler::ast::{Literal, UnaryOperation, BinaryOperation};
use crate::compiler::ast::datatype::DataType;
use crate::compiler::backend::builtin_functions::BARRACUDA_BUILT_IN_FUNCTIONS;

use super::function_tracker::FunctionTracker;
use super::scope_tracker::ScopeTracker;
use super::{SemanticAnalyser, EnvironmentSymbolContext};
use super::super::ast::{
    AbstractSyntaxTree,
    ASTNode,
};



/// BarracudaSemanticAnalyser is a concrete SemanticAnalyser.
pub struct BarracudaSemanticAnalyser {
    symbol_tracker: ScopeTracker,
    scope_counter: ScopeIdGenerator,
    env_vars: HashMap<String, (usize, PrimitiveDataType, Qualifier, String)>,
    functions: HashMap<String, FunctionTracker>
}

impl BarracudaSemanticAnalyser {
 
    /// Parses all pest pair tokens into a valid ASTNode
    pub fn analyse_node(&mut self, node: &ASTNode) -> ASTNode {
        match node {
            ASTNode::IDENTIFIER(identifier_name) => {
                self.analyse_identifier(identifier_name) 
            }
            ASTNode::REFERENCE(identifier_name) => {
                self.analyse_reference(identifier_name)
            }
            ASTNode::DATATYPE(_) => {
                panic!("Malformed AST! Datatypes should not be directly analysed during type checking.");
            }
            ASTNode::QUALIFIER(_) => {
                panic!("Malformed AST! Qualifiers should not be directly analysed during type checking.");
            }
            ASTNode::LITERAL(literal) => {
                self.analyse_literal(literal)
            }
            ASTNode::ARRAY {items, qualifier } => {
                self.analyse_array(items, qualifier)
            }
            ASTNode::UNARY_OP { op, expression } => {
                self.analyse_unary_op(op, expression)
            }
            ASTNode::BINARY_OP { op, lhs, rhs } => {
                self.analyse_binary_op(op, lhs, rhs)
            }
            ASTNode::TERNARY_OP { condition, true_branch, false_branch } => {
                self.analyse_ternary_op(condition, true_branch, false_branch)
            }
            ASTNode::ARRAY_INDEX { index, expression } => {
                self.analyse_array_index(index, expression)
            }
            ASTNode::CONSTRUCT { identifier, datatype, qualifier, expression } => {
                self.analyse_construct_statement(identifier, datatype, qualifier, expression)
            }
            ASTNode::EMPTY_CONSTRUCT { identifier, datatype, qualifier } => {
                self.analyse_empty_construct_statement(identifier, datatype, qualifier)
            }
            ASTNode::EXTERN { identifier } => {
                self.analyse_extern_statement(identifier)
            }
            ASTNode::ASSIGNMENT { identifier, pointer_level, array_index, expression } => {
                self.analyse_assignment_statement(identifier, pointer_level.clone(), array_index, expression)
            }
            ASTNode::PRINT { expression } => {
                self.analyse_print_statement(expression)
            },
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
            ASTNode::PARAMETER { .. } => {
                panic!("Malformed AST! Parameters should not be directly inspected during semantic analysis!")
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
            ASTNode::SCOPE_BLOCK { inner, .. } => {
                self.analyse_scope_block(inner)
            }
            ASTNode::TYPED_NODE { .. } => {
                panic!("Malformed AST! Typed nodes shouldn't be in the AST yet!");
            }
        }
    }

    fn mark_identifier(&mut self, name: &String, datatype: SymbolType) {
        self.symbol_tracker.add_symbol(name, datatype);
    }

    fn type_from_identifier(&mut self, name: &String) -> DataType {
        match self.symbol_tracker.find_symbol(name) {
            Some(symbol) => match symbol {
                SymbolType::Variable(datatype, _) 
                | SymbolType::EnvironmentVariable(_, datatype, _, _) 
                | SymbolType::Parameter(datatype, _) => datatype.clone(),
                _ => panic!("Identifier {} isn't a variable!", name)
            },
            None => panic!("Identifier {} doesn't exist!", name)
        }
    }

    fn qualifier_from_identifier(&mut self, name: &String) -> Qualifier {
        match self.symbol_tracker.find_symbol(name) {
            Some(symbol) => match symbol {
                SymbolType::Variable(_, qualifier) 
                | SymbolType::EnvironmentVariable(_, _, qualifier, _) 
                | SymbolType::Parameter(_, qualifier) => qualifier.clone(),
                _ => panic!("Identifier {} isn't a variable!", name)
            },
            None => panic!("Identifier {} doesn't exist!", name)
        }
    }

    fn analyse_identifier(&mut self, name: &String) -> ASTNode {
        let datatype = self.type_from_identifier(name);
        let qualifier = self.qualifier_from_identifier(name);
        ASTNode::TYPED_NODE { datatype, qualifier, inner: Box::new(ASTNode::IDENTIFIER(name.clone())) }
    }

    fn analyse_reference(&mut self, name: &String) -> ASTNode {
        let datatype = self.type_from_identifier(name);
        let qualifier = self.qualifier_from_identifier(name);
        ASTNode::TYPED_NODE { 
            datatype: DataType::POINTER(Box::new(datatype)), 
            qualifier: qualifier.clone(),
            inner: Box::new(ASTNode::REFERENCE(name.clone())) 
        }
    }

    fn analyse_literal(&mut self, literal: &Literal) -> ASTNode {
        let datatype = match *literal {
            Literal::FLOAT(_) => DataType::PRIMITIVE(PrimitiveDataType::F64),
            Literal::INTEGER(_) => DataType::PRIMITIVE(PrimitiveDataType::I64),
            Literal::BOOL(_) => DataType::PRIMITIVE(PrimitiveDataType::Bool),
            Literal::PACKEDSTRING(_) => DataType::PRIMITIVE(PrimitiveDataType::String),
        };

        ASTNode::TYPED_NODE { 
            datatype, 
            qualifier: Qualifier::CONSTANT,
            inner: Box::new(ASTNode::LITERAL(literal.clone()))
        }
    }

    fn analyse_array(&mut self, items: &Vec<ASTNode>, qualifier: &Box<ASTNode>) -> ASTNode {
        if items.len() == 0 {
            panic!("Cannot create an array of length 0!")
        }
        let mut typed_items: Vec<ASTNode> = vec![];
        for item in items {
            typed_items.push(self.analyse_node(item))
        }
        let datatype = typed_items[0].get_type();
        for item in typed_items.iter().skip(1) {
            let datatype_2 = item.get_type();
            if datatype != datatype_2 {
                panic!("Cannot create array with mismatched types!")
            }
        }
        // Get qualifier from mut/const statement
        let base_qualifier = match **qualifier {
            ASTNode::QUALIFIER(ref q) => q.clone(),
            _ => panic!("Malformed AST! Expected a qualifier node"),
        };
        ASTNode::TYPED_NODE { 
            datatype: DataType::ARRAY(Box::new(datatype), items.len()), 
            qualifier: base_qualifier.clone(),
            inner: Box::new(ASTNode::ARRAY{items: typed_items, qualifier: qualifier.clone()})
        }
    }

    fn analyse_unary_op(&mut self, op: &UnaryOperation, expression: &Box<ASTNode>) -> ASTNode {
        let expression = self.analyse_node(expression);
        let datatype = expression.get_type();
        let qualifier = expression.get_qualifier();
        let datatype = match op {
            UnaryOperation::NOT | UnaryOperation::NEGATE => { 
                match datatype {
                    DataType::PRIMITIVE(_) | DataType::ENVIRONMENTVARIABLE(_) => datatype,
                    _ => panic!("Cannot use operation {:?} on type {:?}", op, datatype)
                }
            }
            UnaryOperation::PTR_DEREF => { 
                match datatype {
                    DataType::POINTER(inner_datatype) => inner_datatype.as_ref().clone(),
                    _ => panic!("Cannot use operation {:?} on type {:?}", op, datatype)
                }
            }
        };
        ASTNode::TYPED_NODE { 
            datatype,
            qualifier,
            inner: Box::new(ASTNode::UNARY_OP { 
                op: op.clone(), 
                expression: Box::new(expression) 
            })
        }
    }

    fn analyse_binary_op(&mut self, op: &BinaryOperation, lhs: &Box<ASTNode>, rhs: &Box<ASTNode>) -> ASTNode {
        let lhs = self.analyse_node(lhs);
        let rhs = self.analyse_node(rhs);
        let lhs_datatype = lhs.get_type();
        let rhs_datatype = rhs.get_type();
        if lhs_datatype != rhs_datatype {
            panic!("Cannot perform operation {:?} with mismatched types! ({:?} vs {:?})", op, lhs_datatype, rhs_datatype)
        }
        let datatype = lhs_datatype;
        
        let datatype = match op {
            BinaryOperation::ADD | BinaryOperation::SUB | BinaryOperation::DIV 
          | BinaryOperation::MUL | BinaryOperation::MOD | BinaryOperation::POW
          | BinaryOperation::LSHIFT | BinaryOperation::RSHIFT => { 
                match datatype {
                    DataType::PRIMITIVE(_) | DataType::ENVIRONMENTVARIABLE(_) => datatype,
                    _ => panic!("Cannot use operation {:?} on type {:?}", op, datatype)
                }
            }
            BinaryOperation::GREATER_THAN | BinaryOperation::LESS_THAN 
          | BinaryOperation::GREATER_EQUAL | BinaryOperation::LESS_EQUAL
          | BinaryOperation::AND | BinaryOperation::OR => { 
                match datatype {
                    DataType::PRIMITIVE(_) | DataType::ENVIRONMENTVARIABLE(_) => DataType::PRIMITIVE(PrimitiveDataType::Bool),
                    _ => panic!("Cannot use operation {:?} on type {:?}", op, datatype)
                }
            }
            BinaryOperation::EQUAL | BinaryOperation::NOT_EQUAL => { 
                DataType::PRIMITIVE(PrimitiveDataType::Bool)
            }
        };
        ASTNode::TYPED_NODE { 
            datatype,
            qualifier: Qualifier::CONSTANT,
            inner: Box::new(ASTNode::BINARY_OP { 
                op: op.clone(), 
                lhs: Box::new(lhs),
                rhs: Box::new(rhs)
            })
        }
    }

    fn analyse_ternary_op(&mut self, condition: &Box<ASTNode>, true_branch: &Box<ASTNode>, false_branch: &Box<ASTNode>) -> ASTNode {
        let condition = self.analyse_node(condition);
        let true_branch = self.analyse_node(true_branch);
        let false_branch = self.analyse_node(false_branch);
        let condition_datatype = condition.get_type();
        let true_branch_datatype = true_branch.get_type();
        let false_branch_datatype = false_branch.get_type();
        if condition_datatype != DataType::PRIMITIVE(PrimitiveDataType::Bool) {
            panic!("Ternary conditions must be booleans! (currently {:?})", condition_datatype)
        }
        if true_branch_datatype != false_branch_datatype {
            panic!("Branches of ternary operator must be the same type! ({:?} vs {:?})", true_branch_datatype, false_branch_datatype)
        }
        let datatype = true_branch_datatype;

        ASTNode::TYPED_NODE { 
            datatype,
            qualifier: Qualifier::CONSTANT,
            inner: Box::new(ASTNode::TERNARY_OP { 
                condition: Box::new(condition), 
                true_branch: Box::new(true_branch),
                false_branch: Box::new(false_branch) 
            })
        }
    }

    fn analyse_array_index(&mut self, index: &Box<ASTNode>, expression: &Box<ASTNode>) -> ASTNode {
        let expression = Box::new(self.analyse_node(expression));
        let index = Box::new(self.analyse_node(index));
        let expression_datatype = expression.get_type();
        let index_datatype = index.get_type();
        // check index is a literal and expression is an array/environmentvariable. Return array innards
        match expression_datatype {
            DataType::ARRAY(inner_type, _size) => {
                match index_datatype {
                    DataType::PRIMITIVE(_) | DataType::ENVIRONMENTVARIABLE(_) => {
                        ASTNode::TYPED_NODE { 
                            datatype: inner_type.as_ref().clone(), 
                            qualifier: expression.get_qualifier(),
                            inner: Box::new(ASTNode::ARRAY_INDEX { index, expression })
                        }
                    }
                    _ => panic!("Can only index arrays with literal values!")
                }
            }
            DataType::ENVIRONMENTVARIABLE(inner_type) => {
                match index_datatype {
                    DataType::PRIMITIVE(_) | DataType::ENVIRONMENTVARIABLE(_) => {
                        ASTNode::TYPED_NODE { 
                            datatype: DataType::PRIMITIVE(inner_type), 
                            qualifier: expression.get_qualifier(),
                            inner: Box::new(ASTNode::ARRAY_INDEX { index, expression })
                        }
                    }
                    _ => panic!("Can only index arrays with literal values!")
                }
            }
            _ => panic!("Can't index a non-array or non-environmentvariable! (indexing {:?} with {:?})", expression_datatype, index_datatype)
        }
    }

    fn analyse_construct_statement(
        &mut self,
        identifier: &Box<ASTNode>,
        datatype: &Box<Option<ASTNode>>,
        qualifier: &Box<ASTNode>,
        expression: &Box<ASTNode>
    ) -> ASTNode {
        // First, analyze the expression and get its type.
        let mut analyzed_expr = self.analyse_node(expression);
        let expression_datatype = analyzed_expr.get_type();
    
        // Extract the declared qualifier from the construct.
        let declared_qualifier = match **qualifier {
            ASTNode::QUALIFIER(ref q) => q.clone(),
            _ => panic!("Malformed AST! Expected a qualifier node"),
        };
    
        // Register the new variable using the expression's type and the declared qualifier.
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            self.mark_identifier(name, SymbolType::Variable(expression_datatype.clone(), declared_qualifier.clone()));
        }
    
        // Override the qualifier in the array literal (if the expression is an array)
        analyzed_expr = match analyzed_expr {
            ASTNode::TYPED_NODE { datatype, qualifier: _, inner } => {
                match *inner {
                    ASTNode::ARRAY { items, qualifier: _ } => {
                        ASTNode::TYPED_NODE {
                            datatype,
                            qualifier: declared_qualifier.clone(),
                            inner: Box::new(ASTNode::ARRAY {
                                items,
                                qualifier: Box::new(ASTNode::QUALIFIER(declared_qualifier.clone())),
                            }),
                        }
                    },
                    other => ASTNode::TYPED_NODE {
                        datatype,
                        qualifier: declared_qualifier.clone(),
                        inner: Box::new(other),
                    }
                }
            },
            other => other,
        };
    
        // Re-analyze the identifier node (this may perform additional processing).
        let identifier_node = Box::new(self.analyse_node(identifier));
        let qualifier_node = Box::new(ASTNode::QUALIFIER(declared_qualifier.clone()));
    
        // If a datatype was provided, ensure that it matches the expression's type.
        if let Some(datatype_node) = datatype.as_ref().clone() {
            let declared_datatype = match datatype_node {
                ASTNode::DATATYPE(dt) => dt,
                _ => panic!("Malformed AST! Node {:?} should have been a datatype but wasn't!", datatype_node),
            };
            if declared_datatype != expression_datatype {
                panic!("Provided data doesn't match given datatype in construct statement! {:?} vs {:?}", declared_datatype, expression_datatype);
            }
            let datatype_box: Box<Option<ASTNode>> = Box::new(Some(ASTNode::DATATYPE(declared_datatype)));
            ASTNode::CONSTRUCT { 
                identifier: identifier_node, 
                datatype: datatype_box, 
                qualifier: qualifier_node, 
                expression: Box::new(analyzed_expr)
            }
        } else {
            let datatype_box: Box<Option<ASTNode>> = Box::new(Some(ASTNode::DATATYPE(expression_datatype)));
            ASTNode::CONSTRUCT { 
                identifier: identifier_node, 
                datatype: datatype_box, 
                qualifier: qualifier_node, 
                expression: Box::new(analyzed_expr)
            }
        }
    }

    fn analyse_empty_construct_statement(&mut self, identifier: &Box<ASTNode>, datatype: &Box<ASTNode>, qualifier: &Box<ASTNode>) -> ASTNode {
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            let datatype = match datatype.as_ref() {
                ASTNode::DATATYPE(datatype) => datatype,
                _ => panic!("Malformed AST! Node {:?} should have been a datatype but wasn't!", datatype)
            };
            let qualifier = match qualifier.as_ref() {
                ASTNode::QUALIFIER(ref q) => q.clone(),
                _ => panic!("Malformed AST! Expected a qualifier node"),
            };
            self.mark_identifier(name, SymbolType::Variable(datatype.clone(), qualifier.clone()));
            let identifier = Box::new(self.analyse_node(identifier));
            let datatype = Box::new(ASTNode::DATATYPE(datatype.clone()));
            let qualifier = Box::new(ASTNode::QUALIFIER(qualifier.clone()));
            ASTNode::EMPTY_CONSTRUCT { identifier, datatype, qualifier }
        } else {
            panic!("Malformed AST! Construct statement should always start with an identifier")
        }
    }

    fn analyse_extern_statement(&mut self, identifier: &Box<ASTNode>) -> ASTNode {
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            match self.env_vars.get(name) {
                Some((usize, datatype, qualifier, string)) => {
                    self.mark_identifier(name, SymbolType::EnvironmentVariable(usize.clone(), DataType::ENVIRONMENTVARIABLE(datatype.clone()), qualifier.clone(), string.clone()));
                    let identifier = Box::new(self.analyse_node(identifier));
                    ASTNode::EXTERN { identifier }
                }
                None => panic!("Tried to declare environment variable {} that doesn't exist!", name)
            }
        } else {
            panic!("Malformed AST! Extern statement should always contain an identifier")
        }
    }

    fn analyse_assignment_statement(&mut self, identifier: &Box<ASTNode>, pointer_level: usize, array_index: &Vec<ASTNode>, expression: &Box<ASTNode>) -> ASTNode {
        let identifier = Box::new(self.analyse_node(identifier));
        let mut identifier_datatype = identifier.get_type();
        let identifier_qualifier = identifier.get_qualifier();

        for _ in 0..pointer_level {
            identifier_datatype = match identifier_datatype {
                DataType::POINTER(datatype) => *datatype,
                _ => panic!("Can't perform pointer assignment on a non-pointer!")
            };
        }

        let mut new_index = Vec::new();
        for index in array_index { 
            let index = self.analyse_node(index);
            let index_datatype = index.get_type();
            match index_datatype {
                DataType::PRIMITIVE(_) | DataType::ENVIRONMENTVARIABLE(_) => {}
                _ => panic!("Can only index arrays with literal values!")
            };
            new_index.push(index);
            identifier_datatype = match identifier_datatype {
                DataType::ARRAY(datatype, _) => *datatype,
                DataType::ENVIRONMENTVARIABLE(datatype) => DataType::ENVIRONMENTVARIABLE(datatype),
                _ => panic!("Can't index a non-array!")
            };
        }

        let expression = self.analyse_node(expression);
        let expression_datatype = expression.get_type();
        let expression = Box::new(expression);

        match identifier_qualifier {
            Qualifier::CONSTANT => panic!("Can't assign to a constant value! {:?}", identifier),
            _ => {}
        }

        //match identifier_datatype {
        //    DataType::PRIMITIVE(_) => panic!("Can't assign to a constant value! {:?}", identifier),
        //    _ => {}
        //}

        if expression_datatype != identifier_datatype {
            panic!("Identifier and expression must be equal in an assignment statement! (Currently {:?} vs {:?})", identifier_datatype, expression_datatype)
        }
        
        ASTNode::ASSIGNMENT { identifier, pointer_level, array_index: new_index, expression }
    }

    fn analyse_print_statement(&mut self, expression: &Box<ASTNode>) -> ASTNode {
        let expression = Box::new(self.analyse_node(expression));
        ASTNode::PRINT { expression }
    }

    fn analyse_return_statement(&mut self, expression: &Box<ASTNode>) -> ASTNode {
        let expression = Box::new(self.analyse_node(expression));
        self.symbol_tracker.add_return_type(&expression.get_type());
        ASTNode::RETURN { expression }
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
        let datatype = condition.get_type();
        match datatype {
            DataType::PRIMITIVE(_) | DataType::ENVIRONMENTVARIABLE(_) => {},
            _ => panic!("Literal values must be used for if statement conditions!")
        }

        ASTNode::BRANCH { condition, if_branch, else_branch }
    }

    fn analyse_while_statement(&mut self, condition: &Box<ASTNode>, body: &Box<ASTNode>) -> ASTNode {
        let condition = Box::new(self.analyse_node(condition));
        let body = Box::new(self.analyse_node(body));
        let datatype = condition.get_type();
        match datatype {
            DataType::PRIMITIVE(_) | DataType::ENVIRONMENTVARIABLE(_) => {},
            _ => panic!("Literal values must be used for while statement conditions!")
        }
        ASTNode::WHILE_LOOP { condition, body }
    }

    fn analyse_for_loop(&mut self, initialization: &Box<ASTNode>, condition: &Box<ASTNode>, advancement: &Box<ASTNode>, body: &Box<ASTNode>) -> ASTNode {
        let initialization = Box::new(self.analyse_node(initialization));
        let condition = Box::new(self.analyse_node(condition));
        let advancement = Box::new(self.analyse_node(advancement));
        let body = Box::new(self.analyse_node(body));
        let datatype = condition.get_type();
        match datatype {
            DataType::PRIMITIVE(_) | DataType::ENVIRONMENTVARIABLE(_) => {},
            _ => panic!("Literal values must be used for for statement conditions!")
        }
        ASTNode::FOR_LOOP { initialization, condition, advancement, body }
    }

    fn analyse_function_definition(&mut self, identifier: &Box<ASTNode>, parameters: &Vec<ASTNode>, return_type: &Box<Option<ASTNode>>, body: &Box<ASTNode>) -> ASTNode {
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            if !self.functions.contains_key(name) {
                self.functions.insert(name.clone(), FunctionTracker::new(
                    parameters.clone(), 
                    return_type.as_ref().clone(), 
                    body.as_ref().clone()
                ));
                ASTNode::FUNCTION {
                    identifier: identifier.clone(),
                    parameters: parameters.clone(),
                    return_type: return_type.clone(),
                    body: body.clone()
                }
            } else {
                panic!("Function {} already exisits!", name)
            }
        } else {
            panic!("Malformed AST! Function names should be identifiers!")
        }
    }

    fn analyse_function_implementation(
        &mut self, 
        parameters: &Vec<DataType>, 
        parameter_names: &Vec<String>, 
        parameter_qualifiers: &Vec<Qualifier>,
        return_type: &Option<DataType>, 
        body: &ASTNode
    ) -> (ASTNode, DataType) {
        self.symbol_tracker.enter_scope();
        for ((identifier, datatype), qualifier) in parameter_names.iter().zip(parameters.iter()).zip(parameter_qualifiers.iter()) { 
            let parameter_datatype = match datatype.clone() {
                DataType::PRIMITIVE(primitive) => DataType::PRIMITIVE(primitive),
                _ => datatype.clone(),
            };
            self.mark_identifier(identifier, SymbolType::Variable(parameter_datatype,qualifier.clone()));
        }
        let body = self.analyse_node(body);
        let real_return_type = self.symbol_tracker.get_return_type().clone();
        self.symbol_tracker.exit_scope();
        if let Some(return_type) = return_type {
            if return_type != &real_return_type {
                panic!("Return type of function did not match declared type! ({:?} vs {:?})", return_type, real_return_type)
            }
        }
        return (body, real_return_type)
    }

    fn analyse_function_call(&mut self, identifier: &Box<ASTNode>, arguments: &Vec<ASTNode>) -> ASTNode {
        let mut typed_arguments: Vec<ASTNode> = vec![];
        for argument in arguments {
            typed_arguments.push(self.analyse_node(argument))
        }
        let mut argument_datatypes: Vec<DataType> = vec![];
        let mut argument_types: Vec<(DataType, Qualifier)> = vec![];
        for argument in typed_arguments.iter() {
            // Push tuple of qualifier and datatype to argument_types
            argument_datatypes.push(argument.get_type());
            argument_types.push((argument.get_type(), argument.get_qualifier()));
        }
        // TODO: Perform a check to see whether an input variable that may be mutable is actually mutated in the function, if not then allow it to be passed into a function with a const parameter qualifier.
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            if self.functions.contains_key(name) {
                let function = self.functions.get(name).unwrap();
                match function.match_function(&argument_types) {
                    Some((implementation_name, datatype)) => {
                        ASTNode::TYPED_NODE {
                            datatype,
                            qualifier: Qualifier::CONSTANT,
                            inner: Box::new(ASTNode::FUNC_CALL {
                                identifier: Box::new(ASTNode::IDENTIFIER(implementation_name)),
                                arguments: typed_arguments,
                            })
                        }
                    }
                    None => {
                        let (parameter_datatypes, parameter_names, parameter_qualifiers, return_type, body) = function.get_innards();
                        let parameter_names = parameter_names.clone();
                        let parameter_qualifiers = parameter_qualifiers.clone();
                        //let parameters_with_qualifiers: Vec<(Option<DataType>, Qualifier)> = parameter_datatypes.iter().cloned().zip(parameter_qualifiers.iter().cloned()).collect();
                        let parameters: Vec<(String, (Option<DataType>, Qualifier))> =
                            parameter_names.iter().cloned()
                            .zip(parameter_datatypes.iter().cloned().zip(parameter_qualifiers.iter().cloned()))
                            .collect();
                        let real_datatypes = self.check_parameter_list(&parameters, &argument_types, name);
                        let (body, return_type) = self.analyse_function_implementation(
                            &real_datatypes.clone(),
                            &parameter_names.clone(),
                            &parameter_qualifiers.clone(),
                            &return_type.clone(),
                            &body.clone()
                        );
                        let function = self.functions.get_mut(name).unwrap();
                        let implementation_name = function.create_implementation(name.clone(), parameter_names, real_datatypes, parameter_qualifiers, return_type.clone(), body);
                        ASTNode::TYPED_NODE {
                            datatype: return_type,
                            qualifier: Qualifier::CONSTANT,
                            inner: Box::new(ASTNode::FUNC_CALL {
                                identifier: Box::new(ASTNode::IDENTIFIER(implementation_name)),
                                arguments: typed_arguments,
                            })
                        }
                    }
                }
            } else {
                for function in BARRACUDA_BUILT_IN_FUNCTIONS {
                    if name == &String::from(format!("__{}", function.to_string().to_lowercase())) {
                        if argument_datatypes == vec![DataType::PRIMITIVE(PrimitiveDataType::F64); function.consume() as usize] {
                            return ASTNode::TYPED_NODE {
                                datatype: DataType::PRIMITIVE(PrimitiveDataType::F64),
                                qualifier: Qualifier::CONSTANT,
                                inner: Box::new(ASTNode::FUNC_CALL {
                                    identifier: Box::new(ASTNode::IDENTIFIER(name.clone())),
                                    arguments: typed_arguments,
                                })
                            }
                        }
                    }
                }
                panic!("Function {} doesn't exist!", name)
            }
        } else {
            panic!("Malformed AST! Function names should be identifiers!")
        }
    }

    fn check_parameter_list(&self, parameters: &Vec<(String, (Option<DataType>, Qualifier))>, arguments: &Vec<(DataType, Qualifier)>, name: &String) -> Vec<DataType> {
        if parameters.len() != arguments.len() {
            panic!("When calling function {}, need to use {} parameters! (Used {})", name, parameters.len(), arguments.len())
        }
        let mut real_types = vec![];
        for ((parameter_name, (parameter_datatype, parameter_qualifier)), (argument_datatype, argument_qualifier)) in parameters.iter().zip(arguments.iter()) {
            let final_datatype = match parameter_datatype {
                Some(parameter_datatype) => {
                    if parameter_datatype == argument_datatype {
                        parameter_datatype.clone()
                    } else {
                        panic!("Type of parameter {:?} in function {} didn't match! ({:?} vs {:?})", parameter_name, name, parameter_datatype, argument_datatype)
                    }
                }
                None => argument_datatype.clone()
            };
            if parameter_qualifier != argument_qualifier {
                panic!("Type qualifier of parameter {:?} in function {} didn't match the input argument! ({:?} vs {:?})", parameter_name, name, parameter_qualifier, argument_qualifier)
            }
            real_types.push(final_datatype);
        }
        real_types
    }

    fn analyse_naked_function_call(&mut self, func_call: &Box<ASTNode>) -> ASTNode {
        let func_call = Box::new(self.analyse_node(func_call));
        ASTNode::NAKED_FUNC_CALL { func_call }
    }

    fn analyse_statement_list(&mut self, statements: &Vec<ASTNode>) -> ASTNode {
        let mut new_statements = Vec::new();
        for statement in statements {
            new_statements.push(self.analyse_node(statement));
        }
        ASTNode::STATEMENT_LIST(new_statements)
    }

    // Currently functions are the only use of scope blocks. If this changes, the method should have enter_scope and exit_scope calls added,
    // and functions should bypass this function with a match statement. It was done this way to prevent functions causing two scopes from being created.
    fn analyse_scope_block(&mut self, inner: &Box<ASTNode>) -> ASTNode {
        let scope = self.scope_counter.next().unwrap();
        let inner = Box::new(self.analyse_node(inner));
        ASTNode::SCOPE_BLOCK { inner, scope }
    }

}

/// AstParser Trait Concrete Implementation 
impl SemanticAnalyser for BarracudaSemanticAnalyser {
    fn default() -> Self {
        Self {
            symbol_tracker: ScopeTracker::new(),
            scope_counter: ScopeIdGenerator::new(),
            env_vars: HashMap::new(),
            functions: HashMap::new()
        }
    }

    /// Parse processes a source string into an abstract syntax tree
    fn analyse(mut self, root_node: ASTNode, env_vars: EnvironmentSymbolContext) -> AbstractSyntaxTree {
        self.scope_counter.next();
        self.env_vars = env_vars.copy_addresses();
        let root = self.analyse_node(&root_node);
        let functions = self.functions;
        AbstractSyntaxTree::new(root, env_vars, functions)
    }
}