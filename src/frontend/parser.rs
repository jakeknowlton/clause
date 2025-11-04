use crate::frontend::ast::*;
use crate::frontend::token::{Token, TokenKind};
use crate::error::{ParseError, ParseErrorKind, Span};

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
        } else if self.match_tokens(&[TokenKind::Break]) {
            self.break_statement()
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

    fn break_statement(&mut self) -> Result<Statement, ParseError> {
        let value = self.expression()?;
        Ok(Statement::Break(BreakStatement { value }))
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
        if self.match_tokens(&[
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Not,
            TokenKind::BNot,
        ]) {
            let operator = self.token_to_unary_op(&self.previous())?;
            let operand = self.unary()?;
            return Ok(Expression::Unary(Box::new(UnaryExpr { operator, operand })));
        }

        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.primary()?;

        // Handle chained indexing: arr[i][j]
        while self.match_tokens(&[TokenKind::LBracket]) {
            let index = self.expression()?;
            self.consume(TokenKind::RBracket, "Expected ']' after array index")?;
            expr = Expression::Index(Box::new(IndexExpr {
                array: expr,
                index,
            }));
        }

        Ok(expr)
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

        // If expression
        if self.match_tokens(&[TokenKind::If]) {
            return self.if_expression();
        }

        // While loop
        if self.match_tokens(&[TokenKind::While]) {
            return self.while_expression();
        }

        // Block
        if self.match_tokens(&[TokenKind::LBrace]) {
            return self.block();
        }

        // Array literal
        if self.match_tokens(&[TokenKind::LBracket]) {
            return self.parse_array_literal();
        }

        // Grouping
        if self.match_tokens(&[TokenKind::LParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RParen, "Expected ')' after expression")?;
            return Ok(Expression::Grouping(Box::new(expr)));
        }

        let token = self.peek();
        Err(ParseError::new(
            ParseErrorKind::InvalidSyntax,
            "Expected expression".to_string(),
        )
        .with_span(Span::new(token.line, token.column, token.lexeme.len())))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, ParseError> {
        let mut elements = Vec::new();

        // Handle empty array []
        if !self.check(&TokenKind::RBracket) {
            elements.push(self.expression()?);

            while self.match_tokens(&[TokenKind::Comma]) {
                elements.push(self.expression()?);
            }
        }

        self.consume(TokenKind::RBracket, "Expected ']' after array elements")?;

        Ok(Expression::ArrayLiteral(elements))
    }

    fn block(&mut self) -> Result<Expression, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }

        self.consume(TokenKind::RBrace, "Expected '}' after block")?;

        Ok(Expression::Block(Block { statements }))
    }

    fn if_expression(&mut self) -> Result<Expression, ParseError> {
        // Parse condition (no parentheses required)
        let condition = self.expression()?;

        // Parse then block (required)
        self.consume(TokenKind::LBrace, "Expected '{' after if condition")?;
        let then_block = match self.block()? {
            Expression::Block(block) => block,
            _ => unreachable!(),
        };

        // Parse else-if and else branches
        let mut else_ifs = Vec::new();
        let mut else_block = None;

        while self.match_tokens(&[TokenKind::Else]) {
            if self.match_tokens(&[TokenKind::If]) {
                // else if branch
                let else_if_condition = self.expression()?;
                self.consume(TokenKind::LBrace, "Expected '{' after else if condition")?;
                let else_if_block = match self.block()? {
                    Expression::Block(block) => block,
                    _ => unreachable!(),
                };
                else_ifs.push((else_if_condition, else_if_block));
            } else {
                // else branch (final)
                self.consume(TokenKind::LBrace, "Expected '{' after else")?;
                else_block = Some(match self.block()? {
                    Expression::Block(block) => block,
                    _ => unreachable!(),
                });
                break; // else must be last
            }
        }

        Ok(Expression::If(Box::new(IfExpr {
            condition,
            then_block,
            else_ifs,
            else_block,
        })))
    }

    fn while_expression(&mut self) -> Result<Expression, ParseError> {
        // Parse condition (no parentheses required)
        let condition = self.expression()?;

        // Parse body block (required)
        self.consume(TokenKind::LBrace, "Expected '{' after while condition")?;
        let body = match self.block()? {
            Expression::Block(block) => block,
            _ => unreachable!(),
        };

        // Parse optional else block
        let else_block = if self.match_tokens(&[TokenKind::Else]) {
            self.consume(TokenKind::LBrace, "Expected '{' after else")?;
            Some(match self.block()? {
                Expression::Block(block) => block,
                _ => unreachable!(),
            })
        } else {
            None
        };

        Ok(Expression::While(Box::new(WhileExpr { condition, body, else_block })))
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        // Check for array type [Type, N]
        if self.check(&TokenKind::LBracket) {
            self.advance(); // consume '['

            let element_type = self.parse_type()?; // recursively parse element type

            self.consume(TokenKind::Comma, "Expected ',' after array element type")?;

            let size_token = self.consume(TokenKind::IntegerLiteral, "Expected integer literal for array size")?;
            let size = size_token.lexeme.parse::<usize>().map_err(|_| {
                ParseError::new(
                    ParseErrorKind::InvalidSyntax,
                    format!("Invalid array size: {}", size_token.lexeme),
                )
                .with_span(Span::new(size_token.line, size_token.column, size_token.lexeme.len()))
            })?;

            self.consume(TokenKind::RBracket, "Expected ']' after array size")?;

            return Ok(Type::Array(Box::new(element_type), size));
        }

        let token = self.peek().clone();

        let type_name = match &token.kind {
            TokenKind::Identifier => &token.lexeme,
            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::ExpectedToken,
                    "Expected type name".to_string(),
                )
                .with_span(Span::new(token.line, token.column, token.lexeme.len())));
            }
        };

        let ty = match type_name.as_str() {
            // Float types
            "F32" => Type::F32,
            "F64" => Type::F64,
            "Bool" => Type::Bool,
            "Void" => Type::Void,
            _ => {
                if type_name.starts_with('I') {
                    let bits_str = &type_name[1..];
                    match bits_str.parse::<u8>() {
                        Ok(bits) if bits >= 1 && bits <= 128 => Type::Signed(bits),
                        Ok(bits) => {
                            return Err(ParseError::new(
                                ParseErrorKind::InvalidSyntax,
                                format!(
                                    "Integer bitwidth must be between 1 and 128, found {}",
                                    bits
                                ),
                            )
                            .with_span(Span::new(token.line, token.column, token.lexeme.len())));
                        }
                        Err(_) => {
                            return Err(ParseError::new(
                                ParseErrorKind::InvalidSyntax,
                                format!("Invalid signed integer type: {}", type_name),
                            )
                            .with_span(Span::new(token.line, token.column, token.lexeme.len())));
                        }
                    }
                } else if type_name.starts_with('U') {
                    let bits_str = &type_name[1..];
                    match bits_str.parse::<u8>() {
                        Ok(bits) if bits >= 1 && bits <= 128 => Type::Unsigned(bits),
                        Ok(bits) => {
                            return Err(ParseError::new(
                                ParseErrorKind::InvalidSyntax,
                                format!(
                                    "Integer bitwidth must be between 1 and 128, found {}",
                                    bits
                                ),
                            )
                            .with_span(Span::new(token.line, token.column, token.lexeme.len())));
                        }
                        Err(_) => {
                            return Err(ParseError::new(
                                ParseErrorKind::InvalidSyntax,
                                format!("Invalid unsigned integer type: {}", type_name),
                            )
                            .with_span(Span::new(token.line, token.column, token.lexeme.len())));
                        }
                    }
                } else {
                    return Err(ParseError::new(
                        ParseErrorKind::InvalidSyntax,
                        format!("Unknown type: {}", type_name),
                    )
                    .with_span(Span::new(token.line, token.column, token.lexeme.len())));
                }
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
            _ => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("Expected binary operator, found {:?}", token.kind),
            )
            .with_span(Span::new(token.line, token.column, token.lexeme.len()))),
        }
    }

    fn token_to_unary_op(&self, token: &Token) -> Result<UnaryOp, ParseError> {
        match token.kind {
            TokenKind::Plus => Ok(UnaryOp::Plus),
            TokenKind::Minus => Ok(UnaryOp::Minus),
            TokenKind::Not => Ok(UnaryOp::LogicalNot),
            TokenKind::BNot => Ok(UnaryOp::BitwiseNot),
            _ => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("Expected unary operator, found {:?}", token.kind),
            )
            .with_span(Span::new(token.line, token.column, token.lexeme.len()))),
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
            let token = self.peek();
            Err(ParseError::new(
                ParseErrorKind::ExpectedToken,
                message.to_string(),
            )
            .with_span(Span::new(token.line, token.column, token.lexeme.len())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::lexer::Lexer;

    // ============================================================================
    // Test Helper Functions
    // ============================================================================

    fn parse(source: &str) -> Result<Program, ParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    fn parse_first_stmt(source: &str) -> Statement {
        let program = parse(source).unwrap();
        assert!(!program.statements.is_empty(), "Program has no statements");
        program.statements[0].clone()
    }

    fn parse_first_expr(source: &str) -> Expression {
        match parse_first_stmt(source) {
            Statement::Expression(expr) => expr,
            _ => panic!("Expected expression statement"),
        }
    }

    fn parse_first_var_decl(source: &str) -> VariableDeclaration {
        match parse_first_stmt(source) {
            Statement::VariableDeclaration(decl) => decl,
            _ => panic!("Expected variable declaration"),
        }
    }

    fn parse_first_yield(source: &str) -> YieldStatement {
        match parse_first_stmt(source) {
            Statement::Yield(yield_stmt) => yield_stmt,
            _ => panic!("Expected yield statement"),
        }
    }

    fn unwrap_binary(expr: Expression) -> Box<BinaryExpr> {
        match expr {
            Expression::Binary(binary) => binary,
            _ => panic!("Expected binary expression, got: {:?}", expr),
        }
    }

    fn unwrap_unary(expr: Expression) -> Box<UnaryExpr> {
        match expr {
            Expression::Unary(unary) => unary,
            _ => panic!("Expected unary expression, got: {:?}", expr),
        }
    }

    fn unwrap_block(expr: Expression) -> Block {
        match expr {
            Expression::Block(block) => block,
            _ => panic!("Expected block expression, got: {:?}", expr),
        }
    }

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
        assert_eq!(decl.type_annotation, Some(Type::Signed(32)));
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

    // ============================================================================
    // Array Parsing Tests
    // ============================================================================

    #[test]
    fn test_parse_array_literal_empty() {
        match parse_first_expr("[]") {
            Expression::ArrayLiteral(elements) => {
                assert_eq!(elements.len(), 0);
            }
            _ => panic!("Expected array literal"),
        }
    }

    #[test]
    fn test_parse_array_literal_single_element() {
        match parse_first_expr("[42]") {
            Expression::ArrayLiteral(elements) => {
                assert_eq!(elements.len(), 1);
                match &elements[0] {
                    Expression::Integer(val) => assert_eq!(val, "42"),
                    _ => panic!("Expected integer element"),
                }
            }
            _ => panic!("Expected array literal"),
        }
    }

    #[test]
    fn test_parse_array_literal_multiple_elements() {
        match parse_first_expr("[1, 2, 3, 4, 5]") {
            Expression::ArrayLiteral(elements) => {
                assert_eq!(elements.len(), 5);
                match &elements[0] {
                    Expression::Integer(val) => assert_eq!(val, "1"),
                    _ => panic!("Expected integer element"),
                }
                match &elements[4] {
                    Expression::Integer(val) => assert_eq!(val, "5"),
                    _ => panic!("Expected integer element"),
                }
            }
            _ => panic!("Expected array literal"),
        }
    }

    #[test]
    fn test_parse_array_literal_with_expressions() {
        match parse_first_expr("[1 + 2, 3 * 4, x]") {
            Expression::ArrayLiteral(elements) => {
                assert_eq!(elements.len(), 3);
                // First element should be a binary expression
                match &elements[0] {
                    Expression::Binary(_) => {}
                    _ => panic!("Expected binary expression"),
                }
                // Third element should be identifier
                match &elements[2] {
                    Expression::Identifier(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier"),
                }
            }
            _ => panic!("Expected array literal"),
        }
    }

    #[test]
    fn test_parse_nested_array_literal() {
        match parse_first_expr("[[1, 2], [3, 4]]") {
            Expression::ArrayLiteral(outer_elements) => {
                assert_eq!(outer_elements.len(), 2);
                match &outer_elements[0] {
                    Expression::ArrayLiteral(inner_elements) => {
                        assert_eq!(inner_elements.len(), 2);
                    }
                    _ => panic!("Expected nested array literal"),
                }
            }
            _ => panic!("Expected array literal"),
        }
    }

    #[test]
    fn test_parse_array_type() {
        let decl = parse_first_var_decl("let x: [I32, 5] = [1, 2, 3, 4, 5]");
        assert_eq!(decl.name, "x");
        match decl.type_annotation {
            Some(Type::Array(elem_type, size)) => {
                assert_eq!(*elem_type, Type::Signed(32));
                assert_eq!(size, 5);
            }
            _ => panic!("Expected array type annotation"),
        }
    }

    #[test]
    fn test_parse_nested_array_type() {
        let decl = parse_first_var_decl("let x: [[I32, 2], 3] = [[1, 2], [3, 4], [5, 6]]");
        match decl.type_annotation {
            Some(Type::Array(elem_type, size)) => {
                assert_eq!(size, 3);
                match *elem_type {
                    Type::Array(inner_elem_type, inner_size) => {
                        assert_eq!(*inner_elem_type, Type::Signed(32));
                        assert_eq!(inner_size, 2);
                    }
                    _ => panic!("Expected nested array type"),
                }
            }
            _ => panic!("Expected array type annotation"),
        }
    }

    #[test]
    fn test_parse_array_indexing() {
        match parse_first_expr("arr[0]") {
            Expression::Index(index_expr) => {
                match index_expr.array {
                    Expression::Identifier(name) => assert_eq!(name, "arr"),
                    _ => panic!("Expected identifier for array"),
                }
                match index_expr.index {
                    Expression::Integer(val) => assert_eq!(val, "0"),
                    _ => panic!("Expected integer for index"),
                }
            }
            _ => panic!("Expected index expression"),
        }
    }

    #[test]
    fn test_parse_array_indexing_with_variable() {
        match parse_first_expr("arr[i]") {
            Expression::Index(index_expr) => {
                match index_expr.array {
                    Expression::Identifier(name) => assert_eq!(name, "arr"),
                    _ => panic!("Expected identifier for array"),
                }
                match index_expr.index {
                    Expression::Identifier(name) => assert_eq!(name, "i"),
                    _ => panic!("Expected identifier for index"),
                }
            }
            _ => panic!("Expected index expression"),
        }
    }

    #[test]
    fn test_parse_array_indexing_with_expression() {
        match parse_first_expr("arr[i + 1]") {
            Expression::Index(index_expr) => {
                match index_expr.index {
                    Expression::Binary(_) => {}
                    _ => panic!("Expected binary expression for index"),
                }
            }
            _ => panic!("Expected index expression"),
        }
    }

    #[test]
    fn test_parse_chained_indexing() {
        match parse_first_expr("matrix[i][j]") {
            Expression::Index(outer_index) => {
                // Outer index should have an Index as its array
                match outer_index.array {
                    Expression::Index(inner_index) => {
                        match inner_index.array {
                            Expression::Identifier(name) => assert_eq!(name, "matrix"),
                            _ => panic!("Expected identifier for innermost array"),
                        }
                        match inner_index.index {
                            Expression::Identifier(name) => assert_eq!(name, "i"),
                            _ => panic!("Expected identifier for first index"),
                        }
                    }
                    _ => panic!("Expected inner index expression"),
                }
                match outer_index.index {
                    Expression::Identifier(name) => assert_eq!(name, "j"),
                    _ => panic!("Expected identifier for second index"),
                }
            }
            _ => panic!("Expected index expression"),
        }
    }

    #[test]
    fn test_parse_array_indexing_assignment() {
        let program = parse("arr[0] = 42").unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Expression(Expression::Binary(binary)) => {
                assert_eq!(binary.operator, BinaryOp::Assign);
                match &binary.left {
                    Expression::Index(_) => {}
                    _ => panic!("Expected index expression on left side"),
                }
            }
            _ => panic!("Expected assignment expression"),
        }
    }

    #[test]
    fn test_parse_array_literal_trailing_comma() {
        // Trailing comma should not be allowed
        assert!(parse("[1, 2, 3,]").is_err());
    }

    #[test]
    fn test_parse_error_unclosed_bracket() {
        assert!(parse("[1, 2, 3").is_err());
    }

    #[test]
    fn test_parse_error_missing_comma() {
        assert!(parse("[1 2 3]").is_err());
    }

    #[test]
    fn test_parse_error_unclosed_index() {
        assert!(parse("arr[0").is_err());
    }
}
