//! This module implements a CFG builder, which takes a [`Module`](`crate::module::Module`) and generates
//! the [`CFG`](`crate::cfg::ControlFlowGraph`) of every function

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
        let instructions = &mut self.func.body;

        let entry_block = self.func.cfg.create_block();
        self.curr_block = Some(entry_block);

        for ins in instructions {
            if self.curr_block.is_none() {
                let new_block = self.func.cfg.create_block();
                self.curr_block = Some(new_block);
            }

            let curr_id = self.curr_block.unwrap();

            match &ins {
                _ => {
                    self.func.cfg.add_ins(curr_id, ins.clone());
                }
            }
        }
    }
}