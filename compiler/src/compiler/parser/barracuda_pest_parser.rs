use crate::pest::Parser;
use super::AstParser;
use super::super::ast::{
    AbstractSyntaxTree,
    ASTNode,
    Literal,
    BinaryOperation,
    UnaryOperation,
    ScopeId,
    environment_symbol_context::EnvironmentSymbolContext
};


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
pub struct PestBarracudaParser;

impl PestBarracudaParser {

    /// Parses source string into an ASTNode.
    fn parse_into_node_tree(source: &str) -> ASTNode {
        match BarracudaParser::parse(Rule::program, source) {
            Ok(pairs) => {
                for pair in pairs {
                    match pair.as_rule() {
                        Rule::global_statement_list => {
                            return Self::parse_pair_node(pair)
                        },
                        _ => {panic!("Program should start with statement list.")}
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
    fn parse_pair_node(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        match pair.as_rule() {
            Rule::identifier =>         { Self::parse_pair_identifier(pair) },
            Rule::reference =>          { Self::parse_pair_reference(pair) },
            Rule::variable =>           { Self::parse_pair_variable(pair) },
            Rule::integer |
            Rule::decimal |
            Rule::boolean =>            { Self::parse_pair_literal(pair) },
            Rule::array =>              { Self::parse_pair_array(pair) },
            Rule::equality |
            Rule::comparison |
            Rule::term |
            Rule::factor |
            Rule::exponent =>           { Self::parse_pair_binary_expression(pair) },
            Rule::unary =>              { Self::parse_pair_unary_expression(pair) },
            Rule::index =>              { Self::parse_pair_array_index(pair) },
            Rule::global_statement_list |
            Rule::statement_list =>     { Self::parse_pair_statement_list(pair) },
            Rule::construct_statement =>{ Self::parse_pair_construct_statement(pair) },
            Rule::external_statement => { Self::parse_pair_external_statement(pair) },
            Rule::assign_statement =>   { Self::parse_pair_assignment_statement(pair) },
            Rule::if_statement =>       { Self::parse_pair_if_statement(pair) },
            Rule::for_statement =>      { Self::parse_pair_for_statement(pair) },
            Rule::while_statement =>    { Self::parse_pair_while_statement(pair) },
            Rule::print_statement =>    { Self::parse_pair_print_statement(pair) },
            Rule::func_statement =>     { Self::parse_pair_function(pair) },
            Rule::func_param =>         { Self::parse_pair_function_parameter(pair) },
            Rule::return_statement =>   { Self::parse_pair_return_statement(pair) },
            Rule::func_call =>          { Self::parse_pair_function_call(pair) },
            Rule::naked_func_call =>    { Self::parse_pair_naked_function_call(pair) },
            Rule::func_arg =>           { Self::parse_pair_function_argument(pair) },
            Rule::global_scope_block |
            Rule::scope_block =>        { Self::parse_pair_scope_block(pair) },
            _ => { panic!("Whoops! Unprocessed pest rule: {:?}", pair.as_rule()) }
        }
    }

    /// Parses a pest token pair into an AST literal
    fn parse_pair_literal(pair: pest::iterators::Pair<Rule>) -> ASTNode {
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
            _ => {panic!("Whoops! Unprocessed literal rule: {:?}", pair.as_rule())}
        }
    }

    fn parse_pair_array(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        ASTNode::ARRAY(pair.into_inner().map(Self::parse_pair_node).collect())
    }

    /// Parses a pest token pair into an AST identifier
    fn parse_pair_identifier(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        ASTNode::IDENTIFIER(String::from(pair.as_str()))
    }

    /// Parses a pest token pair into an AST reference
    fn parse_pair_reference(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        ASTNode::REFERENECE(String::from(&pair.as_str()[1..]))
    }

    /// Parses a pest token pair into an AST variable
    fn parse_pair_variable(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let input = pair.as_str();
        let references = input.chars().take_while(|&c| c == '*').count();
        ASTNode::VARIABLE {
            references,
            identifier: input[references..].to_string()
        }
    }

    /// Parses a pest token pair into an AST binary expression
    fn parse_pair_binary_expression(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();

        // Convert linear list of binary operations of equal precedence
        // Into AST tree of binary operations
        let mut lhs = Self::parse_pair_node(pair.next().unwrap());
        while pair.peek().is_some() {
            let op = Self::parse_pair_binary_op(pair.next().unwrap()).unwrap();
            let rhs = Self::parse_pair_node(pair.next().unwrap());
            lhs = ASTNode::BINARY_OP {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs)
            }
        }

        return lhs
    }

    /// Parses a pest token pair into an AST unary expression
    fn parse_pair_unary_expression(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let primary_or_operator = pair.next().unwrap();
        // Unary
        if pair.peek().is_some() {
            let op = Self::parse_pair_unary_op(primary_or_operator).unwrap();
            let rhs = Self::parse_pair_node(pair.next().unwrap());

            ASTNode::UNARY_OP {
                op,
                expression: Box::new(rhs)
            }
            // Skip as primary
        } else {
            Self::parse_pair_node(primary_or_operator)
        }
    }

    fn parse_pair_array_index(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let primary = pair.next().unwrap();
        let mut expression = Self::parse_pair_node(primary);
        // Unary
        while pair.peek().is_some() {
            let index = Self::parse_pair_node(pair.next().unwrap());

            expression = ASTNode::ARRAY_INDEX {
                index: Box::new(index),
                expression: Box::new(expression)
            };
        }
        expression
    }

    /// Parses a pest token pair into an AST statement list
    fn parse_pair_statement_list(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        ASTNode::STATEMENT_LIST(
            pair.into_inner().map(Self::parse_pair_node).collect()
        )
    }

    /// Parses a pest token pair into an AST construct statement
    fn parse_pair_construct_statement(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();

        let identifier = Self::parse_pair_node(pair.next().unwrap());

        let mut datatype = None;

        // qualifier, datatype or expression
        let datatype_or_expression = Self::parse_pair_node(pair.next().unwrap());
        
        // Datatype match
        let expression = match pair.next() {
            Some(expression_pair) => {
                datatype = Some(datatype_or_expression);
                Self::parse_pair_node(expression_pair)
            }
            None => datatype_or_expression
        };

        ASTNode::CONSTRUCT {
            identifier: Box::new(identifier),
            datatype: Box::new(datatype),
            expression: Box::new(expression)
        }
    }

    /// Parses a pest token pair into an AST external construct statement
    fn parse_pair_external_statement(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let identifier = Self::parse_pair_node(pair.next().unwrap());

        ASTNode::EXTERN {
            identifier: Box::new(identifier)
        }
    }

    /// Parses a pest token pair into an AST assignment statement
    fn parse_pair_assignment_statement(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let identifier = Box::new(Self::parse_pair_node(pair.next().unwrap()));
        let array_index_or_expression = Self::parse_pair_node(pair.next().unwrap());
        let mut array_index = Box::new(None);
        let expression = match pair.next() {
            Some(expression_pair) => {
                array_index = Box::new(Some(array_index_or_expression));
                Box::new(Self::parse_pair_node(expression_pair))
            }
            None => Box::new(array_index_or_expression)
        };

        ASTNode::ASSIGNMENT {
            identifier,
            array_index,
            expression
        }
    }

    /// Parses a pest token pair into an AST if statement
    fn parse_pair_if_statement(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let condition = Self::parse_pair_node(pair.next().unwrap());
        let if_branch = Self::parse_pair_node(pair.next().unwrap());
        let else_branch = match pair.next() {
            Some(item) => Some(Self::parse_pair_node(item)),
            None => None
        };

        ASTNode::BRANCH {
            condition: Box::new(condition),
            if_branch: Box::new(if_branch),
            else_branch: Box::new(else_branch)
        }
    }

    /// Parses a pest token pair into an AST for statement
    fn parse_pair_for_statement(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let initialization = Self::parse_pair_node(pair.next().unwrap());
        let condition = Self::parse_pair_node(pair.next().unwrap());
        let advancement = Self::parse_pair_node(pair.next().unwrap());
        let body = Self::parse_pair_node(pair.next().unwrap());

        ASTNode::FOR_LOOP {
            initialization: Box::new(initialization),
            condition: Box::new(condition),
            advancement: Box::new(advancement),
            body: Box::new(body)
        }
    }

    /// Parses a pest token pair into an AST while statement
    fn parse_pair_while_statement(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let condition = Self::parse_pair_node(pair.next().unwrap());
        let body = Self::parse_pair_node(pair.next().unwrap());

        ASTNode::WHILE_LOOP {
            condition: Box::new(condition),
            body: Box::new(body)
        }
    }

    /// Parses a pest token pair into an AST print statement
    fn parse_pair_print_statement(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let expression = Self::parse_pair_node(pair.next().unwrap());

        ASTNode::PRINT {
            expression: Box::new(expression)
        }
    }

    /// Parses a pest token pair into an AST function statement
    fn parse_pair_function(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let func_identifier = Self::parse_pair_node(pair.next().unwrap());

        let mut parameters = Vec::new();
        while pair.peek().unwrap().as_rule() == Rule::func_param {
            parameters.push(Self::parse_pair_node(pair.next().unwrap()))
        }

        let return_type = if pair.peek().unwrap().as_rule() == Rule::identifier {
            Self::parse_pair_node(pair.next().unwrap())
        } else {
            ASTNode::IDENTIFIER(String::from("void"))
        };

        let body = Self::parse_pair_node(pair.next().unwrap());

        ASTNode::FUNCTION {
            identifier: Box::new(func_identifier),
            parameters,
            return_type: Box::new(return_type),
            body: Box::new(body)
        }
    }

    /// Parses a pest token pair into an AST function parameter.
    /// Function parameters are defined in the function definition.
    fn parse_pair_function_parameter(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let identifier = Self::parse_pair_node(pair.next().unwrap());

        let datatype = match pair.next() {
            Some(datatype_pair) => Some(Self::parse_pair_node(datatype_pair)),
            None => None
        };

        ASTNode::PARAMETER {
            identifier: Box::new(identifier),
            datatype: Box::new(datatype)
        }
    }

    /// Parses a pest token pair into an AST return statement
    fn parse_pair_return_statement(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let expression = Self::parse_pair_node(pair.next().unwrap());

        ASTNode::RETURN {
            expression: Box::new(expression)
        }
    }

    /// Parses a pest token pair into an AST function call statement
    fn parse_pair_function_call(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let identifier = Self::parse_pair_node(pair.next().unwrap());
        let mut arguments = Vec::new();

        while pair.peek().is_some() && pair.peek().unwrap().as_rule() == Rule::func_arg {
            arguments.push(Self::parse_pair_node(pair.next().unwrap()))
        }

        ASTNode::FUNC_CALL {
            identifier: Box::new(identifier),
            arguments
        }
    }

    /// Parses a pest token pair into an AST function call statement
    fn parse_pair_naked_function_call(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let func_call = Self::parse_pair_node(pair.next().unwrap());

        ASTNode::NAKED_FUNC_CALL {
            func_call: Box::new(func_call)
        }
    }

    /// Parse a pest token pair into an AST function argument.
    /// Function Arguments are the expressions in a function call
    fn parse_pair_function_argument(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        Self::parse_pair_node(pair.next().unwrap())
    }

    /// Parses a pest token pair into an AST statement list
    fn parse_pair_scope_block(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        let mut pair = pair.into_inner();
        let body = Self::parse_pair_node(pair.next().unwrap());

        ASTNode::SCOPE_BLOCK {
            inner: Box::new(body),
            scope: ScopeId::default()
        }
    }

    /// Parses a pest token pair into an AST Unary Operation
    fn parse_pair_unary_op(pair: pest::iterators::Pair<Rule>) -> Option<UnaryOperation> {
        match pair.as_rule() {
            Rule::unary_not => Some(UnaryOperation::NOT),
            Rule::unary_neg => Some(UnaryOperation::NEGATE),
            _ => None
        }
    }

    /// Parses a pest token pair into an AST Binary Operation
    fn parse_pair_binary_op(pair: pest::iterators::Pair<Rule>) -> Option<BinaryOperation> {
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
            _ => None
        }
    }

}

/// AstParser Trait Concrete Implementation
impl AstParser for PestBarracudaParser {

    /// PestBarracudaParser has no configuration the
    /// default is just instantiation
    fn default() -> Self {
        Self {}
    }

    /// Parse processes a source string into an abstract syntax tree
    fn parse(self, source: &str, env_vars: EnvironmentSymbolContext) -> AbstractSyntaxTree {
        AbstractSyntaxTree::new(
            Self::parse_into_node_tree(source),
            env_vars
        )
    }
}