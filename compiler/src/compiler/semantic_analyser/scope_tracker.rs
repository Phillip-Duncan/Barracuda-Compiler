use std::collections::HashMap;

use crate::compiler::ast::symbol_table::SymbolType;

pub(crate) struct ScopeTracker {
    scopes: Vec<HashMap<String, SymbolType>>,
}

// A lightweight scope tracker made for semantic analysis.
// The scope tracker used for parsing is much more complex.
// That second scope tracker could/should probably be removed with a better implementation.
// I'm not quite sure how to do that, though.
impl ScopeTracker {
    pub fn new() -> Self {
        ScopeTracker { scopes: vec![HashMap::new()] }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn add_symbol(&mut self, identifier: &String, symbol_type: SymbolType) {
        let current_scope = self
            .scopes
            .last_mut().unwrap_or_else(|| {panic!("No scopes active! Scope tracker malformed!")});
        if current_scope.contains_key(identifier) {
            panic!("Identifier '{}' already declared in this scope", identifier);
        } else {
            current_scope.insert(identifier.to_string(), symbol_type);
        }
    }

    pub fn find_symbol(&self, identifier: &String) -> Option<&SymbolType> {
        for (scope_index, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(identifier) {
                return scope.get(identifier)
            }
        }
        None
    }
}