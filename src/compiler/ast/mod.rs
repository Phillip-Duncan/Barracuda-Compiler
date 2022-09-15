pub(super) mod literals;
pub(super) mod operators;
pub(super) mod ast_node;

pub(super) use self::{
    ast_node::ASTNode,
    literals::Literal,
    operators::{
        UnaryOperation,
        BinaryOperation
    }
};

/// Intermediate Representation of the compiler model
pub struct AbstractSyntaxTree {
    root: ASTNode
}

impl AbstractSyntaxTree {
    pub fn new(root: ASTNode) -> Self {
        Self {
            root
        }
    }

    pub fn into_root(self) -> ASTNode {
        self.root
    }
}