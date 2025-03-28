## 0.4.2 (2025-03-29)

### Fix

- **qualifiers,-barracuda_pest_semantic_analyser,-function_tracker**: Implemented PartialEq for type qualifiers to check equality. Improved error handling output for checking function parameters/arguments. FunctionImplementations now checks qualifiers for matches_arguments method with the FunctionTracker passing this through in the match_function method. The SemanticAnalyser now analyses function parameter qualifiers during a function call and passes through the qualifiers, datatypes and names (for better error information) and will not allow function calls with mismatched qualifiers

## 0.4.1 (2025-03-26)

### Refactor

- **readme,-commitizen-config**: update readme, add in commitizen config for conventional commits
