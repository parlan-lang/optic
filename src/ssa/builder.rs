//! This module implements the Braun et. al. algorithm

#![allow(unused)]

use std::collections::HashMap;

use crate::ssa::*;
use crate::module::{
    instruction:: *,
};

pub struct SsaBuilder {
    curr_def: Vec<HashMap<Vreg, SsaValue>>,
    pub phi_nodes: HashMap<SsaValue, PhiNode>,
    phi_uses: HashMap<SsaValue, Vec<SsaValue>>,
    aliases: HashMap<SsaValue, SsaValue>,
    next_value_id: usize
}

impl SsaBuilder {
    pub fn new(num_blocks: usize) -> Self {
        SsaBuilder {
            curr_def: vec![HashMap::new(); num_blocks],
            phi_nodes: HashMap::new(),
            phi_uses: HashMap::new(),
            aliases: HashMap::new(),
            next_value_id: 0
        }
    }

    pub fn new_value(&mut self) -> SsaValue {
        let id = self.next_value_id;
        self.next_value_id += 1;
        SsaValue(id)
    }    

    pub fn write_variable(&mut self, block: BlockId, vreg: Vreg, val: SsaValue) {
        self.curr_def[block.0].insert(vreg, val);
    }

    pub fn read_variable(&mut self, backward_edges: &[Vec<BlockId>], block: BlockId, vreg: Vreg) -> SsaValue {
        if let Some(&val) = self.curr_def[block.0].get(&vreg) {
            return self.resolve(val);
        }
        self.read_variable_recursive(backward_edges, block, vreg)
    }

    fn read_variable_recursive(&mut self, backward_edges: &[Vec<BlockId>], block: BlockId, vreg: Vreg) -> SsaValue {
        let preds = &backward_edges[block.0];

        if preds.is_empty() {
            let val = self.new_value();
            self.write_variable(block, vreg, val);
            return val;
        }

        if preds.len() == 1 {
            let pred = preds[0];
            let val = self.read_variable(backward_edges, pred, vreg);
            self.write_variable(block, vreg, val);
            return val;
        }

        let phi_val = self.new_value();
        self.write_variable(block, vreg, phi_val);

        let mut operands = Vec::with_capacity(preds.len());
        for i in 0..preds.len() {
            let pred = backward_edges[block.0][i];
            let val = self.read_variable(backward_edges, pred, vreg);
            operands.push(val);
        }

        self.phi_nodes.insert(phi_val, PhiNode { block, vreg, operands: operands.clone() });

        for &op in &operands {
            self.phi_uses.entry(op).or_default().push(phi_val);
        }

        self.try_remove_trivial_phi(phi_val)
    }

    fn try_remove_trivial_phi(&mut self, phi: SsaValue) -> SsaValue {
        let phi_node = match self.phi_nodes.get(&phi) {
            Some(node) => node,
            None => return self.resolve(phi),
        };

        let mut same = None;
        for &op in &phi_node.operands {
            let resolved_op = self.resolve(op);
            if resolved_op == phi || Some(resolved_op) == same {
                continue;
            }
            if same.is_some() {
                return phi;
            }
            same = Some(resolved_op);
        }

        let same_val = match same {
            Some(v) => v,
            None => return phi,
        };

        self.aliases.insert(phi, same_val);
        self.phi_nodes.remove(&phi);

        if let Some(users) = self.phi_uses.remove(&phi) {
            for user in users {
                if let Some(user_node) = self.phi_nodes.get_mut(&user) {
                    let mut replaced = false;
                    for op in &mut user_node.operands {
                        if *op == phi {
                            *op = same_val;
                            replaced = true;
                        }
                    }
                    if replaced {
                        self.phi_uses.entry(same_val).or_default().push(user);
                    }
                    self.try_remove_trivial_phi(user);
                }
            }
        }

        same_val
    }

    pub fn resolve(&self, mut val: SsaValue) -> SsaValue {
        while let Some(&alias) = self.aliases.get(&val) {
            val = alias;
        }
        val
    }
}

