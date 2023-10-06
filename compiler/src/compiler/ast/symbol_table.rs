use std::collections::HashMap;
use crate::compiler::semantic_analyser::function_tracker::{FunctionTracker, FunctionImplementation};

use super::ASTNode;
use super::scope::ScopeId;
use super::datatype::{DataType, PrimitiveDataType};
use std::fmt;


/// Symbol types associated with an identifier
#[derive(Debug, Clone)]
pub enum SymbolType {
    Variable(DataType),
    EnvironmentVariable(usize, DataType, String),
    Parameter(DataType),
    Function {
        func_params: Vec<DataType>,
        func_return: Box<DataType>
    },
}

/// Barracuda Symbols defines the data associated with an identifier.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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

    #[allow(dead_code)] // Linter False Positive
    pub fn identifier(&self) -> &String {
        &self.identifier
    }

    #[allow(dead_code)] // Linter False Positive
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id.clone()
    }

    pub fn symbol_type(&self) -> SymbolType {
        self.symbol_type.clone()
    }

    pub fn is_variable(&self) -> bool {
        match &self.symbol_type {
            SymbolType::Variable(_) => true,
            _ => false
        }
    }

    pub fn is_array(&self) -> bool {
        match &self.symbol_type {
            SymbolType::Variable(datatype) => match datatype {
                DataType::ARRAY(_,_) => true,
                _ => false
            },
            _ => false
        }
    }

    pub fn array_length(&self) -> usize {
        match &self.symbol_type {
            SymbolType::Variable(datatype) => match datatype {
                DataType::ARRAY(_, _) => DataType::get_array_length(datatype),
                _ => 0
            },
            _ => 0
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

    env_var_data: HashMap<String, (usize, PrimitiveDataType, String)>,
    
    functions: HashMap<String, FunctionTracker>,

    /// Main entry scope
    /// Used to find program generation entry point
    main_entry_scope: ScopeId
}


/// Public Methods
impl SymbolTable {

    /// Create an empty symbol table with only one scope, the global scope.
    pub fn new() -> Self {
        let mut symbol_table = SymbolTable {
            scope_map: Default::default(),
            main_entry_scope: ScopeId::global(),
            env_var_data: Default::default(),
            functions: Default::default()
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
    /// @env_variable_ids: Map of environment variable data, identifier:(env_address, datatype, type qualifier)
    /// ast nodes during this process.
    pub(super) fn from(root: &mut ASTNode, env_variable_data: HashMap<String, (usize, PrimitiveDataType, String)>, functions: HashMap<String, FunctionTracker>) -> SymbolTable {
        let mut symbol_table = SymbolTable::new();
        symbol_table.env_var_data = env_variable_data;
        symbol_table.functions = functions;

        symbol_table.main_entry_scope = ScopeId::new(1);
        symbol_table.generate_new_scope(ScopeId::global(), symbol_table.main_entry_scope.clone(), true);
        symbol_table.process_node(root, symbol_table.main_entry_scope.clone()); 
        symbol_table
    }

    pub fn entry_scope(&self) -> ScopeId {
        self.main_entry_scope.clone()
    }

    /// Return copy of functions
    pub fn get_functions(&self) -> HashMap<String, FunctionTracker> {
        self.functions.clone()
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

        // Check if symbol is in global scope
        if let Some(global_scope) = self.scope_map.get(&ScopeId::global()) {
            match global_scope.get_symbol(identifier) {
                Some(symbol) => {return Some(symbol)}
                None => {}
            }
        }

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
}

/// Private Methods
impl SymbolTable {
    /// Create and add a new scope with a valid parent.
    /// @parent: valid parent scope id
    /// @return: new scope id if parent exists otherwise None
    fn generate_new_scope(&mut self, parent: ScopeId, current: ScopeId, is_subroutine: bool) {
        if self.scope_map.contains_key(&parent) {
            let id = current;
            self.scope_map.insert(id.clone(),SymbolScope {
                id: id.clone(),
                parent: Some(parent),
                subroutine: is_subroutine,
                symbols: Default::default()
            });
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
            None => DataType::NONE
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
            None => panic!("Parameters should always have some datatype!") // TODO does this need to be here?
        };

        Some(Symbol::new(identifier, SymbolType::Parameter(datatype)))
    }

    /// Helper function to simplify processing functions where data is stored within deeper
    /// ASTNodes.
    fn process_function(implementation: &FunctionImplementation) -> Symbol {
        // Transform ASTNodes into function identifier and data types
        let func_params = implementation.get_parameter_types().clone();
        let return_type = implementation.get_return_type();
        let symbol_type = SymbolType::Function {
            func_params,
            func_return: Box::new(return_type)
        };
        // Add function to symbol table
        Symbol::new(implementation.get_name(), symbol_type)
    }

    /// Proccess node is a recursive function that iterates through all nodes in an AST.
    /// It identifies symbols and adds them to the symbol table while also generating the scope
    /// tree and assigning scope ids to ASTNode::SCOPE_BLOCKs
    /// @node: head of subtree of AST
    /// @current_scope: Current highest level scope for where we are in the AST. If node is the root
    /// of the AST this should be ScopeId::global()
    fn process_node(&mut self, node: &mut ASTNode, current_scope: ScopeId) {
        let symbol_scope = self.scope_map.get_mut(&current_scope).unwrap();

        match node {
            ASTNode::PARAMETER { identifier, datatype } => {
                match Self::process_parameter(identifier.as_ref(), datatype.as_ref()) {
                    Some(symbol) => symbol_scope.add_symbol(symbol),
                    None => panic!("") // AST Malformed
                };
            }
            ASTNode::CONSTRUCT{ identifier, datatype, .. } => {
                let identifier = match identifier.as_ref() {
                    ASTNode::TYPED_NODE { inner, .. } => inner,
                    _ => identifier
                };
                match Self::process_variable(identifier.as_ref(), datatype) {
                    Some(symbol) => symbol_scope.add_symbol(symbol),
                    None => panic!("") // AST Malformed
                };
            }
            ASTNode::EMPTY_CONSTRUCT{ identifier, datatype } => {
                let identifier = match identifier.as_ref() {
                    ASTNode::TYPED_NODE { inner, .. } => inner,
                    _ => identifier
                };
                match Self::process_variable(identifier.as_ref(), &Some(datatype.as_ref().clone())) {
                    Some(symbol) => symbol_scope.add_symbol(symbol),
                    None => panic!("") // AST Malformed
                };
            }
            ASTNode::EXTERN { identifier} => {
                let identifier_name = identifier.identifier_name().unwrap();

                let env_address = match self.env_var_data.get(&identifier_name) {
                    Some(address) => address.clone(),
                    None => panic!("Extern '{}' found, however '{}' is not a defined external symbol", &identifier_name, &identifier_name)
                };

                // Safe to unwrap as global scope should always be defined
                self.scope_map.get_mut(&ScopeId::global()).unwrap()
                    .add_symbol(Symbol::new(
                    identifier_name,
                    // In the current implementation all environment variables are doubles
                    SymbolType::EnvironmentVariable(
                            env_address.0,
                            DataType::MUTABLE(env_address.1),
                            env_address.2
                        )
                    )
                );
            }
            ASTNode::FUNCTION { identifier, .. } => {

                let identifier = match identifier.as_ref() {
                    ASTNode::IDENTIFIER(name) => name.clone(),
                    _ => panic!("") // AST Malformed
                };
                
                let function = self.functions.get(&identifier).unwrap().clone();
                for implementation in function.get_implementations() {
                    let symbol_scope = self.scope_map.get_mut(&current_scope).unwrap();
                    symbol_scope.add_symbol(Self::process_function(&implementation));
    
                    // Process function body
                    let inner_func_scope = match implementation.get_body() {
                        ASTNode::SCOPE_BLOCK { scope, .. } => scope,
                        _ => panic!("Malformed AST! Function body should be a scope block! Full body: {:?}", implementation.get_body())
                    };
                    self.generate_new_scope(current_scope.clone(), inner_func_scope.clone(), true);
                    let symbol_scope = self.scope_map.get_mut(&inner_func_scope).unwrap();
                    for (identifier, datatype) in implementation.get_parameters().iter().zip(implementation.get_parameter_types().iter()) {
                        symbol_scope.add_symbol(Symbol::new(identifier.clone(), SymbolType::Parameter(datatype.clone())));
                    }
                    for child in implementation.get_body().clone().children() {
                        self.process_node(child, inner_func_scope.clone());
                    }
                }
            }
            ASTNode::SCOPE_BLOCK { inner, scope } => {
                // Assign next scope id
                self.generate_new_scope(current_scope, scope.clone(), false);

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
        fn print_scope(table: &SymbolTable, f: &mut fmt::Formatter<'_>, id: ScopeId, depth: usize) -> fmt::Result {
            let indent = "\t".repeat(depth);
            let indent_plus = "\t".repeat(depth+1);

            writeln!(f, "{}{{\n", indent)?;
            for symbol in table.scope_map.get(&id).unwrap().symbols.values() {
                writeln!(f, "{}{:?}\n", indent_plus, symbol)?;
            }

            for scope in table.children_of(id) {
                print_scope(table, f,scope.id.clone(), depth + 1)?;
            }

            writeln!(f, "{}}}\n", indent)
        }

        print_scope(self, f,ScopeId::global(), 0)?;

        Ok(())
    }
}