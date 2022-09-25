use std::borrow::Borrow;
use std::collections::HashMap;
use super::ASTNode;
use super::scope::{ScopeId, ScopeIdGenerator};
use super::datatype::{DataType, PrimitiveDataType};
use std::fmt;
use std::fmt::format;
use std::thread::{Scope, scope};


/// Symbol types associated with an identifier
#[derive(Debug, Clone)]
pub enum SymbolType {
    Variable(DataType),
    Parameter(DataType),
    Function {
        func_params: Vec<DataType>,
        func_return: Box<PrimitiveDataType> // Cannot be mutable, const or unknown as defaults to void
    },
}

/// Barracuda Symbols defines the data associated with an identifier.
#[derive(Debug, Clone)]
pub struct Symbol {
    identifier: String,      // Identifier known by
    symbol_type: SymbolType, // Identifier type

    scope_id: ScopeId,        // ScopeId belonging to
    declaration_order: usize  // Used to keep track of order of declaration within a scope
}

impl Symbol {
    pub fn new(identifier: String, symbol_type: SymbolType) -> Self {
        Symbol {
            identifier,
            symbol_type,
            scope_id: ScopeId::global(),
            declaration_order: 0,
        }
    }

    pub fn identifier(&self) -> &String {
        &self.identifier
    }

    pub fn symbol_type(&self) -> SymbolType {
        self.symbol_type.clone()
    }

    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.clone()
    }

    pub fn is_mutable(&self) -> bool {
        match &self.symbol_type {
            SymbolType::Variable(datatype) => match datatype {
                DataType::MUTABLE(_) => true,
                DataType::UNKNOWN => true,      // TODO(Connor): This should be removed once the type system is implemented
                _ => false
            },
            _ => false
        }
    }

    pub fn unique_id(&self) -> String {
        format!("{}:{}", self.scope_id, self.identifier)
    }
}


/// Symbol scope is a node in the scope tree of an AST.
/// The scope has an id and an optional parent scope id.
/// The scope holds all symbols defined within its context
#[derive(Debug, Clone)]
pub struct SymbolScope {
    id: ScopeId,
    parent: Option<ScopeId>,
    subroutine: bool,   // If a scope is a sub routine we cannot trust finding symbols in
                        // the parent scope

    symbols: HashMap<String, Symbol>,
}

impl SymbolScope {

    /// Creates a new empty symbol scope
    fn new(id: ScopeId, parent: ScopeId, is_subroutine: bool) -> Self {
        Self {
            id,
            parent: Some(parent),
            subroutine: is_subroutine,
            symbols: Default::default()
        }
    }

    /// Add symbol to scope context
    /// @return true if successful, false if symbol already exists
    fn add_symbol(&mut self, symbol: Symbol) -> bool {
        if self.symbols.contains_key(&symbol.identifier) {
            return false;
        }

        self.symbols.insert(symbol.identifier.clone(), Symbol {
            identifier: symbol.identifier,
            symbol_type: symbol.symbol_type,
            scope_id: self.id.clone(),
            declaration_order: self.symbols.len()
        });
        return true;
    }

    /// Get symbol in scope context by identifier
    /// @return Symbol if found
    fn get_symbol(&self, identifier: &String) -> Option<&Symbol> {
        self.symbols.get(identifier)
    }

    /// Get scope parent id
    fn parent(&self) -> Option<ScopeId> {
        self.parent.clone()
    }
}


/// Symbol table is a auxiliary AST data structure simplifying the scopes
/// of symbols within a program. Each symbol is stored within a unique scope
/// SymbolScopes are organised into a tree where a symbol is 'in scope' if it
/// is the current active scope or any parent scopes up to and including the
/// highest level global scope(id can be retrieved from ScopeId::global()).
/// AST should be 1:1 with a symbol table as the ScopeIDs are only guaranteed
/// to be unique within a single symbol table.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Scopes are stored by ScopeID
    // This implementation is a hashmap to allow for the pruning of scopes without restructuring
    scope_map: HashMap<ScopeId, SymbolScope>,

    /// Generator will generate unique ScopeId for the SymbolTable
    scope_generator: ScopeIdGenerator
}


