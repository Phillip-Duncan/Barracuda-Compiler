// TODO(Connor): Redefine symbol type to include type Variable(Datatype) and move datatypes to
//               DataType enum this is a better representation as currently functions can be
//               recursively defined. In Datatype mutable would be stored as a mutable function makes
//               no sense or is at least a bad idea.

/// Symbol types associated with an identifier
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub enum SymbolType {
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
    Alias(String),
    Function {
        func_label: u64,
        func_params: Vec<SymbolType>,
        func_return: Box<SymbolType>
    },
    Void
}

impl SymbolType {
    /// Convert a string representation to a symbol type
    pub fn from(datatype: String) -> SymbolType {
        match datatype.to_lowercase().trim() {
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
            "bool" => {Self::U8}
            _ => {panic!("Unrecognised datatype used '{}'", datatype)}
        }
    }
}

/// Barracuda Symbols defines the data associated with an identifier.
#[derive(Clone)]
pub struct BarracudaSymbol {
    pub name: String,            // Identifier known by
    pub symbol_type: SymbolType, // Identifier type
    pub mutable: bool,           // Is the symbol mutable? This really should be encoded as a type.
    pub scope_id: usize          // What unique id in the scope does this symbol have.
}