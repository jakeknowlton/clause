use std::fmt;
use super::span::{Span, StackFrame};

// Runtime errors
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub span: Span,
    pub message: String,
    pub backtrace: Vec<StackFrame>,
}

#[derive(Debug, Clone)]
pub enum RuntimeErrorKind {
    DivisionByZero,
    Overflow,
    Underflow,
    IndexOutOfBounds,
    InvalidOperation,
    UndefinedVariable,
    ImmutableAssignment,
    TypeMismatch,
    InvalidConversion,
    BreakOutsideLoop,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Runtime error at {}:{}: {}",
            self.span.line, self.span.column, self.message
        )?;

        if !self.backtrace.is_empty() {
            writeln!(f, "\nStack trace:")?;
            for frame in &self.backtrace {
                writeln!(
                    f,
                    "  at {} ({}:{})",
                    frame.function_name, frame.span.line, frame.span.column
                )?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for RuntimeError {}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind, message: String) -> Self {
        Self {
            kind,
            span: Span::unknown(),
            message,
            backtrace: Vec::new(),
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    pub fn with_backtrace(mut self, backtrace: Vec<StackFrame>) -> Self {
        self.backtrace = backtrace;
        self
    }
}
