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

## Optic Documentation

You can find the documentation of Optic's IR and Optic itself in the [docs](./docs/) folder.

## Contributing

Thank you for considering contributing! We appreciate every kind of contributions!

Before contributing, check out our contribution guidelines [here](./CONTRIBUTING.md)
