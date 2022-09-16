
/// Scope Id defines the unique id associated with every scope regardless
/// of position in the ast tree
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ScopeId {
    id: u64
}

impl ScopeId {
    pub fn default() -> Self {
        ScopeId {
            id: 0,
        }
    }

    // Return global scope id: 0
    pub fn global() -> Self {
        ScopeId::default()
    }

    pub(super) fn new(id: u64) -> Self {
        ScopeId {
            id
        }
    }

    pub(super) fn set(&mut self, scope: ScopeId)  {
        self.id = scope.id;
    }
}

#[derive(Debug, Clone)]
pub struct ScopeIdGenerator {
    previous: ScopeId
}

impl ScopeIdGenerator {
    pub fn new() -> Self {
        Self {
            previous: ScopeId::default()
        }
    }
}

impl Iterator for ScopeIdGenerator {
    type Item = ScopeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.previous = ScopeId::new(self.previous.id + 1);
        Some(self.previous.clone())
    }
}