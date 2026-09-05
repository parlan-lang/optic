//! This module implements the Braun et. al. algorithm

use std::collections::{HashMap, HashSet};

use crate::module::instruction::*;
use crate::ssa::*;

pub struct SsaBuilder {
    curr_def: Vec<HashMap<Vreg, SsaValue>>,
    phi_nodes: HashMap<SsaValue, PhiNode>,
    phi_uses: HashMap<SsaValue, Vec<SsaValue>>,
    aliases: HashMap<SsaValue, SsaValue>,
    incomplete_phis: Vec<HashMap<Vreg, SsaValue>>,
    sealed_blocks: Vec<bool>,
    next_value_id: usize,
}

impl SsaBuilder {
    pub fn new(num_blocks: usize) -> Self {
        SsaBuilder {
            curr_def: vec![HashMap::new(); num_blocks],
            phi_nodes: HashMap::new(),
            phi_uses: HashMap::new(),
            aliases: HashMap::new(),
            incomplete_phis: vec![HashMap::new(); num_blocks],
            sealed_blocks: vec![false; num_blocks],
            next_value_id: 0,
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
        let val = if !self.sealed_blocks[block.0] {
            let phi_val = self.new_value();
            self.incomplete_phis[block.0].insert(vreg, phi_val);
            phi_val
        } else {
            let preds = &backward_edges[block.0];
            if preds.is_empty() {
                self.new_value()
            } else if preds.len() == 1 {
                let pred = preds[0];
                self.read_variable(backward_edges, pred, vreg)
            } else {
                let phi_val = self.new_value();
                self.write_variable(block, vreg, phi_val);
                self.add_phi_operands(backward_edges, block, vreg, phi_val)
            }
        };

        self.write_variable(block, vreg, val);
        val
    }

    fn add_phi_operands(
        &mut self,
        backward_edges: &[Vec<BlockId>],
        block: BlockId,
        vreg: Vreg,
        phi_val: SsaValue,
    ) -> SsaValue {
        let preds = backward_edges[block.0].clone();
        let mut operands = Vec::with_capacity(preds.len());
        for pred in preds {
            let op = self.read_variable(backward_edges, pred, vreg);
            operands.push(op);
        }

        self.phi_nodes.insert(
            phi_val,
            PhiNode {
                block,
                vreg,
                operands: operands.clone(),
            },
        );

        for &op in &operands {
            let resolved_op = self.resolve(op);
            self.phi_uses.entry(resolved_op).or_default().push(phi_val);
        }

        self.try_remove_trivial_phi(phi_val)
    }

    pub fn seal_block(&mut self, backward_edges: &[Vec<BlockId>], block: BlockId) {
        self.sealed_blocks[block.0] = true;
        let incomplete = std::mem::take(&mut self.incomplete_phis[block.0]);
        for (vreg, phi_val) in incomplete {
            self.add_phi_operands(backward_edges, block, vreg, phi_val);
        }
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
        None => self.new_value(),
    };

    self.phi_nodes.remove(&phi);

    self.aliases.insert(phi, same_val);

    if let Some(users) = self.phi_uses.remove(&phi) {
        for user in users {
            if user == phi {
                continue;
            }

            if let Some(mut user_node) = self.phi_nodes.remove(&user) {
                let mut replaced = false;
                for op in &mut user_node.operands {
                    if self.resolve(*op) == phi {
                        *op = same_val;
                        replaced = true;
                    }
                }

                self.phi_nodes.insert(user, user_node);

                if replaced {
                    let target = self.resolve(same_val);
                    self.phi_uses.entry(target).or_default().push(user);
                }
            }
            self.try_remove_trivial_phi(user);
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

pub fn build_ssa(cfg: &mut ControlFlowGraph, param_types: &HashMap<Vreg, Type>) {
    let mut builder = SsaBuilder::new(cfg.blocks.len());
    let mut var_types: HashMap<Vreg, Type> = param_types.clone();

    let rpo_order = cfg.compute_rpo();
    let mut processed_blocks = HashSet::new();

    if let Some(&max_param_id) = param_types.keys().map(|v| &v.0).max() {
        builder.next_value_id = max_param_id + 1;
    }

    if let Some(&entry_block) = rpo_order.first() {
        for &vreg in param_types.keys() {
            builder.write_variable(entry_block, vreg, SsaValue(vreg.0));
        }
    }

    for &block_id in &rpo_order {
        let all_preds_processed = cfg.backward_edges[block_id.0]
            .iter()
            .all(|pred| processed_blocks.contains(pred));

        if all_preds_processed {
            builder.seal_block(&cfg.backward_edges, block_id);
        }

        let block = &mut cfg.blocks[block_id.0];

        for ins in &mut block.instructions {
            match ins {
                Instruction::Ret { val, .. } => {
                    if let Some(vreg_id) = val.as_vreg() {
                        let ssa_id = builder.read_variable(&cfg.backward_edges, block_id, Vreg(vreg_id));
                        *val = Value::Vreg(ssa_id.0);
                    }
                }
                Instruction::Copy { vreg, val, ty } => {
                    if let Some(vreg_id) = val.as_vreg() {
                        let ssa_id = builder.read_variable(&cfg.backward_edges, block_id, Vreg(vreg_id));
                        *val = Value::Vreg(ssa_id.0);
                    }
                    let ssa_id = builder.new_value();
                    var_types.insert(Vreg(*vreg), *ty);
                    builder.write_variable(block_id, Vreg(*vreg), ssa_id);
                    *vreg = ssa_id.0;
                }
                Instruction::Op { vreg, lhs, rhs, ty, .. } => {
                    if let Some(vreg_id) = lhs.as_vreg() {
                        let ssa_id = builder.read_variable(&cfg.backward_edges, block_id, Vreg(vreg_id));
                        *lhs = Value::Vreg(ssa_id.0);
                    }
                    if let Some(vreg_id) = rhs.as_vreg() {
                        let ssa_id = builder.read_variable(&cfg.backward_edges, block_id, Vreg(vreg_id));
                        *rhs = Value::Vreg(ssa_id.0);
                    }
                    let ssa_id = builder.new_value();
                    var_types.insert(Vreg(*vreg), *ty);
                    builder.write_variable(block_id, Vreg(*vreg), ssa_id);
                    *vreg = ssa_id.0;
                }
                Instruction::Call { vreg, args, ty, .. } => {
                    for arg in args {
                        if let Some(vreg_id) = arg.as_vreg() {
                            let ssa_id = builder.read_variable(&cfg.backward_edges, block_id, Vreg(vreg_id));
                            *arg = Value::Vreg(ssa_id.0);
                        }
                    }
                    let ssa_id = builder.new_value();
                    var_types.insert(Vreg(*vreg), *ty);
                    builder.write_variable(block_id, Vreg(*vreg), ssa_id);
                    *vreg = ssa_id.0;
                }
                Instruction::Br { cond, .. } => {
                    if let Some(vreg_id) = cond.as_vreg() {
                        let ssa_id = builder.read_variable(&cfg.backward_edges, block_id, Vreg(vreg_id));
                        *cond = Value::Vreg(ssa_id.0);
                    }
                }
                Instruction::Label(_) | Instruction::Jmp(_) | Instruction::Phi { .. } => {}
            }
        }

        processed_blocks.insert(block_id);
    }

    for &block_id in &rpo_order {
        if !builder.sealed_blocks[block_id.0] {
            builder.seal_block(&cfg.backward_edges, block_id);
        }
    }

    let mut block_phis: HashMap<BlockId, Vec<Instruction>> = HashMap::new();

    for (&phi_val, phi_node) in &builder.phi_nodes {
        if builder.resolve(phi_val) == phi_val {
            let ty = var_types.get(&phi_node.vreg).cloned().unwrap_or(Type::I32);

            let srcs = phi_node
                .operands
                .iter()
                .map(|&op| Value::Vreg(builder.resolve(op).0))
                .collect();

            let phi_ins = Instruction::Phi {
                vreg: phi_val.0,
                srcs,
                ty,
            };

            block_phis.entry(phi_node.block).or_default().push(phi_ins);
        }
    }

    for block in &mut cfg.blocks {
        if let Some(mut phis) = block_phis.remove(&block.id) {
            phis.append(&mut block.instructions);
            block.instructions = phis;
        }
    }

    for block in &mut cfg.blocks {
        for ins in &mut block.instructions {
            let resolve_val = |v: &mut Value| {
                if let Some(vreg_id) = v.as_vreg() {
                    let resolved = builder.resolve(SsaValue(vreg_id));
                    *v = Value::Vreg(resolved.0);
                }
            };

            match ins {
                Instruction::Ret { val, .. } => resolve_val(val),
                Instruction::Copy { val, .. } => resolve_val(val),
                Instruction::Op { lhs, rhs, .. } => {
                    resolve_val(lhs);
                    resolve_val(rhs);
                }
                Instruction::Call { args, .. } => {
                    for arg in args {
                        resolve_val(arg);
                    }
                }
                Instruction::Br { cond, .. } => resolve_val(cond),
                Instruction::Phi { srcs, .. } => {
                    for src in srcs {
                        resolve_val(src);
                    }
                }
                Instruction::Label(_) | Instruction::Jmp(_) => {}
            }
        }
    }
}