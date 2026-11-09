//! This module contains the implemnentation of the 
//! parser (decoder) for the bytecode format of Optic.
//! Most functions try to briefly explain the
//! specific format it parses/decodes. 
//! 
//! This bytecode format is not stable (yet?), and can
//! change from version to version. (This is why it 
//! contains a version number)

use crate::module::function::{Function, Parameter};
use crate::module::instruction::{GlobData, Instruction, OpKind, Type, Value};
use crate::module::Module;
use crate::cfg::ControlFlowGraph;

#[derive(Debug)]
pub enum BytecodeError {
    /// The source bytes isn't valid Optic bytecode
    /// (The first 7 bytes are not 'OPTICBC')
    NotBytecodeSrc,

    /// The parsed string is not UTF8-valid 
    InvalidString (usize),

    /// The parsed type ([`Type`]) is invalid (not defined)
    InvalidType (usize),

    /// The parsed value ([`Value`]) is invalid
    InvalidValue (usize),

    /// The parsed value ([`Value`]) inside a `data` definition was invalid
    /// (it was `void` or `ptr`)
    InvalidDataValue (usize),

    /// The parsed instruction ([`Instruction`]) is invalid
    UnknownInstruction (usize),

    /// The parsed operation ([`OpKind`]) is invalid
    UnknownOp (usize),

    /// The parsed item (either [`Function`] or [`GlobData`]) is invalid
    UnknownItem (usize),

    /// The source bytecode is in a previous version of the format
    UnsupportedBytecodeVersion (u8)
}

impl std::fmt::Display for BytecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BytecodeError::NotBytecodeSrc => writeln!(f, "input was not Optic bytecode. (The first 7 bytes must be OPTICBC)"),
            BytecodeError::InvalidString(byte) => writeln!(f, "invalid string found at byte 0x{byte:X}"),
            BytecodeError::InvalidType(byte) => writeln!(f, "invalid type found at byte 0x{byte:X}"),
            BytecodeError::InvalidValue(byte) => writeln!(f, "invalid value found at byte 0x{byte:X}"),
            BytecodeError::InvalidDataValue(byte) => writeln!(f, "invalid data value found at byte 0x{byte:X}"),
            BytecodeError::UnknownInstruction(byte) => writeln!(f, "unknown instruction found at byte 0x{byte:X}"),
            BytecodeError::UnknownOp(byte) => writeln!(f, "unknown binary operation found at byte 0x{byte:X}"),
            BytecodeError::UnknownItem(byte) => writeln!(f, "unknow item found at byte 0x{byte:X}. (An item can be either a function or a global)"),
            BytecodeError::UnsupportedBytecodeVersion(ver) => writeln!(f, "unsupported bytecode version v{ver}, expected v1"),
        }
    }
}

pub struct BytecodeParser<'a> {
    bytes: &'a [u8], 
    cursor: usize
}

impl<'a> BytecodeParser<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            cursor: 0
        }
    }

    #[inline(always)]
    #[track_caller] // if it panics, we know where it was called
    fn read_bytes(&mut self, num: usize) -> &[u8] {
        self.cursor += num;
        &self.bytes[self.cursor - num..self.cursor]
    }

    #[inline(always)]
    #[track_caller] // if it panics, we know where it was called
    fn peek_bytes(&self, num: usize) -> &[u8] {
        &self.bytes[self.cursor..self.cursor + num]
    }

    /// A string is represented as a lenght followed by some bytes.
    /// The lenght is an `u32` (4 bytes), and determines how many bytes to read
    fn parse_str(&mut self) -> Result<String, BytecodeError> {
        let offset = self.cursor;
        let len = self.read_bytes(4).try_into().map(u32::from_le_bytes).unwrap();
        let src = self.read_bytes(len as usize);
        match String::from_utf8(src.to_vec()) {
            Ok(s) => Ok(s),
            Err(_) => Err(BytecodeError::InvalidString(offset - len as usize - 4))
        }
    }

    fn parse_type(&mut self) -> Result<Type, BytecodeError> {
        match self.read_bytes(1)[0] {
            1 => Ok(Type::I1),
            2 => Ok(Type::I32),
            3 => Ok(Type::F32),
            4 => Ok(Type::Ptr),
            5 => Ok(Type::Void),
            6 => Ok(Type::Ascii),
            _ => Err(BytecodeError::InvalidType(self.cursor - 1))
        }
    }
}

