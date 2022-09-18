use compiler::ast::symbol_table::{SymbolTable, Symbol};
use compiler::ast::ScopeId;

/// Scope Tracker is a utility function for behavioural tracking changes in the current
/// scope id. It wraps around a symbol table and will track the scope with the methods
/// enter_scope and exit_scope. This is helpful for scope tracking with recursive methods
/// on the AST tree.
pub(crate) struct ScopeTracker {
    current_scope: ScopeId,
    symbol_table: Option<SymbolTable>
}

impl ScopeTracker {
    pub fn default() -> Self {
        ScopeTracker {
            current_scope: ScopeId::default(),
            symbol_table: None
        }
    }

    pub fn new(symbol_table: SymbolTable) -> Self {
        ScopeTracker {
            current_scope: ScopeId::global(),
            symbol_table: Some(symbol_table)
        }
    }

    /// Enter a scope
    pub fn enter_scope(&mut self, id: ScopeId) {
        self.current_scope.set(id);
    }

    /// Exit current scope. If no parent scope can be found defaults to global scope
    pub fn exit_scope(&mut self) {
        match &self.symbol_table {
            Some(symbol_table) => {
                self.current_scope.set(symbol_table.parent_of(self.current_scope.clone()))
            }
            None => {
                self.current_scope.set(ScopeId::global());
            }
        }
    }

    /// Attempts to find symbol within currently tracked scope
    pub fn find_symbol(&self, identifier: &String) -> Option<&Symbol> {
        match &self.symbol_table {
            Some(symbol_table) => {
                symbol_table.find_symbol(self.current_scope.clone(), identifier)
            }
            None => None
        }
    }
}