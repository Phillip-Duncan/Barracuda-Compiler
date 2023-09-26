use std::collections::HashMap;

use crate::compiler::ast::{symbol_table::SymbolType, datatype::DataType};

pub(crate) struct ScopeTracker {
    scopes: Vec<HashMap<String, SymbolType>>,
    return_types: Vec<Option<DataType>>,
}

// A lightweight scope tracker made for semantic analysis.
// The scope tracker used for parsing is much more complex.
// That second scope tracker could/should probably be removed with a better implementation.
// I'm not quite sure how to do that, though.
impl ScopeTracker {
    pub fn new() -> Self {
        ScopeTracker { scopes: vec![HashMap::new()], return_types: vec![] }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.return_types.push(None);
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
            self.return_types.pop();
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
        for (_, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(identifier) {
                return scope.get(identifier)
            }
        }
        None
    }

    pub fn add_return_type(&mut self, identifier: &String, datatype: &DataType) {
        let new_type = match self.return_types.last().unwrap() {
            Some(return_type) => {
                if return_type == datatype {
                    datatype.clone()
                } else {
                    panic!("Return types shoudl always be equal in a function! (currently {:?} vs {:?})", return_type, datatype);
                }
            },
            None => datatype.clone()
        };
        self.return_types[self.return_types.len() - 1] = Some(new_type);
    }

    pub fn get_return_type(&self) -> &DataType {
        match self.return_types.last().unwrap() {
            Some(datatype) => datatype,
            None => &DataType::NONE
        }
    }
}