use super::{SemanticAnalyser, EnvironmentSymbolContext};
use super::super::ast::{
    AbstractSyntaxTree,
    ASTNode,
};



/// BarracudaSemanticAnalyser is a concrete SemanticAnalyser.
pub struct BarracudaSemanticAnalyser;

impl BarracudaSemanticAnalyser {

    /// Parses all pest pair tokens into a valid ASTNode
    fn analyse_node(node: ASTNode) -> ASTNode {
        match node {
            _ => { panic!("Whoops! Unidentifiable node: {:?}", node) }
        }
    }
}

/// AstParser Trait Concrete Implementation 
impl SemanticAnalyser for BarracudaSemanticAnalyser {
    fn default() -> Self {
        Self {}
    }

    /// Parse processes a source string into an abstract syntax tree
    fn analyse(self, root_node: ASTNode, env_vars: EnvironmentSymbolContext) -> AbstractSyntaxTree { 
        AbstractSyntaxTree::new(BarracudaSemanticAnalyser::analyse_node(root_node), env_vars)
    }
}