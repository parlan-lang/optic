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
            Value::IntLit(i) => format!("{}", i)
        }
    }

    fn compile_inst(&self, inst: &Instruction) {
        let mut writer = BufWriter::new(&self.out);

        match inst {
            Instruction::Ret { val, ty } => {
                writeln!(writer, "  return ({}){};", self.compile_type(ty), self.compile_value(val));
            }
        }

        writer.flush().unwrap();
    }

    fn compile_func(&self, func: &Function) {
        let mut writer = BufWriter::new(&self.out);
        
        let params = func.params.iter().map(|p| {
            format!("{} vreg_{}", self.compile_type(&p.ty), p.vreg)
        }).collect::<Vec<String>>().join(", ");

        writeln!(
            writer, "\n{} {}({}) {{", 
            self.compile_type(&func.ty), 
            func.name, 
            params
        );
        writer.flush().unwrap();

        for blk in &func.cfg.blocks {
            writeln!(writer, "BB_{}:", blk.id.0);
            writer.flush().unwrap();

            for inst in &blk.instructions {
                self.compile_inst(inst);
            }
        }

        writeln!(writer, "}}");
        
        writer.flush().unwrap();
    }

    pub fn compile(&self) {
        let mut writer = BufWriter::new(&self.out);
        
        writeln!(writer, 
            r#"// Module "{}"
#include <stdint.h>"#, 
            self.module.name
        );
        writer.flush().unwrap();
        
        for func in &self.module.functions {
            self.compile_func(func);
        }

        writer.flush().unwrap();
    }
}