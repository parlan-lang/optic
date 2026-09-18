//! This module implements the C backend for Optic

use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};

use crate::module::{
    instruction::*,
    function::*,
    *
};

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
            Type::Ascii => "char",
            Type::Void => "void",
        }
    }

    fn compile_value(&self, val: &Value) -> String {
        match val {
            Value::IntLit(i) => format!("{}", i),
            Value::FloatLit(f) => format!("{}", f),
            Value::Vreg(v) => format!("vreg_{}", *v),
            Value::GlobSym(s) => format!("glob_{}", s),
            Value::Void => format!("/* VOID */"),
        }
    }

    fn compile_inst(
        &self, 
        inst: &Instruction, 
        header: &mut BufWriter<Vec<u8>>, 
        body: &mut BufWriter<Vec<u8>>,
    ) -> io::Result<()> {
        match inst {
            Instruction::Ret { val, ty } => {
                let ty = match ty {
                    Type::Void => "".to_string(),
                    _ => format!("({})", self.compile_type(ty))
                };
                writeln!(body, "  return {}{};", ty, self.compile_value(val))?;
            },
            Instruction::Copy { vreg, val, ty } => {
                let def = format!("  {} vreg_{};", self.compile_type(ty), *vreg);
                if !header.buffer().lines().any(|l| l.unwrap() == def) {
                    writeln!(header, "{}", def)?;
                }
                writeln!(body, "  vreg_{} = ({}){};", *vreg, self.compile_type(ty), self.compile_value(val))?;
            }
            Instruction::Op { vreg, kind, lhs, rhs, ty } => {
                let (op, res_ty) = match kind {
                    OpKind::Add => ("+", *ty),
                    OpKind::Sub => ("-", *ty),
                    OpKind::Mul => ("*", *ty),
                    OpKind::Div | OpKind::Udiv => ("/", *ty),
                    OpKind::CmpEq => ("==", Type::I1), 
                    OpKind::CmpNe => ("!=", Type::I1),
                    OpKind::CmpSlt | OpKind::CmpUlt => ("<", Type::I1),
                    OpKind::CmpSgt | OpKind::CmpUgt => (">", Type::I1),
                };

                writeln!(header, "  {} vreg_{};", self.compile_type(ty), *vreg)?;
                writeln!(body, "  vreg_{} = ({})(({}){} {} ({}){});", *vreg, self.compile_type(&res_ty), self.compile_type(ty), self.compile_value(lhs), op, self.compile_type(ty), self.compile_value(rhs))?;                
            }
            Instruction::Call { vreg, func, args, ty, discard_value } => {
                let args = args.iter().map(|a| format!("({}){}", self.compile_type(&a.0), self.compile_value(&a.1))).collect::<Vec<String>>().join(", ");

                if *discard_value {
                    writeln!(body, "  {}({});", func, args)?;
                } else {
                    writeln!(header, "  {} vreg_{};", self.compile_type(ty), (*vreg))?;
                    writeln!(body, "  vreg_{} = ({}){}({});", (*vreg), self.compile_type(ty), func, args)?;
                }
            }
            Instruction::Label(label) => {
                writeln!(body, "L_{}:", label)?;
            }
            Instruction::Jmp(dest) => {
                writeln!(body, "  goto L_{};", dest)?;
            }
            Instruction::Br { cond, true_br, false_br } => {
                writeln!(body, "  if ({}) goto L_{}; else goto L_{};", self.compile_value(cond), true_br, false_br)?;
            }
            Instruction::Alloc { vreg, num, ty } => {
                writeln!(header, "  void* vreg_{};", *vreg)?;
                writeln!(body, "  vreg_{} = ({}[{}]){{0}};", *vreg, self.compile_type(ty), *num)?;
            }
            Instruction::Store { ptr, val, ty } => {
                writeln!(body, "  *( ({} *){} ) = {};", self.compile_type(ty), self.compile_value(ptr), self.compile_value(val))?;
            }
            Instruction::Load { vreg, ptr, ty } => {
                writeln!(header, "  {} vreg_{};", self.compile_type(ty), *vreg)?;
                writeln!(body, "  vreg_{} = *( ({} *){} );", *vreg, self.compile_type(ty), self.compile_value(ptr))?;
            }
            Instruction::Offset { vreg, ptr, idx, ty } => {
                writeln!(header, "  void* vreg_{};", *vreg)?;
                writeln!(body, "  vreg_{} = &(({} *){})[{}];", *vreg, self.compile_type(ty), self.compile_value(ptr), *idx)?;
            }
            Instruction::Phi { .. } => {}
        }

        Ok(())
    }

    fn compile_func(&self, func: &Function, body: &mut BufWriter<Vec<u8>>) -> io::Result<()> {
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
            )?;
            return Ok(());
        }
        writeln!(
            body, "\n{} {}({}) {{", 
            self.compile_type(&func.ty), 
            func.name, 
            params
        )?;

        let mut header = BufWriter::new(Vec::new());
        let mut local_body = BufWriter::new(Vec::new());

        for blk in &func.cfg.blocks {
            writeln!(local_body, "// BB_{}:", blk.id.0)?;

            for inst in &blk.instructions {
                self.compile_inst(inst, &mut header, &mut local_body)?;
            }
        }

        body.write(header.buffer()).unwrap();
        body.write(local_body.buffer()).unwrap();
        writeln!(body, "}}")?;
        
        Ok(())
    }

    fn compile_global(&mut self, data: &GlobData, global: &mut BufWriter<Vec<u8>>) -> io::Result<()> {
        if data.is_constant {
            write!(global, "static const uint8_t ")?;
        } else {
            write!(global, "static uint8_t ")?;
        }

        writeln!(
            global, 
            "glob_{}[] = {{ {} }};", 
            data.name, 
            data.val
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<_>>()
                .join(", ")
        )?;

        Ok(())
    }

    pub fn compile(&mut self) {
        let mut header = BufWriter::new(Vec::new());
        let mut body = BufWriter::new(Vec::new());
        
        writeln!(header, 
            r#"// Module "{}"
#include <stdint.h>"#, 
            self.module.name
        ).unwrap();
        
        for global in &self.module.globals {
            self.compile_global(global, &mut header).unwrap();
        }

        for func in &self.module.functions {
            self.compile_func(func,&mut body).unwrap();
        }

        self.out.write(header.buffer()).unwrap();
        self.out.write(body.buffer()).unwrap();
    }
}