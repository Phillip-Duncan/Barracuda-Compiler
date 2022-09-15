use std::collections::HashMap;
use super::symbols::BarracudaSymbol;
use std::cmp::max;

/// BarracudaScope tracks symbols throughout the scopes in an AST.
/// Symbols being identifiers in a program such as variables and functions.
pub struct BarracudaScope {
    /// Stack of symbol lists currently in scope.
    scope_levels: Vec<HashMap<String, BarracudaSymbol>>,
    scope_mutable_size: usize, // Can be calculated from scope but is manually maintained for ease
    scope_mutable_max_size: usize, // Kept track to calculate the variable space that needs to be reserved
    pub scope_total_mutable: usize
}

impl BarracudaScope {

    /// Create an empty scope with no symbols defined.
    pub fn new () -> BarracudaScope {
        BarracudaScope {
            scope_levels: vec![],
            scope_mutable_size: 0,
            scope_mutable_max_size: 0,
            scope_total_mutable: 0
        }
    }

    /// Get symbol if it is within the current scope.
    /// @symbol_name: string identifier of symbol
    /// @return symbol if found, None otherwise
    pub fn get_symbol(&self, symbol_name: &String) -> Option<BarracudaSymbol> {
        // Checks if symbol is found in one of the scope levels
        for scope in self.scope_levels.iter().rev() {
            if let Some(symbol) = scope.get(&*symbol_name) {
                return Some(symbol.clone())
            }
        }

        return None
    }

    /// Return next available unique id for a mutable symbol
    pub fn next_mutable_id(&self) -> usize {
        return self.scope_total_mutable;
    }

    /// Add a symbol to the current scope.
    pub fn add_symbol(&mut self, symbol: BarracudaSymbol) {
        if symbol.mutable {
            self.scope_mutable_size += 1;
            self.scope_total_mutable += 1;
        }

        self.scope_mutable_max_size = max(self.scope_mutable_max_size, self.scope_mutable_size);

        // Add symbol to current scope level
        self.scope_levels.last_mut().unwrap().insert(symbol.name.clone(), symbol);
    }

    /// Add a scope level. Symbols added in this scope will be removed from scope on removal of
    /// the level.
    pub fn add_level(&mut self) {
        self.scope_levels.push(HashMap::new());
    }

    /// Removes level from scope and associated symbols in the most recent scope.
    pub fn remove_level(&mut self) {
        // Peek level to remove
        let level = self.scope_levels.last().unwrap();

        // Count mutable symbols in level and remove from mutable size of scope.
        let mutable_symbol_count = level.values().filter(|sym| sym.mutable).count();
        self.scope_mutable_size -= mutable_symbol_count;

        // Remove level
        self.scope_levels.pop();
    }
}