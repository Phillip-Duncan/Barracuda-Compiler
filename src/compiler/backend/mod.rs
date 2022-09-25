mod barracuda_bytecode_generator;
mod program_code_builder;

use super::ast::AbstractSyntaxTree;
use super::program_code::ProgramCode;

// Abstract Definitions

/// BackEndGenerator takes an AbstractSyntaxTree and generate ProgramCode
pub trait BackEndGenerator {
    /// Generate default generator configuration
    fn default() -> Self;

    /// Generate program code from an abstract syntax tree
    fn generate(self, tree: AbstractSyntaxTree) -> ProgramCode;
}

// Concrete Definition Export
pub use self::barracuda_bytecode_generator::BarracudaByteCodeGenerator;

