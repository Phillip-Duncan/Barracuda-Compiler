
use crate::compiler::ast::{ASTNode, datatype::DataType};

use super::BarracudaSemanticAnalyser;

pub(crate) struct FunctionTracker {
    name: String,
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
    pub fn new(name: String, parameters: Vec<ASTNode>, return_type: ASTNode, body: ASTNode) -> Self {
        let mut parameter_types = vec![];
        for parameter in parameters {
            let datatype = match parameter {
                ASTNode::PARAMETER { datatype, .. } => match datatype.as_ref() {
                    Some(datatype) => match datatype {
                        ASTNode::DATATYPE(datatype) => Some(datatype.clone()),
                        _ => panic!("Malformed AST! Node {:?} should have been a datatype but wasn't!", datatype)
                    },
                    None => None
                },
                _ => panic!("Malformed AST! Parameter wasn't a parameter, instead it was {:?}", parameter)
            };
            parameter_types.push(datatype);
        }
        // TODO optional return types
        let return_type = match return_type {
            ASTNode::DATATYPE(datatype) => Some(datatype),
            _ => panic!("Malformed AST! Return type wasn't a datatype, instead it was {:?}", return_type)
        };
        FunctionTracker { 
            name,
            parameters: parameter_types,
            return_type,
            body,
            implementations: Vec::new()
        }
    }

    pub fn match_or_create(&mut self, arguments: Vec<DataType>, semantic_analyser: &mut BarracudaSemanticAnalyser) -> (String, DataType) {
        for implementation in &self.implementations {
            if implementation.matches_arguments(&arguments) {
                return (implementation.name(), implementation.return_type())
            }
        }
        let implementation = FunctionImplementation::new(
            &self.name, 
            &self.parameters, 
            &arguments, 
            &self.return_type, 
            &self.body,
            semantic_analyser);
        let return_val = (implementation.name(), implementation.return_type());
        self.implementations.push(implementation);
        return return_val;
    }
}

pub(crate) struct FunctionImplementation {
    name: String,
    parameters: Vec<DataType>,
    return_type: DataType,
    body: ASTNode
}

impl FunctionImplementation {
    pub fn new(name: &String, parameters: &Vec<Option<DataType>>, arguments: &Vec<DataType>, return_type: &Option<DataType>, body: &ASTNode, semantic_analyser: &mut BarracudaSemanticAnalyser) -> Self {
        if parameters.len() != arguments.len() {
            panic!("When calling function {}, need to use {} parameters! (Used {})", name, parameters.len(), arguments.len())
        }
        let mut param_types = vec![];
        for (parameter, argument) in parameters.iter().zip(arguments.iter()) {
            let datatype = match parameter {
                Some(parameter) => {
                    if parameter == argument {
                        parameter.clone()
                    } else {
                        panic!("Type of parameter in function {} didn't match! ({:?} vs {:?})", name, parameters.len(), arguments.len())
                    }
                }
                None => argument.clone()
            };
            param_types.push(datatype);
        }
        let (body, return_type) = semantic_analyser.analyse_function_implementation(parameters, return_type, body);
        FunctionImplementation { 
            name: name.to_string(),
            parameters: param_types,
            return_type,
            body,
        }
    }

    pub fn matches_arguments(&self, arguments: &Vec<DataType>) -> bool {
        self.parameters.iter().zip(arguments.iter()).all(|(a, b)| a == b)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn return_type(&self) -> DataType {
        self.return_type
    }
}