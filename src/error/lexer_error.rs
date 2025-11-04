use std::fmt;
use super::span::Span;

// Lexer errors
#[derive(Debug, Clone)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum LexerErrorKind {
    UnexpectedCharacter,
    UnterminatedString,
    InvalidNumber,
    InvalidEscape,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexer error at {}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for LexerError {}

impl LexerError {
    pub fn new(kind: LexerErrorKind, message: String) -> Self {
        Self {
            kind,
            span: Span::unknown(),
            message,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }
}
