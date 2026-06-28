//! This module contains the definitions of the CFG (Control Flow Graph) of Optic
//! 
//! having a CFG makes easier some optimizations and analisys

#![allow(unused)]

pub mod builder;

use crate::module::instruction::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockId(pub usize);

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>
}

#[derive(Debug, Clone, Default)]
pub struct ControlFlowGraph {
    pub blocks: Vec<BasicBlock>,
    pub forward_edges: Vec<Vec<BlockId>>,
    pub backward_edges: Vec<Vec<BlockId>>
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        ControlFlowGraph::default()
    }

    pub fn create_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len());
        let block = BasicBlock {
            id,
            instructions: Vec::new()
        };
        self.blocks.push(block);
        self.forward_edges.push(Vec::new());
        self.backward_edges.push(Vec::new());
        id
    }

    pub fn add_ins(&mut self, block_id: BlockId, ins: Instruction) {
        if let Some(block) = self.blocks.get_mut(block_id.0) {
            block.instructions.push(ins);
        }
    }

    pub fn add_edge(&mut self, from: BlockId, to: BlockId) {
        self.forward_edges[from.0].push(to);
        self.backward_edges[to.0].push(from);
    }

    pub fn successors(&self, id: BlockId) -> &[BlockId] {
        &self.forward_edges[id.0]
    }

    pub fn predecessors(&self, id: BlockId) -> &[BlockId] {
        &self.backward_edges[id.0]
    }
}