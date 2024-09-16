use super::ast::ASTNode;
pub mod barracuda_pest_parser;

/// Parser handles interpretation of high-level tokens into the intermediate
/// representation. Put another way the parser turns a source string into an
/// abstract syntax tree.
pub trait AstParser {
    /// Creates a default configuration of an AstParser
    fn default() -> Self;

    /// Parse a source string into an Abstract Syntax Tree
    fn parse(self, source: &str, precision: usize) -> ASTNode;
}

// Concrete Definition Export
pub use self::barracuda_pest_parser::PestBarracudaParser;