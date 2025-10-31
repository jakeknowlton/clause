use crate::ast::{
    BinaryExpr, BinaryOp, Block, Expression, Program, Statement, UnaryExpr, UnaryOp,
    VariableDeclaration,
};
use crate::environment::Environment;
use crate::value::Value;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn with_environment(environment: Environment) -> Self {
        Interpreter { environment }
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    pub fn execute_program(&mut self, program: &Program) -> Result<Value, String> {
        let mut last_value = Value::Void;
        for statement in &program.statements {
            last_value = self.execute_statement(statement)?;
        }
        Ok(last_value)
    }

    pub fn execute_statement(&mut self, statement: &Statement) -> Result<Value, String> {
        match statement {
            Statement::VariableDeclaration(var_decl) => self.execute_variable_declaration(var_decl),
            Statement::Yield(yield_stmt) => self.evaluate_expression(&yield_stmt.value),
            Statement::Expression(expr) => self.evaluate_expression(expr),
        }
    }

    fn execute_variable_declaration(
        &mut self,
        var_decl: &VariableDeclaration,
    ) -> Result<Value, String> {
        let mut value = self.evaluate_expression(&var_decl.initializer)?;

        // Type annotation checking and conversion (if present)
        if let Some(type_annotation) = &var_decl.type_annotation {
            value = self.check_type(&value, type_annotation)?;
        }

        let mutable = var_decl.mutable;
        self.environment
            .define(var_decl.name.clone(), value.clone(), mutable)?;

        Ok(value)
    }

    fn check_type(
        &self,
        value: &Value,
        type_annotation: &crate::ast::Type,
    ) -> Result<Value, String> {
        use crate::ast::Type;

        match (value, type_annotation) {
            (
                Value::SignedInt {
                    value: val,
                    bits: value_bits,
                },
                Type::Signed(type_bits),
            ) => {
                if value_bits == type_bits {
                    Ok(value.clone())
                } else {
                    // Convert to the specified bitwidth with validation
                    // TODO: Remove automatic conversion
                    Value::new_signed(*val, *type_bits)
                }
            }
            (
                Value::UnsignedInt {
                    value: val,
                    bits: value_bits,
                },
                Type::Unsigned(type_bits),
            ) => {
                if value_bits == type_bits {
                    Ok(value.clone())
                } else {
                    // Convert to the specified bitwidth with validation
                    // TODO: Remove automatic conversion
                    Value::new_unsigned(*val, *type_bits)
                }
            }
            (Value::Single(_), Type::F32) => Ok(value.clone()),
            (Value::Double(_), Type::F64) => Ok(value.clone()),
            (Value::Boolean(_), Type::Bool) => Ok(value.clone()),
            (Value::Void, Type::Void) => Ok(value.clone()),

            _ => Err(format!(
                "Type mismatch: expected {}, found {}",
                type_annotation,
                value.type_name()
            )),
        }
    }

    /// Evaluate an expression to a value
    pub fn evaluate_expression(&mut self, expression: &Expression) -> Result<Value, String> {
        match expression {
            Expression::Integer(s) => self.parse_integer(s),
            Expression::Float(s) => self.parse_float(s),
            Expression::Boolean(b) => Ok(Value::Boolean(*b)),
            Expression::Void => Ok(Value::Void),
            Expression::Identifier(name) => self.environment.get(name),
            Expression::Binary(binary) => self.evaluate_binary(binary),
            Expression::Unary(unary) => self.evaluate_unary(unary),
            Expression::Block(block) => self.evaluate_block(block),
            Expression::Grouping(expr) => self.evaluate_expression(expr),
        }
    }

    /// Parse an integer literal (handles all formats and underscores)
    /// Defaults to 64-bit signed integers for untyped literals
    fn parse_integer(&self, s: &str) -> Result<Value, String> {
        let s = s.replace('_', "");

        let result = if s.starts_with("0x") || s.starts_with("0X") {
            i128::from_str_radix(&s[2..], 16)
        } else if s.starts_with("0b") || s.starts_with("0B") {
            i128::from_str_radix(&s[2..], 2)
        } else if s.starts_with("0o") || s.starts_with("0O") {
            i128::from_str_radix(&s[2..], 8)
        } else {
            s.parse::<i128>()
        };

        let value = result.map_err(|e| format!("Invalid integer literal: {}", e))?;
        Value::new_signed(value, 64)
    }

    // Defaults to 64-bit float for untyped literals
    fn parse_float(&self, s: &str) -> Result<Value, String> {
        let s = s.replace('_', "");
        let value = s
            .parse::<f64>()
            .map_err(|e| format!("Invalid float literal: {}", e))?;
        Ok(Value::Double(value))
    }

    fn evaluate_binary(&mut self, binary: &BinaryExpr) -> Result<Value, String> {
        // Handle assignment operators
        if self.is_assignment_operator(&binary.operator) {
            return self.evaluate_assignment_binary(binary);
        }

        // Handle short-circuit evaluation for logical operators
        match binary.operator {
            BinaryOp::LogicalAnd => {
                let left = self.evaluate_expression(&binary.left)?;
                match left {
                    Value::Boolean(false) => Ok(Value::Boolean(false)),
                    Value::Boolean(true) => {
                        let right = self.evaluate_expression(&binary.right)?;
                        match right {
                            Value::Boolean(_) => Ok(right),
                            _ => Err(format!("Expected Boolean, found {}", right.type_name())),
                        }
                    }
                    _ => Err(format!("Expected Boolean, found {}", left.type_name())),
                }
            }
            BinaryOp::LogicalOr => {
                let left = self.evaluate_expression(&binary.left)?;
                match left {
                    Value::Boolean(true) => Ok(Value::Boolean(true)),
                    Value::Boolean(false) => {
                        let right = self.evaluate_expression(&binary.right)?;
                        match right {
                            Value::Boolean(_) => Ok(right),
                            _ => Err(format!("Expected Boolean, found {}", right.type_name())),
                        }
                    }
                    _ => Err(format!("Expected Boolean, found {}", left.type_name())),
                }
            }
            _ => {
                // For all other operators, evaluate both sides first
                let left = self.evaluate_expression(&binary.left)?;
                let right = self.evaluate_expression(&binary.right)?;
                self.apply_binary_operator(&binary.operator, &left, &right)
            }
        }
    }

    /// Check if an operator is an assignment operator
    fn is_assignment_operator(&self, op: &BinaryOp) -> bool {
        matches!(
            op,
            BinaryOp::Assign
                | BinaryOp::PlusAssign
                | BinaryOp::MinusAssign
                | BinaryOp::MultAssign
                | BinaryOp::DivAssign
                | BinaryOp::ModAssign
                | BinaryOp::AndAssign
                | BinaryOp::OrAssign
                | BinaryOp::XorAssign
                | BinaryOp::LShiftAssign
                | BinaryOp::RShiftAssign
        )
    }

    /// Evaluate an assignment expression
    fn evaluate_assignment_binary(&mut self, binary: &BinaryExpr) -> Result<Value, String> {
        // Get the target identifier
        let target = match &binary.left {
            Expression::Identifier(name) => name,
            _ => return Err("Invalid assignment target".to_string()),
        };

        // Evaluate the right-hand side
        let new_value = self.evaluate_expression(&binary.right)?;

        // Apply the assignment operator
        match binary.operator {
            BinaryOp::Assign => {
                self.environment.assign(target, new_value.clone())?;
                Ok(new_value)
            }
            BinaryOp::PlusAssign => {
                let current = self.environment.get(target)?;
                let result = current.add(&new_value)?;
                self.environment.assign(target, result.clone())?;
                Ok(result)
            }
            BinaryOp::MinusAssign => {
                let current = self.environment.get(target)?;
                let result = current.subtract(&new_value)?;
                self.environment.assign(target, result.clone())?;
                Ok(result)
            }
            BinaryOp::MultAssign => {
                let current = self.environment.get(target)?;
                let result = current.multiply(&new_value)?;
                self.environment.assign(target, result.clone())?;
                Ok(result)
            }
            BinaryOp::DivAssign => {
                let current = self.environment.get(target)?;
                let result = current.divide(&new_value)?;
                self.environment.assign(target, result.clone())?;
                Ok(result)
            }
            BinaryOp::ModAssign => {
                let current = self.environment.get(target)?;
                let result = current.modulo(&new_value)?;
                self.environment.assign(target, result.clone())?;
                Ok(result)
            }
            BinaryOp::AndAssign => {
                let current = self.environment.get(target)?;
                let result = current.bitwise_and(&new_value)?;
                self.environment.assign(target, result.clone())?;
                Ok(result)
            }
            BinaryOp::OrAssign => {
                let current = self.environment.get(target)?;
                let result = current.bitwise_or(&new_value)?;
                self.environment.assign(target, result.clone())?;
                Ok(result)
            }
            BinaryOp::XorAssign => {
                let current = self.environment.get(target)?;
                let result = current.bitwise_xor(&new_value)?;
                self.environment.assign(target, result.clone())?;
                Ok(result)
            }
            BinaryOp::LShiftAssign => {
                let current = self.environment.get(target)?;
                let result = current.left_shift(&new_value)?;
                self.environment.assign(target, result.clone())?;
                Ok(result)
            }
            BinaryOp::RShiftAssign => {
                let current = self.environment.get(target)?;
                let result = current.right_shift(&new_value)?;
                self.environment.assign(target, result.clone())?;
                Ok(result)
            }
            _ => unreachable!("Non-assignment operator passed to evaluate_assignment_binary"),
        }
    }

    /// Apply a binary operator to two values
    fn apply_binary_operator(
        &self,
        operator: &BinaryOp,
        left: &Value,
        right: &Value,
    ) -> Result<Value, String> {
        match operator {
            // Arithmetic
            BinaryOp::Add => left.add(right),
            BinaryOp::Subtract => left.subtract(right),
            BinaryOp::Multiply => left.multiply(right),
            BinaryOp::Divide => left.divide(right),
            BinaryOp::Modulo => left.modulo(right),

            // Bitwise
            BinaryOp::BitwiseAnd => left.bitwise_and(right),
            BinaryOp::BitwiseOr => left.bitwise_or(right),
            BinaryOp::BitwiseXor => left.bitwise_xor(right),
            BinaryOp::LeftShift => left.left_shift(right),
            BinaryOp::RightShift => left.right_shift(right),

            // Comparison
            BinaryOp::Equal => left.equals(right),
            BinaryOp::NotEqual => left.not_equals(right),
            BinaryOp::LessThan => left.less_than(right),
            BinaryOp::LessEqual => left.less_equal(right),
            BinaryOp::GreaterThan => left.greater_than(right),
            BinaryOp::GreaterEqual => left.greater_equal(right),

            // Logical operators already handled above
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                unreachable!("Logical operators should be handled in evaluate_binary")
            }

            // Assignment operators should not reach here
            _ => {
                unreachable!("Assignment operators should be handled in evaluate_assignment_binary")
            }
        }
    }

    /// Evaluate a unary expression
    fn evaluate_unary(&mut self, unary: &UnaryExpr) -> Result<Value, String> {
        let operand = self.evaluate_expression(&unary.operand)?;
        match unary.operator {
            UnaryOp::Plus => Ok(operand),
            UnaryOp::Minus => operand.negate(),
            UnaryOp::LogicalNot => operand.logical_not(),
            UnaryOp::BitwiseNot => operand.bitwise_not(),
        }
    }

    /// Evaluate a block expression
    fn evaluate_block(&mut self, block: &Block) -> Result<Value, String> {
        // Push a new scope for the block
        self.environment.push_scope();

        let mut result = Value::Void;

        // Execute each statement in the block
        for statement in &block.statements {
            result = self.execute_statement(statement)?;
        }

        // Pop the block's scope
        self.environment.pop_scope();

        Ok(result)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn interpret(source: &str) -> Result<Value, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| e.message)?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| e.message)?;
        let mut interpreter = Interpreter::new();
        interpreter.execute_program(&program)
    }

    #[test]
    fn test_integer_literals() {
        assert_eq!(
            interpret("42").unwrap(),
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
        assert_eq!(
            interpret("0xFF").unwrap(),
            Value::SignedInt {
                value: 255,
                bits: 64
            }
        );
        assert_eq!(
            interpret("0b1010").unwrap(),
            Value::SignedInt {
                value: 10,
                bits: 64
            }
        );
        assert_eq!(
            interpret("0o17").unwrap(),
            Value::SignedInt {
                value: 15,
                bits: 64
            }
        );
        assert_eq!(
            interpret("1_000_000").unwrap(),
            Value::SignedInt {
                value: 1_000_000,
                bits: 64
            }
        );
    }

    #[test]
    fn test_float_literals() {
        assert_eq!(interpret("3.14").unwrap(), Value::Double(3.14));
        assert_eq!(interpret("1.5e2").unwrap(), Value::Double(150.0));
    }

    #[test]
    fn test_boolean_literals() {
        assert_eq!(interpret("true").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("false").unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_void_literal() {
        assert_eq!(interpret("void").unwrap(), Value::Void);
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(
            interpret("5 + 3").unwrap(),
            Value::SignedInt { value: 8, bits: 64 }
        );
        assert_eq!(
            interpret("10 - 4").unwrap(),
            Value::SignedInt { value: 6, bits: 64 }
        );
        assert_eq!(
            interpret("2 * 6").unwrap(),
            Value::SignedInt {
                value: 12,
                bits: 64
            }
        );
        assert_eq!(
            interpret("15 / 3").unwrap(),
            Value::SignedInt { value: 5, bits: 64 }
        );
        assert_eq!(
            interpret("10 % 3").unwrap(),
            Value::SignedInt { value: 1, bits: 64 }
        );
    }

    #[test]
    fn test_arithmetic_precedence() {
        assert_eq!(
            interpret("2 + 3 * 4").unwrap(),
            Value::SignedInt {
                value: 14,
                bits: 64
            }
        );
        assert_eq!(
            interpret("(2 + 3) * 4").unwrap(),
            Value::SignedInt {
                value: 20,
                bits: 64
            }
        );
    }

    #[test]
    fn test_float_arithmetic() {
        assert_eq!(interpret("2.5 + 1.5").unwrap(), Value::Double(4.0));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(interpret("5 < 10").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("5 > 10").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("5 <= 5").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("5 >= 10").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("5 == 5").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("5 != 10").unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_logical_operators() {
        assert_eq!(interpret("true && true").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("true && false").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("false || true").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("false || false").unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_logical_short_circuit() {
        // These would cause errors if not short-circuited
        assert_eq!(
            interpret("false && (1 / 0)").unwrap(),
            Value::Boolean(false)
        );
        assert_eq!(interpret("true || (1 / 0)").unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_bitwise_operators() {
        assert_eq!(
            interpret("5 & 3").unwrap(),
            Value::SignedInt { value: 1, bits: 64 }
        );
        assert_eq!(
            interpret("5 | 3").unwrap(),
            Value::SignedInt { value: 7, bits: 64 }
        );
        assert_eq!(
            interpret("5 ^ 3").unwrap(),
            Value::SignedInt { value: 6, bits: 64 }
        );
        assert_eq!(
            interpret("~5").unwrap(),
            Value::SignedInt {
                value: -6,
                bits: 64
            }
        );
        assert_eq!(
            interpret("2 << 3").unwrap(),
            Value::SignedInt {
                value: 16,
                bits: 64
            }
        );
        assert_eq!(
            interpret("16 >> 2").unwrap(),
            Value::SignedInt { value: 4, bits: 64 }
        );
    }

    #[test]
    fn test_unary_operators() {
        assert_eq!(
            interpret("-5").unwrap(),
            Value::SignedInt {
                value: -5,
                bits: 64
            }
        );
        assert_eq!(
            interpret("+5").unwrap(),
            Value::SignedInt { value: 5, bits: 64 }
        );
        assert_eq!(interpret("!true").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("!false").unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_variable_declaration() {
        assert_eq!(
            interpret("let x = 42\nx").unwrap(),
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
        assert_eq!(
            interpret("fix y = 10\ny").unwrap(),
            Value::SignedInt {
                value: 10,
                bits: 64
            }
        );
    }

    #[test]
    fn test_variable_assignment() {
        assert_eq!(
            interpret("let x = 10\nx = 20\nx").unwrap(),
            Value::SignedInt {
                value: 20,
                bits: 64
            }
        );
    }

    #[test]
    fn test_immutable_assignment_error() {
        let result = interpret("fix x = 10\nx = 20");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("immutable"));
    }

    #[test]
    fn test_undefined_variable_error() {
        let result = interpret("x");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable"));
    }

    #[test]
    fn test_block_expression() {
        let source = "{\n  let x = 5\n  let y = 10\n  <- x + y\n}";
        assert_eq!(
            interpret(source).unwrap(),
            Value::SignedInt {
                value: 15,
                bits: 64
            }
        );
    }

    #[test]
    fn test_block_scoping() {
        let source = "let x = 1\n{\n  let x = 2\n}\nx";
        assert_eq!(
            interpret(source).unwrap(),
            Value::SignedInt { value: 1, bits: 64 }
        );
    }

    #[test]
    fn test_nested_blocks() {
        let source = "let x = 1\n{\n  let y = 2\n  {\n    let z = 3\n    <- x + y + z\n  }\n}";
        assert_eq!(
            interpret(source).unwrap(),
            Value::SignedInt { value: 6, bits: 64 }
        );
    }

    #[test]
    fn test_type_annotation() {
        assert_eq!(
            interpret("let x: i32 = 42\nx").unwrap(),
            Value::SignedInt {
                value: 42,
                bits: 32
            }
        );
        assert_eq!(
            interpret("let y: F32 = 3.14\ny").unwrap(),
            Value::Single(3.14)
        );
    }

    #[test]
    fn test_division_by_zero() {
        let result = interpret("10 / 0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Division by zero"));
    }

    #[test]
    fn test_complex_expression() {
        let source = "let a = 5\nlet b = 10\nlet c = a * b + 20\nc";
        assert_eq!(
            interpret(source).unwrap(),
            Value::SignedInt {
                value: 70,
                bits: 64
            }
        );
    }

    #[test]
    fn test_logical_and_requires_boolean() {
        let result = interpret("5 && true");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Expected Boolean"));
        assert!(err.contains("I64"));

        let result2 = interpret("true && 5");
        assert!(result2.is_err());
        let err2 = result2.unwrap_err();
        assert!(err2.contains("Expected Boolean"));
        assert!(err2.contains("I64"));
    }

    #[test]
    fn test_logical_or_requires_boolean() {
        let result = interpret("5 || false");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Expected Boolean"));
        assert!(err.contains("I64"));

        let result2 = interpret("false || 5");
        assert!(result2.is_err());
        let err2 = result2.unwrap_err();
        assert!(err2.contains("Expected Boolean"));
        assert!(err2.contains("I64"));
    }

    #[test]
    fn test_logical_not_requires_boolean() {
        let result = interpret("!5");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Logical NOT requires boolean"));
        assert!(err.contains("I64"));

        let result2 = interpret("!0");
        assert!(result2.is_err());
        assert!(
            result2
                .unwrap_err()
                .contains("Logical NOT requires boolean")
        );

        let result3 = interpret("!3.14");
        assert!(result3.is_err());
        let err3 = result3.unwrap_err();
        assert!(err3.contains("Logical NOT requires boolean"));
        assert!(err3.contains("F64"));
    }

    #[test]
    fn test_unsigned_integer_type_annotation() {
        let source = "let x: u32 = 42\nx";
        assert_eq!(
            interpret(source).unwrap(),
            Value::UnsignedInt {
                value: 42,
                bits: 32
            }
        );

        let source2 = "let y: u64 = 100\ny";
        assert_eq!(
            interpret(source2).unwrap(),
            Value::UnsignedInt {
                value: 100,
                bits: 64
            }
        );
    }

    #[test]
    fn test_unsigned_integer_arithmetic() {
        let source = "let x: u32 = 10\nlet y: u32 = 20\nx + y";
        assert_eq!(
            interpret(source).unwrap(),
            Value::UnsignedInt {
                value: 30,
                bits: 32
            }
        );

        let source2 = "let a: u64 = 100\nlet b: u64 = 50\na - b";
        assert_eq!(
            interpret(source2).unwrap(),
            Value::UnsignedInt {
                value: 50,
                bits: 64
            }
        );
    }

    #[test]
    fn test_unsigned_negative_value_error() {
        let result = interpret("let x: u32 = -5");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Cannot assign negative integer")
        );
    }

    #[test]
    fn test_unsigned_bitwise_operations() {
        let source = "let x: u32 = 5\nlet y: u32 = 3\nx & y";
        assert_eq!(
            interpret(source).unwrap(),
            Value::UnsignedInt { value: 1, bits: 32 }
        );

        let source2 = "let a: u32 = 5\nlet b: u32 = 3\na | b";
        assert_eq!(
            interpret(source2).unwrap(),
            Value::UnsignedInt { value: 7, bits: 32 }
        );
    }

    #[test]
    fn test_unsigned_comparison() {
        let source = "let x: u32 = 10\nlet y: u32 = 20\nx < y";
        assert_eq!(interpret(source).unwrap(), Value::Boolean(true));

        let source2 = "let a: u64 = 100\nlet b: u64 = 50\na > b";
        assert_eq!(interpret(source2).unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_mixed_signed_unsigned_not_allowed() {
        let result = interpret("let x: i32 = 10\nlet y: u32 = 20\nx + y");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot add"));
    }
}
