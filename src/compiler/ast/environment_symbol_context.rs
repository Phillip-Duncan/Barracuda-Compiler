use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EnvironmentSymbolContext {
    environment_variable_addresses: HashMap<String, usize>
}

impl EnvironmentSymbolContext {
    pub fn new() -> Self {
        Self {
            environment_variable_addresses: Default::default()
        }
    }

    pub fn add_symbol(&mut self, identifier: String, address: usize) -> bool {
        self.environment_variable_addresses.insert(identifier, address).is_some()
    }

    pub fn into(self) -> HashMap<String, usize> {
        return self.environment_variable_addresses;
    }
}