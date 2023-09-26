
use crate::compiler::ast::{ASTNode, datatype::DataType};

pub(crate) struct FunctionTracker {
    parameter_names: Vec<String>,
    parameters: Vec<Option<DataType>>,
    return_type: Option<DataType>,
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
    pub fn new(parameters: Vec<ASTNode>, return_type: ASTNode, body: ASTNode) -> Self {
        let mut parameter_names = vec![];
        let mut parameter_types = vec![];
        for parameter in parameters {
            match parameter {
                ASTNode::PARAMETER { datatype, identifier } => {
                    let datatype = match datatype.as_ref() {
                        Some(datatype) => match datatype {
                            ASTNode::DATATYPE(datatype) => Some(datatype.clone()),
                            _ => panic!("Malformed AST! Node {:?} should have been a datatype but wasn't!", datatype)
                        },
                        None => None
                    };
                    let identifier = match identifier.as_ref() {
                        ASTNode::IDENTIFIER(name) => name.clone(),
                        _ => panic!("Malformed AST! Node {:?} should have been a datatype but wasn't!", datatype)
                    };
                    parameter_names.push(identifier);
                    parameter_types.push(datatype);
                },
                _ => panic!("Malformed AST! Parameter wasn't a parameter, instead it was {:?}", parameter)
            };
        }
        // TODO optional return types
        let return_type = match return_type {
            ASTNode::DATATYPE(datatype) => Some(datatype),
            _ => panic!("Malformed AST! Return type wasn't a datatype, instead it was {:?}", return_type) 
        };
        FunctionTracker {
            parameter_names,
            parameters: parameter_types,
            return_type,
            body,
            implementations: Vec::new()
        }
    }

    pub fn match_function(&self, arguments: &Vec<DataType>) -> Option<(String, DataType)> {
        for implementation in &self.implementations {
            if implementation.matches_arguments(arguments) {
                return Some((implementation.name(), implementation.return_type()))
            }
        }
        return None
    }

    pub fn get_innards(&self) -> (&Vec<Option<DataType>>, &Vec<String>, &Option<DataType>, &ASTNode) {
        (&self.parameters, &self.parameter_names, &self.return_type, &self.body)
    }

    pub fn create_implementation(&mut self, name: String, parameters: Vec<DataType>, return_type: DataType, body: ASTNode) -> String {
        let name = format!("{}:{}", name, self.implementations.len());
        let implementation = FunctionImplementation::new(name, parameters, return_type, body);
        let implementation_name = implementation.name();
        self.implementations.push(implementation);
        return implementation_name;
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
        FunctionImplementation { name, parameters, return_type, body }
    }

    pub fn matches_arguments(&self, arguments: &Vec<DataType>) -> bool {
        self.parameters.iter().zip(arguments.iter()).all(|(a, b)| a == b)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn return_type(&self) -> DataType {
        self.return_type.clone()
    }
}