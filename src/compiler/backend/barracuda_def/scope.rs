use std::collections::HashMap;
use super::symbols::BarracudaSymbol;
use std::cmp::max;

pub struct BarracudaScope {
    scope_levels: Vec<HashMap<String, BarracudaSymbol>>,
    scope_mutable_size: usize, // Can be calculated from scope but is manually maintained for ease
    scope_mutable_max_size: usize, // Kept track to calculate the variable space that needs to be reserved
    pub scope_total_mutable: usize
}

impl BarracudaScope {
    pub fn new () -> BarracudaScope {
        BarracudaScope {
            scope_levels: vec![],
            scope_mutable_size: 0,
            scope_mutable_max_size: 0,
            scope_total_mutable: 0
        }
    }

    pub fn get_symbol(&self, symbol_name: &String) -> Option<BarracudaSymbol> {
        // Checks if symbol is found in scope levels
        for scope in self.scope_levels.iter().rev() {
            if let Some(symbol) = scope.get(&*symbol_name) {
                return Some(symbol.clone())
            }
        }

        return None
    }

    pub fn next_mutable_id(&self) -> usize {
        return self.scope_total_mutable;
    }

    pub fn add_symbol(&mut self, symbol: BarracudaSymbol) {

        if symbol.mutable {
            self.scope_mutable_size += 1;
            self.scope_total_mutable += 1;
        }
        self.scope_mutable_max_size = max(self.scope_mutable_max_size, self.scope_mutable_size);

        self.scope_levels.last_mut().unwrap().insert(symbol.name.clone(), symbol);
    }

    pub fn add_level(&mut self) {
        self.scope_levels.push(HashMap::new());
    }

    pub fn remove_level(&mut self) {
        let level = self.scope_levels.last().unwrap();
        let mutable_symbol_count = level.values().filter(|sym| sym.mutable).count();
        self.scope_mutable_size -= mutable_symbol_count;
        self.scope_levels.pop();
    }
}