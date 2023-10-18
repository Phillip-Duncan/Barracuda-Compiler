use std::collections::{HashMap, HashSet};
use crate::compiler::ast::symbol_table::SymbolType;
use super::symbol_table::{SymbolTable, Symbol};
use super::ScopeId;

/// Scope Tracker is a utility function for behavioural tracking changes in the current
/// scope id. It wraps around a symbol table and will track the scope with the methods
/// enter_scope and exit_scope. This is helpful for scope tracking with recursive methods
/// on the AST tree.
///
/// Symbols have to be tracked according to how the caller defines when they are available using the
/// add_symbol method. Internally the tracker uses this as it cannot know exactly when something is
/// brought into scope in the current scope. For example:
/// {
///     let a = 0;      // allocate_for('a')
///     print(b);       // find_symbol('b') ! Without tracking this may return valid even if the
///                     //                    backend has not allocated for it.
///     let b = 0;      // allocate_for('b')
/// }
///
/// ## Note for future maintainers
///   + Resource id management with local_var_ids and parameter_ids is stretching the
///     responsibilities of this class a bit far and will likely be better off with rework.
///   + Certain assumptions about the parent backend are being made. For instance parameter count
///     resetting upon entering scopes is a great hack for resetting parameter count between function
///     instantiations however it is just a guess based on the current backend and may cause undefined
///     behaviour with extensions in the future.
pub(crate) struct ScopeTracker {
    symbol_table: Option<SymbolTable>,
    current_scope: ScopeId,

    // Locally keep track of symbols in scope
    symbols_in_scope: HashSet<(ScopeId, String)>,

    // Resource allocation ids

    // Maps symbol.unqiue_id() -> localvar_id or param_id
    local_var_ids: HashMap<String, usize>,
    parameter_ids: HashMap<String, usize>,
    array_ids: HashMap<String, usize>,
    local_var_count: usize,
    active_parameter_count: usize,
    array_count: usize,
}



impl ScopeTracker {
    pub fn default() -> Self {
        ScopeTracker {
            current_scope: ScopeId::default(),
            symbol_table: None,
            symbols_in_scope: HashSet::default(),

            local_var_ids: Default::default(),
            parameter_ids: Default::default(),
            array_ids: Default::default(),
            local_var_count: 0,
            active_parameter_count: 0,
            array_count: 0,
        }
    }

    pub fn new(symbol_table: SymbolTable) -> Self {
        ScopeTracker {
            current_scope: symbol_table.entry_scope(),
            symbol_table: Some(symbol_table),
            symbols_in_scope: HashSet::default(),

            local_var_ids: Default::default(),
            parameter_ids: Default::default(),
            array_ids: Default::default(),
            local_var_count: 0,
            active_parameter_count: 0,
            array_count: 0,
        }
    }

    /// Enter a scope
    pub fn enter_scope(&mut self, id: ScopeId) {
        self.current_scope.set(id);

        // Parameters are never added between scopes
        self.active_parameter_count = 0;
    }

    /// Exit current scope. If no parent scope can be found defaults to global scope.
    /// Upon exiting symbols in the current scope are removed and can no longer be referenced.
    /// @return the number of mutable variables dropped leaving the scope
    pub fn exit_scope(&mut self) -> usize {
        // Get new current scope
        let new_scope_id = match &self.symbol_table {
            Some(symbol_table) => {
                symbol_table.parent_of(self.current_scope.clone())
            }
            None => { ScopeId::global() }
        };

        // Remove tracked symbols that only exist in current scope
        let localvars_removed = self.symbols_in_scope.iter()
            .filter(|(scope, identifier)|
                self.find_symbol(identifier).and_then(|symbol| Some(symbol.is_variable()))
                    .unwrap_or(false)
                && scope.eq(&self.current_scope)
            ).count();

        self.symbols_in_scope.retain(|symbol| symbol.0 != self.current_scope );
        self.local_var_count -= localvars_removed;

        // Set new scope
        self.current_scope.set(new_scope_id);

        return localvars_removed;
    }

