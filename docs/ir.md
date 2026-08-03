# Optic's IR Documentation

This is the official documentation of Optic's IR, here you will find a extensive especification of every you need to know about it.

## Table Of Contents

- [Global Symbols & Virtual Registers](#global-symbols--virtual-registers)
- [Type System](#type-system)
- [Functions](#functions)
  - [External Functions](#external-functions)
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

Optic's IR is explictly-typed, these means every single instruction needs an explicit type. The type of an instruction can be specified by two ways depending on the type of the instruction.

A instruction can either compute & store a value, or execute an action:
- **if the instruction computes & stores a value**: it always starts with the destination virtual register, followed by an assing symbol, a dot and the type. for example: `%vreg =.i32 copy 23` 
- **if the instruction just executes an action:** the type is specified after the instruction mnemonic and a dot. for example: `ret.i32 42`

> [!NOTE]
> Optic doesn't currently features any kind of type checking, and it's not planned to add it for now. 

## Functions

Functions are defined with the `define` instruction, followed by the name, the parameter list, the return type and the function's body. 
This is a simple function in Optic's IR:

```
define @main() i32 {
    ret.i32 42
}
```

### External Functions

You can define an external function using `extern`, for example:

```
define extern @printf(%fmt ptr, ...) i32
```

An external function is a function which body *is not defined in the current file*. An external function cannot have a body, because it is supposed to be defined somewhere else.

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
| A integer literal, prefix with `-` to denote a negative literal | `2`, `-1` |

### Labels 

A label is a tag that denotes a position in the IR, an instruction can jump conditionally or unconditionally to them.

A label starts with `#`, for example: `#my_label`

### Instructions

This is a list of all instructions, its mnemonics, syntax, description and an example. 

| Mnemonic | Syntax | Description | Example |
| :-- | :-- | :-- | :-- |
| `copy` | `copy VALUE` | Copies a value into a register | `%r =.i32 copy 42` |
| `ret` | `ret.TYPE VALUE` | returns from the current function with a Value | `ret.i32 42` |
| `add` | `add VALUE, VALUE` | adds two integer values | `%r =.i32 add 2, 2` |
| `sub` | `sub VALUE, VALUE` | substracts two integer values | `%r =.i32 sub 2, 2` |
| `mul` | `mul VALUE, VALUE` | multiplies two integer values | `%r =.i32 mul 2, 2` |
| `div` | `div VALUE, VALUE` | divides two integer values (signed) | `%r =.i32 div 2, 2` |
| `udiv` | `udiv VALUE, VALUE` | divides two integer values (unsigned) | `%r =.i32 udiv 2, 2` |
| `call` | `call FUNC(VALUES,...)` | calls a function with the specified arguments | `%r =.i32 call @add(2, 2)` |
| `jmp` | `jmp LABEL` | jumps inconditionaly to a label | `jmp #end` |
| `br` | `br VREG, LABEL, LABEL` | jumps conditionaly to a label or another based on the value on the register | `br %cond, #then, #else` |

#### `cmp` instruction family

The `cmp` instruction is a special case, because is not a single instruction, it's a "family of instructions".

All of them return a boolean value of type `i1`, and has a special syntax: `cmp.KIND.TYPE VALUE, VALUE`, where `TYPE` is the type of the values. these are the possible `KIND`s of comparitions:

| Kind | Meaning |
| :-- | :-- |
| `eq` | equal to |
| `ne` | not equal to |
| `slt`/`ult` | signed/unsigned less than |
| `sgt`/`ugt` | signed/unsigned greater than |

This is a simple example: `%r =.i1 cmp.eq.i32 %x, 5`

### Globals

You can define a global variable using the `data` instruction. this is an example:

```
data @msg =.str "Hello, World!\0"
```

A global variable can have any value that can be know at compile-time, like a number or a literal string. Currently there is a string type, but it may be removed later
