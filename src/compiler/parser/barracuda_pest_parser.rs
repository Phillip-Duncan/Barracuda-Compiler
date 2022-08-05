use crate::pest::Parser;
use super::AstParser;
use super::super::ast::{
    AbstractSyntaxTree,
    ASTNode,
    Literal,
    BinaryOperation,
    UnaryOperation
};

#[derive(Parser)]
#[grammar = "barracuda.pest"]
struct BarracudaParser;


pub struct PestBarracudaParser;

impl PestBarracudaParser {
    fn parse_into_node_tree(source: &str) -> ASTNode {
        match BarracudaParser::parse(Rule::program, source) {
            Ok(pairs) => {
                for pair in pairs {
                    match pair.as_rule() {
                        Rule::statement_list => {
                            return Self::parse_pair_node(pair)
                        },
                        _ => {}
                    }
                }
            },
            Err(error) => {
                panic!("Syntax Error: {}", error)
            }
        }
        panic!("Program has been parsed without error but is empty.")
    }

    fn parse_pair_node(pair: pest::iterators::Pair<Rule>) -> ASTNode {
        match pair.as_rule() {
            Rule::identifier => {
                ASTNode::IDENTIFIER(String::from(pair.as_str()))
            },
            Rule::integer => {
                ASTNode::LITERAL(Literal::INTEGER(pair.as_str().parse().unwrap()))
            },
            Rule::decimal => {
                ASTNode::LITERAL(Literal::FLOAT(pair.as_str().parse().unwrap()))
            },
            Rule::boolean => {
                ASTNode::LITERAL(Literal::BOOL(pair.as_str().parse().unwrap()))
            },
            Rule::equality | Rule::comparison |
            Rule::term | Rule::factor |
            Rule::exponent => {
                let mut pair = pair.into_inner();

                // Convert linear list of binary operations of equal precedence
                // Into AST tree of binary operations
                let mut lhs = Self::parse_pair_node(pair.next().unwrap());
                while pair.peek().is_some() {
                    let op = Self::parse_pair_binary(pair.next().unwrap()).unwrap();
                    let rhs = Self::parse_pair_node(pair.next().unwrap());
                    lhs = ASTNode::BINARY_OP {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs)
                    }
                }

                return lhs
            },
            Rule::unary => {
                let mut pair = pair.into_inner();
                let primary_or_operator = pair.next().unwrap();
                // Unary
                if pair.peek().is_some() {
                    let op = Self::parse_pair_unary(primary_or_operator).unwrap();
                    let rhs = Self::parse_pair_node(pair.next().unwrap());

                    ASTNode::UNARY_OP {
                        op,
                        expression: Box::new(rhs)
                    }
                    // Skip as primary
                } else {
                    Self::parse_pair_node(primary_or_operator)
                }
            },
            Rule::statement_list => {
                ASTNode::STATEMENT_LIST(
                    pair.into_inner().map(Self::parse_pair_node).collect()
                )
            },
            Rule::construct_statement => {
                let mut pair = pair.into_inner();

                let identifier = Self::parse_pair_node(pair.next().unwrap());

                let mut datatype = None;
                let datatype_or_expression = Self::parse_pair_node(pair.next().unwrap());
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
            },
            Rule::assign_statement => {
                let mut pair = pair.into_inner();
                ASTNode::ASSIGNMENT {
                    identifier: Box::new(Self::parse_pair_node(pair.next().unwrap())),
                    expression: Box::new(Self::parse_pair_node(pair.next().unwrap()))
                }
            },
            Rule::if_statement => {
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
            },
            Rule::for_statement => {
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
            },
            Rule::while_statement => {
                let mut pair = pair.into_inner();
                let condition = Self::parse_pair_node(pair.next().unwrap());
                let body = Self::parse_pair_node(pair.next().unwrap());
                ASTNode::WHILE_LOOP {
                    condition: Box::new(condition),
                    body: Box::new(body)
                }
            },
            Rule::print_statement => {
                let mut pair = pair.into_inner();
                let expression = Self::parse_pair_node(pair.next().unwrap());

                ASTNode::PRINT {
                    expression: Box::new(expression)
                }
            },
            Rule::func_statement => {
                let mut pair = pair.into_inner();
                let func_identifier = Self::parse_pair_node(pair.next().unwrap());

                let mut parameters = Vec::new();
                while pair.peek().unwrap().as_rule() == Rule::func_param {
                    parameters.push(Self::parse_pair_node(pair.next().unwrap()))
                }

                let return_type = if pair.peek().unwrap().as_rule() == Rule::identifier {
                    Some(Self::parse_pair_node(pair.next().unwrap()))
                } else {
                    None
                };

                let body = Self::parse_pair_node(pair.next().unwrap());

                ASTNode::FUNCTION {
                    identifier: Box::new(func_identifier),
                    parameters,
                    return_type: Box::new(return_type),
                    body: Box::new(body)
                }
            },
            Rule::func_param => {
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
            },
            Rule::func_arg => {
                let mut pair = pair.into_inner();
                Self::parse_pair_node(pair.next().unwrap())
            }
            Rule::func_call => {
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
            Rule::return_statement => {
                let mut pair = pair.into_inner();
                let expression = Self::parse_pair_node(pair.next().unwrap());

                ASTNode::RETURN {
                    expression: Box::new(expression)
                }
            }
            _ => { panic!("Whoops") }
        }
    }

    fn parse_pair_unary(pair: pest::iterators::Pair<Rule>) -> Option<UnaryOperation> {
        match pair.as_rule() {
            Rule::unary_not => Some(UnaryOperation::NOT),
            Rule::unary_neg => Some(UnaryOperation::NEGATE),
            _ => None
        }
    }
    
    fn parse_pair_binary(pair: pest::iterators::Pair<Rule>) -> Option<BinaryOperation> {
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

impl AstParser for PestBarracudaParser {
    fn default() -> Self {
        Self {}
    }

    fn parse(self, source: &str) -> AbstractSyntaxTree {
        AbstractSyntaxTree::new(Self::parse_into_node_tree(source))
    }
}