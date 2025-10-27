use crate::ast::*;
use crate::token::{Token, TokenKind};
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub token: Token,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at {}:{}: {} (found '{}')",
            self.token.line, self.token.column, self.message, self.token.lexeme
        )
    }
}

impl std::error::Error for ParseError {}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(Program { statements })
    }

    // Statements

    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_tokens(&[TokenKind::Let, TokenKind::Fix]) {
            self.variable_declaration()
        } else if self.match_tokens(&[TokenKind::Yield]) {
            self.yield_statement()
        } else {
            self.expression_statement()
        }
    }

    fn variable_declaration(&mut self) -> Result<Statement, ParseError> {
        let mutable = self.previous().kind == TokenKind::Let;

        let name = self.consume(TokenKind::Identifier, "Expected variable name")?;
        let name_str = name.lexeme.clone();

        let type_annotation = if self.match_tokens(&[TokenKind::Colon]) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(TokenKind::Assign, "Expected '=' after variable name")?;

        let initializer = self.expression()?;

        Ok(Statement::VariableDeclaration(VariableDeclaration {
            mutable,
            name: name_str,
            type_annotation,
            initializer,
        }))
    }

    fn yield_statement(&mut self) -> Result<Statement, ParseError> {
        let value = self.expression()?;
        Ok(Statement::Yield(YieldStatement { value }))
    }

    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        Ok(Statement::Expression(expr))
    }

    // Expressions (in precedence order)

    fn expression(&mut self) -> Result<Expression, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.logical_or()?;

        while self.match_tokens(&[
            TokenKind::Assign,
            TokenKind::PlusAssign,
            TokenKind::MinusAssign,
            TokenKind::MultAssign,
            TokenKind::DivAssign,
            TokenKind::ModAssign,
            TokenKind::AndAssign,
            TokenKind::OrAssign,
            TokenKind::XorAssign,
            TokenKind::LShiftAssign,
            TokenKind::RShiftAssign,
        ]) {
            let operator = self.token_to_binary_op(&self.previous())?;
            let right = self.logical_or()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    fn logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.logical_and()?;

        while self.match_tokens(&[TokenKind::LOr]) {
            let right = self.logical_and()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator: BinaryOp::LogicalOr,
                right,
            }));
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.bitwise_or()?;

        while self.match_tokens(&[TokenKind::LAnd]) {
            let right = self.bitwise_or()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator: BinaryOp::LogicalAnd,
                right,
            }));
        }

        Ok(expr)
    }

    fn bitwise_or(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.bitwise_xor()?;

        while self.match_tokens(&[TokenKind::Or]) {
            let right = self.bitwise_xor()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator: BinaryOp::BitwiseOr,
                right,
            }));
        }

        Ok(expr)
    }

    fn bitwise_xor(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.bitwise_and()?;

        while self.match_tokens(&[TokenKind::Xor]) {
            let right = self.bitwise_and()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator: BinaryOp::BitwiseXor,
                right,
            }));
        }

        Ok(expr)
    }

    fn bitwise_and(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.equality()?;

        while self.match_tokens(&[TokenKind::And]) {
            let right = self.equality()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator: BinaryOp::BitwiseAnd,
                right,
            }));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.relational()?;

        while self.match_tokens(&[TokenKind::Eq, TokenKind::Ne]) {
            let operator = self.token_to_binary_op(&self.previous())?;
            let right = self.relational()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    fn relational(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.bitwise_shift()?;

        while self.match_tokens(&[TokenKind::Lt, TokenKind::Le, TokenKind::Gt, TokenKind::Ge]) {
            let operator = self.token_to_binary_op(&self.previous())?;
            let right = self.bitwise_shift()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    fn bitwise_shift(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.additive()?;

        while self.match_tokens(&[TokenKind::LShift, TokenKind::RShift]) {
            let operator = self.token_to_binary_op(&self.previous())?;
            let right = self.additive()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    fn additive(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.multiplicative()?;

        while self.match_tokens(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.token_to_binary_op(&self.previous())?;
            let right = self.multiplicative()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    fn multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[TokenKind::Mult, TokenKind::Div, TokenKind::Mod]) {
            let operator = self.token_to_binary_op(&self.previous())?;
            let right = self.unary()?;
            expr = Expression::Binary(Box::new(BinaryExpr {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, ParseError> {
        if self.match_tokens(&[TokenKind::Plus, TokenKind::Minus, TokenKind::Not, TokenKind::BNot]) {
            let operator = self.token_to_unary_op(&self.previous())?;
            let operand = self.unary()?;
            return Ok(Expression::Unary(Box::new(UnaryExpr { operator, operand })));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        // Literals
        if self.match_tokens(&[TokenKind::IntegerLiteral]) {
            return Ok(Expression::Integer(self.previous().lexeme.clone()));
        }

        if self.match_tokens(&[TokenKind::FloatLiteral]) {
            return Ok(Expression::Float(self.previous().lexeme.clone()));
        }

        if self.match_tokens(&[TokenKind::BooleanLiteral]) {
            let value = self.previous().lexeme == "true";
            return Ok(Expression::Boolean(value));
        }

        if self.match_tokens(&[TokenKind::VoidLiteral]) {
            return Ok(Expression::Void);
        }

        // Identifier
        if self.match_tokens(&[TokenKind::Identifier]) {
            return Ok(Expression::Identifier(self.previous().lexeme.clone()));
        }

        // Block
        if self.match_tokens(&[TokenKind::LBrace]) {
            return self.block();
        }

        // Grouping
        if self.match_tokens(&[TokenKind::LParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RParen, "Expected ')' after expression")?;
            return Ok(Expression::Grouping(Box::new(expr)));
        }

        Err(ParseError {
            message: "Expected expression".to_string(),
            token: self.peek().clone(),
        })
    }

    fn block(&mut self) -> Result<Expression, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }

        self.consume(TokenKind::RBrace, "Expected '}' after block")?;

        Ok(Expression::Block(Block { statements }))
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let token = self.peek().clone();

        let type_name = match &token.kind {
            TokenKind::Identifier => &token.lexeme,
            _ => {
                return Err(ParseError {
                    message: "Expected type name".to_string(),
                    token,
                })
            }
        };

        let ty = match type_name.as_str() {
            "I8" => Type::I8,
            "I16" => Type::I16,
            "I32" => Type::I32,
            "I64" => Type::I64,
            "U8" => Type::U8,
            "U16" => Type::U16,
            "U32" => Type::U32,
            "U64" => Type::U64,
            "F16" => Type::F16,
            "F32" => Type::F32,
            "F64" => Type::F64,
            "Bool" => Type::Bool,
            "Void" => Type::Void,
            _ => {
                return Err(ParseError {
                    message: format!("Unknown type: {}", type_name),
                    token,
                })
            }
        };

        self.advance();
        Ok(ty)
    }

    // Helper methods

    fn token_to_binary_op(&self, token: &Token) -> Result<BinaryOp, ParseError> {
        match token.kind {
            TokenKind::Assign => Ok(BinaryOp::Assign),
            TokenKind::PlusAssign => Ok(BinaryOp::PlusAssign),
            TokenKind::MinusAssign => Ok(BinaryOp::MinusAssign),
            TokenKind::MultAssign => Ok(BinaryOp::MultAssign),
            TokenKind::DivAssign => Ok(BinaryOp::DivAssign),
            TokenKind::ModAssign => Ok(BinaryOp::ModAssign),
            TokenKind::AndAssign => Ok(BinaryOp::AndAssign),
            TokenKind::OrAssign => Ok(BinaryOp::OrAssign),
            TokenKind::XorAssign => Ok(BinaryOp::XorAssign),
            TokenKind::LShiftAssign => Ok(BinaryOp::LShiftAssign),
            TokenKind::RShiftAssign => Ok(BinaryOp::RShiftAssign),
            TokenKind::Eq => Ok(BinaryOp::Equal),
            TokenKind::Ne => Ok(BinaryOp::NotEqual),
            TokenKind::Lt => Ok(BinaryOp::LessThan),
            TokenKind::Le => Ok(BinaryOp::LessEqual),
            TokenKind::Gt => Ok(BinaryOp::GreaterThan),
            TokenKind::Ge => Ok(BinaryOp::GreaterEqual),
            TokenKind::LShift => Ok(BinaryOp::LeftShift),
            TokenKind::RShift => Ok(BinaryOp::RightShift),
            TokenKind::Plus => Ok(BinaryOp::Add),
            TokenKind::Minus => Ok(BinaryOp::Subtract),
            TokenKind::Mult => Ok(BinaryOp::Multiply),
            TokenKind::Div => Ok(BinaryOp::Divide),
            TokenKind::Mod => Ok(BinaryOp::Modulo),
            _ => Err(ParseError {
                message: format!("Expected binary operator, found {:?}", token.kind),
                token: token.clone(),
            }),
        }
    }

    fn token_to_unary_op(&self, token: &Token) -> Result<UnaryOp, ParseError> {
        match token.kind {
            TokenKind::Plus => Ok(UnaryOp::Plus),
            TokenKind::Minus => Ok(UnaryOp::Minus),
            TokenKind::Not => Ok(UnaryOp::LogicalNot),
            TokenKind::BNot => Ok(UnaryOp::BitwiseNot),
            _ => Err(ParseError {
                message: format!("Expected unary operator, found {:?}", token.kind),
                token: token.clone(),
            }),
        }
    }

    fn match_tokens(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.peek().kind == kind
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, ParseError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                message: message.to_string(),
                token: self.peek().clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    // ============================================================================
    // Test Helper Functions
    // ============================================================================

    /// Parse a source string into a Program AST
    fn parse(source: &str) -> Result<Program, ParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    /// Parse source and extract the first statement
    fn parse_first_stmt(source: &str) -> Statement {
        let program = parse(source).unwrap();
        assert!(!program.statements.is_empty(), "Program has no statements");
        program.statements[0].clone()
    }

    /// Parse source and extract the first expression statement
    fn parse_first_expr(source: &str) -> Expression {
        match parse_first_stmt(source) {
            Statement::Expression(expr) => expr,
            _ => panic!("Expected expression statement"),
        }
    }

    /// Parse source and extract the first variable declaration
    fn parse_first_var_decl(source: &str) -> VariableDeclaration {
        match parse_first_stmt(source) {
            Statement::VariableDeclaration(decl) => decl,
            _ => panic!("Expected variable declaration"),
        }
    }

    /// Parse source and extract the first yield statement
    fn parse_first_yield(source: &str) -> YieldStatement {
        match parse_first_stmt(source) {
            Statement::Yield(yield_stmt) => yield_stmt,
            _ => panic!("Expected yield statement"),
        }
    }

    /// Unwrap a Binary expression or panic
    fn unwrap_binary(expr: Expression) -> Box<BinaryExpr> {
        match expr {
            Expression::Binary(binary) => binary,
            _ => panic!("Expected binary expression, got: {:?}", expr),
        }
    }

    /// Unwrap a Unary expression or panic
    fn unwrap_unary(expr: Expression) -> Box<UnaryExpr> {
        match expr {
            Expression::Unary(unary) => unary,
            _ => panic!("Expected unary expression, got: {:?}", expr),
        }
    }

    /// Unwrap a Block expression or panic
    fn unwrap_block(expr: Expression) -> Block {
        match expr {
            Expression::Block(block) => block,
            _ => panic!("Expected block expression, got: {:?}", expr),
        }
    }

    /// Unwrap a Grouping expression or panic
    fn unwrap_grouping(expr: Expression) -> Box<Expression> {
        match expr {
            Expression::Grouping(grouped) => grouped,
            _ => panic!("Expected grouping expression, got: {:?}", expr),
        }
    }

    #[test]
    fn test_parse_simple_expression() {
        let program = parse("42").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_binary_expression() {
        let program = parse("1 + 2").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_variable_declaration() {
        let decl = parse_first_var_decl("let x = 42");
        assert_eq!(decl.name, "x");
        assert!(decl.mutable);
    }

    #[test]
    fn test_parse_integer_literal() {
        match parse_first_expr("42") {
            Expression::Integer(val) => assert_eq!(val, "42"),
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_parse_float_literal() {
        match parse_first_expr("3.14") {
            Expression::Float(val) => assert_eq!(val, "3.14"),
            _ => panic!("Expected float literal"),
        }
    }

    #[test]
    fn test_parse_boolean_literal() {
        match parse_first_expr("true") {
            Expression::Boolean(val) => assert!(val),
            _ => panic!("Expected boolean literal"),
        }
    }

    #[test]
    fn test_parse_void_literal() {
        match parse_first_expr("void") {
            Expression::Void => {}
            _ => panic!("Expected void literal"),
        }
    }

    #[test]
    fn test_parse_identifier() {
        match parse_first_expr("myVar") {
            Expression::Identifier(name) => assert_eq!(name, "myVar"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_parse_operator_precedence() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let binary = unwrap_binary(parse_first_expr("1 + 2 * 3"));
        assert_eq!(binary.operator, BinaryOp::Add);
        // Right side should be multiplication
        let right_binary = unwrap_binary(binary.right);
        assert_eq!(right_binary.operator, BinaryOp::Multiply);
    }

    #[test]
    fn test_parse_grouping_overrides_precedence() {
        // (1 + 2) * 3 should parse as (1 + 2) * 3
        let binary = unwrap_binary(parse_first_expr("(1 + 2) * 3"));
        assert_eq!(binary.operator, BinaryOp::Multiply);
        // Left side should be grouping with addition
        let grouped = unwrap_grouping(binary.left);
        let left_binary = unwrap_binary(*grouped);
        assert_eq!(left_binary.operator, BinaryOp::Add);
    }

    #[test]
    fn test_parse_logical_operators() {
        // Should parse as (true && false) || true
        let binary = unwrap_binary(parse_first_expr("true && false || true"));
        assert_eq!(binary.operator, BinaryOp::LogicalOr);
    }

    #[test]
    fn test_parse_bitwise_operators() {
        // Should respect bitwise operator precedence
        let binary = unwrap_binary(parse_first_expr("1 & 2 | 3 ^ 4"));
        assert_eq!(binary.operator, BinaryOp::BitwiseOr);
    }

    #[test]
    fn test_parse_comparison_operators() {
        let binary = unwrap_binary(parse_first_expr("1 < 2"));
        assert_eq!(binary.operator, BinaryOp::LessThan);
    }

    #[test]
    fn test_parse_equality_operators() {
        let binary = unwrap_binary(parse_first_expr("x == y"));
        assert_eq!(binary.operator, BinaryOp::Equal);
    }

    #[test]
    fn test_parse_shift_operators() {
        let binary = unwrap_binary(parse_first_expr("1 << 2"));
        assert_eq!(binary.operator, BinaryOp::LeftShift);
    }

    #[test]
    fn test_parse_unary_minus() {
        let unary = unwrap_unary(parse_first_expr("-42"));
        assert_eq!(unary.operator, UnaryOp::Minus);
        match unary.operand {
            Expression::Integer(val) => assert_eq!(val, "42"),
            _ => panic!("Expected integer operand"),
        }
    }

    #[test]
    fn test_parse_unary_not() {
        let unary = unwrap_unary(parse_first_expr("!true"));
        assert_eq!(unary.operator, UnaryOp::LogicalNot);
    }

    #[test]
    fn test_parse_unary_bitwise_not() {
        let unary = unwrap_unary(parse_first_expr("~42"));
        assert_eq!(unary.operator, UnaryOp::BitwiseNot);
    }

    #[test]
    fn test_parse_nested_unary() {
        let outer = unwrap_unary(parse_first_expr("--42"));
        assert_eq!(outer.operator, UnaryOp::Minus);
        let inner = unwrap_unary(outer.operand);
        assert_eq!(inner.operator, UnaryOp::Minus);
    }

    #[test]
    fn test_parse_assignment() {
        let binary = unwrap_binary(parse_first_expr("x = 42"));
        assert_eq!(binary.operator, BinaryOp::Assign);
        match binary.left {
            Expression::Identifier(name) => assert_eq!(name, "x"),
            _ => panic!("Expected identifier on left"),
        }
    }

    #[test]
    fn test_parse_compound_assignment() {
        let binary = unwrap_binary(parse_first_expr("x += 5"));
        assert_eq!(binary.operator, BinaryOp::PlusAssign);
    }

    #[test]
    fn test_parse_chained_assignment() {
        // x = y = 5 should parse as (x = y) = 5 with left associativity
        let binary = unwrap_binary(parse_first_expr("x = y = 5"));
        assert_eq!(binary.operator, BinaryOp::Assign);
    }

    #[test]
    fn test_parse_let_declaration() {
        let decl = parse_first_var_decl("let x = 42");
        assert_eq!(decl.name, "x");
        assert!(decl.mutable);
        assert!(decl.type_annotation.is_none());
    }

    #[test]
    fn test_parse_fix_declaration() {
        let decl = parse_first_var_decl("fix x = 42");
        assert_eq!(decl.name, "x");
        assert!(!decl.mutable);
    }

    #[test]
    fn test_parse_declaration_with_type() {
        let decl = parse_first_var_decl("let x: I32 = 42");
        assert_eq!(decl.name, "x");
        assert!(decl.mutable);
        assert_eq!(decl.type_annotation, Some(Type::I32));
    }

    #[test]
    fn test_parse_declaration_with_expression() {
        let decl = parse_first_var_decl("let x = 1 + 2");
        assert_eq!(decl.name, "x");
        let binary = unwrap_binary(decl.initializer);
        assert_eq!(binary.operator, BinaryOp::Add);
    }

    #[test]
    fn test_parse_empty_block() {
        let block = unwrap_block(parse_first_expr("{}"));
        assert_eq!(block.statements.len(), 0);
    }

    #[test]
    fn test_parse_block_with_statements() {
        let block = unwrap_block(parse_first_expr("{ let x = 5\nlet y = 10 }"));
        assert_eq!(block.statements.len(), 2);
    }

    #[test]
    fn test_parse_block_with_yield() {
        let block = unwrap_block(parse_first_expr("{ let x = 5\n<- x * 2 }"));
        assert_eq!(block.statements.len(), 2);
        match &block.statements[1] {
            Statement::Yield(_) => {}
            _ => panic!("Expected yield statement"),
        }
    }

    #[test]
    fn test_parse_nested_blocks() {
        let outer = unwrap_block(parse_first_expr("{ { 42 } }"));
        assert_eq!(outer.statements.len(), 1);
        match &outer.statements[0] {
            Statement::Expression(Expression::Block(_)) => {}
            _ => panic!("Expected nested block"),
        }
    }

    #[test]
    fn test_parse_yield_statement() {
        let yield_stmt = parse_first_yield("<- 42");
        match yield_stmt.value {
            Expression::Integer(val) => assert_eq!(val, "42"),
            _ => panic!("Expected integer in yield"),
        }
    }

    #[test]
    fn test_parse_yield_with_expression() {
        let yield_stmt = parse_first_yield("<- x + y");
        let binary = unwrap_binary(yield_stmt.value);
        assert_eq!(binary.operator, BinaryOp::Add);
    }

    #[test]
    fn test_parse_multiple_statements() {
        let program = parse("let x = 5\nlet y = 10\nx + y").unwrap();
        assert_eq!(program.statements.len(), 3);
    }

    #[test]
    fn test_parse_complex_expression() {
        // Just verify it parses without error
        let program = parse("1 + 2 * 3 - 4 / 2").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_error_missing_expression() {
        assert!(parse("let x =").is_err());
    }

    #[test]
    fn test_parse_error_missing_equals() {
        assert!(parse("let x 42").is_err());
    }

    #[test]
    fn test_parse_error_unclosed_paren() {
        assert!(parse("(1 + 2").is_err());
    }

    #[test]
    fn test_parse_error_unclosed_block() {
        assert!(parse("{ let x = 5").is_err());
    }
}
