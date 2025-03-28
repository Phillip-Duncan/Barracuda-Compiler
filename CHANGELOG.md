## 0.5.0 (2025-03-29)

### Feat

- **Qualifiers**: Proper implementation of type qualifiers (const/mut). Implemented the const/mut keywords and mostly ensured their correctness w.r.t assignment. Everything is const by default. Implemented const for arrays, allowing for statically allocated constant arrays for memory saving. New ops (LDCUX and LDCUPTR) implemented for the loading of ptr/values from constant memory. Updated several components (e.g., function parameters) to use the const/mut keywords

### Fix

- **tests**: Fix the last 7 tests that were previously failing due to qualifier changes. Additionally remove an erronous print statement and fix doctests of ASTNode being incorrectly generated for latest rust version
- **qualifiers,-barracuda_pest_semantic_analyser,-function_tracker**: Implemented PartialEq for type qualifiers to check equality. Improved error handling output for checking function parameters/arguments. FunctionImplementations now checks qualifiers for matches_arguments method with the FunctionTracker passing this through in the match_function method. The SemanticAnalyser now analyses function parameter qualifiers during a function call and passes through the qualifiers, datatypes and names (for better error information) and will not allow function calls with mismatched qualifiers
- **Cargo**: Change version to 0.4.0, this was previously arbitarily changed incorrectly
- **arrays**: add sync operations and fix static arrays
- **tests**: update tests for previous changes
- **analyse_array_index**: fix the index value for this method
- **analyse_array_index**: Handle environmentvariable type
- **userspace**: Add userspace size to track the size of the generated user space, this is used for implementation MALLOC of userspace. Add PEST rule for type qualifiers (e.g., const, mut). integration TBC. add method set_environment_variable_count, this is used to count total env_vars from implementer cfg, not those used in program, as this is the proper stride that should be used. Old one was incorrect. Add size method to PrimitiveDataType to return memory size of each primitive for use in ADD_P. Made ENVIRONMENTVARIABLE a DataType, this is to allow for ENV VARS to be values, pointers, arrays all at once and still satisfy semantic analysis. Add a new new built ins for memory/pointers. These should only be used with expert knowledge but are exposed temporarily until arrays are completely finished (including all cases). Add size parameter to BarracudaIR::Array. Propagates the size of the array through to generation. Temporarily make all variable/parameter types MUTABLE. This is a fix until types/qualifiers are implemented properly for arrays. add semantic analysis for assigning to CONST, causes a panic. Strictly defining non-assignment to CONST values.

### Refactor

- **barracuda_pest_semantic_analyser**: removed constructing memory for an allocation that already exists
- **program_code_builder**: dead code removal
- **datatype**: dead code and unreachable condition removal
- **lib**: dead code removal
- **barracuda_bytecode_generator**: dead code removal. Since most of the qualifier analysis is done in the semantic analyser, a lot of qualifier usage in functions could be removed
- **mod**: dead code removal
- **bct_parser**: dead code removal
- **readme,-commitizen-config**: update readme, add in commitizen config for conventional commits
- **ops**: Add and/or logical ops, along with lshift and rshift
- **arrays**: Make arrays be initialized directly into userspace rather than requiring allocation at runtime. Print is now a function and not a statement as this had caused issues with some function names. Change typo: REFERENENCE to REFERENCE. Change pack strings function to recognise special characters (e.g. \n) and pack these as a single char. Add new operations for reading/writing specific types, this is required for memory safety
- **strings**: Add string implementation and numerical precision. Naive string implementation, TODO: type/semantic analysis for this. Numerical precision as an input as this allows strings to work (strings get packed into f32/f64 values). Numerical precision can be used more in future for optimization/bit packing/etc. Add new utils.rs file for utilities used by multiple modules, currently only contains string packing method.

## 0.4.2 (2025-03-29)

### Fix

- **qualifiers,-barracuda_pest_semantic_analyser,-function_tracker**: Implemented PartialEq for type qualifiers to check equality. Improved error handling output for checking function parameters/arguments. FunctionImplementations now checks qualifiers for matches_arguments method with the FunctionTracker passing this through in the match_function method. The SemanticAnalyser now analyses function parameter qualifiers during a function call and passes through the qualifiers, datatypes and names (for better error information) and will not allow function calls with mismatched qualifiers

## 0.4.1 (2025-03-26)

### Refactor

- **readme,-commitizen-config**: update readme, add in commitizen config for conventional commits
