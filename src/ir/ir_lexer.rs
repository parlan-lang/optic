//! This module implements the lexer/tokenizer of the IR

#![allow(unused)]

/// Represents the type of the token
/// 
/// A [`Token`] can have one of several types (e.g., [`TokenKind::Define`]), and this enum represents
/// all of the posible types that a [`Token`] can have
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
// Keywords
    Ret,
    Define,
    Extern,
    Copy,
    Add,
    Sub,
    Mul,
    Div,
    Udiv,
    Call,
    Jmp,
    Cmp,
    Eq, Ne,
    Slt, Ult,
    Sgt, Ugt,
    Br,
    Data,
    Constant,

// Types
    I32,
    I1,
    Ptr,
    Str,

// Delimiters
    Dot,
    Dots, 
    Comma,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    Assing,

// Literals
    GlobSym,
    Vreg, 
    Label,
    IntLit,
    StrLit,

// Sentinels
    Eof,
    Error,
}

/// Represents a token
/// 
/// A [`Token`] is the minimal, meaningful unit of the source code
#[derive(Debug, Clone)]
pub struct Token {
    /// A tuple that contains the start and end of the token' lexeme
    pub span: (u32, u32),
    pub kind: TokenKind
} 

impl Token {
    pub fn new(span: (u32, u32), kind: TokenKind) -> Self {
        return Token {
            span,
            kind
        };
    }

    pub fn get_span(&self) -> std::ops::Range<usize> {
        self.span.0 as usize..self.span.1 as usize
    }
}

/// A Lexer/Tokenizer for the textual IR
/// 
/// [`IrLexer`] transforms the source code into a stream of [`Token`]s.  
/// It exposes the function [`IrLexer::next_token`], which returns the next token in the source code
pub struct IrLexer {
    /// the source code as bytes
    source: Vec<u8>,
    cursor: u32,
}

impl IrLexer {
    pub fn new(source: &str) -> Self {
        return IrLexer {
            source: source.as_bytes().to_vec(),
            cursor: 0
        };
    }

    fn peek(&self) -> u8 {
        return self.source[self.cursor as usize];
    }

    fn is_numeric(&self) -> bool {
        return self.peek() >= b'0' && self.peek() <= b'9';
    }

    fn is_alphanumeric(&self) -> bool {
        return (self.peek() >= b'a' && self.peek() <= b'z') ||
               (self.peek() >= b'A' && self.peek() <= b'Z') ||
               self.peek() == b'_' ||
               self.is_numeric();
    }

    fn get_keyword(&self, start: u32) -> TokenKind {
        match &self.source[start as usize..self.cursor as usize] {
            b"ret" => TokenKind::Ret,
            b"i32" => TokenKind::I32,
            b"i1" => TokenKind::I1,
            b"ptr" => TokenKind::Ptr,
            b"str" => TokenKind::Str,
            b"define" => TokenKind::Define,
            b"extern" => TokenKind::Extern,
            b"copy" => TokenKind::Copy,
            b"add" => TokenKind::Add,
            b"sub" => TokenKind::Sub,
            b"mul" => TokenKind::Mul,
            b"div" => TokenKind::Div,
            b"udiv" => TokenKind::Udiv,
            b"call" => TokenKind::Call,
            b"jmp" => TokenKind::Jmp,
            b"cmp" => TokenKind::Cmp,
            b"eq" => TokenKind::Eq, b"ne" => TokenKind::Ne,
            b"slt" => TokenKind::Slt, b"ult" => TokenKind::Ult,
            b"sgt" => TokenKind::Sgt, b"ugt" => TokenKind::Ugt,
            b"br" => TokenKind::Br,
            b"data" => TokenKind::Data,
            b"constant" => TokenKind::Constant,
            _ => TokenKind::Error
        }
    }

    fn is_at_end(&self) -> bool {
        return self.cursor as usize >= self.source.len();
    }

