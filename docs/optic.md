# Optic Documentation

This is the official documentation of the Optic compilation backend. if you want the Optic's IR documentation, see it [here](./ir.md)

## Table Of Contents

- [Getting Started](#getting-started)
  - [Installation](#installation)
  - [Usage](#usage)
- [Pipeline](#pipeline)
  - [IR Parser](#ir-parser)
  - [CFG Builder](#cfg-builder)
  - [SSA Builder](#ssa-builder)
  - [Out of SSA](#out-of-ssa)
  - [Codegen](#codegen)

## Getting Started

### Installation

#### Prerequisites

To compile Optic, you need a Rust toolchain installed in your system, you can install it using [`rustup`](https://rustup.rs).

#### Building

To build Optic, clone its repository and compile it:

```bash
# clone the repo
git clone https://github.com/parlan-lang/optic.git
cd optic

# compile it
cargo build --release
```

After a successful build, you'll find an `optic` executable in `./target/release/optic`

### Usage

Write your code in Optic's IR in a file with a name like `prog.opt`, then run this command to compile it into C:

```bash
./target/release/optic prog.opt -o prog.c
```

This will create a `prog.c` file containing the output C code.

*you can run `./target/release/optic --help` to get more information about flags supported by Optic*

## Pipeline

### IR Parser

The first step in the Optic pipeline is the [IR parser](../src/ir/ir_parser.rs)

The parser translates the textual IR into a in-memory representation, which is divided into hierarchies. The parser outputs a [`Module`](../src/module/mod.rs), which represents the current compilation unit (the current file being compiled), this module contains the functions and globals.

### CFG Builder

After the IR parser converts the textual IR into a `Module`, the [`CFG builder`](../src/cfg/builder.rs) takes a specific function and generates it Control Flow Graph from its linear IR instructions.

The [`CFG`](../src/cfg/mod.rs) represent how the code executes, dividing the code into a graph of basic blocks. This CFG is not in SSA form since the IR is not in SSA form.

The CFG builder ensures that every basic block ends with a terminal instruction, which is an instruction that changes the control flow like `jmp`, `br` or `ret`. If it finds a basic block that doesn't end with a terminal instruction, then it throws an error and ends the compilation.

### SSA Builder

Once the CFG builder constructs the CFG of every function in the `Module`, the [`SSA builder`](../src/ssa/builder.rs) rebuilds the CFG into SSA form using the Braun et. al. algorithm ("Simple and Efficient Construction of Static Single Assignment Form", you can find the pdf [here](https://c9x.me/compile/bib/braun13cc.pdf)).

In the non-SSA CFG, you can reassign any virtual register multiple times, but in SSA form every variable is only assigned once. So, Braun et. al. algorithm inserts $\phi$-nodes anywhere the control flow converges

### Out of SSA

Before going generating the final code, we need to go out of SSA form first. To archieve this, Optic uses a simple algorithm to delete all $\phi$-nodes in the code:

1. **Split Critical Edges**: A Critical Edge occurs when a basic block have multiple predecessors and multiple succesors. We eliminate these Critical Edges by inserting a dummy basic block. 
2. **Lower $\phi$-nodes into Moves**: Then we convert all the $\phi$-nodes into moves. We do this by inserting a copy instruction at the predecessor block just before branching
3. **Sequentialize Parallel Moves**: The we eliminate the parallel moves. Because $\phi$-nodes cannot be executed simoultanously, sometimes we can lost data if two registers swap, so we insert a temporal register to eliminate this problem
4. **Eliminate all $\phi$-nodes**: After all of this, we can delete all $\phi$-nodes sefely. 

#### Codegen

The final step of the pipeline is the [codegen](../src/codegen/), where the in-memory IR is converted into the final output.

Currently, Optic only supports targeting C, but more backends are planned in the future

