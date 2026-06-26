#![allow(unused)]

use crate::module::instruction::*;

/// Represents a parameters
/// 
/// A [`Parameter`] contains all the related information about one parameter of a [`Function`], its name and type
#[derive(Debug, Clone)]
pub struct Parameter {
    name: String,
    ty: Type
}

/// Represents a function
/// 
/// A [`Function`] contains all the related information about one function in the current [`Module`](`crate::cfg::module::Module`)
/// being compiled, like the name, the [`Parameter`]s, the return type and body
#[derive(Debug, Clone)]
pub struct Function {
    name: String,
    params: Vec<Parameter>,
    ty: Type,
    body: Vec<Instruction>
}