    /// Returns the next token moving the cursor
    pub fn next_token(&mut self) -> Token {
        while !self.is_at_end() {
            if self.peek() == b' '  ||
               self.peek() == b'\n' ||
               self.peek() == b'\r' ||
               self.peek() == b'\t' { self.cursor += 1; continue; }
            break;
        }

        if self.is_at_end() {
            return Token::new((self.cursor, self.cursor), TokenKind::Eof);
        }

        match self.peek() {
            b'.' => {
                self.cursor += 1;
                if &self.source[self.cursor as usize..self.cursor as usize + 2] == b".." {
                    self.cursor += 2;
                    return Token::new((self.cursor - 3, self.cursor), TokenKind::Dots)
                }
                Token::new((self.cursor - 1, self.cursor), TokenKind::Dot)
            }
            b',' => {
                self.cursor += 1;
                Token::new((self.cursor - 1, self.cursor), TokenKind::Comma)
            }
            b'(' => {
                self.cursor += 1;
                Token::new((self.cursor - 1, self.cursor), TokenKind::Lparen)
            }
            b')' => {
                self.cursor += 1;
                Token::new((self.cursor - 1, self.cursor), TokenKind::Rparen)
            }
            b'{' => {
                self.cursor += 1;
                Token::new((self.cursor - 1, self.cursor), TokenKind::Lbrace)
            }
            b'}' => {
                self.cursor += 1;
                Token::new((self.cursor - 1, self.cursor), TokenKind::Rbrace)
            }
            b'=' => {
                self.cursor += 1;
                Token::new((self.cursor - 1, self.cursor), TokenKind::Assing)
            }
            b'"' => {
                self.cursor += 1;
                let start = self.cursor;

                while !self.is_at_end() && self.peek() != b'"' { self.cursor += 1; }
                self.cursor += 1;

                Token::new((start, self.cursor - 1), TokenKind::StrLit)
            }
            b'@' => {
                self.cursor += 1; // do not include the `@`
                let start = self.cursor;

                while !self.is_at_end() && self.is_alphanumeric() { self.cursor += 1; }

                Token::new((start, self.cursor), TokenKind::GlobSym)
            }
            b'%' => {
                self.cursor += 1; // do not include the `%`
                let start = self.cursor;

                while !self.is_at_end() && self.is_alphanumeric() { self.cursor += 1; }

                Token::new((start, self.cursor), TokenKind::Vreg)
            }
            b'#' => {
                self.cursor += 1; // do not include the `#`
                let start = self.cursor;

                while !self.is_at_end() && self.is_alphanumeric() { self.cursor += 1; }

                Token::new((start, self.cursor), TokenKind::Label)
            }
            b'-' => {
                let start = self.cursor;
                self.cursor += 1;

                while !self.is_at_end() && self.is_numeric() { self.cursor += 1; }

                Token::new((start,self.cursor), TokenKind::IntLit)
            }
            b'0'..=b'9' => {
                let start = self.cursor;

                while !self.is_at_end() && self.is_numeric() { self.cursor += 1; }

                Token::new((start,self.cursor), TokenKind::IntLit)
            },
            b'a'..=b'z' |
            b'A'..=b'Z' |
            b'_' => {
                let start = self.cursor;

                while !self.is_at_end() && self.is_alphanumeric() {self.cursor += 1; }

                let token_type = self.get_keyword(start);

                if token_type == TokenKind::Error {
                    eprintln!("error: unknown instruction nmemonic `{}`", str::from_utf8(&self.source[start as usize..self.cursor as usize]).unwrap());
                    return Token::new((start,self.cursor), TokenKind::Error)
                }

                Token::new((start, self.cursor), token_type)
            }
            _ => {
                eprintln!("error: unknown start of token `{}`", self.peek() as char);
                Token::new((0,00), TokenKind::Error)
            }
        }
    }

    // Returns the next token without moving the cursor
    pub fn peek_token(&mut self) -> Token {
        let cursor = self.cursor;
        let tk = self.next_token();
        self.cursor = cursor;
        return tk;
    }
}