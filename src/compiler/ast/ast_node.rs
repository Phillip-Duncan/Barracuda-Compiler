use super::literals::Literal;
use super::operators::{UnaryOperation, BinaryOperation};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ASTNode {
    /// Identifier is a string sequence representative of a symbol. That is
    /// a variable, symbol or constant.
    /// # Example:
    ///     let hello = 4;
    ///         ^^^^^ -> Identifier
    IDENTIFIER(String),

    /// Literal is a constant value used within an expression.
    /// # Example:
    ///     let hello = 4;
    ///                 ^ -> Literal
    ///     let world = false;
    ///                 ^^^^^ -> Literal
    LITERAL(Literal),

    /// Unary operation is an expression operation with only one argument
    ///
    /// # Example:
    ///     let x = !(true);
    ///             ^ -> Unary Operator
    UNARY_OP {
        op: UnaryOperation,
        expression: Box<ASTNode>
    },

    /// Binary operation is an expression operation with two arguments.
    ///
    /// # Syntax:
    ///     <lhs> <op> <rhs>
    ///
    /// # Example:
    ///     let x = 40 + 2.0;
    ///                ^ -> Binary Operator
    BINARY_OP {
        op: BinaryOperation,
        lhs: Box<ASTNode>,
        rhs: Box<ASTNode>
    },

    /// Construction statement defines a variable for use in future statements in scope.
    ///
    /// # Syntax:
    ///     let <identifier> (: <datatype>)? = <expression>;
    ///
    /// # Example:
    ///     let x = 36;
    ///     ^^^^^^^^^^^ -> Construction Statement
    ///
    ///     let y: u32 = 42;
    ///
    CONSTRUCT {
        identifier: Box<ASTNode>,
        datatype: Box<Option<ASTNode>>,
        expression: Box<ASTNode>
    },

    /// Assignment statement assigns a new value to a variable within scope.
    ///
    /// # Syntax:
    ///     <identifier> = <expression>;
    ///
    /// # Example:
    ///     let x = 42;
    ///     x = 20;
    ///     ^^^^^^ -> Assignment
    ASSIGNMENT {
        identifier: Box<ASTNode>,
        expression: Box<ASTNode>
    },

    /// Print statement will display the result of an expression to stdout of interpreter
    ///
    /// # Syntax:
    ///     print <expression>;
    ///
    /// # Example:
    ///     print 12*12;    -> '144'
    ///     ^^^^^^^^^^^ -> Print Statement
    PRINT {
        expression: Box<ASTNode>
    },

    /// Return statement will return the result of an expression to function caller.
    ///
    /// # Syntax:
    ///     return <expression>;
    ///
    /// # Example:
    ///     fn my_function() {
    ///         return 30;
    ///     }   ^^^^^^^^^^ -> Return Statement
    ///
    ///     print my_function();    -> '30'
    RETURN {
        expression: Box<ASTNode>
    },

    /// Branch statement, also known as an if statement will conditionally run a section of code if
    /// the result of a condition expression is non zero. If an else statement is defined this section
    /// of code will be run on a zero condition.
    ///
    /// # Syntax:
    ///     if <expression> { ... }
    ///     (else if <expression> {...})*
    ///     (else { ... })?
    ///
    /// # Example:
    ///     let x: bool = false;
    ///     if x {
    ///         print 1;    // Skipped
    ///     } else {
    ///         print 2;    -> '2'
    ///     }
    ///     ^^^^^^^^^^^^ -> If Statement + Else Block
    BRANCH {
        condition: Box<ASTNode>,
        if_branch: Box<ASTNode>,
        else_branch: Box<Option<ASTNode>>
    },

    /// While statement or loop, will iteratively run a section of code if a condition is non-zero
    /// at each iteration.
    ///
    /// # Syntax:
    ///     while <expression> { ... }
    ///
    /// # Example:
    ///     let x = 0;
    ///     while x < 5 {
    ///         x = x + 1;
    ///         print x;
    ///     }               -> '1' '2' '3' '4' '5'
    ///     ^^^^^^^^^^^^^   -> While Statement
    WHILE_LOOP {
        condition: Box<ASTNode>,
        body: Box<ASTNode>
    },

    /// For statement or loop, will iteratively run a section of code if a condition is non-zero.
    /// Additionally the syntax supports inclusion of statements that are run initially and each
    /// iteration. Traditionally these are used to define an iteration variable and increment it.
    ///
    /// # Syntax:
    ///     for (<initial>; <condition>; <advancement>) { ... }
    ///
    /// # Example:
    ///     for (let i = 1; i < 6; i = i + 1) {
    ///         print i;
    ///     }                                   -> '1' '2' '3' '4' '5'
    ///     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ -> For Statement
    FOR_LOOP {
        initialization: Box<ASTNode>,
        condition: Box<ASTNode>,
        advancement: Box<ASTNode>,
        body: Box<ASTNode>
    },

    /// Function parameters are defined when defining a function. They carry an identifier and
    /// an optional datatype.
    ///
    /// # Syntax:
    ///     <identifier> (: <datatype>)?
    ///
    /// # Example:
    ///     fn my_func(x: u32, y: u32) {
    ///                ^^^^^^ -> Function Parameter
    ///         return x + y;
    ///     }
    PARAMETER {
        identifier: Box<ASTNode>,
        datatype: Box<Option<ASTNode>>
    },

    /// Functions are callable sections of code that have defined 0 or more function parameters and
    /// a return type.
    ///
    /// # Syntax:
    ///     fn <identifier>( (<parameter>, )* ) (:datatype)? {...}
    ///
    /// # Example:
    ///     fn add_and_square(a: f64, b: f64) : f64 {
    ///         let sum = a + b;
    ///         return sum * sum;
    ///     }
    ///     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ -> Function Definition
    ///     print add_and_square(1, 3);               -> '16'
    FUNCTION {
        identifier: Box<ASTNode>,
        parameters: Vec<ASTNode>,
        return_type: Box<Option<ASTNode>>,
        body: Box<ASTNode>
    },

    /// Function Call expressions will call a function with relevant argument expressions and be
    /// processed and replaced with return value of the function.
    ///
    /// # Syntax:
    ///     fn <identifier>( (<parameter>, )* ) (:datatype)? {...}
    ///
    /// # Example:
    ///     fn add_and_square(a: f64, b: f64) : f64 {
    ///         let sum = a + b;
    ///         return sum * sum;
    ///     }
    ///     print add_and_square(1, 8-5);   -> '16'
    ///           ^^^^^^^^^^^^^^^^^^^^^^^   -> Function Call
    FUNC_CALL {
        identifier: Box<ASTNode>,
        arguments: Vec<ASTNode>
    },

    /// Statement list or code block or code section is a collection of statements that should
    /// be run linearly.
    ///
    /// # Syntax:
    ///     (statement)*
    ///
    /// # Example:
    ///     let x = 10;  -> Statement  |
    ///     let y = 30;  -> Statement  } Statement List
    ///     print x * y; -> Statement  |
    STATEMENT_LIST(Vec<ASTNode>)
}