#![allow(unused)]

use crate::module::instruction::*;

/// Represents a parameters
/// 
/// A [`Parameter`] contains all the related information about one parameter of a [`Function`], its virtual register and type
#[derive(Debug, Clone)]
pub struct Parameter {
    pub vreg: usize,
    pub ty: Type
}

/// Represents a function
/// 
/// A [`Function`] contains all the related information about one function in the current [`Module`](`crate::cfg::module::Module`)
/// being compiled, like the name, the [`Parameter`]s, the return type and body
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub ty: Type,
    pub body: Vec<Instruction>
}