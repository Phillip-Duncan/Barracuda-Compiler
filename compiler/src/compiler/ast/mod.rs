pub(super) mod literals;
pub(super) mod operators;
pub(super) mod ast_node;
pub(super) mod scope;
pub(super) mod symbol_table;
pub(super) mod datatype;
pub(super) mod scope_tracker;

use std::collections::HashMap;

pub(super) use self::{
    ast_node::ASTNode,
    literals::Literal,
    scope::ScopeId,
    operators::{
        UnaryOperation,
        BinaryOperation
    },
    scope_tracker::ScopeTracker
};

pub mod environment_symbol_context;

pub use self::environment_symbol_context::EnvironmentSymbolContext;

use self::symbol_table::SymbolTable;

use super::semantic_analyser::function_tracker::FunctionTracker;

/// Intermediate Representation of the compiler model
/// This model is represented as a tree using the ASTNode enum.
/// Each node on this tree is representative of a statement or expression
/// involved in the construction of a program.
pub struct AbstractSyntaxTree {
    root: ASTNode,
    symbol_table: SymbolTable
}

impl AbstractSyntaxTree {
    pub fn new(root: ASTNode, env_vars: EnvironmentSymbolContext, functions: HashMap<String, FunctionTracker>) -> Self {
        let mut root = root;
        let symbol_table = SymbolTable::from(&mut root, env_vars.into(), functions.clone());

        Self {
            root,
            symbol_table
        }
    }

    /// Return cloned copy of symbol table
    pub fn get_symbol_table(&self) -> SymbolTable {
        self.symbol_table.clone()
    }

    /// Return copy of functions
    pub fn get_functions(&self) -> HashMap<String, FunctionTracker> {
        self.symbol_table.get_functions()
    }

    /// Convert AST into ASTNode
    pub fn into_root(self) -> ASTNode {
        self.root
    }
}