    /// Adds symbol identifier to currently valid tracked symbols
    /// Variables and parameter symbols are automatically given a valid linear
    /// resource id when added.
    pub fn add_symbol(&mut self, identifier: String) {
        // Add to tracked symbols currently in scope
        self.symbols_in_scope.insert((self.current_scope.clone(), identifier.clone()));

        // Assign resource ids to symbols
        let symbol = self.find_symbol(&identifier).unwrap();
        match symbol.symbol_type() {
            SymbolType::Variable(_) => {
                if symbol.is_array() {
                    let unique_id = symbol.unique_id();
                    let array_count = self.array_count;
                    self.array_count += symbol.array_length();
                    self.array_ids.insert(unique_id.clone(), array_count);
                    self.local_var_ids.insert(unique_id, self.local_var_count);
                    self.local_var_count += 1;
                } else {
                    let unique_id = symbol.unique_id();
                    self.local_var_ids.insert(unique_id, self.local_var_count);
                    self.local_var_count += 1;
                }
            }
            SymbolType::EnvironmentVariable(_, _, _) => {}
            SymbolType::Parameter(_) => {
                let unique_id = symbol.unique_id();
                self.parameter_ids.insert(unique_id, self.active_parameter_count);
                self.active_parameter_count += 1;
            }
            SymbolType::Function { .. } => {}
        }

    }

    /// Attempts to find the symbol within the currently tracked scope and parent scopes.
    /// Will only return symbols that have been explicitly added using add_symbol. This is
    /// to prevent symbols being in scope before they have been declared just because they will
    /// eventually exist within the current scope. @see The ScopeTracker docs for details.
    pub fn find_symbol(&self, identifier: &String) -> Option<&Symbol> {
        // Check if identifier has been declared in scope
        if !self.symbols_in_scope.iter().any(|symbol| identifier.eq(&symbol.1)) {
            return None
        }

        // Return real symbol if found
        match &self.symbol_table {
            Some(symbol_table) => {
                symbol_table.find_symbol(self.current_scope.clone(), identifier)
            }
            None => None
        }
    }
}

impl ScopeTracker {

    /// Returns a valid local variable resource id if symbol with identifier exists in scope
    /// and is a mutable variable. Local id is a valid linear index of mutable variables
    /// currently in scope. As an example look at the following section of code:
    ///     let a = 16;     // localvar_id = 0
    ///     let b = 32;     // localvar_id = 1
    ///     if a > 0 {
    ///         let c = 4;  // localvar_id = 2
    ///         let d = 2;  // localvar_id = 3
    ///     } else {
    ///         let e = 8;  // localvar_id = 2
    ///     }
    ///     let f = 64;     // localvar_id = 2
    pub(crate) fn get_local_id(&self, identifier: &String) -> Option<usize> {
        match self.find_symbol(&identifier) {
            Some(symbol) => {
                match self.local_var_ids.get(&symbol.unique_id()) {
                    Some(id) => Some(id.clone()),
                    None => None
                }
            }
            None => None
        }
    }

    pub(crate) fn get_array_id(&self, identifier: &String) -> Option<usize> {
        match self.find_symbol(&identifier) {
            Some(symbol) => match self.array_ids.get(&symbol.unique_id()) {
                Some(id) => Some(id.clone()),
                None => None
            }
            None => None
        }
    }


    /// Returns a valid parameter resource id if a symbol with identifier exists and is a parameter.
    /// Similar to get_local_id however using a separate id generator
    pub(crate) fn get_param_id(&self, identifier: &String) -> Option<usize> {
        match self.find_symbol(&identifier) {
            Some(symbol) => match self.parameter_ids.get(&symbol.unique_id()) {
                Some(id) => Some(id.clone()),
                None => None
            }
            None => None
        }
    }
}