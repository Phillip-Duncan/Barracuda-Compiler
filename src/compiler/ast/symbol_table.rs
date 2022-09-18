use std::collections::HashMap;
use super::ASTNode;
use super::scope::{ScopeId, ScopeIdGenerator};
use super::datatype::{DataType, PrimitiveDataType};
use std::fmt;


/// Symbol types associated with an identifier
#[derive(Debug, Clone)]
pub enum SymbolType {
    Variable(DataType),
    Function {
        func_params: Vec<DataType>,
        func_return: Box<PrimitiveDataType> // Cannot be mutable, const or unknown as defaults to void
    },
}

/// Barracuda Symbols defines the data associated with an identifier.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub identifier: String,      // Identifier known by
    pub symbol_type: SymbolType, // Identifier type
}


/// Symbol scope is a node in the scope tree of an AST.
/// The scope has an id and an optional parent scope id.
/// The scope holds all symbols defined within its context
#[derive(Debug, Clone)]
pub struct SymbolScope {
    id: ScopeId,
    parent: Option<ScopeId>,

    symbols: HashMap<String, Symbol>,
}

impl SymbolScope {

    /// Creates a new empty symbol scope
    fn new(id: ScopeId, parent: ScopeId) -> Self {
        Self {
            id,
            parent: Some(parent),
            symbols: Default::default()
        }
    }

    /// Add symbol to scope context
    /// @return true if successful, false if symbol already exists
    fn add_symbol(&mut self, symbol: Symbol) -> bool {
        if self.symbols.contains_key(&symbol.identifier) {
            return false;
        }

        self.symbols.insert(symbol.identifier.clone(), symbol);
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
    pub(super) fn find_symbol(&self, current_scope: ScopeId, identifier: &String) -> Option<&Symbol> {
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
                // Check parent for symbol
                match symbol_scope.parent() {
                    Some(parent_id) => self.find_symbol(parent_id, identifier),

                    // Symbol does not exist
                    None => None
                }
            }
        }
    }

}

/// Private Methods
impl SymbolTable {
    /// Create and add a new scope with a valid parent.
    /// @parent: valid parent scope id
    /// @return: new scope id if parent exists otherwise None
    fn generate_new_scope(&mut self, parent: ScopeId) -> Option<ScopeId> {
        if self.scope_map.contains_key(&parent) {
            let id = self.scope_generator.next().unwrap(); // Generator always valid
            self.scope_map.insert(id.clone(),SymbolScope {
                id: id.clone(),
                parent: Some(parent),
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

        Some( Symbol {
            identifier,
            symbol_type: SymbolType::Variable(datatype)
        })
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

        // Add function to symbol table
        Some(Symbol {
            identifier,
            symbol_type: SymbolType::Function {
                func_params,
                func_return: Box::new(return_type)
            }
        })
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
                match Self::process_variable(identifier.as_ref(), datatype.as_ref()) {
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
            ASTNode::FUNCTION { identifier, parameters, return_type, body:_ } => {
                match Self::process_function(identifier.as_ref(), parameters.as_ref(), return_type.as_ref()) {
                    Some(symbol) => symbol_scope.add_symbol(symbol),
                    None => panic!("") // AST Malformed
                };

                // Process function body
                let inner_func_scope= self.generate_new_scope(current_scope).unwrap();
                for child in node.children() {
                    self.process_node(child, inner_func_scope.clone());
                }
            }
            ASTNode::SCOPE_BLOCK { inner, scope } => {
                // Assign next scope id
                let assigned_id = self.generate_new_scope(current_scope).unwrap();
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