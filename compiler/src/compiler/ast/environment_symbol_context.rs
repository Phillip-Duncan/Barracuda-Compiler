use super::datatype::PrimitiveDataType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EnvironmentSymbolContext {
    environment_variable_addresses: HashMap<String, (usize, PrimitiveDataType, String)>
}

impl EnvironmentSymbolContext {
    pub fn new() -> Self {
        Self {
            environment_variable_addresses: Default::default()
        }
    }

    #[allow(dead_code)] // Linter False Positive
    pub fn add_symbol(&mut self, identifier: String, address: usize, datatype: PrimitiveDataType, qualifier: String) -> bool {
        self.environment_variable_addresses.insert(identifier, (address, datatype, qualifier)).is_some()
    }

    pub fn into(self) -> HashMap<String, (usize, PrimitiveDataType, String)> {
        return self.environment_variable_addresses;
    }
}