//! this module implements the parser of the IR

use std::collections::HashMap;

use crate::ir::ir_lexer::*;
use crate::module::instruction::Value::GlobSym;
use crate::module::{
    Module, 
    instruction::*, 
    function::*,
};
use crate::cfg::*;

pub struct IrParser {
    lexer: IrLexer,
    src: String,
    curr_vreg: usize,
    local: HashMap<String, usize> // local scope of the current function
}

impl IrParser {
    pub fn new(src: &str) -> Self {
        IrParser {
            lexer: IrLexer::new(src),
            src: src.to_string(),
            curr_vreg: 0,
            local: HashMap::new()
        }
    }

    /// Shorthand for `self.lexer.next_token()`
    #[inline(always)]
    fn next(&mut self) -> Token {
        self.lexer.next_token()
    }

    /// Shorthand for `self.lexer.peek_token()`
    #[inline(always)]
    fn peek(&mut self) -> Token {
        self.lexer.peek_token()
    }

    /// Returns the current virtual register and advances
    #[inline(always)]
    fn next_vreg(&mut self) -> usize {
        let curr = self.curr_vreg;
        self.curr_vreg += 1;
        curr
    }

    /// Consumes and returns the next [`Token`] if it matches the expected [`TokenKind`]
    /// 
    /// # Panics
    /// 
    /// This will `panic` if the next [`Token`] does not match the expected type 
    fn eat(&mut self, kind: TokenKind) -> Token {
        if self.peek().kind != kind {
            eprintln!("error: expected token of kind {:?}, found `{:?}` instead", kind, self.peek().kind);
            panic!();
        }
        self.next()
    }

    /// Parses a [`Value`] and returns it
    /// 
    /// Panics
    /// 
    /// This will `panic` in case the next [`Token`] is not a valid [`Value`]
    fn parse_value(&mut self) -> Value {
        let tk = self.next();
        match tk.kind {
            TokenKind::IntLit => {
                let n = &self.src[tk.get_span()];
                if n.starts_with('-') {
                    let num = n.parse::<isize>().unwrap();
                    return Value::IntLit(num as usize);
                }

                Value::IntLit(n.parse().unwrap())
            }
            TokenKind::FloatLit => {
                let n = &self.src[tk.get_span()];
                Value::FloatLit(n.parse().unwrap())
            }
            TokenKind::Vreg => {
                let vreg = &self.src[tk.get_span()];

                match self.local.get(vreg) {
                    Some(vreg) => Value::Vreg(*vreg),
                    None => {
                        eprintln!("error: undeclared virtual register {:?}", vreg);
                        panic!()
                    }
                }
            }
            TokenKind::GlobSym => {
                let name = &self.src[tk.get_span()];

                GlobSym(name.to_string())
            }
            _ => {
                eprintln!("error: expected a value, found {:?} instead", tk.kind);
                panic!()
            }
        }
    }

    /// Parses a [`Type`] and returns it
    /// 
    /// Panics
    /// 
    /// This will `panic` in case the next [`Token`] is not a valid [`Type`] 
    fn parse_type(&mut self) -> Type {
        match self.next().kind {
            TokenKind::I32 => Type::I32,
            TokenKind::I1 => Type::I1,
            TokenKind::F32 => Type::F32,
            TokenKind::Ptr => Type::Ptr,
            TokenKind::Ascii => Type::Ascii,
            TokenKind::Void => Type::Void,
            _ => {
                eprintln!("error: expected a type, found {:?} instead", self.peek().kind);
                panic!()
            }
        }
    }

    /// Parses the `ret` instruction
    fn parse_ins_ret(&mut self) -> Instruction {
        self.eat(TokenKind::Ret);

        let ty = self.parse_type();

        if ty == Type::Void {
            return Instruction::Ret { val: Value::Void, ty }
        }

        let val = self.parse_value();

        Instruction::Ret { val, ty }
    }

    /// Parses the `copy` instruction
    fn parse_ins_copy(&mut self, vreg: usize) -> Instruction {
        self.eat(TokenKind::Copy);

        let ty = self.parse_type();
        let val = self.parse_value();

        Instruction::Copy { vreg, val, ty }
    }

    fn parse_ins_op(&mut self, vreg: usize) -> Instruction {
        let kind = match self.next().kind {
            TokenKind::Add => OpKind::Add,
            TokenKind::Sub => OpKind::Sub,
            TokenKind::Mul => OpKind::Mul,
            TokenKind::Div => OpKind::Div,
            TokenKind::Udiv => OpKind::Udiv,
            TokenKind::Ceq => OpKind::CmpEq,
            TokenKind::Cne => OpKind::CmpNe,
            TokenKind::Cslt => OpKind::CmpSlt,
            TokenKind::Cult => OpKind::CmpUlt,
            TokenKind::Csgt => OpKind::CmpSgt,
            TokenKind::Cugt => OpKind::CmpUgt,
            _ => panic!()
        };

        let ty = self.parse_type();

        let lhs = self.parse_value();
        self.eat(TokenKind::Comma);
        let rhs = self.parse_value();

        Instruction::Op { vreg, kind, lhs, rhs, ty}
    }

