//! This module implements the C backend for Optic

#![allow(unused)]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::os::windows;

use crate::module::{
    instruction::*,
    function::*,
    *
};

pub struct CBackend<'a> {
    out:    File,
    module: &'a Module 
}

impl<'a> CBackend<'a> {
    pub fn new(out: &str, module: &'a Module) -> Self {
        CBackend {
            out:  File::create(out).expect("error: could not open or create the output file"),
            module
        }
    }

    fn compile_type(&self, ty: &Type) -> &str {
        match ty {
            Type::I32 => "int32_t"
        }
    }

    fn compile_value(&self, val: &Value) -> String {
        match val {
            Value::IntLit(i) => format!("{}", i),
            Value::Vreg(v) => format!("vreg_{}", v),
        }
    }

    fn compile_inst(&self, inst: &Instruction, header: &mut BufWriter<Vec<u8>>, body: &mut BufWriter<Vec<u8>>) {
        match inst {
            Instruction::Ret { val, ty } => {
                writeln!(body, "  return ({}){};", self.compile_type(ty), self.compile_value(val));
            },
            Instruction::Copy { vreg, val, ty } => {
                writeln!(header, "{} vreg_{};", self.compile_type(ty), *vreg);
                writeln!(body, "  vreg_{} = ({}){};", *vreg, self.compile_type(ty), self.compile_value(val));
            }
            _ => todo!()
        }
    }

    fn compile_func(&self, func: &Function, header: &mut BufWriter<Vec<u8>>, body: &mut BufWriter<Vec<u8>>) {
        let params = func.params.iter().map(|p| {
            format!("{} vreg_{}", self.compile_type(&p.ty), p.vreg)
        }).collect::<Vec<String>>().join(", ");

        writeln!(
            body, "\n{} {}({}) {{", 
            self.compile_type(&func.ty), 
            func.name, 
            params
        );

        for blk in &func.cfg.blocks {
            writeln!(body, "BB_{}:", blk.id.0);

            for inst in &blk.instructions {
                self.compile_inst(inst, header, body);
            }
        }

        writeln!(body, "}}");
    }

    pub fn compile(&mut self) {
        let mut header = BufWriter::new(Vec::new());
        let mut body = BufWriter::new(Vec::new());
        
        writeln!(header, 
            r#"// Module "{}"
#include <stdint.h>"#, 
            self.module.name
        );
        
        for func in &self.module.functions {
            self.compile_func(func, &mut header, &mut body);
        }

        self.out.write(header.buffer()).unwrap();
        self.out.write(body.buffer()).unwrap();
    }
}