pub fn build_ssa(cfg: &mut ControlFlowGraph) {
    let mut builder = SsaBuilder::new(cfg.blocks.len());

    let mut var_types: HashMap<usize, Type> = HashMap::new();

    let backward_edges = &cfg.backward_edges;
    let blocks = &mut cfg.blocks;

    for block in blocks.iter_mut() {
        let block_id = block.id;

        for ins in &mut block.instructions {
            match ins {
                Instruction::Ret { val, .. } => {
                    if let Some(vreg_id) = val.as_vreg(){
                        let ssa_id = builder.read_variable(backward_edges,block_id,Vreg(vreg_id));
                        *val = Value::Vreg(ssa_id.0);
                    };
                }
                Instruction::Copy { vreg, val, ty } => {
                    if let Some(vreg_id) = val.as_vreg(){
                        let ssa_id = builder.read_variable(backward_edges,block_id,Vreg(vreg_id));
                        *val = Value::Vreg(ssa_id.0);
                    };
                    let ssa_id = builder.new_value();
                    var_types.insert(*vreg, *ty);
                    builder.write_variable(block_id,Vreg(*vreg),ssa_id);
                    *vreg = ssa_id.0;
                }
                Instruction::Op { vreg, lhs, rhs, ty, .. } => {
                    if let Some(vreg_id) = lhs.as_vreg(){
                        let ssa_id = builder.read_variable(backward_edges,block_id,Vreg(vreg_id));
                        *lhs = Value::Vreg(ssa_id.0);
                    };
                    if let Some(vreg_id) = rhs.as_vreg(){
                        let ssa_id = builder.read_variable(backward_edges,block_id,Vreg(vreg_id));
                        *rhs = Value::Vreg(ssa_id.0);
                    };
                    let ssa_id = builder.new_value();
                    var_types.insert(*vreg, *ty);
                    builder.write_variable(block_id,Vreg(*vreg),ssa_id);
                    *vreg = ssa_id.0;
                }
                Instruction::Call { vreg, args, ty, .. } => {
                    for arg in args {
                        if let Some(vreg_id) = arg.as_vreg(){
                            let ssa_id = builder.read_variable(backward_edges,block_id,Vreg(vreg_id));
                            *arg = Value::Vreg(ssa_id.0);
                        };
                    }
                    let ssa_id = builder.new_value();
                    var_types.insert(*vreg, *ty);
                    builder.write_variable(block_id,Vreg(*vreg),ssa_id);
                    *vreg = ssa_id.0;
                }
                Instruction::Label(_) | Instruction::Jmp(_) | Instruction::Phi { .. } => {}
            }
        }
    }

    let mut block_phis: HashMap<BlockId, Vec<Instruction>> = HashMap::new();
    
    for (&phi_val, phi_node) in &builder.phi_nodes {
        if builder.resolve(phi_val) == phi_val {
            let ty = var_types.get(&phi_node.vreg.0).cloned().unwrap();

            let srcs = phi_node.operands.iter()
                .map(|&op| Value::Vreg(builder.resolve(op).0))
                .collect();

            let phi_ins = Instruction::Phi {
                vreg: phi_val.0,
                srcs,
                ty
            };

            block_phis.entry(phi_node.block).or_default().push(phi_ins);
        }
    }

    for block in blocks {
        if let Some(mut phis) = block_phis.remove(&block.id) {
            phis.append(&mut block.instructions);
            block.instructions = phis;
        }

        for ins in &mut block.instructions {
            let resolve_use = |val: &mut Value| {
                val.map_vreg(|id| builder.resolve(SsaValue(id)).0);
            };

            match ins {
                Instruction::Ret { val, .. } => resolve_use(val),
                Instruction::Copy { val, .. } => resolve_use(val),
                Instruction::Op { lhs, rhs, .. } => { resolve_use(lhs); resolve_use(rhs); }
                Instruction::Call { args, ..} => for arg in args { resolve_use(arg); },
                Instruction::Phi { srcs, .. } => for src in srcs { resolve_use(src); },
                Instruction::Label(_) | Instruction::Jmp(_) => {}
            }
        }
    }
}