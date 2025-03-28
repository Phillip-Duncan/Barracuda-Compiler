#[derive(Debug, Clone)]
pub enum Qualifier {
    CONSTANT,
    MUTABLE,
}

impl Qualifier {
    pub fn from_str(qualifier: String) -> Qualifier {
        match qualifier.to_lowercase().trim() {
            "const" => {Qualifier::CONSTANT},
            "mut" => {Qualifier::MUTABLE},
            _ => {Qualifier::CONSTANT}
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            Qualifier::CONSTANT => "const".to_string(),
            Qualifier::MUTABLE => "mut".to_string(),
        }
    }
}

impl PartialEq for Qualifier {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Qualifier::CONSTANT, Qualifier::CONSTANT) => true,
            (Qualifier::MUTABLE, Qualifier::MUTABLE) => true,
            _ => false,
        }
    }
}