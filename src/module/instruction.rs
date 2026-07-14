#![allow(unused)]

/// Represents a value that can be used by an instruction
/// 
/// A [`Value`] is any value that can be used by an instruction, e.g., an integer literal
/// or a virtual register
#[derive(Debug, Clone)]
pub enum Value {
    IntLit(usize),
    Vreg(usize),
}

/// The Type of an [`Instruction`]
/// 
/// Every [`Instruction`] have a type (e.g., [`Type::I32`]), and this enum represents all of the posible types
/// that an [`Instruction`] can have
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    I32,
}

/// Represents a single instruction and its data
/// 
/// A [`Instruction`] contains all the information related to that single instruction
/// that it represents
#[derive(Debug, Clone)]
pub enum Instruction {
    Ret {
        val: Value,
        ty: Type 
    },
    Copy {
        vreg: usize,
        val: Value,
        ty: Type
    }
}