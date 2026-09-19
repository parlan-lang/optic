//! This module implements the error reporting system

use std::panic::Location;

use crate::ir::ir_lexer::*;

fn offset_to_line_col(tk: &Token, src: &str) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, c) in src.char_indices() {
        if i >= tk.span.0 as usize {
            break;
        }

        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

/// Helper function to report errors
#[track_caller]
pub fn _report_error_(tk: &Token, src: &str, file_name: &str, args: std::fmt::Arguments) {
    let caller = Location::caller();
    let (line, col) = offset_to_line_col(tk, src);
    let line_content = src.lines().nth(line - 1).unwrap_or("");

    eprintln!("\x1b[1;31merror at\x1b[0m `{}:{}:{}` : {}", file_name, line, col, args);
    eprintln!("   \x1b[1;34m|\x1b[0m");
    eprintln!("\x1b[1;34m{:>2} |\x1b[0m {}", line, line_content);
    eprintln!("   \x1b[1;34m|\x1b[0m {}\x1b[1;31m{}\x1b[0m", " ".repeat(col - 1), "^".repeat((tk.span.1 - tk.span.0) as usize));
    eprintln!("   \x1b[90m= emitted by the compiler at `{}:{}:{}`\x1b[0m", caller.file(), caller.line(), caller.column());

    std::process::exit(1);
}

/// Reports an error
/// 
/// ### Syntax: 
/// 
/// ```no_run
/// error!(file_name, source, token, format args)
/// ```
#[macro_export]
macro_rules! error {
    ($file:expr, $src:expr, $tk:expr, $($arg:tt)*) => {
        $crate::error::_report_error_($tk, $src, $file, format_args!($($arg)*));
    };
}