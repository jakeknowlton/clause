use super::span::Span;
use std::fmt;

// Parser errors
#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    UnexpectedToken,
    ExpectedToken,
    InvalidSyntax,
    UnexpectedEof,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at {}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    pub fn new(kind: ParseErrorKind, message: String) -> Self {
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
