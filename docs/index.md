# Optic's IR Documentation

This is the official documentation of Optic's IR, here you will find a extensive especification of every single instruction that the last version of Optic support, which is `0.2.x`

> [!NOTE]
> Currently, Optic is under active, early-stage development, this mean the IR can change dramatically between diferent versions.  
> Also, not all updates changes this docs, some updates may only change something internal related to the pipeline or the CLI, but not the IR 

## Global Symbols & Virtual Registers

A global symbol is one that can be accessed from anywhere from the current module, these always start with `@`. A global symbol can be a variable or a global variable.

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

## Instructions & Values

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