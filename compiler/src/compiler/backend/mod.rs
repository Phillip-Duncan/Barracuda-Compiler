mod barracuda_bytecode_generator;
mod program_code_builder;
pub mod analysis;
pub mod builtin_functions;

use super::ast::AbstractSyntaxTree;
use barracuda_common::ProgramCode;

// Abstract Definitions

/// BackEndGenerator takes an AbstractSyntaxTree and generate ProgramCode
pub trait BackEndGenerator {
    /// Generate default generator configuration
    fn default() -> Self;

    /// Generate program code from an abstract syntax tree
    fn generate(self, tree: AbstractSyntaxTree) -> ProgramCode;

    fn add_environment_variable(&mut self);

    fn set_precision(&mut self, precision: usize);
}

// Concrete Definition Export
pub use self::barracuda_bytecode_generator::BarracudaByteCodeGenerator;

