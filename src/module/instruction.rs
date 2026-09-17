/// Represents a value that can be used by an instruction
/// 
/// A [`Value`] is any value that can be used by an instruction, e.g., an integer literal
/// or a virtual register
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    IntLit(usize),
    FloatLit(f64),
    Vreg(usize),
    GlobSym(String),
    // The value returned by `ret void`
    Void,
}

impl Value {
    pub fn as_vreg(&self) -> Option<usize> {
        match self {
            Value::Vreg(id) => Some(*id),
            _ => None
        }
    }
}

/// The Type of an [`Instruction`]
/// 
/// Every [`Instruction`] have a type (e.g., [`Type::I32`]), and this enum represents all of the posible types
/// that an [`Instruction`] can have
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    I32,
    I1,
    F32,
    Ptr,
    Ascii,
    Void,
}

/// The kind of a binary or boolean operation 
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpKind {
    Add,
    Sub,
    Mul,
    Div,
    Udiv,
    CmpEq, CmpNe,
    CmpSlt, CmpUlt,
    CmpSgt, CmpUgt,
}

/// Represents a single instruction and its data
/// 
/// A [`Instruction`] contains all the information related to that single instruction
/// that it represents
#[derive(Debug, Clone)]
pub enum Instruction {
    Ret {
        val: Value,
        ty: Type 
    },
    Copy {
        vreg: usize,
        val: Value,
        ty: Type
    },
    Op {
        vreg: usize,
        kind: OpKind,
        lhs: Value,
        rhs: Value,
        ty: Type,
    },
    Call {
        vreg: usize,
        func: String,
        args: Vec<(Type, Value)>,
        ty: Type,
        discard_value: bool
    },
    Label (String),
    Jmp (String),
    Br {
        cond: Value,
        true_br: String,
        false_br: String
    },
    Alloc {
        vreg: usize,
        num: u32, 
        ty: Type
    },
    Store {
        ptr: Value,
        val: Value,
        ty: Type
    },
    Load {
        vreg: usize,
        ptr: Value,
        ty: Type
    },
    Phi {
        vreg: usize,
        srcs: Vec<Value>,
        ty: Type
    }
}

impl Instruction {
    pub fn is_label(&self) -> bool {
        matches!(self, Instruction::Label(_))
    }

    pub fn is_terminator(&self) -> bool {
        match self {
            Instruction::Jmp(_) |
            Instruction::Br { .. } |
            Instruction::Ret { .. } => true,
            _ => false,
        }
    }

    pub fn get_label_name(&self) -> Option<&String> {
        match self {
            Instruction::Label(name) => Some(name),
            _ => None
        }
    }
}

/// Represents a global data (the `data` instruction)
#[derive(Debug, Clone)]
pub struct GlobData {
    pub name: String,
    pub val: Vec<u8>,
    pub is_constant: bool,
}