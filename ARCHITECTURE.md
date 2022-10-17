# Summary
The purpose of this file is to give an overview of the project structure and to provide as a jumping off point for 
where to add modifications without reading the entire code base. Documentation in the codebase is thorough and so the
code or the doc website should be used for further details. 
This repo is a rust workspace of three rust crates. These crates are barracuda common, barracuda compiler, and barracuda cpu vm.

+ **[Barracuda Common](#barracuda-common)** 
+ **[Barracuda Compiler](#barracuda-compiler)**
+ **[Barracuda CPU VM](#barracuda-cpu-vm)**

#Barracuda Common
Barracuda common acts as a utility library for the compiler and cpu vm of shared code. 
For instance the definitions of operators is used by both the compiler for generation 
and by the cpu vm for context therefore it is stored in the common library. 
This provides a single point of truth and makes the project easier to maintain overall.

### program_code
Program code stores the internal definition of what a ready to be executed barracuda program contains. This is stored in the module file but requires other shared modules contained within such as operator and instruction definitions.
These definitions do not describe computation of the operators this is done by the emulator.

### parser
The parser module contains a trait definition of `ProgramCodeParser` that defines 
how program code can be loaded from a file. Right now the only implementation for this stored in `bct_parser.rs`
loads barracuda code from .bct files which are plain text files. The format leaves open the possibility of binary formats
in the future using the same interface.

### cli_utility
This module contains utility functions for loading some command line arguments. This is to allow the compiler
and the vm to have a shared command interface style. The only modules here presently is `cli_env_var_descriptor` which
parses environment variable command arguments with their unique syntax.

# Barracuda Compiler
The compiler starts with two files `main.rs` for the binary target and `lib.rs` for the DLL target. Main contains 
the context for processing the cll as its configuration is small due to the use of the clap library. Lib contains the 
translation interface functions using ffi so that they are exposed when compiling as a DLL and is used to generate c headers.
The core of the compiler's functionality is stored within the compiler module. 

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

# Barracuda CPU VM
The cpu virtual machine starts in `main.rs`. This file contains the context for processing the cli configuration
and using that context to run the emulator either in execution or visual debugging mode given the `-d` flag. 

### emulator
The emulator modules main class is stored in `main.rs` called `ThreadContext`. `ThreadContext` can be loaded with
a program code and configured to run under different expected circumstances. The execution of program code relies on
some external utility classes. To execute instructions and operations, `instuction_executor.rs` and 
`operation_executor.rs` are used and contain definitions for interfacing with ThreadContext to execute a instruction. 
These definitions centralise around four key steps.
+ pop arguments to the instruction/operator
+ cast arguments as relevant types
+ apply operation in rust code
+ push result(s) to the stack

Additionally, in `emulator_heap.rs` is a manager class for giving an interface for allocating memory on the heap in a way
that is transparent to the executing program code but is inspectable and safe for the emulator itself.

### visualiser
The visualiser is a wrapper around `ThreadContext` defined in the `emulator` sibling module. It contains defintions
for different widgets used to control the TUI interface. 
