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

#[derive(Clone)]
pub struct BarracudaSymbol {
    pub(crate) name: String,
    pub(crate) symbol_type: SymbolType,
    pub(crate) mutable: bool,
    pub(crate) scope_id: usize
}