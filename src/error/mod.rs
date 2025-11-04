mod span;
mod lexer_error;
mod parse_error;
mod type_error;
mod runtime_error;

pub use span::Span;
pub use lexer_error::{LexerError, LexerErrorKind};
pub use parse_error::{ParseError, ParseErrorKind};
pub use type_error::{TypeError, TypeErrorKind};
pub use runtime_error::{RuntimeError, RuntimeErrorKind};

use std::fmt;

// Main error type for the entire language
#[derive(Debug)]
pub enum ClauseError {
    Lexer(LexerError),
    Parser(ParseError),
    Type(TypeError),
    Runtime(RuntimeError),
}

impl fmt::Display for ClauseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClauseError::Lexer(e) => write!(f, "{}", e),
            ClauseError::Parser(e) => write!(f, "{}", e),
            ClauseError::Type(e) => write!(f, "{}", e),
            ClauseError::Runtime(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ClauseError {}

// Conversion from String for backward compatibility
impl From<String> for ClauseError {
    fn from(s: String) -> Self {
        ClauseError::Runtime(RuntimeError {
            kind: RuntimeErrorKind::InvalidOperation,
            span: Span::unknown(),
            message: s,
            backtrace: Vec::new(),
        })
    }
}

impl From<&str> for ClauseError {
    fn from(s: &str) -> Self {
        ClauseError::from(s.to_string())
    }
}

// Conversions from specific error types
impl From<LexerError> for ClauseError {
    fn from(e: LexerError) -> Self {
        ClauseError::Lexer(e)
    }
}

impl From<ParseError> for ClauseError {
    fn from(e: ParseError) -> Self {
        ClauseError::Parser(e)
    }
}

impl From<TypeError> for ClauseError {
    fn from(e: TypeError) -> Self {
        ClauseError::Type(e)
    }
}

impl From<RuntimeError> for ClauseError {
    fn from(e: RuntimeError) -> Self {
        ClauseError::Runtime(e)
    }
}