/// Public Methods
impl SymbolTable {

    /// Create an empty symbol table with only one scope, the global scope.
    pub fn new() -> Self {
        let mut symbol_table = SymbolTable {
            scope_map: Default::default(),
            scope_generator: ScopeIdGenerator::new()
        };

        // Add root global scope
        symbol_table.scope_map.insert(ScopeId::global(), SymbolScope {
            id: ScopeId::global(),
            parent: None,
            subroutine: false,
            symbols: Default::default()
        });

        symbol_table
    }

    /// Generate a Symbol table from a root ASTNode
    /// Each node in the AST is processed to identify the scope structure and symbols.
    /// @root: Root node of an Abstract Syntax Tree. Mutable as scope ids are assigned to scope
    /// ast nodes during this process.
    pub(super) fn from(root: &mut ASTNode) -> SymbolTable {
        let mut symbol_table = SymbolTable::new();
        symbol_table.process_node(root, ScopeId::global());
        symbol_table
    }

    /// Find a symbol in the symbol table given the scope within.
    /// Identifiers are only valid for finding symbols with context to the scope at which asking from.
    /// This is because scopes allow for identifiers to be reused so an identifier may map to different
    /// symbols depending on the context.
    ///
    /// If multiple symbols exist in scope then the most recently defined will be returned.
    /// Example:
    /// let x = 0;
    /// {
    ///     let x = 2;
    ///     print(x) -> '2'
    /// }
    /// print(x) -> '0'
    ///
    /// @current_scope: scope id of the current scope. This is stored with ASTNode::SCOPE_BLOCK::scope
    /// @identifier: symbol identifier name
    /// @return Symbol if found within scope or parent scopes, otherwise None
    pub fn find_symbol(&self, current_scope: ScopeId, identifier: &String) -> Option<&Symbol> {
        // Get scope from id
        let symbol_scope = match self.scope_map.get(&current_scope) {
            Some(symbol) => symbol,

            // Invalid scope id
            None => return None
        };

        // Check if current scope has symbol
        match symbol_scope.get_symbol(identifier) {
            Some(symbol) => Some(symbol),
            None => {
                // Check parent for symbol if not a subroutine
                if symbol_scope.parent.is_some() && !symbol_scope.subroutine {
                    let parent_id = symbol_scope.parent().unwrap();
                    self.find_symbol(parent_id, identifier)
                } else {
                    None
                }
            }
        }
    }

    /// Get symbols in scope returns a copy of all symbols in scope at current_scope.
    pub fn get_symbols_in_scope(&self, current_scope: ScopeId) -> Vec<Symbol>
    {
        // Get relevant scope
        let scope = match self.scope_map.get(&current_scope) {
            Some(scope) => { scope }
            None => { return Vec::default() }
        };

        // Get current scope symbols
        let mut symbols = scope.symbols.values()
            .map(|symbol| symbol.clone()).collect();

        // Add parent scope symbols if not subroutine
        if scope.parent.is_some() && !scope.subroutine {
            let parent_id = scope.parent.clone().unwrap();
            let mut parent_symbols = self.get_symbols_in_scope(parent_id);
            parent_symbols.append(&mut symbols);
            symbols = parent_symbols;
        };

        return symbols
    }

    /// Returns a list of all scopes in the symbol table
    pub fn get_valid_scope_ids(&self) -> Vec<ScopeId> {
        self.scope_map.keys().map(|id| id.clone()).collect()
    }
}