impl<'a> BytecodeParser<'a> {
    /// A module starts with 'OPTICBC' (79, 80, 84, 73, 67, 66, 67), 
    /// followed by the bytecode version, the name of the module, and 
    /// a sequence of functions and globals finished with 0xE
    pub fn parse_module(&'a mut self) -> Result<Module, BytecodeError> {
        let mut globals = Vec::new();
        let mut functions = Vec::new();

        if self.read_bytes(7) != b"OPTICBC" {
            return Err(BytecodeError::NotBytecodeSrc);
        }

        // IMPORTANT: Every time the bytecode format changes in a 
        // non backward-compatible way, this line must be updated
        if self.read_bytes(1).try_into().map(u8::from_le_bytes).unwrap() != 1 {
            return Err(BytecodeError::UnsupportedBytecodeVersion(self.bytes[self.cursor - 1].into()));
        }

        let name = self.parse_str()?;

        while self.read_bytes(1)[0] != 0xE {
            match self.peek_bytes(1)[0] {
                0xF => functions.push(self.parse_function()?),
                0xC => globals.push(self.parse_global()?),
                _ => return Err(BytecodeError::UnknownItem(self.cursor - 1))
            }
        }

        Ok(Module { 
            name, 
            globals, 
            functions 
        })
    }

    /// A global starts with 0xC, followed by a 1 byte boolean (to determine if it's 
    /// constant or not), its name, and a sequence of data values finished with 0xE.
    /// 
    /// Every data value starts with a type (1 byte), followed by some bytes containing
    /// the actual value (the type determines how many bytes to read).
    fn parse_global(&mut self) -> Result<GlobData, BytecodeError> {
        let is_constant = self.read_bytes(1)[0] == 1;
        let name = self.parse_str()?;

        let mut val = Vec::new();
        while self.peek_bytes(1)[0] != 0xE {
            let bytes = match self.parse_type()? {
                Type::I32 => self.read_bytes(4),
                Type::I1 => self.read_bytes(1),
                Type::F32 => self.read_bytes(4),
                Type::Ascii => &self.parse_str().unwrap().into_bytes(),
                Type::Void | Type::Ptr => return Err(BytecodeError::InvalidDataValue(self.cursor)),
            };
            bytes.iter().for_each(|b| val.push(*b));
        }

        Ok(GlobData { name, val, is_constant })
    }

    /// A function starts with 0xF, followed by a 1 byte boolean (to determine if it's
    /// extern or not), a sequence of parameters finished with 0xE, a type, and a 
    /// sequence of instructions also finished with 0xE.
    /// 
    /// A parameter only contains a type, but if its 0 then it's a vaarg.
    fn parse_function(&mut self) -> Result<Function, BytecodeError> {
        let name = self.parse_str()?;
        let is_extern = self.read_bytes(1)[0] == 1;

        let mut params = Vec::new();
        let mut vreg = 0;
        loop {
            match self.read_bytes(1)[0] {
                0xE => break,
                0x0 => {
                    params.push(Parameter { vreg, ty: Type::I1, is_vaarg: true });
                    vreg += 1;
                },
                _ => {
                    params.push(Parameter { vreg, ty: self.parse_type()?, is_vaarg: false });
                    vreg += 1;
                }
            }
        }

        let ty = self.parse_type()?;

        let mut body = Vec::new();
        loop {
            match self.peek_bytes(1)[0] {
                0xE => {
                    self.read_bytes(1);
                    break;
                }
                _ => body.push(self.parse_ins()?),
            }
        }

        Ok(Function { 
            name, 
            params, 
            ty, 
            body, 
            cfg: ControlFlowGraph::new(), 
            is_extern
        })
    }
} 

