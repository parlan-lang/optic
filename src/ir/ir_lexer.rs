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

// Types
    I32,

// Delimiters
    Dot,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,

// Literals
    GlobSym, 
    IntLit,

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
            b"define" => TokenKind::Define,
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
                Token::new((self.cursor - 1, self.cursor), TokenKind::Dot)
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
            b'@' => {
                self.cursor += 1; // do not include the `@`
                let start = self.cursor;

                while !self.is_at_end() && self.is_alphanumeric() { self.cursor += 1; }

                Token::new((start, self.cursor), TokenKind::GlobSym)
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