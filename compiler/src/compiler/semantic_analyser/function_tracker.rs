
use crate::compiler::ast::{ASTNode, datatype::DataType};

pub(crate) struct FunctionTracker {
    name: String,
    parameters: Vec<ASTNode>,
    return_type: ASTNode,
    body: ASTNode,
    implementations: Vec<FunctionImplementation>,
}

/*
    Tracks a specific function when doing semantic analysis.
    When a function definition is found during semantic analysis, 
        it should be added to the functions in the semantic analyser using FunctionTracker::new().
    When a function call is found during semantic analysis, 
        it should be checked against the relevant FunctionTracker using match_or_create().
*/
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