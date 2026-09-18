# Optic's IR Documentation

This is the official documentation of Optic's IR, here you will find a extensive especification of every you need to know about it.

## Table Of Contents

- [Global Symbols & Virtual Registers](#global-symbols--virtual-registers)
- [Type System](#type-system)
- [Functions](#functions)
  - [Extern Functions](#extern-functions)
  - [Variadic Functions](#variadic-functions)
- [Instructions, Globals & Values](#instructions-globals--values)
  - [Values](#values)
  - [Instructions](#instructions)
  - [Globals](#globals)


> [!NOTE]
> Currently, Optic is under active, early-stage development, this mean the IR can change dramatically between diferent versions.

## Global Symbols & Virtual Registers

A global symbol is one that can be accessed from anywhere from the current module, these always start with `@`. A global symbol can be a function or a global variable.

A virtual register is what in high-level languages is called a variable, these always start with `%`. you can define an infinite number of virtual registers, and unlike other IRs (such as LLVM's) Optic's IR *is not* in SSA (Static Single Assignment) form, this means you can reassing a virtual register anytime.

## Type System

Optic's IR is *explictly-typed*, however it's not *strongly-typed*. This means the compiler does not perform any type-checking pass, and it is not planned to be added soon.

Every instructions that produces or manipulates data needs a type, like `load`, `add`, etc. But instructions that doesn't manipulate data directly, like `jmp` or `br`, doesn't need an explicit type.  
There are also some instructions that always produce a value of the same type. For example, `alloc` and `offset` always produces a pointer. And these instructions also need a type, but because they need to know with what data type they are working, even though they'll generate the exact same type as its output.

## Functions

Functions are defined with the `define` instruction, followed by the name, the parameter list, the return type and the function's body. 
This is a simple function in Optic's IR:

```
define @main() i32 {
    ret i32 42
}
```

### Extern Functions

You can define an extern function by writing `extern` between `define` and the function's name, for example:

```
define extern @printf(ptr, ...) i32
```

An extern function is a function which body *is not defined in the current file*. An extern function cannot have a body, because it is supposed to be defined somewhere else.

The parameter list of extern functions only contains the types of the parameters, but not the name. 

### Variadic Functions

You can define a function that takes a variadic number of arguments using `...` as the last parameter. 

Currently, there's no way to manage these arguments, so the only place where variadic functions are useful is in external functions, like `printf`.

## Instructions, Globals & Values

### Values

A Value is just a immediate value or virtual register, a Value can be assigned to a virtual register, and an instruction may return a Value. 

This is a list of all value's descriptions and multiple examples

| Description | Example |
| :-- | :-- |
| A virtual register | `%x` |
| An integer literal, prefix with `-` to denote a negative literal | `2`, `-1` |
| An Floating point number literal | `0.1`, `25.5` |
| A global symbol | `@pi` |

### Labels 

A label is a tag that denotes a position in the IR, an instruction can jump conditionally (with `br`) or unconditionally (with `jmp`) to them.

A label starts with `#`, for example: `#my_label`

### Instructions

This is a list of all instructions, its mnemonics, syntax, description and an example. 

| Mnemonic | Syntax | Description | Example |
| :-- | :-- | :-- | :-- |
| `copy` | `copy TYPE VALUE` | Copies a value into a register | `%r = copy i32 42` |
| `ret` | `ret TYPE VALUE` | returns from the current function with a Value | `ret i32 42` |
| `add` | `add TYPE VALUE, VALUE` | adds two integer values | `%r = add i32 2, 2` |
| `sub` | `sub TYPE VALUE, VALUE` | substracts two integer values | `%r = sub i32 2, 2` |
| `mul` | `mul TYPE VALUE, VALUE` | multiplies two integer values | `%r = mul i32 2, 2` |
| `div` | `div TYPE VALUE, VALUE` | divides two integer values (signed) | `%r = div i32 2, 2` |
| `udiv` | `udiv TYPE VALUE, VALUE` | divides two integer values (unsigned) | `%r = udiv i32 2, 2` |
| `call` | `call TYPE FUNC(TYPE VALUES, ...)` | calls a function with the specified arguments | `%r = call i32 @add(i32 2, i32 2)` |
| `jmp` | `jmp LABEL` | jumps inconditionaly to a label | `jmp #end` |
| `br` | `br VREG, LABEL, LABEL` | jumps conditionaly to a label or another based on the value on the register | `br %cond, #then, #else` |
| `alloc` | `alloc TYPE[, NUM]` | allocates enough size in the stack to store a value of type `TYPE`, `NUM`-times (or 1 if not provided) | `%ptr = alloc i32, 5` |
| `store` | `store TYPE PTR, VALUE` | stores `VALUE` (of type `TYPE`) inside `PTR` | `store i32 %ptr, 5` |
| `load` | `load TYPE PTR` | loads the value of type `TYPE` stored in `PTR` | `%r = load i32 %ptr` |
| `offset` | `offset PTR, TYPE, IDX` | calculates the offset of an element at index `IDX` in an array of values of type `TYPE`. `PTR` is the base pointer of the array | `%r = offset %ptr, i32, 0` | 

#### comparison instructions family

Comparison instructios are a special case, because is not a single instruction, it's a "family of instructions".

All of them return a boolean value of type `i1`, and has a special syntax: `cPREFIX TYPE VALUE, VALUE`, where `TYPE` is the type of the values and the `PREFIX` is the type of comparison. These are all the allowed prefixes: 

| Kind | Meaning |
| :-- | :-- |
| `eq` | equal to |
| `ne` | not equal to |
| `slt`/`ult` | signed/unsigned less than |
| `sgt`/`ugt` | signed/unsigned greater than |

This is a simple example: `%r = ceq i32 %x, 5`

### Globals

You can define a global variable writing `data`, optionally followed by `constant`, the name of the global (which needs to start with `@`), a `=` sign, and brackets containing the data. this is an example:

```
data @msg = { ascii "Hello, World!", i1 0 }
```

A global variable only stores raw bytes, this means the example above is storing an array of 14 bytes. To access the data inside of a global variable you need to use the `load` instruction, because a global variable is just a memory address.
