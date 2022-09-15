use super::literals::Literal;
use super::operators::{UnaryOperation, BinaryOperation};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ASTNode {
    IDENTIFIER(String),
    LITERAL(Literal),
    UNARY_OP {
        op: UnaryOperation,
        expression: Box<ASTNode>
    },
    BINARY_OP {
        op: BinaryOperation,
        lhs: Box<ASTNode>,
        rhs: Box<ASTNode>
    },
    CONSTRUCT {
        identifier: Box<ASTNode>,
        datatype: Box<Option<ASTNode>>,
        expression: Box<ASTNode>
    },
    ASSIGNMENT {
        identifier: Box<ASTNode>,
        expression: Box<ASTNode>
    },
    PRINT {
        expression: Box<ASTNode>
    },
    RETURN {
        expression: Box<ASTNode>
    },
    BRANCH {
        condition: Box<ASTNode>,
        if_branch: Box<ASTNode>,
        else_branch: Box<Option<ASTNode>>
    },
    WHILE_LOOP {
        condition: Box<ASTNode>,
        body: Box<ASTNode>
    },
    FOR_LOOP {
        initialization: Box<ASTNode>,
        condition: Box<ASTNode>,
        advancement: Box<ASTNode>,
        body: Box<ASTNode>
    },
    PARAMETER {
        identifier: Box<ASTNode>,
        datatype: Box<Option<ASTNode>>
    },
    FUNCTION {
        identifier: Box<ASTNode>,
        parameters: Vec<ASTNode>,
        return_type: Box<Option<ASTNode>>,
        body: Box<ASTNode>
    },
    FUNC_CALL {
        identifier: Box<ASTNode>,
        arguments: Vec<ASTNode>
    },

    STATEMENT_LIST(Vec<ASTNode>)
}