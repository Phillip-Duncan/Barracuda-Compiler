
/// Literals are defined constants within a program. See ASTNode for more detail on their usage.
/// They are divided by their representation in text.
///
/// Note: signed literals are stored in the AST as a negate unary operation. For example
/// -32 <=> UNARY_OP{UnaryOperation::NEGATE, Literal::INTEGER(32)}
#[derive(Debug, Clone)]
pub enum Literal {
    /// Form: %d.%d
    FLOAT(f64),

    /// Form: %d
    INTEGER(u64),

    /// Form: "%c*"
    // STRING(String),

    /// Form: false | true
    BOOL(bool),

    /// Form: %d.%d
    /// 8 characters packed into a single f64
    PACKEDSTRING(f64), 
}