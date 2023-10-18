# Summary
The purpose of this file is to give an overview of the project structure and to provide as a jumping off point for 
where to add modifications without reading the entire code base. Documentation in the codebase is thorough and so the
code or the doc website should be used for further details. 
This repo is a rust workspace of two rust crates. These crates are barracuda common and barracuda compiler. 
The existence of the common crate is an artifact of when a third vm crate existed.

+ **[Barracuda Common](#barracuda-common)** 
+ **[Barracuda Compiler](#barracuda-compiler)**

# Barracuda Common
Barracuda common acts as a utility library for the compiler.
### program_code
Program code stores the internal definition of what a ready to be executed barracuda program contains. This is stored in the module file but requires other shared modules contained within such as operator and instruction definitions.
These definitions do not describe computation of the operators this is done by the emulator.

### parser
The parser module contains a trait definition of `ProgramCodeParser` that defines 
how program code can be loaded from a file. Right now the only implementation for this stored in `bct_parser.rs`
loads barracuda code from .bct files which are plain text files. The format leaves open the possibility of binary formats
in the future using the same interface.

### cli_utility
This module contains utility functions for loading some command line arguments. 
The only modules here presently is `cli_env_var_descriptor` which
parses environment variable command arguments with their unique syntax.

# Barracuda Compiler
The compiler starts with two files `main.rs` for the binary target and `lib.rs` for the DLL target. Main contains 
the context for processing the cll as its configuration is small due to the use of the clap library. Lib contains the 
translation interface functions using ffi so that they are exposed when compiling as a DLL and is used to generate c headers.
The core of the compiler's functionality is stored within the compiler module. 

See compiler_flow.png for the path of compilation.

### compiler
The compiler `mod.rs` file contains the definition of `Compiler` which accepts two type definitions for the
parser to use, and the generator to use. This allows for data driven configuration of the compiler if future parser/generators
are written and need to substitute. The definitions of the traits `AstParser` and `BackEndGenerator` are stored within 
the lower level modules ast and backend.

Adding new language features requires modifying the ast for an intermediate representation of that feature. Modifying
the pest grammar file `barracuda.pest`. Providing the substitution of the pest tokens to the AST nodes in 
the parser implementation. Then finally adding the backend representation from the AST nodes.

### compiler/ast
This module stores the relevant definitions of the AbstractSyntaxTree in the `mod.rs` file. These definitions
require a lot of other classes for defining nodes, datatypes, symbol tables. The odd file out here is the 
`scope_tracker.rs` file which is a utility function for traversing the symbol table scopes given the current generation 
context.

### compiler/parser
This module defines the implementations for the `AstParser` trait defined in `mod.rs`. Presently only one
implementation exists which is `barracuda_pest_parser.rs`. This implementation as the name states relies on the
pest library to tokenize the input source code. The definitions for this tokenization can be found in the `barracuda.pest` 
file at the root of this crate. 

### compiler/backend
This module defines the translation from an abstract syntax tree to executable `ProgramCode` through the trait
`BackEndGenerator`. One implementation is written for this trait called `barracuda_bytecode_generator.rs`. 
This implementation relies on of a utility class in `program_code_builder.rs` that provides utilities in creating 
program code. Utilities such as creating and referencing labels in an unknown sized generated program code. 

Additionally this module contains a sub module called analysis which contains stack_estimator that will guess the stack
size required to execute a program in ProgramCode. This guess relies on some generation knowledge and is therefore stored
at this level.