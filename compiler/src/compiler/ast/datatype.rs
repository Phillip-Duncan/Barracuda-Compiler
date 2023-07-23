use super::ast_node::ASTNode;

/// Primitive Data types supported by the AST Model
#[derive(Debug, Clone)]
pub enum PrimitiveDataType {
    F64,
    F32,
    U64,
    U32,
    U16,
    U8,
    S64,
    S32,
    S16,
    S8,
    Void
}

impl PrimitiveDataType {
    /// Convert a string representation to a primitive data type
    pub fn parse(datatype: String) -> Option<PrimitiveDataType> {
        Some(match datatype.to_lowercase().trim() {
            "f64" => {Self::F64},
            "f32" => {Self::F32},
            "u64" => {Self::U64},
            "u32" => {Self::U32},
            "u16" => {Self::U16},
            "u8" =>  {Self::U8},
            "s64" => {Self::S64},
            "s32" => {Self::S32},
            "s16" => {Self::S16},
            "s8" =>  {Self::S8},
            "bool" => {Self::U8},
            "void" => {Self::Void}
            _ => {return None}
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
// Dubious implementation of datatypes that crowbars in arrays.
// I will think about this more later on in the implementation.
// When a full type system is implemented much of the other code in this file will be needed.
pub enum DataType {
    MUTABLE(PrimitiveDataType),
    CONST(PrimitiveDataType),
    ARRAY(usize),
    UNKNOWN
}

impl DataType {

    /// Get datatype from assumed array AST node (e.g. let a: [4] = [1,2,3,4];)
    pub fn from(node: &ASTNode) -> Self {
        match node {
            ASTNode::LITERAL(super::Literal::INTEGER(value)) => DataType::ARRAY(value.clone() as usize),
            ASTNode::IDENTIFIER(_) => DataType::MUTABLE(PrimitiveDataType::Void), // As datatypes are not properly implemented
            _ => panic!("{:?}", node)
        }
    }
}

