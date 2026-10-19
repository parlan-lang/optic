# **Optic IR Documentation**

Welcome to the official documentation for Optic IR—A low-level, platform-agnostic intermediate representation used by the Optic compilation backend

Here you willl find a complete technical specification and reference guide for working with Opti IR, with examples and detailed explanantions about everything you need to know about Optic IR. 

> [!WARNING]
> *Optic is still in early-stage development. The IR is not stable yet, so its format, syntax, and features may change dramatically between versions*

---

# Table Of Contents

1. [Program Structure](#1-program-structure)
   - [Modules](#modules)
   - [Globals](#globals)
   - [Functions](#functions)
   - [Basic Blocks](#basic-blocks)
2. [Type System](#2-type-system)
3. [Values & Literals](#3-values--literals)
4. [Instruction Set Reference]()
   - [Arithmetic Operations](#arithmetic-operations)
   - [Comparison Operations](#comparison-operations)
   - [Terminator/Branching Operations](#terminatorbranching-instructions)
   - [Memory operations](#memory-operations)
   - [Function Calls](#function-calls)
5. [IR Invariants & Correctness](#ir-invariants--correctness)
6. [Complete Example](#complete-example)
  
---

# 1. Program Structure

## Modules

A module is what we call a *compilation-unit*, this means that every module is compiled separately from others. Every module is contained by a separated IR file. 

Currently, Optic's CLI (Command Line Interface) doesn't support compiling multiples files at the same time. However, you can compile two modules separately and link them with each other using an external linker.

## Globals

A module can contain multiple *globals*, which are essenctially equivalent to global variables in higher-level languages like C. A global only contains raw bytes, it doesn't actually have a concrete type, and they're accesed as pointers by memory operations.

This is an example of a global definition:
```
data constant @PI = { f32 3.14159 }
```

We use the `data` keyword, the name of the global which must start with `@`, and the byte section.  
The byte section is enclosed by brackets `{}`, and contains values of form `TYPE VALUE` separated by commas, you can put as many of these as you want. All of these values will be converted into raw bytes, like an array.

You also have the `ascii` type, which cannot be used on any place other than a global. You type `ascii` followed by a string enclosed by quotes `"`, inside the global's byte section. For example:
```
data constant @hello = { ascii "Hello, World!", i1 0 } // null terminated
```

## Functions

Functions in Optic IR are the same as in other high-level languages. This is an example:
```
define @square(%x i32) i32 {
  %0 = mul i32 %x, %x
  ret i32 %0
}
```

We define a function using the `define` keyword, the name of the function which must start with `@`, the parameter list, the return type and the function body.  

### Extern Functions

Extern (or external) functions are functions which bodies are not defined in the current file. They are declared by placing `extern` after `define` and before the function's name. This is an example:
```
define extern @printf(ptr, ...) i32
```

Extern functions doesn't have named parameters, this means that you only need to list the types of the parameters. Extern functions also doesn't have a body, because it must be defined elsewhere; Adding a body to an extern function will make Optic throw an error and refuse to compile.

In the example, we can also see `...`, which denotes that the function is variadic (can take a variadic number of parameters). These is only useful in extern functions since Optic's IR doesn't have any way to manipulate these parameters.

## Basic Blocks

A function's body is composed by a sequence of basic block; every basic block is sequence of instructions without any branching. A basic block is usually limited by labels: a label starts a new basic block, finishing the last one. 

Every basic block has one strict rule: it MUST end with a terminator instruction. A terminator instruction is an instruction that changes the control flow by dividing, looping or terminating execution. See [all the terminator instructions](#terminatorbranching-instructions)

---

# 2. Type System

Optic IR is a statically-typed language, which means you need to specify explicit type for instructions, functions, values, etc. 
Optic supports all the following primitive types:

| Type | Description |
| :--: | :-- |
| `i32` | 32-bit integer |
| `i1` | 1-bit integer (a boolean) |
| `f32` | 32-bit (single-presition) float |
| `ptr` | Opaque pointer type |
| `void` | Void type |

*There's also the `ascii` type, but this one is only usable on a global definition. See [more here](#globals)*

Optic also have arrays, collections of elements of the same type. These can be created using the `alloc` instruction, and access elements through `offset` (which returns the pointer to that element). This is a simple, runnable example:
```
define @main() i32 {
  // create an array of 5 i32s
  %my_arr = alloc i32, 5
  
  // access the first element position and store a value
  %0 = offset %my_arr, i32, 0
  store i32 %0, 5

  ret i32 0
}
``` 

Optic doesn't currently support for user-defined types, but they will be supported in future versions.

> [!NOTE] 
> Optic currently doesn't feature a type checker, this means it does not check if the types in the IR are correct or not.  
> Future versions may implement a type checker or verifier.

---

# 3. Values & Literals

A value can be either a literal (e.g., `2`), a virtual register (e.g., `%x`) or a global (e.g., `@msg`). A value is exactly that, *a value* that an instruction can use. 

A literal is a kind of value that contains data of some primitive type, such as `i32`. The exact type of a literal may change depending on where it's used. Optic currently supports the following literals:

| Literal | Description | Example(s) |
| :--: | :-- | :-- |
| Integer | An integer number, may be negated (2's complement) using `-` | `2`, `-50` |
| Float | A floating-point number, may be negated using `-` | `2.0`, `-3.14` |
| String | An ASCII character sequence enclosed with quotes (only used inside globals) | `"Hello"`, `"World"` | 

# 4. Instruction Set Reference

<!--
**NOTE**: This section could be improved by providing better explanations of 
what does the instructions do exactly 
(e.g., explaining the difference beetween ordered and unordered FP comparisons). 
-->

## Arithmetic Operations

| Instruction | Syntax (EBNF) | Description | Example |
| :--: | :-- | :-- | :-- |
| ADD | `Vreg "=" "add" Type Value, Value` | Integer addition | `%0 = add i32 2, 2` |
| SUB | `Vreg "=" "sub" Type Value, Value` | Integer substraction | `%0 = sub i32 2, 2` |
| MUL | `Vreg "=" "mul" Type Value, Value` | Integer multiplication | `%0 = mul i32 2, 2` |
| DIV | `Vreg "=" "div" Type Value, Value` | Integer division (signed) | `%0 = div i32 2, 2` |
| UDIV | `Vreg "=" "udiv" Type Value, Value` | Integer division (unsigned) | `%0 = udiv i32 2, 2` |
| NEG | `Vreg "=" "neg" Type Value` | Integer negation | `%0 = neg i32 2` |
| FADD | `Vreg "=" "fadd" Type Value, Value` | FP addition | `%0 = fadd f32 2.0, 2.0` |
| FSUB | `Vreg "=" "fsub" Type Value, Value` | FP substraction | `%0 = fsub f32 2.0, 2.0` |
| FMUL | `Vreg "=" "fmul" Type Value, Value` | FP multiplication | `%0 = fmul f32 2.0, 2.0` |
| FDIV | `Vreg "=" "fdiv" Type Value, Value` | FP division | `%0 = fdiv f32 2.0, 2.0` |
| FNEG | `Vreg "=" "fneg" Type Value` | FP negation | `%0 = fneg f32 2.0` |

## Comparison Operations

| Instruction | Syntax (EBNF) | Description | Example |
| :--: | :-- | :-- | :-- |
| CEQ | `Vreg "=" "ceq" Type Value, Value` | Integer Equal-To | `%0 = ceq i32 2, 2` |
| CNE | `Vreg "=" "cne" Type Value, Value` | Integer Not-Equal | `%0 = cne i32 2, 2` |
| CSLT | `Vreg "=" "cslt" Type Value, Value` | Integer Less-Than (signed) | `%0 = cslt i32 2, 2` |
| CULT | `Vreg "=" "cult" Type Value, Value` | Integer Less-Than (unsigned) | `%0 = cult i32 2, 2` |
| FCOEQ | `Vreg "=" "fcoeq" Type Value, Value` | FP Equal-To (ordered) | `%0 = fcoeq f32 2.0, 2.0` |
| FCUEQ | `Vreg "=" "fcueq" Type Value, Value` | FP Equal-To (unordered) | `%0 = fcueq f32 2.0, 2.0` |
| FCONE | `Vreg "=" "fcone" Type Value, Value` | FP Not-Equal (ordered) | `%0 = fcone f32 2.0, 2.0` |
| FCUNE | `Vreg "=" "fcune" Type Value, Value` | FP Not-Equal (unordered) | `%0 = fcune f32 2.0, 2.0` |
| FCOLT | `Vreg "=" "fcolt" Type Value, Value` | FP Less-Than (ordered) | `%0 = fcolt f32 2.0, 2.0` |
| FCULT | `Vreg "=" "fcult" Type Value, Value` | FP Less-Than | `%0 = fcult f32 2.0, 2.0` |
| FCORD | `Vreg "=" "fcord" Type Value, Value` | FP Ordered Cmp | `%0 = fcord f32 2.0` |
| FCUNO | `Vreg "=" "fneg" Type Value, Value` | FP Unordered Cmp | `%0 = fcuno f32 2.0` |

## Terminator/Branching Instructions

| Instruction | Syntax (EBNF) | Description | Example |
| :--: | :-- | :-- | :-- |
| RET | `"ret" Type Value` | Return a value | `ret i32 2` (or `ret void`) |
| JMP | `"jmp" Label` | Unconditional Jump | `jmp #then_br` |
| BR | `"br" Value, Label, Label` | Conditional Jump | `br %0, #then_br, #else_br` |

## Memory Operations

| Instruction | Syntax (EBNF) | Description | Example |
| :--: | :-- | :-- | :-- |
| COPY | `Vreg = "copy" Type Value` | Copy value | `%0 = copy i32 2` |
| ALLOC | `Vreg = "alloc" Type[, Value]` | Allocate memory in stack for `Type` times 1 or `Value` | `%0 = alloc i32, 5` |
| LOAD | `Vreg = "load" Type Value` | Load value of type `Type` from a pointet | `%0 = load i32 %ptr` |
| STORE | `"store" Type Value, Value` | Store value of type `Type` onto a pointer | `store i32 %ptr, 2` |

## Function Calls

To call a function, you use the `call` instruction, with the following EBNF syntax:

```
[Vreg "="] "call" Type GlobalSym "(" { Type Value "," } ")" 
```

A simple example:

```
%0  = call i32 @add(i32 2, i32 2)
```

If the destination virtual register is ommited, then the value will be discarded.

# IR Invariants & Correctness

The Optic IR has some invariants, these are rules that must remain true through all the program, always. These are the Optic IR invariants:

1. *Virtual registers MUST start with `%`*
2. *Global symbols (function names and/or globals' name) MUST start with `@`*
3. *Every basic block MUST end with a terminator instruction*
4. *Extern functions MUST NO have a body*
5. *Non-Extern functions MUST have a body*
  
--Note that these invariant may change, deleted, or new invariants may be added--

## Correctness

Optic currently doesn't feature a verifier for the IR, this means even if your IR code doesn't break any invariants, it may still be wrong. For example, types; Optic doesn't verify types at all in the IR, this means that you *technically* can, for example, return an `i32` value from a function of type `f32` without Optic complaining.

Futures versions of Optic will (probably) include some sort of verifier for the IR. For now, you need to manually double-check your code to ensure that the IR is correct, otherwise your program may compile but behave in unexpected ways.

# Complete Example

This is a complete, working example of the Optic IR, using most of its instructions and features:

```
// a comment!

// define some globals
data constant @PI = { f32 3.1415927 }
data constant @msg = { ascii "area of the circle is: %f", i1 0 }

// define an extern, variadic function
define extern @printf(ptr, ...) i32

// define more functions
define @area_of_circle(%radius f32) f32 {
  // formula: πr^2 
  %0 = fmul f32 %radius, %radius
  %1 = load f32 @PI
  %2 = fmul f32 %1, %0
  ret f32 %2
}

define @main() i32 {
  %radius = copy f32 5.0
  %area = call f32 @area_of_circle(f32 %radius)

  call i32 @printf(ptr @msg, f32 %area)

  ret i32 0
}
```

To compile it, save that example into a `test.opt` file, and run this commands:
```bash
cargo run -- test.opt -o test.c
clang test.c -o test -w
```

*If you are on windows, add `.exe`. You can use other C compiler, not only Clang*
