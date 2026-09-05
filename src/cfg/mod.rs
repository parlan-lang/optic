//! This module contains the definitions of the CFG (Control Flow Graph) of Optic
//! 
//! having a CFG makes easier some optimizations and analisys

#![allow(unused)]

pub mod builder;

use crate::module::instruction::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    pub fn get_block(&self, id: BlockId) -> &BasicBlock {
        &self.blocks[id.0]
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

    pub fn compute_rpo(&self) -> Vec<BlockId> {
        let mut visited = vec![false; self.blocks.len()];
        let mut post_order = Vec::with_capacity(self.blocks.len());

        let entry_block = BlockId(0);
        self.dfs_post_order(entry_block, &mut visited, &mut post_order);

        post_order.reverse();
        post_order
    }

    fn get_block_label(&self, id: usize) -> Option<String> {
        self.blocks[id].instructions.iter().find_map(|ins| match ins {
            Instruction::Label(lbl) => Some(lbl.clone()),
            _ => None
        })
    }

    fn dfs_post_order(
        &self,
        curr: BlockId,
        visited: &mut [bool],
        post_order: &mut Vec<BlockId>,
    ) {
        visited[curr.0] = true;

        for &succ in &self.forward_edges[curr.0] {
            if !visited[succ.0] {
                self.dfs_post_order(succ, visited, post_order);
            }
        }

        post_order.push(curr);
    }

    pub fn split_critical_edges(&mut self) {
        let mut critical_edges = Vec::new();

        for src in &self.blocks {
            let succs = self.successors(src.id);
            if succs.len() > 1 {
                for dst in succs {
                    let preds = self.predecessors(*dst);
                    if preds.len() > 1 {
                        critical_edges.push((src.id, *dst));
                    }
                }
            }
        }

        critical_edges.sort_unstable();
        critical_edges.dedup();

        for (src, dst) in critical_edges {
            // 1. Get the label string of dst
            let dst_label = self
                .get_block_label(dst.0)
                .unwrap_or_else(|| panic!("internal error: block with id {} is missing an Instruction::Label", dst.0));

            // 2. Create new block with its own label and a Jmp to dst
            let blk = self.create_block();
            let blk_label = format!("critical_blk_{}", blk.0);

            self.blocks[blk.0].instructions.push(Instruction::Label(blk_label.clone()));
            self.blocks[blk.0].instructions.push(Instruction::Jmp(dst_label.clone()));

            // Update CFG edges
            self.forward_edges[blk.0] = vec![dst];
            self.backward_edges[blk.0] = vec![src];

            if let Some(target) = self.forward_edges[src.0].iter_mut().find(|id| **id == dst) {
                *target = blk;
            }

            if let Some(target) = self.backward_edges[dst.0].iter_mut().find(|id| **id == src) {
                *target = blk;
            }

            // 3. Rewrite src's branch target from dst_label to blk_label
            for ins in &mut self.blocks[src.0].instructions {
                match ins {
                    Instruction::Jmp(label) if *label == dst_label => {
                        *label = blk_label.to_string();
                    }
                    Instruction::Br { true_br, false_br, .. } => {
                        if *true_br == dst_label {
                            *true_br = blk_label.to_string();
                        }
                        if *false_br == dst_label {
                            *false_br = blk_label.to_string();
                        }
                    }
                    _ => continue
                } 
            }
        }
    }

    pub fn lower_phis_to_moves(&mut self, next_vreg: &mut usize) {
        let block_ids: Vec<usize> = self.blocks.iter().map(|b| b.id.0).collect();

        for dst_id in block_ids {
            let phis: Vec<(usize, Vec<Value>, Type)> = self.blocks[dst_id]
                .instructions
                .iter()
                .filter_map(|ins| match ins {
                    Instruction::Phi { vreg, srcs, ty } => Some((*vreg, srcs.clone(), *ty)),
                    _ => None,
                })
                .collect();

            if phis.is_empty() {
                continue;
            }

            let preds = self.backward_edges[dst_id].clone();

            for (pred_idx, &pred) in preds.iter().enumerate() {
                let mut parallel_set = Vec::new();

                for (dst_vreg, srcs, ty) in &phis {
                    parallel_set.push(Move {
                        dst: *dst_vreg,
                        src: srcs[pred_idx].clone(),
                        ty: *ty,
                    });
                }

                let copies = sequentialize_parallel_moves(parallel_set, next_vreg);

                // Insert before the terminator instruction (e.g. Branch/Jump)
                let instrs = &mut self.blocks[pred.0].instructions;
                let insert_pos = instrs
                    .iter()
                    .rposition(|ins| ins.is_terminator())
                    .unwrap_or(instrs.len());

                instrs.splice(insert_pos..insert_pos, copies);
            }

            // Remove Phi instructions from dst block (FIXED)
            self.blocks[dst_id]
                .instructions
                .retain(|ins| !matches!(ins, Instruction::Phi { .. }));
        }
    }

    pub fn get_max_vreg(&self) -> usize {
        let mut max = 0;
    
        for block in &self.blocks {
            for ins in &block.instructions {
                match ins {
                    Instruction::Phi { vreg, srcs, .. } => {
                        max = max.max(*vreg);
                        for src in srcs {
                            if let Value::Vreg(id) = src { max = max.max(*id) }
                        }
                    }
                    Instruction::Copy { vreg, val, .. } => {
                        max = max.max(*vreg);
                        if let Value::Vreg(id) = val { max = max.max(*id) }
                    }
                    Instruction::Op { vreg, lhs, rhs, .. } => {
                        max = max.max(*vreg);
                        if let Value::Vreg(id) = lhs { max = max.max(*id) }
                        if let Value::Vreg(id) = rhs { max = max.max(*id) }
                    }
                    Instruction::Call { vreg, args, .. } => {
                        max = max.max(*vreg);
                        for arg in args {
                            if let Value::Vreg(id) = arg { max = max.max(*id) }
                        }
                    }
                    _ => continue
                }
            }
        }
    
        max
    }
}

/// Helper struct that represents a move, like a `copy` instruction
#[derive(PartialEq)]
struct Move { 
    dst: usize, 
    src: Value, 
    ty: Type 
}

fn sequentialize_parallel_moves(moves: Vec<Move>, next_vreg: &mut usize) -> Vec<Instruction> {
    let mut pending: Vec<Move> = moves
        .into_iter()
        .filter(|m| match &m.src {
            Value::Vreg(s) => m.dst != *s,
            _ => true,
        })
        .collect();

    let mut instructions = Vec::new();

    while !pending.is_empty() {
        // Find a move whose destination is not read by any other pending move
        let ready_idx = pending.iter().enumerate().position(|(m_idx, m)| {
            !pending.iter().enumerate().any(|(other_idx, other)| {
                m_idx != other_idx && matches!(&other.src, Value::Vreg(s) if *s == m.dst)
            })
        });

        if let Some(idx) = ready_idx {
            let m = pending.remove(idx);
            instructions.push(Instruction::Copy {
                vreg: m.dst,
                val: m.src,
                ty: m.ty,
            });
        } else {
            // Break cycle using a temp vreg
            let cycle_move = &pending[0];
            let temp_vreg = *next_vreg;
            *next_vreg += 1;

            instructions.push(Instruction::Copy {
                vreg: temp_vreg,
                val: Value::Vreg(cycle_move.dst),
                ty: cycle_move.ty,
            });

            let old_dst = cycle_move.dst;
            for m in pending.iter_mut() {
                if matches!(&m.src, Value::Vreg(s) if *s == old_dst) {
                    m.src = Value::Vreg(temp_vreg);
                }
            }
        }
    }

    instructions
}