    fn parse_ins_call(&mut self, vreg: usize, discard_value: bool) -> Instruction {
        self.eat(TokenKind::Call);

        let ty = self.parse_type();

        let func_tk = self.eat(TokenKind::GlobSym);
        let func = self.src[func_tk.get_span()].to_string();

        self.eat(TokenKind::Lparen);

        let mut args = Vec::new();

        if self.peek().kind != TokenKind::Rparen {
            args.push((self.parse_type(), self.parse_value()));
            while self.peek().kind == TokenKind::Comma {
                self.eat(TokenKind::Comma);
                args.push((self.parse_type(), self.parse_value()));
            }
        }

        self.eat(TokenKind::Rparen);

        Instruction::Call { vreg, func, args, ty, discard_value }
    }

    fn parse_ins_jmp(&mut self) -> Instruction {
        self.eat(TokenKind::Jmp);

        let label = self.eat(TokenKind::Label);

        Instruction::Jmp(self.src[label.get_span()].to_string())
    }

    fn parse_ins_br(&mut self) -> Instruction {
        self.eat(TokenKind::Br);

        let cond = self.parse_value();

        self.eat(TokenKind::Comma);
        let true_br = self.eat(TokenKind::Label);
        self.eat(TokenKind::Comma);
        let false_br = self.eat(TokenKind::Label);

        Instruction::Br { cond, true_br: self.src[true_br.get_span()].to_string(), false_br: self.src[false_br.get_span()].to_string() }
    }

    fn parse_ins_alloc(&mut self, vreg: usize) -> Instruction {
        self.eat(TokenKind::Alloc);

        let ty = self.parse_type();

        let num = if self.peek().kind == TokenKind::Comma {
            self.next();
            let num_tk = self.eat(TokenKind::IntLit);
            self.src[num_tk.get_span()].parse::<u32>().unwrap()
        } else {
            1
        };

        Instruction::Alloc { vreg, ty, num }
    }

    fn parse_ins_store(&mut self) -> Instruction {
        self.eat(TokenKind::Store);

        let ty = self.parse_type();

        let ptr = self.parse_value();

        self.eat(TokenKind::Comma);

        let val = self.parse_value();

        Instruction::Store { ptr, val, ty }
    }

    fn parse_ins_load(&mut self, vreg: usize) -> Instruction {
        self.eat(TokenKind::Load);

        let ty = self.parse_type();

        let ptr = self.parse_value();

        Instruction::Load { vreg, ptr, ty }
    }

    fn parse_ins_offset(&mut self, vreg: usize) -> Instruction {
        self.eat(TokenKind::Offset);

        let ptr = self.parse_value();

        self.eat(TokenKind::Comma);

        let ty = self.parse_type();

        self.eat(TokenKind::Comma);

        let idx_tk = self.eat(TokenKind::IntLit);
        let idx = self.src[idx_tk.get_span()].parse::<usize>().unwrap();

        Instruction::Offset { vreg, ptr, idx, ty }
    }

    /// Parses an instruction
    /// 
    /// Panics
    /// 
    /// This will `panic` in case the next [`Token`] is not a valid instruction
    fn parse_ins(&mut self) -> Instruction {
        match self.peek().kind {
            TokenKind::Ret => self.parse_ins_ret(),
            TokenKind::Jmp => self.parse_ins_jmp(),
            TokenKind::Br => self.parse_ins_br(),
            TokenKind::Call => self.parse_ins_call(0, true),
            TokenKind::Store => self.parse_ins_store(),
            TokenKind::Label => {
                let label = self.next();
                Instruction::Label(self.src[label.get_span()].to_string())
            },
            TokenKind::Vreg => {
                let curr_tk = self.next(); // we already know this is Vreg
                let vreg_name = self.src[curr_tk.get_span()].to_string();
                let vreg = match self.local.get(&vreg_name) {
                    Some(v) => *v,
                    None => {
                        let v = self.next_vreg();
                        self.local.insert(vreg_name, v);
                        v
                    }
                };

                self.eat(TokenKind::Assing);

                match self.peek().kind {
                    TokenKind::Copy => self.parse_ins_copy(vreg),
                    TokenKind::Call => self.parse_ins_call(vreg, false),
                    TokenKind::Alloc => self.parse_ins_alloc(vreg),
                    TokenKind::Load => self.parse_ins_load(vreg),
                    TokenKind::Offset => self.parse_ins_offset(vreg),
                    TokenKind::Add  | TokenKind::Sub  |
                    TokenKind::Mul  | TokenKind::Div  |
                    TokenKind::Ceq  | TokenKind::Cne  |
                    TokenKind::Cslt | TokenKind::Cult |
                    TokenKind::Csgt | TokenKind::Cugt => self.parse_ins_op(vreg),
                    _ => {
                        eprintln!("error: expected an instruction, found {:?} instead", self.peek().kind);
                        panic!()
                    }
                }
            }
            _ => {
                eprintln!("error: expected an instruction, found {:?} instead", self.peek().kind);
                panic!()
            }
        }
    }

