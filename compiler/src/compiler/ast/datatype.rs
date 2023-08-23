use super::ast_node::ASTNode;

/// Primitive Data types supported by the AST Model
#[derive(Debug, Clone, PartialEq)]
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
    ARRAY(Box<DataType>, usize)
}

impl DataType {
    pub fn from(node: &ASTNode) -> Self {
        match node {
            ASTNode::DATATYPE(datatype) => datatype.clone(),
            _ => panic!("Node {:?} must be a datatype node to convert it to a datatype", node)
        }
    }

    pub fn from_str(datatype: String) -> Self {
        DataType::MUTABLE(PrimitiveDataType::parse(datatype).unwrap())
    }
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DataType::MUTABLE(this_type), DataType::MUTABLE(other_type))
            | (DataType::CONST(this_type), DataType::CONST(other_type)) => this_type == other_type,
            (DataType::POINTER(this_inner), DataType::POINTER(other_inner)) => this_inner == other_inner,
            (DataType::ARRAY(this_inner, this_size), DataType::ARRAY(other_inner, other_size)) => {
                this_inner == other_inner && this_size == other_size
            },
            (_, _) => false,
        }
    }
}