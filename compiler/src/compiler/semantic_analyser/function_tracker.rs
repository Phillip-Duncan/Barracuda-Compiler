
use crate::compiler::ast::{ASTNode, datatype::DataType};

pub(crate) struct FunctionTracker {
    name: String,
    parameters: Vec<ASTNode>,
    return_type: ASTNode,
    body: ASTNode,
    implementations: Vec<FunctionImplementation>,
}

impl FunctionTracker {
    pub fn new(name: String, parameters: Vec<ASTNode>, return_type: ASTNode, body: ASTNode) -> Self {
        FunctionTracker { 
            name,
            parameters,
            return_type,
            body,
            implementations: Vec::new()
        }
    }

    pub fn match_or_create(self, arguments: Vec<DataType>) -> (String, DataType) {
        for implementation in self.implementations {
            if implementation.matches_arguments(arguments) {
                return (implementation.name(), implementation.return_type())
            }
        }

        (String::from(""), DataType::NONE)
    }
}

pub(crate) struct FunctionImplementation {
    name: String,
    parameters: Vec<DataType>,
    return_type: DataType,
    body: ASTNode
}

impl FunctionImplementation {
    pub fn new(name: String, parameters: Vec<DataType>, return_type: DataType, body: ASTNode) -> Self {
        FunctionImplementation { 
            name,
            parameters,
            return_type,
            body,
        }
    }

    pub fn matches_arguments(self, arguments: Vec<DataType>) -> bool {
        self.parameters.iter().zip(arguments.iter()).all(|(a, b)| a == b)
    }

    pub fn name(self) -> String {
        self.name.clone()
    }

    pub fn return_type(self) -> DataType {
        self.return_type.clone()
    }
}