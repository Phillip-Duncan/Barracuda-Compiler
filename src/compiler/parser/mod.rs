use super::ast::AbstractSyntaxTree;
pub mod barracuda_pest_parser;

/// Parser handles interpretation of high-level tokens into the intermediate
/// representation.
pub trait AstParser {
    fn default() -> Self;
    fn parse(self, source: &str) -> AbstractSyntaxTree;
}

// Concrete Definition Export
pub use self::barracuda_pest_parser::PestBarracudaParser;
