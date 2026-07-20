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

/* Virtual Register Aliases */

#[derive(Debug, Clone)]
pub struct VregAlias (Vec<usize>);

impl VregAlias {
    pub fn new() -> Self { 
        VregAlias(Vec::new())
    }

    fn ensure_vreg(&mut self, vreg: usize) {
        if vreg >= self.0.len() {
            let old_len = self.0.len();
            self.0.resize(vreg + 1, 0);
            for i in old_len..=vreg {
                self.0[i] = i;
            }
        }
    }

    pub fn find(&mut self, vreg: usize) -> usize {
        self.ensure_vreg(vreg);
        if self.0[vreg] == vreg {
            vreg
        } else {
            let root = self.find(self.0[vreg]);
            self.0[vreg] = root;
            root
        }
    }

    pub fn union(&mut self, vreg1: usize, vreg2: usize) {
        let root1 = self.find(vreg1);
        let root2 = self.find(vreg2);
        if root1 != root2 {
            self.0[root2] = root1;
        }
    }
}