use super::ast_node::ASTNode;

/// Primitive Data types supported by the AST Model
#[derive(Debug, Clone)]
pub enum PrimitiveDataType {
    F64,
    F32,
    F16,
    F8,
    U64,
    U32,
    U16,
    U8,
    Bool
}

impl PrimitiveDataType {
    /// Convert a string representation to a primitive data type
    pub fn parse(datatype: String) -> Option<PrimitiveDataType> {
        Some(match datatype.to_lowercase().trim() {
            "f64" => {Self::F64},
            "f32" => {Self::F32},
            "f16" => {Self::F16},
            "f8"  => {Self::F8},
            "u64" => {Self::U64},
            "u32" => {Self::U32},
            "u16" => {Self::U16},
            "u8" =>  {Self::U8},
            "bool" => {Self::Bool}
            _ => {return None}
        })
    }
}

#[derive(Debug, Clone)]
pub enum DataType {
    MUTABLE(PrimitiveDataType),
    CONST(PrimitiveDataType),
    ARRAY(Box<DataType>, usize),
}

impl DataType {
    pub fn from(node: &ASTNode) -> Self {
        panic!("Datatypes not implemented! {:?}", node);
    }
}

