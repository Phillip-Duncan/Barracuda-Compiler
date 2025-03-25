# Barracuda Compiler 🦈⚙️
Welcome to the repo of the Barracuda compiler. [Barracuda](https://github.com/Phillip-Duncan/Barracuda) is a header only CUDA library for running concurrent bytecode 
on GPU cores. This repo provides helpful tools in generating this bytecode from a natural high level language.

## Installation 🛠️
Installation requires having rust-stable 1.64.0+ installed. This will install the rust toolchain 
as well as the **cargo** package manager used to control dependencies, test, and documentation.

+ Generate all targets use: `cargo build`
+ Generate a target use: `cargo build <target> (--bin | --lib)?`
    + Target: common, compiler, vm_emulator.

    
## Usage 💻

### Compiler
The compiler itself can be used by:

`barracuda_compiler <filename.bc> -o <outputfile.bct>`

This will generate a `filename.bct` file by default if no output file is specified. The output can 
also be directly printed using the flag `--stdout`

Environment variables can be specified as existing in the target host environment via the `--env` command. Where each 
variable has the syntax of `identifier(:host_index)?`. If no host index is specified one is given based on the order
of the given variables. As an example of the usage see below.

`barracuda_compiler <filename.bc> --env X:0 Y:1 Z:2 OUTPUT:3`

## Examples ℹ️
The best example programs can be found in the related barracuda-vm-testing repository.
More can be found in the test suite in `compiler/lib.rs`
Example programs can be found in `compiler/examples/barracuda`

## Tests 🧪
All tests can be run out of the box using `cargo test`, this will run all integration and unit tests within the workspace.
Individual tests can be specified and run using `cargo test <test_name>`.

## Documentation 📄
Documentation can be generated from `cargo doc`. This will create a static website in `target/doc` that can be
navigated to get a better understanding of the codebase. If you are looking for a higher level overview of the
codebase rather than particulars please see [ARCHITECTURE.md](ARCHITECTURE.md).

## Contributing 💯

All contributions are more than welcome. This project has been developed by a small team and is currently solo-maintained. As of v0.4.0 this project uses [Commitizen](https://commitizen-tools.github.io/commitizen/) with [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) (CC) for consistency and simplicity of project maintainance and contribution. All contributions are required to follow the CC specification, using the [Commitizen](https://pypi.org/project/commitizen/) tool is likely the simplest way to achieve this.

Recommendations for contributions:

* New features are less important than bug fixes, if creating a new feature please add new tests and make sure all existing tests either pass or you have fixed them to account for the changes and discussed why it was necessary.
* Bug fixes and defensive systems (e.g., type checking) are always welcome, but make sure you test any and all changes and submit test code (where different from existing infrastructure) to the PR/issue. It is also a good idea to run performance benchmarks (e.g., Mandelbrot) to quantify any performance improvements/degradation.
* Refactors should only be performed if they contribute to performance gains or if they significantly increase the simplicity, readability, understanding or debugging of the compiler.

**There are some (significant) changes that may require implementation from the Barracuda VM side first before compiler change. For these changes, please submit an issue/request in the Barracuda VM repository discussing which new additions/modifications are required. If this is a breaking change then this will require collaboration between maintainers of the VM and compiler and should be done on separate branches of both in lockstep.**
