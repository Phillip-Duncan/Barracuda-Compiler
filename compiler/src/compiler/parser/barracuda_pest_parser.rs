use crate::compiler::ast::datatype::DataType;
use crate::pest::Parser;
use super::AstParser;
use super::super::ast::{
    ASTNode,
    Literal,
    BinaryOperation,
    UnaryOperation,
    ScopeId
};

use crate::compiler::utils::pack_string_to_f64_array;

/// Pest Barracuda Parser parses a string into a series of tokens.
/// These tokens are defined as a Context-Free-Grammar in the src/barracuda.pest file.
/// The tokens generated from this parser are then formalised into the generic abstract
/// syntax tree implementation.
#[derive(Parser)]
#[grammar = "barracuda.pest"]
struct BarracudaParser;

/// PestBarracudaParser is a concrete AstParser.
/// It uses the pest library to generate a token sequence from a source string
/// that is then converted into a AbstractSyntaxTree.
pub struct PestBarracudaParser {
    precision: usize
}

impl PestBarracudaParser {

    /// Parses source string into an ASTNode.
    fn parse_into_node_tree(&self, source: &str) -> ASTNode {
        match BarracudaParser::parse(Rule::program, source) {
            Ok(pairs) => {
                for pair in pairs {
                    match pair.as_rule() {
                        Rule::global_statement_list => {
                            return self.parse_pair_node(pair)
                        },
                        _ => { panic!("Program should start with statement list.") }
                    }
                }
            },
            Err(error) => {
                panic!("Syntax Error: {}", error)
            }
        }
        panic!("Program has been parsed without error but is empty.")
    }

    /// Parses all pest pair tokens into a valid ASTNode
    fn parse_pair_node(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        match pair.as_rule() {
            Rule::identifier =>         { self.parse_pair_identifier(pair) },
            Rule::reference =>          { self.parse_pair_reference(pair) },
            Rule::primitive_datatype => { self.parse_pair_primitive_datatype(pair) },
            Rule::pointer_datatype =>   { self.parse_pair_pointer_datatype(pair) },
            Rule::array_datatype =>     { self.parse_pair_array_datatype(pair) },
            Rule::integer |
            Rule::decimal |
            Rule::boolean =>            { self.parse_pair_literal(pair) },
            Rule::string =>             { self.parse_pair_string(pair) },
            Rule::array =>              { self.parse_pair_array(pair) },
            Rule::equality |
            Rule::comparison |
            Rule::term |
            Rule::factor |
            Rule::exponent =>           { self.parse_pair_binary_expression(pair) },
            Rule::ternary =>            { self.parse_pair_ternary_expression(pair) },
            Rule::unary |
            Rule::pointer =>            { self.parse_pair_unary_expression(pair) },
            Rule::index =>              { self.parse_pair_array_index(pair) },
            Rule::global_statement_list |
            Rule::statement_list =>     { self.parse_pair_statement_list(pair) },
            Rule::full_qualified_construct_statement => { self.parse_pair_full_qualified_construct_statement(pair) },
            Rule::full_construct_statement => { self.parse_pair_full_construct_statement(pair) },
            Rule::inferred_construct_statement => { self.parse_pair_inferred_construct_statement(pair) },
            Rule::empty_construct_statement => { self.parse_pair_empty_construct_statement(pair) },
            Rule::external_statement => { self.parse_pair_external_statement(pair) },
            Rule::assign_statement =>   { self.parse_pair_assignment_statement(pair) },
            Rule::if_statement =>       { self.parse_pair_if_statement(pair) },
            Rule::for_statement =>      { self.parse_pair_for_statement(pair) },
            Rule::while_statement =>    { self.parse_pair_while_statement(pair) },
            Rule::print_statement =>    { self.parse_pair_print_statement(pair) },
            Rule::func_statement =>     { self.parse_pair_function(pair) },
            Rule::func_param =>         { self.parse_pair_function_parameter(pair) },
            Rule::return_statement =>   { self.parse_pair_return_statement(pair) },
            Rule::func_call =>          { self.parse_pair_function_call(pair) },
            Rule::naked_func_call =>    { self.parse_pair_naked_function_call(pair) },
            Rule::func_arg =>           { self.parse_pair_function_argument(pair) },
            Rule::global_scope_block |
            Rule::scope_block =>        { self.parse_pair_scope_block(pair) },
            _ => { panic!("Whoops! Unprocessed pest rule: {:?}", pair.as_rule()) }
        }
    }

