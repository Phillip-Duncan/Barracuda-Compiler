use std::collections::HashMap;

use crate::compiler::PrimitiveDataType;
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
    env_vars: HashMap<String, (usize, PrimitiveDataType, String)>,
    functions: HashMap<String, FunctionTracker>
}

impl BarracudaSemanticAnalyser {
 
    /// Parses all pest pair tokens into a valid ASTNode
    pub fn analyse_node(&mut self, node: &ASTNode) -> ASTNode {
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
            ASTNode::EMPTY_CONSTRUCT { identifier, datatype } => {
                self.analyse_empty_construct_statement(identifier, datatype)
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
                SymbolType::Variable(datatype) 
                | SymbolType::EnvironmentVariable(_, datatype, _) 
                | SymbolType::Parameter(datatype) => datatype.clone(),
                _ => panic!("Identifier {} isn't a variable!", name)
            },
            None => panic!("Identifier {} doesn't exist!", name)
        }
    }

    fn analyse_identifier(&mut self, name: &String) -> ASTNode {
        let datatype = self.type_from_identifier(name);
        ASTNode::TYPED_NODE { datatype, inner: Box::new(ASTNode::IDENTIFIER(name.clone())) }
    }

    fn analyse_reference(&mut self, name: &String) -> ASTNode {
        let datatype = self.type_from_identifier(name);
        ASTNode::TYPED_NODE { 
            datatype: DataType::POINTER(Box::new(datatype)), 
            inner: Box::new(ASTNode::REFERENECE(name.clone())) 
        }
    }

    fn analyse_literal(&mut self, literal: &Literal) -> ASTNode {
        let datatype = match *literal {
            Literal::FLOAT(_) => DataType::CONST(PrimitiveDataType::F64),
            Literal::INTEGER(_) => DataType::CONST(PrimitiveDataType::I64),
            Literal::BOOL(_) => DataType::CONST(PrimitiveDataType::Bool)
        };

        ASTNode::TYPED_NODE { 
            datatype, 
            inner: Box::new(ASTNode::LITERAL(literal.clone()))
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
        let datatype = typed_items[0].get_type();
        for item in typed_items.iter().skip(1) {
            let datatype_2 = item.get_type();
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
        let datatype = expression.get_type();
        let datatype = match op {
            UnaryOperation::NOT | UnaryOperation::NEGATE => { 
                match datatype {
                    DataType::CONST(_) | DataType::MUTABLE(_) => datatype,
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
        let expression_datatype = expression.get_type();
        let index_datatype = index.get_type();
        // check index is a literal and expression is an array. Return array innards
        if let DataType::ARRAY(inner_type, _size) = expression_datatype {
            if let DataType::CONST(_) | DataType::MUTABLE(_)  = index_datatype {
                ASTNode::TYPED_NODE { 
                    datatype: inner_type.as_ref().clone(), 
                    inner: Box::new(ASTNode::ARRAY_INDEX { index, expression })
                }
            } else {
                panic!("Can only index arrays with literal values!")
            }
        } else {
            panic!("Can't index a non-array! (indexing {:?} with {:?})", expression_datatype, index_datatype)
        }
    }

    fn analyse_construct_statement(&mut self, identifier: &Box<ASTNode>, datatype: &Box<Option<ASTNode>>, expression: &Box<ASTNode>) -> ASTNode {
        let expression = Box::new(self.analyse_node(expression));
        let expression_datatype = expression.get_type();
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            self.mark_identifier(name, SymbolType::Variable(expression_datatype.clone()));
            let identifier = Box::new(self.analyse_node(identifier));
            if let Some(datatype) = datatype.as_ref().clone() {
                let datatype = match datatype {
                    ASTNode::DATATYPE(datatype) => datatype,
                    _ => panic!("Malformed AST! Node {:?} should have been a datatype but wasn't!", datatype)
                };
                if datatype != expression_datatype {
                    panic!("Provided data doesn't match given datatype in construct statement! {:?} vs {:?}", datatype, expression_datatype);
                }
                let datatype: Box<Option<ASTNode>> = Box::new(Some(ASTNode::DATATYPE(datatype)));
                ASTNode::CONSTRUCT { identifier, datatype, expression: expression.clone() }   
            } else {
                let datatype = Box::new(Some(ASTNode::DATATYPE(expression_datatype)));
                ASTNode::CONSTRUCT { identifier, datatype, expression: expression.clone() }   
            }
        } else {
            panic!("Malformed AST! Construct statement should always start with an identifier")
        }
    }

    fn analyse_empty_construct_statement(&mut self, identifier: &Box<ASTNode>, datatype: &Box<ASTNode>) -> ASTNode {
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            let datatype = match datatype.as_ref() {
                ASTNode::DATATYPE(datatype) => datatype,
                _ => panic!("Malformed AST! Node {:?} should have been a datatype but wasn't!", datatype)
            };
            self.mark_identifier(name, SymbolType::Variable(datatype.clone()));
            let identifier = Box::new(self.analyse_node(identifier));
            let datatype = Box::new(ASTNode::DATATYPE(datatype.clone()));
            ASTNode::EMPTY_CONSTRUCT { identifier, datatype }
        } else {
            panic!("Malformed AST! Construct statement should always start with an identifier")
        }
    }

    fn analyse_extern_statement(&mut self, identifier: &Box<ASTNode>) -> ASTNode {
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            match self.env_vars.get(name) {
                Some((usize, datatype, string)) => {
                    self.mark_identifier(name, SymbolType::EnvironmentVariable(usize.clone(), DataType::MUTABLE(datatype.clone()), string.clone()));
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
                DataType::CONST(_) | DataType::MUTABLE(_) => {}
                _ => panic!("Can only index arrays with literal values!")
            };
            new_index.push(index);
            identifier_datatype = match identifier_datatype {
                DataType::ARRAY(datatype, _) => *datatype,
                _ => panic!("Can't index a non-array!")
            };
        }

        let expression = self.analyse_node(expression);
        let expression_datatype = expression.get_type();
        let expression = Box::new(expression);

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
            DataType::CONST(_) | DataType::MUTABLE(_) => {},
            _ => panic!("Literal values must be used for if statement conditions!")
        }

        ASTNode::BRANCH { condition, if_branch, else_branch }
    }

    fn analyse_while_statement(&mut self, condition: &Box<ASTNode>, body: &Box<ASTNode>) -> ASTNode {
        let condition = Box::new(self.analyse_node(condition));
        let body = Box::new(self.analyse_node(body));
        let datatype = condition.get_type();
        match datatype {
            DataType::CONST(_) | DataType::MUTABLE(_) => {},
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
            DataType::CONST(_) | DataType::MUTABLE(_) => {},
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
        return_type: &Option<DataType>, 
        body: &ASTNode
    ) -> (ASTNode, DataType) {
        self.symbol_tracker.enter_scope();
        for (identifier, datatype) in parameter_names.iter().zip(parameters.iter()) { 
            self.mark_identifier(identifier, SymbolType::Parameter(datatype.clone()));
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
        let mut argument_types: Vec<DataType> = vec![];
        for argument in &typed_arguments {
            argument_types.push(argument.get_type())
        }
        if let ASTNode::IDENTIFIER(name) = identifier.as_ref() {
            if self.functions.contains_key(name) {
                let function = self.functions.get(name).unwrap();
                match function.match_function(&argument_types) {
                    Some((implementation_name, datatype)) => {
                        ASTNode::TYPED_NODE {
                            datatype,
                            inner: Box::new(ASTNode::FUNC_CALL {
                                identifier: Box::new(ASTNode::IDENTIFIER(implementation_name)),
                                arguments: typed_arguments,
                            })
                        }
                    }
                    None => {
                        let (parameters, parameter_names, return_type, body) = function.get_innards();
                        let parameter_names = parameter_names.clone();
                        let real_parameters = self.check_parameter_list(parameters, &argument_types, name);
                        let (body, return_type) = self.analyse_function_implementation(
                            &real_parameters.clone(),
                            &parameter_names.clone(),
                            &return_type.clone(),
                            &body.clone()
                        );
                        let function = self.functions.get_mut(name).unwrap();
                        let implementation_name = function.create_implementation(name.clone(), parameter_names, real_parameters, return_type.clone(), body);
                        ASTNode::TYPED_NODE {
                            datatype: return_type,
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
                        if argument_types == vec![DataType::CONST(PrimitiveDataType::F64); function.consume() as usize] {
                            return ASTNode::TYPED_NODE {
                                datatype: DataType::CONST(PrimitiveDataType::F64),
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

    fn check_parameter_list(&self, parameters: &Vec<Option<DataType>>, arguments: &Vec<DataType>, name: &String) -> Vec<DataType> {
        if parameters.len() != arguments.len() {
            panic!("When calling function {}, need to use {} parameters! (Used {})", name, parameters.len(), arguments.len())
        }
        let mut real_types = vec![];
        for (parameter, argument) in parameters.iter().zip(arguments.iter()) {
            let datatype = match parameter {
                Some(parameter) => {
                    if parameter == argument {
                        parameter.clone()
                    } else {
                        panic!("Type of parameter in function {} didn't match! ({:?} vs {:?})", name, parameter, argument)
                    }
                }
                None => argument.clone()
            };
            real_types.push(datatype);
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