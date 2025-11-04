use crate::ast::{BinaryOp, Type, UnaryOp};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct TypedProgram {
    pub statements: Vec<TypedStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedStatement {
    VariableDeclaration(TypedVariableDeclaration),
    Yield(TypedYieldStatement),
    Expression(TypedExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedVariableDeclaration {
    pub mutable: bool,
    pub name: String,
    pub var_type: Type, // Resolved type
    pub initializer: TypedExpression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedYieldStatement {
    pub value: TypedExpression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpression {
    // Literals with resolved types
    Integer(String, Type), // value, resolved type
    Float(String, Type),   // value, resolved type
    Boolean(bool),
    Void,

    // Identifier with resolved type
    Identifier(String, Type),

    // Binary operations with resolved type
    Binary(Box<TypedBinaryExpr>),

    // Unary operations with resolved type
    Unary(Box<TypedUnaryExpr>),

    // Block expression with resolved type
    Block(TypedBlock),

    // Grouped expression
    Grouping(Box<TypedExpression>),
}

impl TypedExpression {
    pub fn get_type(&self) -> Type {
        match self {
            TypedExpression::Integer(_, ty) => ty.clone(),
            TypedExpression::Float(_, ty) => ty.clone(),
            TypedExpression::Boolean(_) => Type::Bool,
            TypedExpression::Void => Type::Void,
            TypedExpression::Identifier(_, ty) => ty.clone(),
            TypedExpression::Binary(binary) => binary.result_type.clone(),
            TypedExpression::Unary(unary) => unary.result_type.clone(),
            TypedExpression::Block(block) => block.block_type.clone(),
            TypedExpression::Grouping(expr) => expr.get_type(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedBinaryExpr {
    pub left: TypedExpression,
    pub operator: BinaryOp,
    pub right: TypedExpression,
    pub result_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedUnaryExpr {
    pub operator: UnaryOp,
    pub operand: TypedExpression,
    pub result_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedBlock {
    pub statements: Vec<TypedStatement>,
    pub block_type: Type,
}

// Display implementations for debugging
impl fmt::Display for TypedProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.statements {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

impl fmt::Display for TypedStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypedStatement::VariableDeclaration(decl) => write!(f, "{}", decl),
            TypedStatement::Yield(yield_stmt) => write!(f, "{}", yield_stmt),
            TypedStatement::Expression(expr) => write!(f, "{}", expr),
        }
    }
}

impl fmt::Display for TypedVariableDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keyword = if self.mutable { "let" } else { "fix" };
        write!(
            f,
            "{} {}: {} = {}",
            keyword, self.name, self.var_type, self.initializer
        )
    }
}

impl fmt::Display for TypedYieldStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<- {}", self.value)
    }
}

impl fmt::Display for TypedExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypedExpression::Integer(val, ty) => write!(f, "{}:{}", val, ty),
            TypedExpression::Float(val, ty) => write!(f, "{}:{}", val, ty),
            TypedExpression::Boolean(val) => write!(f, "{}", val),
            TypedExpression::Void => write!(f, "void"),
            TypedExpression::Identifier(name, ty) => write!(f, "{}:{}", name, ty),
            TypedExpression::Binary(binary) => {
                write!(f, "({} {} {})", binary.left, binary.operator, binary.right)
            }
            TypedExpression::Unary(unary) => write!(f, "({}{})", unary.operator, unary.operand),
            TypedExpression::Block(block) => write!(f, "{}", block),
            TypedExpression::Grouping(expr) => write!(f, "({})", expr),
        }
    }
}

impl fmt::Display for TypedBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ ")?;
        for stmt in &self.statements {
            write!(f, "{}; ", stmt)?;
        }
        write!(f, "}}")
    }
}
