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

### Emulator
The emulator can be executed by giving a .bct file to run where output will be sent to stdout. This is given by 
the following usage:

`barracuda_emulator <filename.bct>`

The emulator can be run as a visual debugger by attaching the debug flag `-d`. This will launch a TUI interface so that 
you can step through instructions using SPACE and continue execution until finished with ENTER.

Environment variables can be specified as existing in the target host environment via the `--env` command. Where each
variable has the syntax of `identifier(:host_index)?(=init_value)?`. If no host index is specified one is given based on the order
of the given variables. If no value is specified it is set as zero. As an example of the usage see below.

`barracuda_emulator <filename.bct> --env X=1 Y=0.5 Z=1 OUTPUT`

## Examples
Example programs can be found both in `compiler/examples/barracuda` and `vm_emulator/examples`.

## Tests
All tests can be run out of the box using `cargo test`, this will run all integration and unit tests within the workspace.
Individual tests can be specified and run using `cargo test <test_name>`.

## Documentation
Documentation can be generated from `cargo doc`. This will create a static website in `target/doc` that can be
navigated to get a better understanding of the codebase. If you are looking for a higher level overview of the
codebase rather than particulars please see [ARCHITECTURE.md](ARCHITECTURE.md).