impl BytecodeParser<'_> {
    /// A value starts with a 1 byte discriminant followed by some bytes. 
    /// The discriminat determines how many bytes to read
    fn parse_value(&mut self) -> Result<Value, BytecodeError> {
        match self.read_bytes(1)[0] {
            0 => Ok(Value::IntLit(self.read_bytes(8).try_into().map(usize::from_le_bytes).unwrap())),
            1 => Ok(Value::FloatLit(self.read_bytes(8).try_into().map(f64::from_le_bytes).unwrap())),
            2 => Ok(Value::Vreg(self.parse_vreg())),
            3 => Ok(Value::GlobSym(self.parse_str()?)),
            4 => Ok(Value::Void),
            _ => Err(BytecodeError::InvalidValue(self.cursor - 1))   
        }
    }

    /// A vreg is just an `u32` (4 bytes). This means a function can have
    /// up to 4,294,967,295 virtual registers, more than enough.
    #[inline]
    fn parse_vreg(&mut self) -> usize {
        self.read_bytes(4).try_into().map(u32::from_le_bytes).unwrap() as usize
    }
    
    fn parse_ins_ret(&mut self) -> Result<Instruction, BytecodeError> {
        let ty = self.parse_type()?;
        let val = self.parse_value()?;
        
        Ok(Instruction::Ret { val, ty })   
    }

    fn parse_ins_copy(&mut self) -> Result<Instruction, BytecodeError> {
        let ty = self.parse_type()?;
        let vreg = self.parse_vreg();
        let val = self.parse_value()?;
        
        Ok(Instruction::Copy { vreg, val, ty })
    }

    fn parse_ins_op(&mut self) -> Result<Instruction, BytecodeError> {
        let ty = self.parse_type()?;
        let kind = match self.read_bytes(1)[0] {
            0 => OpKind::Add,
            1 => OpKind::Sub,
            2 => OpKind::Mul,
            3 => OpKind::Div,
            4 => OpKind::Udiv,
            5 => OpKind::Fadd,
            6 => OpKind::Fsub,
            7 => OpKind::Fmul,
            8 => OpKind::Fdiv,
            9 => OpKind::Neg,
            10 => OpKind::Fneg,
            11 => OpKind::Or,
            12 => OpKind::And,
            13 => OpKind::Xor,
            14 => OpKind::CmpEq,
            15 => OpKind::CmpNe,
            16 => OpKind::CmpSlt,
            17 => OpKind::CmpUlt,
            18 => OpKind::FCmpOEq,
            19 => OpKind::FCmpONe,
            20 => OpKind::FCmpOlt,
            21 => OpKind::FCmpUEq,
            22 => OpKind::FCmpUNe,
            23 => OpKind::FCmpUlt,
            24 => OpKind::FCmpOrd,
            25 => OpKind::FCmpUno,
            _ => return Err(BytecodeError::UnknownOp(self.cursor - 1))
        };
        let vreg = self.parse_vreg();

        let lhs = self.parse_value()?;
        let rhs = self.parse_value()?;

        Ok(Instruction::Op { vreg, kind, lhs, rhs, ty })
    }

    fn parse_ins_call(&mut self) -> Result<Instruction, BytecodeError> {
        let ty = self.parse_type()?;
        let vreg = self.parse_vreg();
        let func = self.parse_str()?;

        let mut args = Vec::new();
        while self.peek_bytes(1)[0] != 0xE {
            args.push((self.parse_type()?, self.parse_value()?));
        }
        // consume the 0xE (end) byte
        self.read_bytes(1);

        // Discard value is only available on 
        // the textual IR, but this may change 
        // in future versions
        Ok(Instruction::Call { vreg, func, args, ty, discard_value: false })
    }

    fn parse_ins_jmp(&mut self) -> Result<Instruction, BytecodeError> {
        let label = self.parse_str()?;
        
        Ok(Instruction::Jmp(label))
    }

    fn parse_ins_br(&mut self) -> Result<Instruction, BytecodeError> {
        let cond = self.parse_value()?;
        let true_br = self.parse_str()?;
        let false_br = self.parse_str()?;
        
        Ok(Instruction::Br { cond, true_br, false_br })
    }

    fn parse_ins_alloc(&mut self) -> Result<Instruction, BytecodeError> {
        let ty = self.parse_type()?;
        let vreg = self.parse_vreg();
        let num = self.read_bytes(4).try_into().map(u32::from_le_bytes).unwrap();
        
        Ok(Instruction::Alloc { vreg, num, ty })
    }

    fn parse_ins_store(&mut self) -> Result<Instruction, BytecodeError> {
        let ty = self.parse_type()?;
        let ptr = self.parse_value()?;
        let val = self.parse_value()?;
        
        Ok(Instruction::Store { ptr, val, ty })
    }

    fn parse_ins_load(&mut self) -> Result<Instruction, BytecodeError> {
        let ty = self.parse_type()?;
        let vreg = self.parse_vreg();
        let ptr = self.parse_value()?;

        Ok(Instruction::Load { vreg, ptr, ty })
    }

    fn parse_ins_offset(&mut self) -> Result<Instruction, BytecodeError> {
        let ty = self.parse_type()?;
        let vreg = self.parse_vreg();
        let ptr = self.parse_value()?;
        let idx = self.read_bytes(4).try_into().map(u32::from_le_bytes).unwrap() as usize;

        Ok(Instruction::Offset { vreg, ptr, idx, ty })
    }

    /// An instruction is a 1 byte discriminant followed by some bytes.
    /// The discriminant determines how to parse these bytes
    fn parse_ins(&mut self) -> Result<Instruction, BytecodeError> {
        match self.read_bytes(1)[0] {
            0 => self.parse_ins_ret(),
            1 => self.parse_ins_copy(),
            2 => self.parse_ins_op(),
            3 => self.parse_ins_call(),
            4 => self.parse_ins_jmp(),
            5 => self.parse_ins_br(),
            6 => self.parse_ins_alloc(),
            7 => self.parse_ins_store(),
            8 => self.parse_ins_load(),
            9 => self.parse_ins_offset(),
            _ => Err(BytecodeError::UnknownInstruction(self.cursor - 1))
        }
    }
}