/// Private Methods
impl SymbolTable {
    /// Create and add a new scope with a valid parent.
    /// @parent: valid parent scope id
    /// @return: new scope id if parent exists otherwise None
    fn generate_new_scope(&mut self, parent: ScopeId, is_subroutine: bool) -> Option<ScopeId> {
        if self.scope_map.contains_key(&parent) {
            let id = self.scope_generator.next().unwrap(); // Generator always valid
            self.scope_map.insert(id.clone(),SymbolScope {
                id: id.clone(),
                parent: Some(parent),
                subroutine: is_subroutine,
                symbols: Default::default()
            });

            Some(id)
        } else {
            None
        }
    }

    /// Helper function to simplify processing variables where data is stored within deeper
    /// ASTNodes.
    fn process_variable(identifier: &ASTNode, datatype: &Option<ASTNode>) -> Option<Symbol> {
        // Get inner string
        let identifier = match identifier {
            ASTNode::IDENTIFIER(name) => name.clone(),
            _ => panic!("")    // AST Malformed
        };

        // Identify the variable type
        let datatype = match datatype {
            Some(datatype_node) => DataType::from(datatype_node),
            None => DataType::UNKNOWN
        };

        Some(Symbol::new(identifier, SymbolType::Variable(datatype)))
    }

    /// Helper function to simplify processing parameters where data is stored within deeper
    /// ASTNodes.
    fn process_parameter(identifier: &ASTNode, datatype: &Option<ASTNode>) -> Option<Symbol> {
        // Get inner string
        let identifier = match identifier {
            ASTNode::IDENTIFIER(name) => name.clone(),
            _ => panic!("")    // AST Malformed
        };

        // Identify the variable type
        let datatype = match datatype {
            Some(datatype_node) => DataType::from(datatype_node),
            None => DataType::UNKNOWN
        };

        Some(Symbol::new(identifier, SymbolType::Parameter(datatype)))
    }

    /// Helper function to simplify processing functions where data is stored within deeper
    /// ASTNodes.
    fn process_function(identifier: &ASTNode, parameters: &Vec<ASTNode>, return_type: &ASTNode) -> Option<Symbol> {
        // Transform ASTNodes into function identifier and data types
        let identifier = match identifier {
            ASTNode::IDENTIFIER(name) => name.clone(),
            _ => panic!("") // AST Malformed
        };

        // Collect parameters types
        let func_params = parameters.into_iter()
            .map(|param| match param {
                ASTNode::PARAMETER{ identifier:_, datatype } => {
                    match datatype.as_ref() {
                        Some(datatype_node) => DataType::from(datatype_node),
                        None => DataType::UNKNOWN
                    }
                }
                _ => panic!("")  // AST Malformed
            }).collect();

        let return_type = match DataType::from(return_type) {
            // Strip datatype as return type can only be primitive
            DataType::MUTABLE(primitive_type) => primitive_type,

            // Unknown return type
            _ => panic!("")
        };

        let symbol_type = SymbolType::Function {
            func_params,
            func_return: Box::new(return_type)
        };

        // Add function to symbol table
        Some(Symbol::new(identifier, symbol_type))
    }