    /// Parses a pest token pair into an AST literal
    fn parse_pair_literal(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        match pair.as_rule() {
            Rule::integer => {
                ASTNode::LITERAL(Literal::INTEGER(pair.as_str().parse().unwrap()))
            },
            Rule::decimal => {
                ASTNode::LITERAL(Literal::FLOAT(pair.as_str().parse().unwrap()))
            },
            Rule::boolean => {
                ASTNode::LITERAL(Literal::BOOL(pair.as_str().parse().unwrap()))
            },
            _ => { panic!("Whoops! Unprocessed literal rule: {:?}", pair.as_rule()) }
        }
    }

    fn parse_pair_string(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let string = pair.as_str();
        let string = &string[1..string.len() - 1];
        let string = pack_string_to_f64_array(string, self.precision);
        ASTNode::ARRAY(
            string
                .into_iter()
                .map(|x| ASTNode::LITERAL(Literal::PACKEDSTRING(x)))
                .collect(),
        )
    }

    fn parse_pair_array(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        ASTNode::ARRAY(pair.into_inner().map(|p| self.parse_pair_node(p)).collect())
    }

    /// Parses a pest token pair into an AST identifier
    fn parse_pair_identifier(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        ASTNode::IDENTIFIER(String::from(pair.as_str()))
    }

    /// Parses a pest token pair into an AST reference
    fn parse_pair_reference(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        ASTNode::REFERENECE(String::from(&pair.as_str()[1..]))
    }

    fn parse_pair_primitive_datatype(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        ASTNode::DATATYPE(DataType::from_str(pair.as_str().to_owned()))
    }

    fn parse_pair_pointer_datatype(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let sub_datatype = pair.next().unwrap();
        let sub_datatype = self.parse_pair_node(sub_datatype);
        let sub_datatype = match sub_datatype {
            ASTNode::DATATYPE(datatype) => datatype,
            _ => panic!("Datatype not found in array (ASTNode: {:?})", sub_datatype),
        };
        let sub_datatype = Box::new(sub_datatype);
        ASTNode::DATATYPE(DataType::POINTER(sub_datatype))
    }

    fn parse_pair_array_datatype(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let sub_datatype = pair.next().unwrap();
        let sub_datatype = self.parse_pair_node(sub_datatype);
        let sub_datatype = match sub_datatype {
            ASTNode::DATATYPE(datatype) => datatype,
            _ => panic!("Datatype not found in array (ASTNode: {:?})", sub_datatype),
        };
        let sub_datatype = Box::new(sub_datatype);
        let size = pair.as_str().parse().unwrap();
        ASTNode::DATATYPE(DataType::ARRAY(sub_datatype, size))
    }

    /// Parses a pest token pair into an AST binary expression
    fn parse_pair_binary_expression(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();

        // Convert linear list of binary operations of equal precedence
        // Into AST tree of binary operations
        let mut lhs = self.parse_pair_node(pair.next().unwrap());
        while pair.peek().is_some() {
            let op = self.parse_pair_binary_op(pair.next().unwrap()).unwrap();
            let rhs = self.parse_pair_node(pair.next().unwrap());
            lhs = ASTNode::BINARY_OP {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }

        return lhs;
    }

    /// Parses a pest token pair into an AST ternary expression
    fn parse_pair_ternary_expression(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let condition = self.parse_pair_node(pair.next().unwrap());
        let true_branch = self.parse_pair_node(pair.next().unwrap());
        let false_branch = self.parse_pair_node(pair.next().unwrap());
        return ASTNode::TERNARY_OP {
            condition: Box::new(condition),
            true_branch: Box::new(true_branch),
            false_branch: Box::new(false_branch),
        };
    }

    /// Parses a pest token pair into an AST unary expression
    fn parse_pair_unary_expression(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let primary_or_operator = pair.next().unwrap();
        // Unary
        if pair.peek().is_some() {
            let op = self.parse_pair_unary_op(primary_or_operator).unwrap();
            let rhs = self.parse_pair_node(pair.next().unwrap());

            ASTNode::UNARY_OP {
                op,
                expression: Box::new(rhs),
            }
        // Skip as primary
        } else {
            self.parse_pair_node(primary_or_operator)
        }
    }

    fn parse_pair_array_index(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let primary = pair.next().unwrap();
        let mut expression = self.parse_pair_node(primary);
        // Unary
        while pair.peek().is_some() {
            let index = self.parse_pair_node(pair.next().unwrap());

            expression = ASTNode::ARRAY_INDEX {
                index: Box::new(index),
                expression: Box::new(expression),
            };
        }
        expression
    }

    /// Parses a pest token pair into an AST statement list
    fn parse_pair_statement_list(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        ASTNode::STATEMENT_LIST(pair.into_inner().map(|p| self.parse_pair_node(p)).collect())
    }

    /// Parses a pest token pair into an AST construct statement, with datatype
    fn parse_pair_full_qualified_construct_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let qualifier = self.parse_pair_node(pair.next().unwrap());
        let identifier = self.parse_pair_node(pair.next().unwrap());
        let datatype = self.parse_pair_node(pair.next().unwrap());
        let expression = self.parse_pair_node(pair.next().unwrap());

        ASTNode::CONSTRUCT {
            identifier: Box::new(identifier),
            datatype: Box::new(Some(datatype)),
            expression: Box::new(expression),
        }
    }

