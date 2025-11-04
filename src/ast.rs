use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VariableDeclaration(VariableDeclaration),
    Yield(YieldStatement),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    pub mutable: bool, // true for 'let', false for 'fix'
    pub name: String,
    pub type_annotation: Option<Type>,
    pub initializer: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YieldStatement {
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literals
    Integer(String),
    Float(String),
    Boolean(bool),
    Void,
    ArrayLiteral(Vec<Expression>),

    // Identifier
    Identifier(String),

    // Binary operations
    Binary(Box<BinaryExpr>),

    // Unary operations
    Unary(Box<UnaryExpr>),

    // Array indexing
    Index(Box<IndexExpr>),

    // Block expression
    Block(Block),

    // Grouped expression
    Grouping(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Expression,
    pub operator: BinaryOp,
    pub right: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexExpr {
    pub array: Expression,
    pub index: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Assignment
    Assign,
    PlusAssign,
    MinusAssign,
    MultAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LShiftAssign,
    RShiftAssign,

    // Logical
    LogicalOr,
    LogicalAnd,

    // Bitwise
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,

    // Equality
    Equal,
    NotEqual,

    // Relational
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,

    // Bitwise shift
    LeftShift,
    RightShift,

    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Plus,
    Minus,
    LogicalNot,
    BitwiseNot,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Signed(u8),   // i1 through i128
    Unsigned(u8), // u1 through u128
    F32,
    F64,
    Bool,
    Void,
    Array(Box<Type>, usize), // [Type, N] - element type and size
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Signed(bits) => write!(f, "i{}", bits),
            Type::Unsigned(bits) => write!(f, "u{}", bits),
            Type::F32 => write!(f, "F32"),
            Type::F64 => write!(f, "F64"),
            Type::Bool => write!(f, "Bool"),
            Type::Void => write!(f, "Void"),
            Type::Array(elem_type, size) => write!(f, "[{}, {}]", elem_type, size),
        }
    }
}

// Pretty printing for AST
impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::VariableDeclaration(decl) => write!(f, "{}", decl),
            Statement::Yield(yield_stmt) => write!(f, "{}", yield_stmt),
            Statement::Expression(expr) => write!(f, "{}", expr),
        }
    }
}

impl fmt::Display for VariableDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keyword = if self.mutable { "let" } else { "fix" };
        write!(f, "{} {}", keyword, self.name)?;
        if let Some(ty) = &self.type_annotation {
            write!(f, ": {}", ty)?;
        }
        write!(f, " = {}", self.initializer)
    }
}

impl fmt::Display for YieldStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<- {}", self.value)
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Integer(val) => write!(f, "{}", val),
            Expression::Float(val) => write!(f, "{}", val),
            Expression::Boolean(val) => write!(f, "{}", val),
            Expression::Void => write!(f, "void"),
            Expression::ArrayLiteral(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Expression::Identifier(name) => write!(f, "{}", name),
            Expression::Binary(binary) => {
                write!(f, "({} {} {})", binary.left, binary.operator, binary.right)
            }
            Expression::Unary(unary) => write!(f, "({}{})", unary.operator, unary.operand),
            Expression::Index(index) => write!(f, "{}[{}]", index.array, index.index),
            Expression::Block(block) => write!(f, "{}", block),
            Expression::Grouping(expr) => write!(f, "({})", expr),
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ ")?;
        for stmt in &self.statements {
            write!(f, "{}; ", stmt)?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            BinaryOp::Assign => "=",
            BinaryOp::PlusAssign => "+=",
            BinaryOp::MinusAssign => "-=",
            BinaryOp::MultAssign => "*=",
            BinaryOp::DivAssign => "/=",
            BinaryOp::ModAssign => "%=",
            BinaryOp::AndAssign => "&=",
            BinaryOp::OrAssign => "|=",
            BinaryOp::XorAssign => "^=",
            BinaryOp::LShiftAssign => "<<=",
            BinaryOp::RShiftAssign => ">>=",
            BinaryOp::LogicalOr => "||",
            BinaryOp::LogicalAnd => "&&",
            BinaryOp::BitwiseOr => "|",
            BinaryOp::BitwiseXor => "^",
            BinaryOp::BitwiseAnd => "&",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::LessThan => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::GreaterThan => ">",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::LeftShift => "<<",
            BinaryOp::RightShift => ">>",
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
        };
        write!(f, "{}", op)
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            UnaryOp::Plus => "+",
            UnaryOp::Minus => "-",
            UnaryOp::LogicalNot => "!",
            UnaryOp::BitwiseNot => "~",
        };
        write!(f, "{}", op)
    }
}
