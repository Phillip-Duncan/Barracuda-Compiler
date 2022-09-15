/// Barracuda Defs contains specific helper functions for defining symbols and scope
/// TODO(Connor): The Scope tracker and symbols feel more generic than to the backend. In future
///               it would be better to redesign them and generate this data for the AST.
pub(super) mod symbols;
pub(super) mod scope;