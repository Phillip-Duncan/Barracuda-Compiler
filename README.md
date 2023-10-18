# Barracuda Compiler
Welcome to the repo of the barracuda compiler. [Barracuda]() is a header only CUDA library for running concurrent bytecode 
on GPU cores. This repo provides helpful tools in generating this bytecode from a natural high level language. This is
accomplished by the compiler itself but is aided by the debugging tools offered in the CPU VM Emulator.

## Installation
Installation requires having rust-stable installed at the time of writing 1.64.0. This will install the rust toolchain 
as well as the **cargo** package manager used to control dependencies, test, and documentation.

+ Generate all targets use: `cargo build`
+ Generate a target use: `cargo build <target> (--bin | --lib)?`
    + Target: common, compiler, vm_emulator.

    
## Usage

### Compiler
The compiler itself can be used by:

`barracuda_compiler <filename.bc> -o <outputfile.bct>`

This will generate a `filename.bct` file by default if no output file is specified. The output can 
also be directly printed using the flag `--stdout`

Environment variables can be specified as existing in the target host environment via the `--env` command. Where each 
variable has the syntax of `identifier(:host_index)?`. If no host index is specified one is given based on the order
of the given variables. As an example of the usage see below.

`barracuda_compiler <filename.bc> --env X:0 Y:1 Z:2 OUTPUT:3`

## Examples
The best example programs can be found in the related barracuda-vm-testing repository.
More can be found in the test suite in `compiler/lib.rs`
Example programs can be found in `compiler/examples/barracuda`

## Tests
All tests can be run out of the box using `cargo test`, this will run all integration and unit tests within the workspace.
Individual tests can be specified and run using `cargo test <test_name>`.

## Documentation
Documentation can be generated from `cargo doc`. This will create a static website in `target/doc` that can be
navigated to get a better understanding of the codebase. If you are looking for a higher level overview of the
codebase rather than particulars please see [ARCHITECTURE.md](ARCHITECTURE.md).
