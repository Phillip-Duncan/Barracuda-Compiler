#[derive(Debug)]
pub(crate) enum UnaryOperation {
    NOT,
    NEGATE
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub(crate) enum BinaryOperation {
    ADD,
    SUB,
    DIV,
    MUL,
    MOD,
    POW,

    EQUAL,
    NOT_EQUAL,
    GREATER_THAN,
    LESS_THAN,
    GREATER_EQUAL,
    LESS_EQUAL
}