    /// Parses a function
    fn parse_function(&mut self) -> Function {
        let curr_vreg = self.curr_vreg; // current virtual register number before parsing the body

        self.eat(TokenKind::Define);

        let is_extern = if self.peek().kind == TokenKind::Extern {
            self.next();
            true
        } else {
            false
        };

        let name_span = self.eat(TokenKind::GlobSym).get_span();
        let name = self.src[name_span].to_string();

        self.eat(TokenKind::Lparen);

        let mut params = Vec::<Parameter>::new();

        if is_extern {
            while self.peek().kind != TokenKind::Rparen {
                if self.peek().kind == TokenKind::Dots {
                    self.next();
                    params.push(Parameter { vreg: self.next_vreg(), ty: Type::I1, is_vaarg: true });
                    break; // it must be the last parameter
                }
                params.push(Parameter { vreg: self.next_vreg(), ty: self.parse_type(), is_vaarg: false });

                if self.peek().kind != TokenKind::Rparen {
                    self.eat(TokenKind::Comma);
                }
            }
        }

        if self.peek().kind != TokenKind::Rparen {
            let param = self.eat(TokenKind::Vreg);
            let vreg = self.next_vreg();
            let ty = self.parse_type();
            params.push(Parameter { vreg, ty, is_vaarg: false });
            self.local.insert(self.src[param.get_span()].to_string(), vreg);

            while self.peek().kind == TokenKind::Comma {
                self.eat(TokenKind::Comma);

                if self.peek().kind == TokenKind::Dots {
                    self.eat(TokenKind::Dots);
                    params.push(Parameter { vreg: 0, ty: Type::I1, is_vaarg: true }); // i chosed Type::I1 as the default, but it will be never be readed
                    break; // it must be the last argument
                }

                let param = self.eat(TokenKind::Vreg);
                let vreg = self.next_vreg();
                let ty = self.parse_type();
                params.push(Parameter { vreg, ty, is_vaarg: false });
                self.local.insert(self.src[param.get_span()].to_string(), vreg);
            }
        }

        self.eat(TokenKind::Rparen);

        let ty = self.parse_type();

        if is_extern {
            self.curr_vreg = curr_vreg; // reset the virtual registers
            return Function { name, params, ty, body: Vec::new(), cfg: ControlFlowGraph::new(), is_extern }
        }

        self.eat(TokenKind::Lbrace);

        let mut body = Vec::new();

        while self.peek().kind != TokenKind::Rbrace {
            body.push(self.parse_ins());
        }

        self.eat(TokenKind::Rbrace);

        self.curr_vreg = curr_vreg; // reset the virtual registers
        
        Function { name, params, ty, body, cfg: ControlFlowGraph::new(), is_extern }
    }

    fn parse_global_value(&mut self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        
        self.eat(TokenKind::Lbrace);

        while self.peek().kind != TokenKind::Rbrace {
            match self.parse_type() {
                Type::I32 => {
                    let tk = self.eat(TokenKind::IntLit);
                    self.src[tk.get_span()].parse::<i32>().unwrap().to_le_bytes().iter().for_each(|b| bytes.push(*b));
                },
                Type::I1 => {
                    let tk = self.eat(TokenKind::IntLit);
                    if self.src[tk.get_span()].parse::<i8>().unwrap() > 0 {
                        bytes.push(1);
                    } else {
                        bytes.push(0);
                    }
                },
                Type::F32 => {
                    let tk = self.eat(TokenKind::IntLit);
                    self.src[tk.get_span()].parse::<f32>().unwrap().to_le_bytes().iter().for_each(|b| bytes.push(*b));
                },
                Type::Ptr => panic!("error: data instructions don't support pointers"),
                Type::Ascii => {
                    let tk = self.eat(TokenKind::AsciiLit);
                    self.src[tk.get_span()].as_bytes().iter().for_each(|b| bytes.push(*b));
                },
                Type::Void => panic!("error: data instructions don't support void"),
            };

            if self.peek().kind != TokenKind::Rbrace {
                self.eat(TokenKind::Comma);
            }
        }

        self.eat(TokenKind::Rbrace);

        bytes
    }

    fn parse_global(&mut self) -> GlobData {
        self.eat(TokenKind::Data);

        let is_constant = if self.peek().kind == TokenKind::Constant {
            self.next();
            true
        } else {
            false
        };

        let name_tk = self.eat(TokenKind::GlobSym);
        let name = self.src[name_tk.get_span()].to_string();

        self.eat(TokenKind::Assing);

        let val = self.parse_global_value();

        GlobData { name, val, is_constant }
    }

    /// Parses a module
    pub fn parse_module(&mut self, name: &str) -> Module {
        let mut functions = Vec::new();
        let mut globals = Vec::new();

        while self.peek().kind != TokenKind::Eof {
            match self.peek().kind {
                TokenKind::Data => globals.push(self.parse_global()),
                TokenKind::Define => functions.push(self.parse_function()),
                _ => {
                    eprintln!("error: expected `define` or `data`, found {:?}", self.peek());
                    panic!();
                }
            }
        }

        Module {
            name: name.to_string(),
            functions,
            globals
        }
    }
}
