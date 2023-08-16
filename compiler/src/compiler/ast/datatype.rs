use super::ast_node::ASTNode;

/// Primitive Data types supported by the AST Model
#[derive(Debug, Clone)]
pub enum PrimitiveDataType {
    F128,
    F64,
    F32,
    F16,
    F8,
    I128,
    I64,
    I32,
    I16,
    I8,
    Bool
}

impl PrimitiveDataType {
    /// Convert a string representation to a primitive data type
    pub fn parse(datatype: String) -> Option<PrimitiveDataType> {
        Some(match datatype.to_lowercase().trim() {
            "f128" => {Self::F128},
            "f64" => {Self::F64},
            "f32" => {Self::F32},
            "f16" => {Self::F16},
            "f8"  => {Self::F8},
            "i128" => {Self::I128},
            "i64" => {Self::I64},
            "i32" => {Self::I32},
            "i16" => {Self::I16},
            "i8" =>  {Self::I8},
            "bool" => {Self::Bool}
            _ => {return None}
        })
    }
}

#[derive(Debug, Clone)]
pub enum DataType {
    MUTABLE(PrimitiveDataType),
    CONST(PrimitiveDataType),
    POINTER(Box<DataType>),
    ARRAY(Box<DataType>, usize),
}

impl DataType {
    pub fn from(node: &ASTNode) -> Self {
        panic!("Datatypes not implemented! {:?}", node);
    }

    pub fn from_str(datatype: String) -> Self {
        DataType::MUTABLE(PrimitiveDataType::parse(datatype).unwrap())
    }
}

