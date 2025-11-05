use super::span::Span;
use std::fmt;

// Type checker errors
#[derive(Debug, Clone)]
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum TypeErrorKind {
    TypeMismatch,
    UndefinedVariable,
    InvalidOperation,
    IncompatibleTypes,
    ConstraintSolvingFailure,
    ArraySizeMismatch,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Type error at {}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for TypeError {}

impl TypeError {
    pub fn new(kind: TypeErrorKind, message: String) -> Self {
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