    /// Proccess node is a recursive function that iterates through all nodes in an AST.
    /// It identifies symbols and adds them to the symbol table while also generating the scope
    /// tree and assigning scope ids to ASTNode::SCOPE_BLOCKs
    /// @node: head of subtree of AST
    /// @current_scope: Current highest level scope for where we are in the AST. If node is the root
    /// of the AST this should be ScopeId::global()
    fn process_node(&mut self, node: &mut ASTNode, current_scope: ScopeId) {
        let mut symbol_scope = self.scope_map.get_mut(&current_scope).unwrap();

        match node {
            ASTNode::PARAMETER { identifier, datatype } => {
                match Self::process_parameter(identifier.as_ref(), datatype.as_ref()) {
                    Some(symbol) => symbol_scope.add_symbol(symbol),
                    None => panic!("") // AST Malformed
                };
            }
            ASTNode::CONSTRUCT{ identifier, datatype, expression:_ } => {
                match Self::process_variable(identifier.as_ref(), datatype.as_ref()) {
                    Some(symbol) => symbol_scope.add_symbol(symbol),
                    None => panic!("") // AST Malformed
                };
            }
            ASTNode::FUNCTION { identifier, parameters, return_type, body } => {
                match Self::process_function(identifier.as_ref(), parameters.as_ref(), return_type.as_ref()) {
                    Some(symbol) => symbol_scope.add_symbol(symbol),
                    None => panic!("") // AST Malformed
                };

                // Process function body
                let inner_func_scope= self.generate_new_scope(current_scope, true).unwrap();
                for param in parameters {
                    self.process_node(param, inner_func_scope.clone());
                }
                for child in body.children() {
                    self.process_node(child, inner_func_scope.clone());
                }

                if let ASTNode::SCOPE_BLOCK {inner:_, scope} = body.as_mut() {
                    scope.set(inner_func_scope);
                }
            }
            ASTNode::SCOPE_BLOCK { inner, scope } => {
                // Assign next scope id
                let assigned_id = self.generate_new_scope(current_scope, false).unwrap();
                scope.set(assigned_id);

                // Process inner node with new scope
                self.process_node(inner, scope.clone());
            }
            _ => {
                // If not nodes we are searching for iterate over children
                for child in node.children() {
                    self.process_node(child, current_scope.clone());
                }
            }
        }
    }

    /// Collect the children scopes of a scope id.
    /// Because internally scopes are unidirectional trees upward
    /// this function is O(n) where n is the number of scopes.
    /// @id: Parent scope
    /// @return: child scopes of the parent scope with associated id
    fn children_of(&self, id: ScopeId) -> Vec<&SymbolScope> {
        self.scope_map.values()
            .filter(|scope|
                scope.parent()
                    .and_then(|parent| Some(parent == id))
                    .unwrap_or(false))
            .collect()
    }

    /// Get parent scope id from a scope id
    /// @id: Scope to get the parent of
    /// @return: Parent scope id of id if found otherwise returns the global scope
    pub(super) fn parent_of(&self, id: ScopeId) -> ScopeId {
        match self.scope_map.get(&id) {
            Some(scope) => {
                match scope.parent.clone() {
                    Some(parent_id) => parent_id,
                    None => ScopeId::global()
                }
            },
            None => ScopeId::global()
        }
    }
}



/// Formatting of Symbol Tree allows for symbol tables to be written as a string.
impl fmt::Display for SymbolTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_scope(table: &SymbolTable, f: &mut fmt::Formatter<'_>, id: ScopeId, depth: usize) {
            let indent = "\t".repeat(depth);
            let indent_plus = "\t".repeat(depth+1);

            writeln!(f, "{}{{\n", indent);
            for symbol in table.scope_map.get(&id).unwrap().symbols.values() {
                writeln!(f, "{}{:?}\n", indent_plus, symbol);
            }

            for scope in table.children_of(id) {
                print_scope(table, f,scope.id.clone(), depth + 1);
            }

            writeln!(f, "{}}}\n", indent);
        }

        print_scope(self, f,ScopeId::global(), 0);

        Ok(())
    }
}


/// SymbolTable Module Tests
#[cfg(test)]
mod tests {
    use crate::compiler::ast::{ASTNode, BinaryOperation, Literal, ScopeId};
    use crate::compiler::ast::symbol_table::SymbolTable;

