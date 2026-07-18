//! This module implements a CFG builder, which takes a [`Module`](`crate::module::Module`) and generates
//! the [`CFG`](`crate::cfg::ControlFlowGraph`) of every function

use std::collections::HashMap;
use std::vec;

use crate::cfg::*;
use crate::module::function::*;

pub struct CfgBuilder<'a> {
    func: &'a mut Function,
    curr_block: Option<BlockId>
}

impl<'a> CfgBuilder<'a> {
    pub fn new(func: &'a mut Function) -> Self {
        CfgBuilder { 
            func,
            curr_block: None
        }
    }

    pub fn build_cfg(&mut self) {
        // take the instructions out of the body
        let instructions = std::mem::take(&mut self.func.body);

        let mut label_map: HashMap<String, BlockId> = HashMap::new();

        let mut curr_block_ins = Vec::new();
        let mut next_block_id = 0;

        let mut finish_curr_block = |ins: &mut Vec<Instruction>, cfg: &mut ControlFlowGraph, next_block_id: &mut usize| {
            if ins.is_empty() { return None };

            let id = BlockId(*next_block_id);
            *next_block_id += 1;

            let block = BasicBlock {
                id, 
                instructions: std::mem::take(ins)
            };

            cfg.blocks.push(block);
            cfg.forward_edges.push(Vec::new());
            cfg.backward_edges.push((Vec::new()));

            Some(id)
        };

        for ins in instructions {
            if ins.is_label() {
                if let Some(id) = finish_curr_block(&mut curr_block_ins, &mut self.func.cfg, &mut next_block_id) {
                    self.curr_block = Some(id);
                }

                let label_name = ins.get_label_name().unwrap().clone();
                let incoming_block_id = BlockId(next_block_id);
                label_map.insert(label_name, incoming_block_id);
            }

            curr_block_ins.push(ins);

            if curr_block_ins.last().map_or(false, |i| i.is_terminator()) {
                if let Some(id) = finish_curr_block(&mut curr_block_ins, &mut self.func.cfg, &mut next_block_id) {
                    self.curr_block = Some(id);
                }
            }
        }

        finish_curr_block(&mut curr_block_ins, &mut self.func.cfg, &mut next_block_id);

        self.populate_edges(&label_map);
    }

    fn populate_edges(&mut self, label_map: &HashMap<String, BlockId>) {
        let cfg = &mut self.func.cfg;
        let num_blocks = cfg.blocks.len();

        for i in 0..num_blocks {
            let block_id = cfg.blocks[i].id;

            if let Some(terminator) = cfg.blocks[i].instructions.last() {
                match terminator {
                    Instruction::Jmp(dest) => {
                        if let Some(&dest_id) = label_map.get(dest) {
                            cfg.add_edge(block_id, dest_id);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}