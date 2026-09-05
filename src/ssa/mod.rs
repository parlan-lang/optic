//! This module contains the definitions to implement the Braun et. al. algorithm to convert the non-SSA [`Control Flow Graph`](`ControlFlowGraph`) into SSA form

pub mod builder;

use crate::cfg::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vreg(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SsaValue(pub usize);

pub struct PhiNode {
    pub block: BlockId,
    pub vreg: Vreg,
    pub operands: Vec<SsaValue>
}