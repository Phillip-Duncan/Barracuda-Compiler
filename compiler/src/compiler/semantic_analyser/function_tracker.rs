
use crate::compiler::ast::ASTNode;

pub(crate) struct FunctionTracker {
    name: String,
    parameters: Vec<ASTNode>,
    return_type: ASTNode,
    body: ASTNode,
    implementations: Vec<ASTNode>,
}

impl FunctionTracker {
    pub fn new(name: String, parameters: Vec<ASTNode>, return_type: ASTNode, body: ASTNode,) -> Self {
        FunctionTracker { 
            name,
            parameters,
            return_type,
            body,
            implementations: Vec::new()
        }
    }
}