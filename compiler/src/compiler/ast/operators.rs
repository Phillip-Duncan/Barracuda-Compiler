/// Unary Operations are mathematical symbolic functions with one argument.
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum UnaryOperation {
    NOT,        // ! <rhs>
    NEGATE,     // - <rhs>
    PTR_DEREF,  // * <rhs>
}

/// Binary Operations are mathematical symbolic functions with two arguments.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum BinaryOperation {
    ADD,            // <lhs> + <rhs>
    SUB,            // <lhs> - <rhs>
    DIV,            // <lhs> / <rhs>
    MUL,            // <lhs> * <rhs>
    MOD,            // <lhs> % <rhs>
    POW,            // <lhs> ^ <rhs>

    EQUAL,          // <lhs> == <rhs>
    NOT_EQUAL,      // <lhs> != <rhs>
    GREATER_THAN,   // <lhs> >  <rhs>
    LESS_THAN,      // <lhs> <  <rhs>
    GREATER_EQUAL,  // <lhs> >= <rhs>
    LESS_EQUAL      // <lhs> <= <rhs>
}