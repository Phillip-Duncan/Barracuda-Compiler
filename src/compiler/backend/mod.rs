mod barracuda_bytecode_generator;
mod barracuda_def;

use super::ast::AbstractSyntaxTree;

// Abstract Definitions
pub(crate) trait CodeToken {
    fn repr(&self) -> String;
}

pub(crate) trait BackEndGenerator {
    fn default() -> Self;
    fn generate(self, tree: AbstractSyntaxTree) -> Vec<Box<dyn CodeToken>>;
}

// Concrete Definition Export
pub use self::barracuda_bytecode_generator::BarracudaByteCodeGenerator;
