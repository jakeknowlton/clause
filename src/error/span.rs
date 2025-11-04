// Span information for error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Span {
    pub fn new(line: usize, column: usize, length: usize) -> Self {
        Self {
            line,
            column,
            length,
        }
    }

    pub fn unknown() -> Self {
        Self {
            line: 0,
            column: 0,
            length: 0,
        }
    }
}

// Stack frame for runtime errors
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub span: Span,
}
