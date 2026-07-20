//! this module implements the parser of the IR

#![allow(unused)]

use std::collections::HashMap;

use crate::ir::ir_lexer::*;
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
            _ => {
                eprintln!("error: expected a type, found {:?} instead", self.peek().kind);
                panic!()
            }
        }
    }

    /// Parses the `ret` instruction
    fn parse_ins_ret(&mut self) -> Instruction {
        self.eat(TokenKind::Ret);
        self.eat(TokenKind::Dot);

        let ty = self.parse_type();

        let val = self.parse_value();

        Instruction::Ret { val, ty }
    }

    /// Parses the `copy` instruction
    fn parse_ins_copy(&mut self, vreg: usize, ty: Type) -> Instruction {
        self.eat(TokenKind::Copy);

        let val = self.parse_value();

        Instruction::Copy { vreg, val, ty }
    }

    fn parse_ins_op(&mut self, vreg: usize, ty: Type) -> Instruction {
        let kind = match self.next().kind {
            TokenKind::Add => OpKind::Add,
            TokenKind::Sub => OpKind::Sub,
            TokenKind::Mul => OpKind::Mul,
            TokenKind::Div => OpKind::Div,
            TokenKind::Udiv => OpKind::Udiv,
            _ => panic!()
        };

        let lhs = self.parse_value();
        self.eat(TokenKind::Comma);
        let rhs = self.parse_value();

        Instruction::Op { vreg, kind, lhs, rhs, ty }
    }

    fn parse_ins_call(&mut self, vreg: usize, ty: Type) -> Instruction {
        self.eat(TokenKind::Call);

        let func_tk = self.eat(TokenKind::GlobSym);
        let func = self.src[func_tk.get_span()].to_string();

        self.eat(TokenKind::Lparen);

        let mut args = Vec::new();

        if self.peek().kind != TokenKind::Rparen {
            args.push(self.parse_value());
            while self.peek().kind == TokenKind::Comma {
                self.eat(TokenKind::Comma);
                args.push(self.parse_value());
            }
        }

        self.eat(TokenKind::Rparen);

        Instruction::Call { vreg, func, args, ty }
    }

    fn parse_ins_jmp(&mut self) -> Instruction {
        self.eat(TokenKind::Jmp);

        let label = self.eat(TokenKind::Label);

        Instruction::Jmp(self.src[label.get_span()].to_string())
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
            TokenKind::Label => {
                let label = self.next();
                Instruction::Label(self.src[label.get_span()].to_string())
            },
            TokenKind::Vreg => {
                let curr_tk = self.next(); // we already know this is Vreg
                let vreg_name = self.src[curr_tk.get_span()].to_string();
                let mut vreg = 0;
                if let Some(v) = self.local.get(&vreg_name) {
                    vreg = *v;
                } else {
                    vreg = self.next_vreg();
                    self.local.insert(vreg_name, vreg);
                }

                self.eat(TokenKind::Assing);
                self.eat(TokenKind::Dot);
                let ty = self.parse_type();

                match self.peek().kind {
                    TokenKind::Copy => self.parse_ins_copy(vreg, ty),
                    TokenKind::Call => self.parse_ins_call(vreg, ty),
                    TokenKind::Add | TokenKind::Sub |
                    TokenKind::Mul | TokenKind::Div => self.parse_ins_op(vreg, ty),
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

        let name_span = self.eat(TokenKind::GlobSym).get_span();
        let name = self.src[name_span].to_string();

        self.eat(TokenKind::Lparen);

        let mut params = Vec::<Parameter>::new();

        if self.peek().kind != TokenKind::Rparen {
            let param = self.eat(TokenKind::Vreg);
            let vreg = self.next_vreg();
            let ty = self.parse_type();
            params.push(Parameter { vreg, ty });
            self.local.insert(self.src[param.get_span()].to_string(), vreg);

            while self.peek().kind == TokenKind::Comma {
                self.eat(TokenKind::Comma);
                let param = self.eat(TokenKind::Vreg);
                let vreg = self.next_vreg();
                let ty = self.parse_type();
                params.push(Parameter { vreg, ty });
                self.local.insert(self.src[param.get_span()].to_string(), vreg);
            }
        }

        self.eat(TokenKind::Rparen);

        let ty = self.parse_type();

        self.eat(TokenKind::Lbrace);

        let mut body = Vec::new();

        while self.peek().kind != TokenKind::Rbrace {
            body.push(self.parse_ins());
        }

        self.eat(TokenKind::Rbrace);

        self.curr_vreg = curr_vreg; // reset the virtual registers
        
        Function { name, params, ty, body, cfg: ControlFlowGraph::new() }
    }

    /// Parses a module
    pub fn parse_module(&mut self, name: &str) -> Module {
        let mut functions = Vec::new();

        while self.peek().kind != TokenKind::Eof {
            functions.push(self.parse_function());
        }

        Module {
            name: name.to_string(),
            functions
        }
    }
}
