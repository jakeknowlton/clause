use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}('{}') at {}:{}",
            self.kind, self.lexeme, self.line, self.column
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Let,
    Fix,
    Fun,
    Return,
    If,
    Else,
    While,
    Break,

    // Literals
    IntegerLiteral,
    FloatLiteral,
    BooleanLiteral,
    VoidLiteral,

    // Identifier
    Identifier,

    // Assignment operators
    Assign,       // =
    PlusAssign,   // +=
    MinusAssign,  // -=
    MultAssign,   // *=
    DivAssign,    // /=
    ModAssign,    // %=
    AndAssign,    // &=
    OrAssign,     // |=
    XorAssign,    // ^=
    LShiftAssign, // <<=
    RShiftAssign, // >>=

    // Comparison operators
    Eq, // ==
    Ne, // !=
    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=

    // Logical operators
    LAnd, // &&
    LOr,  // ||
    Not,  // !

    // Bitwise operators
    And,    // &
    Or,     // |
    Xor,    // ^
    BNot,   // ~
    LShift, // <<
    RShift, // >>

    // Arithmetic operators
    Plus,  // +
    Minus, // -
    Mult,  // *
    Div,   // /
    Mod,   // %

    // Yield operator
    Yield, // <-

    // Delimiters
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
    Colon,     // :
    Comma,     // ,
    Semicolon, // ;

    // Special
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