    /// Parses a pest token pair into an AST construct statement, with datatype
    fn parse_pair_full_construct_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let identifier = self.parse_pair_node(pair.next().unwrap());
        let datatype = self.parse_pair_node(pair.next().unwrap());
        let expression = self.parse_pair_node(pair.next().unwrap());

        ASTNode::CONSTRUCT {
            identifier: Box::new(identifier),
            datatype: Box::new(Some(datatype)),
            expression: Box::new(expression),
        }
    }

    /// Parses a pest token pair into an AST construct statement, without datatype
    fn parse_pair_inferred_construct_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let identifier = self.parse_pair_node(pair.next().unwrap());
        let expression = self.parse_pair_node(pair.next().unwrap());

        ASTNode::CONSTRUCT {
            identifier: Box::new(identifier),
            datatype: Box::new(None),
            expression: Box::new(expression),
        }
    }

    /// Parses a pest token pair into an AST construct statement, without expression
    fn parse_pair_empty_construct_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let identifier = self.parse_pair_node(pair.next().unwrap());
        let datatype = self.parse_pair_node(pair.next().unwrap());

        ASTNode::EMPTY_CONSTRUCT {
            identifier: Box::new(identifier),
            datatype: Box::new(datatype),
        }
    }

    /// Parses a pest token pair into an AST external construct statement
    fn parse_pair_external_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let identifier = self.parse_pair_node(pair.next().unwrap());

        ASTNode::EXTERN {
            identifier: Box::new(identifier),
        }
    }

    /// Parses a pest token pair into an AST assignment statement
    fn parse_pair_assignment_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let pointer_level = pair.next().unwrap().as_str().len();
        let identifier = Box::new(self.parse_pair_node(pair.next().unwrap()));
        let mut array_index = Vec::new();
        let mut expression = self.parse_pair_node(pair.next().unwrap());
        while let Some(_) = pair.peek() {
            array_index.push(expression);
            expression = self.parse_pair_node(pair.next().unwrap());
        }
        let expression = Box::new(expression);

        ASTNode::ASSIGNMENT {
            identifier,
            pointer_level,
            array_index,
            expression,
        }
    }

    /// Parses a pest token pair into an AST if statement
    fn parse_pair_if_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let condition = self.parse_pair_node(pair.next().unwrap());
        let if_branch = self.parse_pair_node(pair.next().unwrap());
        let else_branch = match pair.next() {
            Some(item) => Some(self.parse_pair_node(item)),
            None => None,
        };

        ASTNode::BRANCH {
            condition: Box::new(condition),
            if_branch: Box::new(if_branch),
            else_branch: Box::new(else_branch),
        }
    }

    /// Parses a pest token pair into an AST for statement
    fn parse_pair_for_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let initialization = self.parse_pair_node(pair.next().unwrap());
        let condition = self.parse_pair_node(pair.next().unwrap());
        let advancement = self.parse_pair_node(pair.next().unwrap());
        let body = self.parse_pair_node(pair.next().unwrap());

        ASTNode::FOR_LOOP {
            initialization: Box::new(initialization),
            condition: Box::new(condition),
            advancement: Box::new(advancement),
            body: Box::new(body),
        }
    }

    /// Parses a pest token pair into an AST while statement
    fn parse_pair_while_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let condition = self.parse_pair_node(pair.next().unwrap());
        let body = self.parse_pair_node(pair.next().unwrap());

        ASTNode::WHILE_LOOP {
            condition: Box::new(condition),
            body: Box::new(body),
        }
    }

    /// Parses a pest token pair into an AST print statement
    fn parse_pair_print_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let expression = self.parse_pair_node(pair.next().unwrap());

        ASTNode::PRINT {
            expression: Box::new(expression),
        }
    }

    /// Parses a pest token pair into an AST function statement
    fn parse_pair_function(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let func_identifier = self.parse_pair_node(pair.next().unwrap());

        let mut parameters = Vec::new();
        while pair.peek().is_some() && pair.peek().unwrap().as_rule() == Rule::func_param {
            parameters.push(self.parse_pair_node(pair.next().unwrap()))
        }

        let return_type = if pair.peek().is_some() && pair.peek().unwrap().as_rule() != Rule::global_scope_block {
            Some(self.parse_pair_node(pair.next().unwrap()))
        } else {
            None
        };

        let body = self.parse_pair_node(pair.next().unwrap());

        ASTNode::FUNCTION {
            identifier: Box::new(func_identifier),
            parameters,
            return_type: Box::new(return_type),
            body: Box::new(body),
        }
    }

    /// Parses a pest token pair into an AST function parameter.
    /// Function parameters are defined in the function definition.
    fn parse_pair_function_parameter(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let identifier = self.parse_pair_node(pair.next().unwrap());

        let datatype = match pair.next() {
            Some(datatype_pair) => Some(self.parse_pair_node(datatype_pair)),
            None => None,
        };

        ASTNode::PARAMETER {
            identifier: Box::new(identifier),
            datatype: Box::new(datatype),
        }
    }

    /// Parses a pest token pair into an AST return statement
    fn parse_pair_return_statement(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let expression = self.parse_pair_node(pair.next().unwrap());

        ASTNode::RETURN {
            expression: Box::new(expression),
        }
    }

    /// Parses a pest token pair into an AST function call statement
    fn parse_pair_function_call(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let identifier = self.parse_pair_node(pair.next().unwrap());
        let mut arguments = Vec::new();

        while pair.peek().is_some() && pair.peek().unwrap().as_rule() == Rule::func_arg {
            arguments.push(self.parse_pair_node(pair.next().unwrap()))
        }

        ASTNode::FUNC_CALL {
            identifier: Box::new(identifier),
            arguments,
        }
    }

    /// Parses a pest token pair into an AST function call statement
    fn parse_pair_naked_function_call(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let func_call = self.parse_pair_node(pair.next().unwrap());

        ASTNode::NAKED_FUNC_CALL {
            func_call: Box::new(func_call),
        }
    }

    /// Parse a pest token pair into an AST function argument.
    /// Function Arguments are the expressions in a function call
    fn parse_pair_function_argument(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        self.parse_pair_node(pair.next().unwrap())
    }

    /// Parses a pest token pair into an AST statement list
    fn parse_pair_scope_block(&self, pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let body = self.parse_pair_node(pair.next().unwrap());

        ASTNode::SCOPE_BLOCK {
            inner: Box::new(body),
            scope: ScopeId::default(),
        }
    }

    /// Parses a pest token pair into an AST Unary Operation
    fn parse_pair_unary_op(&self, pair: pest::iterators::Pair<Rule>) -> Option<UnaryOperation> {
        match pair.as_rule() {
            Rule::unary_not => Some(UnaryOperation::NOT),
            Rule::unary_neg => Some(UnaryOperation::NEGATE),
            Rule::dereference => Some(UnaryOperation::PTR_DEREF),
            _ => None,
        }
    }

    /// Parses a pest token pair into an AST Binary Operation
    fn parse_pair_binary_op(&self, pair: pest::iterators::Pair<Rule>) -> Option<BinaryOperation> {
        match pair.as_rule() {
            Rule::add => Some(BinaryOperation::ADD),
            Rule::sub => Some(BinaryOperation::SUB),
            Rule::div => Some(BinaryOperation::DIV),
            Rule::mul => Some(BinaryOperation::MUL),
            Rule::modulus => Some(BinaryOperation::MOD),
            Rule::pow => Some(BinaryOperation::POW),
            Rule::equal => Some(BinaryOperation::EQUAL),
            Rule::not_equal => Some(BinaryOperation::NOT_EQUAL),
            Rule::greater_than => Some(BinaryOperation::GREATER_THAN),
            Rule::less_than => Some(BinaryOperation::LESS_THAN),
            Rule::greater_equal => Some(BinaryOperation::GREATER_EQUAL),
            Rule::less_equal => Some(BinaryOperation::LESS_EQUAL),
            _ => None,
        }
    }

}

/// AstParser Trait Concrete Implementation
impl AstParser for PestBarracudaParser {

    /// PestBarracudaParser has no configuration the
    /// default is just instantiation
    fn default() -> Self {
        Self {
            precision: 32,
        }
    }

    /// Parse processes a source string into an  
    fn parse(mut self, source: &str, precision: usize) -> ASTNode {
        self.precision = precision;
        self.parse_into_node_tree(source)
    }
}
