use super::datatype::DataType;
use super::literals::Literal;
use super::operators::{UnaryOperation, BinaryOperation};
use super::scope::ScopeId;
use std::borrow::BorrowMut;
use super::qualifiers::Qualifier;

#[cfg_attr(doctest, doc = " ````no_test")] // Ignore doctests for this module
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum ASTNode {
    ///
    /// Identifier is a string sequence representative of a symbol. That is
    /// a variable, symbol or constant.
    /// # Example:
    ///     let hello = 4;
    ///         ^^^^^ -> Identifier
    IDENTIFIER(String),

    /// Reference represents a reference to an identifier. 
    /// In the example below, hello_ref points to the memory address of hello.
    /// 
    /// # Example:
    ///     let hello = 4;
    ///     let hello_ref = &hello;
    ///                     ^^^^^^ -> Reference
    REFERENCE(String),

    /// # Example:
    ///     let hello: i64 = 4;
    ///                ^^^ -> Datatype
    DATATYPE(DataType),


    /// # Example:
    ///     let const hello = 4;
    ///         ^^^^^ -> Qualifier
    QUALIFIER(Qualifier),


    /// Literal is a constant value used within an expression.
    /// # Example:
    ///     let hello = 4;
    ///                 ^ -> Literal
    ///     let world = false;
    ///                 ^^^^^ -> Literal
    LITERAL(Literal),

    /// Array is a list of expressions.
    /// # Example:
    ///     let array : [4] = [1, 2, 3+4, 5];
    ///                       ^^^^^^^^^^^^^^ -> Array
    ARRAY {
        items: Vec<ASTNode>,
        qualifier: Box<ASTNode>
    },

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

    /// Ternary operation is an expression operation with three arguments.
    ///
    /// # Syntax:
    ///     <condition> ? <true_branch> : <false_branch>
    ///
    /// # Example:
    ///     let x = boolean ? 1.0 : 2.0;
    ///                     ^     ^     -> Ternary Operator
    TERNARY_OP {
        condition: Box<ASTNode>,
        true_branch: Box<ASTNode>,
        false_branch: Box<ASTNode>
    },

    /// Array index allows a specific element to be accessed from an array.
    /// # Example:
    ///     let array : [4] = [5,8,11,14];
    ///     let example = array[2]; // example will now hold 11
    ///                        ^^^ -> Array index
    ARRAY_INDEX {
        index: Box<ASTNode>,
        expression: Box<ASTNode>
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
        qualifier: Box<ASTNode>,
        expression: Box<ASTNode>
    },

    /// Empty construct statement is a construct statement that does not provide any value to the cons
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
    EMPTY_CONSTRUCT {
        identifier: Box<ASTNode>,
        datatype: Box<ASTNode>,
        qualifier: Box<ASTNode>
    },

    /// External statement defines a external variable for use in future statements in scope.
    EXTERN {
        identifier: Box<ASTNode>
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
        pointer_level: usize,
        array_index: Vec<ASTNode>,
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
        datatype: Box<Option<ASTNode>>,
        qualifier: Box<ASTNode>
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

    NAKED_FUNC_CALL {
        func_call: Box<ASTNode>
    },

    /// Statement list is a collection of statements that should
    /// be run linearly.
    ///
    /// # Syntax:
    ///     (<statement>)*
    ///
    /// # Example:
    ///     let x = 10;  -> Statement  |
    ///     let y = 30;  -> Statement  } Statement List
    ///     print x * y; -> Statement  |
    STATEMENT_LIST(Vec<ASTNode>),

    /// Scope Block defines all nodes after inner as existing in the same scope.
    ///
    /// # Syntax:
    ///     { statement_list }
    ///
    /// # Example:
    SCOPE_BLOCK {
        inner: Box<ASTNode>,
        scope: ScopeId
    },

    /// Print statement will display the result of an expression to stdout of interpreter
    ///
    /// # Syntax:
    ///     print <expression>;
    ///
    /// # Example:
    ///     print 12*12;    -> '144'
    ///     ^^^^^^^^^^^ -> Print Statement
    TYPED_NODE {
        datatype: DataType,
        qualifier: Qualifier,
        inner: Box<ASTNode>
    },
}

impl ASTNode {
    /// Returns children of a ASTNode.
    /// This method is helpful when searching the AST for specific nodes
    /// without worrying about the implementation details of non target nodes
    pub(crate) fn children(&mut self) -> Vec<&mut ASTNode> {
        let mut output = vec![];

        match self {
            ASTNode::IDENTIFIER(_) => {}
            ASTNode::REFERENCE(_) => {}
            ASTNode::DATATYPE(_) => {}
            ASTNode::QUALIFIER(_) => {}
            ASTNode::LITERAL(_) => {}
            ASTNode::ARRAY {items, qualifier} => {
                for item in items {
                    output.push(item.borrow_mut());
                }
                output.push(qualifier.as_mut());
            }
            ASTNode::UNARY_OP { op: _, expression } => {
                output.push(expression.as_mut());
            }
            ASTNode::BINARY_OP { op: _, lhs, rhs } => {
                output.push(lhs.as_mut());
                output.push(rhs.as_mut());
            }
            ASTNode::TERNARY_OP { condition, true_branch, false_branch } => {
                output.push(condition.as_mut());
                output.push(true_branch.as_mut());
                output.push(false_branch.as_mut());
            }
            ASTNode::ARRAY_INDEX { index: _, expression } => {
                output.push(expression.as_mut());
            }
            ASTNode::CONSTRUCT { identifier, datatype, qualifier, expression } => {
                output.push(identifier.as_mut());

                if datatype.is_some() {
                    output.push(datatype.as_mut().as_mut().unwrap());
                }
                output.push(qualifier.as_mut());
                output.push(expression.as_mut());
            }
            ASTNode::EMPTY_CONSTRUCT { identifier, datatype, qualifier } => {
                output.push(identifier.as_mut());
                output.push(datatype.as_mut());
                output.push(qualifier.as_mut());
            }
            ASTNode::EXTERN {identifier} => {
                output.push(identifier.as_mut());
            }
            ASTNode::ASSIGNMENT { identifier, array_index, expression, .. } => {
                output.push(identifier.as_mut());
                for index in array_index {
                    output.push(index);
                }
                output.push(expression.as_mut());
            }
            ASTNode::PRINT { expression } => {
                output.push(expression.as_mut());
            }
            ASTNode::RETURN { expression } => {
                output.push(expression.as_mut());
            }
            ASTNode::BRANCH { condition, if_branch, else_branch } => {
                output.push(condition.as_mut());
                output.push(if_branch.as_mut());
                if else_branch.is_some() {
                    output.push(else_branch.as_mut().as_mut().unwrap());
                }
            }
            ASTNode::WHILE_LOOP { condition, body } => {
                output.push(condition.as_mut());
                output.push(body.as_mut());
            }
            ASTNode::FOR_LOOP { initialization, condition, advancement, body } => {
                output.push(initialization.as_mut());
                output.push(condition.as_mut());
                output.push(advancement.as_mut());
                output.push(body.as_mut());
            }
            ASTNode::PARAMETER { identifier, datatype, qualifier } => {
                output.push(identifier.as_mut());
                output.push(qualifier.as_mut());
                if datatype.is_some() {
                    output.push(datatype.as_mut().as_mut().unwrap());
                }
            }
            ASTNode::FUNCTION { identifier, parameters, return_type, body } => {
                output.push(identifier);
                for param in parameters {
                    output.push(param.borrow_mut());
                }
                if return_type.is_some() {
                    output.push(return_type.as_mut().as_mut().unwrap());
                }
                output.push(body.as_mut())
            }
            ASTNode::FUNC_CALL { identifier, arguments } => {
                output.push(identifier.as_mut());
                for arg in arguments {
                    output.push(arg.borrow_mut());
                }
            }
            ASTNode::NAKED_FUNC_CALL { func_call } => {
                output.push(func_call.as_mut());
            }
            ASTNode::STATEMENT_LIST(statements) => {
                for statement in statements {
                    output.push(statement.borrow_mut());
                }
            }
            ASTNode::SCOPE_BLOCK { inner, scope: _ } => {
                output.push(inner.as_mut());
            }
            ASTNode::TYPED_NODE { inner, .. } => {
                output.push(inner.as_mut());
            }
        }

        output
    }

    /// Utility function for simplifying extracting string out of identifier node
    pub(crate) fn identifier_name(&self) -> Option<String> {
        match self {
            ASTNode::TYPED_NODE { inner, .. } => match inner.as_ref() {
                ASTNode::IDENTIFIER(name) => Some(name.clone()),
                _ => None
            }
            ASTNode::IDENTIFIER(name) => Some(name.clone()),
            _ => None
        }
    }

    /// Utility function for simplifying extracting type out of node
    pub(crate) fn get_type(&self) -> DataType {
        match self {
            ASTNode::TYPED_NODE { datatype, .. } => datatype.clone(),
            _ => panic!("Malformed AST! Node {:?} was meant to be a TYPED_NODE but wasn't!", self)
        }
    }

    pub(crate) fn get_qualifier(&self) -> Qualifier {
        match self {
            ASTNode::TYPED_NODE { qualifier, .. } => qualifier.clone(),
            _ => panic!("Malformed AST! Node {:?} was meant to be a TYPED_NODE but wasn't!", self)
        }
    }

}