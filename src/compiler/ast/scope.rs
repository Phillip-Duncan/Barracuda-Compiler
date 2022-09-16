
/// Scope Id defines the unique id associated with every scope regardless
/// of position in the ast tree
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScopeId {
    id: u64
}

impl ScopeId {
    pub fn default() -> Self {
        ScopeId {
            id: 0,
        }
    }

    pub(super) fn new(id: u64) -> Self {
        ScopeId {
            id
        }
    }
}