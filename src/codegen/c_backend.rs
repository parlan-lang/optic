//! This module implements the C backend for Optic

#![allow(unused)]

use std::fs::File;
use std::io::{BufRead, BufWriter, Write};
use std::collections::HashMap;

use crate::module::{
    instruction::*,
    function::*,
    *
};
use crate::cfg::*;

pub struct CBackend<'a> {
    out:     File,
    module:  &'a Module,
}

impl<'a> CBackend<'a> {
    pub fn new(out: &str, module: &'a Module) -> Self {
        CBackend {
            out:  File::create(out).expect("error: could not open or create the output file"),
            module,
        }
    }

    fn compile_type(&self, ty: &Type) -> &str {
        match ty {
            Type::I32 => "uint32_t",
            Type::I1 => "uint8_t",
            Type::F32 => "float", // `float` is almost always of 4 bytes (32 bits)
            Type::Ptr => "void*",
            Type::Str => "char*",
        }
    }

    fn compile_value(&self, val: &Value) -> String {
        match val {
            Value::IntLit(i) => format!("{}", i),
            Value::FloatLit(f) => format!("{}", f),
            Value::Vreg(v) => format!("vreg_{}", *v),
            Value::GlobSym(s) => format!("glob_{}", s),
        }
    }

    fn compile_inst(
        &self, 
        inst: &Instruction, 
        global: &mut BufWriter<Vec<u8>>, 
        header: &mut BufWriter<Vec<u8>>, 
        body: &mut BufWriter<Vec<u8>>,
    ) {
        match inst {
            Instruction::Ret { val, ty } => {
                writeln!(body, "  return ({}){};", self.compile_type(ty), self.compile_value(val));
            },
            Instruction::Copy { vreg, val, ty } => {
                let def = format!("  {} vreg_{};", self.compile_type(ty), *vreg);
                if !header.buffer().lines().any(|l| l.unwrap() == def) {
                    writeln!(header, "{}", def);
                }
                writeln!(body, "  vreg_{} = ({}){};", *vreg, self.compile_type(ty), self.compile_value(val));
            }
            Instruction::Op { vreg, kind, lhs, rhs, ty, .. } => {
                let op = match kind {
                    OpKind::Add => "+",
                    OpKind::Sub => "-",
                    OpKind::Mul => "*",
                    OpKind::Div | OpKind::Udiv => "/",
                    OpKind::CmpEq => "==", OpKind::CmpNe => "!=",
                    OpKind::CmpSlt | OpKind::CmpUlt => "<",
                    OpKind::CmpSgt | OpKind::CmpUgt => ">",
                };

                writeln!(header, "  {} vreg_{};", self.compile_type(ty), (*vreg));
                writeln!(body, "  vreg_{} = ({})({} {} {});", (*vreg), self.compile_type(ty), self.compile_value(lhs), op, self.compile_value(rhs));                
            }
            Instruction::Call { vreg, func, args, ty } => {
                let args = args.iter().map(|v| self.compile_value(v)).collect::<Vec<String>>().join(",");

                writeln!(header, "  {} vreg_{};", self.compile_type(ty), (*vreg));
                writeln!(body, "  vreg_{} = ({}){}({});", (*vreg), self.compile_type(ty), func, args);
            }
            Instruction::Label(label) => {
                writeln!(body, "L_{}:", label);
            }
            Instruction::Jmp(dest) => {
                writeln!(body, "  goto L_{};", dest);
            }
            Instruction::Br { cond, true_br, false_br } => {
                writeln!(body, "  if ({}) goto L_{}; else goto L_{};", self.compile_value(cond), true_br, false_br);
            }
            Instruction::Phi { vreg, srcs, ty } => {}
        }
    }

    fn compile_func(&self, func: &Function, global: &mut BufWriter<Vec<u8>>, body: &mut BufWriter<Vec<u8>>) {
        let params = func.params.iter().map(|p| {
            if p.is_vaarg { "...".to_string() }
            else { format!("{} vreg_{}", self.compile_type(&p.ty), p.vreg) }
        }).collect::<Vec<String>>().join(", ");

        if func.is_extern {
            writeln!(
                body, "\nextern {} {}({});",
                self.compile_type(&func.ty),
                func.name,
                params
            );
            return;
        }
        writeln!(
            body, "\n{} {}({}) {{", 
            self.compile_type(&func.ty), 
            func.name, 
            params
        );

        let mut header = &mut BufWriter::new(Vec::new());
        let mut local_body = &mut BufWriter::new(Vec::new());

        for blk in &func.cfg.blocks {
            writeln!(local_body, "// BB_{}:", blk.id.0);

            for inst in &blk.instructions {
                self.compile_inst(inst, global, header, local_body);
            }
        }

        body.write(header.buffer()).unwrap();
        body.write(local_body.buffer()).unwrap();
        writeln!(body, "}}");
    }

    fn compile_global(&mut self, data: &GlobData, global: &mut BufWriter<Vec<u8>>) {
        if data.is_constant {
            write!(global, "const ");
        }
        match &data.val {
            GlobValue::Int(n) => {
                writeln!(global, "{} glob_{} = {};", self.compile_type(&data.ty), data.name, n);
            }
            GlobValue::Str(s) => {
                writeln!(global, "{} glob_{} = \"{}\";", self.compile_type(&data.ty), data.name, s);
            }
        }
    }

    pub fn compile(&mut self) {
        let mut header = BufWriter::new(Vec::new());
        let mut body = BufWriter::new(Vec::new());
        
        writeln!(header, 
            r#"// Module "{}"
#include <stdint.h>"#, 
            self.module.name
        );
        
        for global in &self.module.globals {
            self.compile_global(global, &mut header);
        }

        for func in &self.module.functions {

            self.compile_func(func, &mut header, &mut body);
        }

        self.out.write(header.buffer()).unwrap();
        self.out.write(body.buffer()).unwrap();
    }
}