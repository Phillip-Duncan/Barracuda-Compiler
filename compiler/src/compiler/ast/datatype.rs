use super::ast_node::ASTNode;

/// Primitive Data types supported by the AST Model
#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub fn size(&self) -> usize {
        match self {
            PrimitiveDataType::F128 => 16,
            PrimitiveDataType::F64 => 8,
            PrimitiveDataType::F32 => 4,
            PrimitiveDataType::F16 => 2,
            PrimitiveDataType::F8 => 1,
            PrimitiveDataType::I128 => 16,
            PrimitiveDataType::I64 => 8,
            PrimitiveDataType::I32 => 4,
            PrimitiveDataType::I16 => 2,
            PrimitiveDataType::I8 => 1,
            PrimitiveDataType::Bool => 1
        }
    }
}

#[derive(Debug, Clone)]
pub enum DataType {
    MUTABLE(PrimitiveDataType),
    CONST(PrimitiveDataType),
    ENVIRONMENTVARIABLE(PrimitiveDataType),
    POINTER(Box<DataType>),
    ARRAY(Box<DataType>, usize),
    NONE
}

impl DataType {
    pub fn from(node: &ASTNode) -> Self {
        match node {
            ASTNode::DATATYPE(datatype) => datatype.clone(),
            _ => panic!("Node {:?} must be a datatype node to convert it to a datatype", node)
        }
    }

    pub fn from_str(datatype: String) -> Self {
        // Create different qualified types based on datatype qualifier (MUTABLE/CONST). CONST by default.
        if datatype == "none" {
            DataType::NONE
        } else if datatype.contains("mut") {
            DataType::MUTABLE(PrimitiveDataType::parse(datatype.replace("mut", "")).unwrap())
        } else if datatype.contains("const") {
            DataType::CONST(PrimitiveDataType::parse(datatype.replace("const", "")).unwrap())
        } else {
            DataType::MUTABLE(PrimitiveDataType::parse(datatype).unwrap())
        }
    }

    pub fn get_array_length(datatype: &Self) -> usize {
        match datatype {
            DataType::ARRAY(inner, size) => {
                size * DataType::get_array_length(inner)
            }
            _ => 1
        }
    }
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Currently all primitive types are considered equal as everything is just a float on the VM.
            // This will need to be changed if proper integer operations are implemented.
            (DataType::MUTABLE(_), DataType::MUTABLE(_))
            | (DataType::CONST(_), DataType::CONST(_))
            | (DataType::MUTABLE(_), DataType::CONST(_))
            | (DataType::CONST(_), DataType::MUTABLE(_))
            | (DataType::ENVIRONMENTVARIABLE(_), DataType::MUTABLE(_))
            | (DataType::ENVIRONMENTVARIABLE(_), DataType::CONST(_))
            | (DataType::MUTABLE(_), DataType::ENVIRONMENTVARIABLE(_))
            | (DataType::CONST(_), DataType::ENVIRONMENTVARIABLE(_))
            | (DataType::ENVIRONMENTVARIABLE(_), DataType::ENVIRONMENTVARIABLE(_)) => true,
            (DataType::POINTER(this_inner), DataType::POINTER(other_inner)) => this_inner == other_inner,
            (DataType::ARRAY(this_inner, this_size), DataType::ARRAY(other_inner, other_size)) => {
                this_inner == other_inner && this_size == other_size
            },
            (DataType::NONE, DataType::NONE) => true,
            (_, _) => false,
        }
    }
}