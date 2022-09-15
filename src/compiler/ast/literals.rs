

#[derive(Debug)]
pub enum Literal {
    FLOAT(f64),
    INTEGER(u64),
    STRING(String),
    BOOL(bool)
}