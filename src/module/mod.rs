//! This module contains the definitions and implementations
//! of the related to [`Module`]
//! 
//! In this module you will find the definitions of [`Module`], [`Function`](function::Function), [`Instruction`](`instruction::Instruction`), and others.
//! These types represents the structure and contains the data of the program

#![allow(unused)]

pub mod instruction;
pub mod function;

/// Contains the data of the current translate-unit being compiled
/// 
/// A [`Module`] contains information about the current translate-unit that's being
/// compiled, all the functions and global symbols
#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub functions: Vec<function::Function>
}