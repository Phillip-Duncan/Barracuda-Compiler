use super::datatype::PrimitiveDataType;
use super::qualifiers::Qualifier;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EnvironmentSymbolContext {
    environment_variable_addresses: HashMap<String, (usize, PrimitiveDataType, Qualifier, String)>
}

impl EnvironmentSymbolContext {
    pub fn new() -> Self {
        Self {
            environment_variable_addresses: Default::default()
        }
    }

    #[allow(dead_code)] // Linter False Positive
    pub fn add_symbol(&mut self, identifier: String, address: usize, datatype: PrimitiveDataType, qualifier: Qualifier, ptr_levels: String) -> bool {
        self.environment_variable_addresses.insert(identifier, (address, datatype, qualifier, ptr_levels)).is_some()
    }

    pub fn into(self) -> HashMap<String, (usize, PrimitiveDataType, Qualifier, String)> {
        return self.environment_variable_addresses;
    }

    pub fn copy_addresses(&self) -> HashMap<String, (usize, PrimitiveDataType, Qualifier, String)> {
        return self.environment_variable_addresses.clone();
    }
}