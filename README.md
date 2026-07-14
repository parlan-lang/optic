# The Optic Compilation Backend

Optic is the official compilation backend for the [Parlan programming language](https://github.com/parlan-lang/parlan). 


## Current Status

Optic is currently in **active, early-stage development**. 

At this moment, Optic does not yet support enough features to compile full Parlan programs and cannot be used as Parlan's primary backend. 


## Optic's IR (Intermediate Representation)

Optic uses its own Intermediate Representation (IR). Here is an example of a basic program in Optic's IR:

```
define @main() i32 {
    ret.i32 42
}
```

Note: While development is ongoing, this simple return-integer program is currently the primary syntax fully supported for compilation

## Requierements & Installation

### Prerequisites

To compile and run Optic, you need to have the Rust toolchain installed on your system. if you don't have Rust installed, you can get it via [rustup](https://rustup.rs)

### Building

To clone the repository and build Optic, run the folowing commands:

``` bash
git clone https://github.com/parlan-lang/optic
cd optic
cargo build --release
```

After a successful build, you will find an `optic` executable at: `./target/release/optic`

## Usage

Once compiled, you can run the compiler directly. For example, to compile an Optic IR file `prog.opt` into C99:

``` bash
./target/release/optic prog.opt -o out.c
```

*If you need more help, run Optic with the `--help` flag to see a list of all supported options and flags* 