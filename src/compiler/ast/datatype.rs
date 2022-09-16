use super::ast_node::ASTNode;

/// Primitive Data types supported by thr AST Model
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
pub enum DataType {
    MUTABLE(PrimitiveDataType),
    CONST(PrimitiveDataType),
    UNKNOWN
}

impl DataType {

    /// Get datatype from assumed identifier ast node
    pub fn from(node: &ASTNode) -> Self {
        let datatype_str = match node {
            ASTNode::IDENTIFIER(name) => name.clone(),
            _ => panic!("{:?}", node)
        };

        // TODO(Connor): Currently assumes that datatype is mutable
        match PrimitiveDataType::parse(datatype_str) {
            Some(primitive_type) => {DataType::MUTABLE(primitive_type)}
            None => { DataType::UNKNOWN }
        }
    }
}

