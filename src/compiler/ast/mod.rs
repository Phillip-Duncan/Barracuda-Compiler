pub(super) mod literals;
pub(super) mod operators;
pub(super) mod ast_node;
pub(super) mod scope;
pub(super) mod symbol_table;
mod datatype;

pub(super) use self::{
    ast_node::ASTNode,
    literals::Literal,
    scope::ScopeId,
    operators::{
        UnaryOperation,
        BinaryOperation
    }
};

use self::symbol_table::SymbolTable;

/// Intermediate Representation of the compiler model
/// This model is represented as a tree using the ASTNode enum.
/// Each node on this tree is representative of a statement or expression
/// involved in the construction of a program.
pub struct AbstractSyntaxTree {
    root: ASTNode,
    symbol_table: SymbolTable
}

impl AbstractSyntaxTree {
    pub fn new(root: ASTNode) -> Self {
        let mut root = root;
        let symbol_table = SymbolTable::from(&mut root);

        Self {
            root,
            symbol_table
        }
    }

    pub fn into_root(self) -> ASTNode {
        self.root
    }
}