    /// Generates an example ast directly using ASTNodes. Example program is equivalent to:
    ///     fn add(x: f64, y: f64) -> f64 {
    ///         let z: f64 = 2;
    ///         return x + y + z;
    ///     }
    ///
    ///     let a: f64 = 10.0;
    ///     let b: f64 = 25.0;
    ///     if true {
    ///         let c = add(a, b);
    ///     } else {
    ///         let c = 5;
    ///     }
    fn generate_test_ast() -> ASTNode {
        let f64_datatype = Box::new(Some(ASTNode::IDENTIFIER(String::from("f64"))));
        let ast = ASTNode::STATEMENT_LIST(vec![
            // fn add(x: f64, y: f64) -> f64
            ASTNode::FUNCTION {
                identifier: Box::new(ASTNode::IDENTIFIER(String::from("add"))),
                parameters: vec![
                    ASTNode::PARAMETER {
                        identifier: Box::new(ASTNode::IDENTIFIER(String::from("x"))),
                        datatype: f64_datatype.clone()
                    },
                    ASTNode::PARAMETER {
                        identifier: Box::new(ASTNode::IDENTIFIER(String::from("y"))),
                        datatype: f64_datatype.clone()
                    }
                ],
                return_type: Box::new(f64_datatype.clone().unwrap()),
                // {
                body: Box::new(ASTNode::SCOPE_BLOCK {
                    scope: ScopeId::default(),
                    inner: Box::new(ASTNode::STATEMENT_LIST(vec![
                        // let z: f64 = 2.0;
                        ASTNode::CONSTRUCT {
                            identifier: Box::new(ASTNode::IDENTIFIER(String::from("z"))),
                            datatype: f64_datatype.clone(),
                            expression: Box::new(ASTNode::LITERAL(Literal::FLOAT(2.0)))
                        },
                        // return x + y + z;
                        ASTNode::RETURN {
                            expression: Box::new(ASTNode::BINARY_OP {
                                op: BinaryOperation::ADD,
                                lhs: Box::new(ASTNode::BINARY_OP {
                                    op: BinaryOperation::ADD,
                                    lhs: Box::new(ASTNode::IDENTIFIER(String::from("x"))),
                                    rhs: Box::new(ASTNode::IDENTIFIER(String::from("y")))
                                }),
                                rhs: Box::new(ASTNode::IDENTIFIER(String::from("z")))
                            })
                        }
                    ]))
                })
                // }
            },
            // let a: f64 = 10.0;
            ASTNode::CONSTRUCT {
                identifier: Box::new(ASTNode::IDENTIFIER(String::from("a"))),
                datatype: f64_datatype.clone(),
                expression: Box::new(ASTNode::LITERAL(Literal::FLOAT(10.0)))
            },
            // let b: f64 = 25.0;
            ASTNode::CONSTRUCT {
                identifier: Box::new(ASTNode::IDENTIFIER(String::from("b"))),
                datatype: f64_datatype.clone(),
                expression: Box::new(ASTNode::LITERAL(Literal::FLOAT(25.0)))
            },
            // if true
            ASTNode::BRANCH {
                condition: Box::new(ASTNode::LITERAL(Literal::BOOL(true))),
                // let c = add(x,y)
                if_branch: Box::new(ASTNode::SCOPE_BLOCK {
                    scope: ScopeId::default(),
                    inner: Box::new(ASTNode::CONSTRUCT {
                        identifier: Box::new(ASTNode::IDENTIFIER(String::from("c"))),
                        datatype: Box::new(None),
                        expression: Box::new(ASTNode::FUNC_CALL {
                            identifier: Box::new(ASTNode::IDENTIFIER(String::from("add"))),
                            arguments: vec![
                                ASTNode::IDENTIFIER(String::from("a")),
                                ASTNode::IDENTIFIER(String::from("b")),
                            ]
                        })
                    }),
                }),
                // let c = 5.0
                else_branch: Box::new(Some(ASTNode::SCOPE_BLOCK {
                    scope: ScopeId::default(),
                    inner: Box::new(ASTNode::CONSTRUCT {
                        identifier: Box::new(ASTNode::IDENTIFIER(String::from("c"))),
                        datatype: Box::new(None),
                        expression: Box::new(ASTNode::LITERAL(Literal::FLOAT(5.0)))
                    }),
                }))
            }
        ]);

        return ast;
    }

    #[test]
    fn symbol_table_generation() {
        let mut ast = generate_test_ast();
        let symbol_table = SymbolTable::from(&mut ast);
        assert_eq!(symbol_table.scope_map.len(), 4);


    }
}
