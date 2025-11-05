use crate::analysis::typed_ast::{
    TypedBinaryExpr, TypedBlock, TypedExpression, TypedIfExpr, TypedIndexExpr, TypedProgram,
    TypedStatement, TypedUnaryExpr, TypedVariableDeclaration, TypedWhileExpr,
};
use crate::error::{RuntimeError, RuntimeErrorKind};
use crate::frontend::ast::{BinaryOp, Type, UnaryOp};
use crate::runtime::environment::Environment;
use crate::runtime::value::Value;

// Control flow signal for break statements
#[derive(Debug, Clone)]
enum ControlFlow {
    None,
    Break(Value),
}

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn execute_program(&mut self, program: &TypedProgram) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Void;
        for statement in &program.statements {
            last_value = self.execute_statement(statement)?;
        }
        Ok(last_value)
    }

    fn execute_statement(&mut self, statement: &TypedStatement) -> Result<Value, RuntimeError> {
        match statement {
            TypedStatement::VariableDeclaration(var_decl) => {
                self.execute_variable_declaration(var_decl)
            }
            TypedStatement::Yield(yield_stmt) => self.evaluate_expression(&yield_stmt.value),
            TypedStatement::Break(_) => {
                // Break should never be executed at the top level
                // It's only valid inside loops and handled there
                Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidOperation,
                    "break statement outside of loop".to_string(),
                ))
            }
            TypedStatement::Expression(expr) => self.evaluate_expression(expr),
        }
    }

    fn execute_variable_declaration(
        &mut self,
        var_decl: &TypedVariableDeclaration,
    ) -> Result<Value, RuntimeError> {
        // Type is already resolved, just evaluate the initializer
        let value = self.evaluate_expression(&var_decl.initializer)?;

        let mutable = var_decl.mutable;
        self.environment
            .define(var_decl.name.clone(), value.clone(), mutable)?;

        Ok(value)
    }

    fn get_array_idx(&mut self, value: &Value) -> Result<usize, RuntimeError> {
        match *value {
            Value::SignedInt { value, .. } => {
                if value < 0 {
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidOperation,
                        format!("Array index cannot be negative: {}", value),
                    ));
                }
                Ok(value as usize)
            }
            Value::UnsignedInt { value, .. } => Ok(value as usize),
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::InvalidOperation,
                "Array index must be an integer".to_string(),
            )),
        }
    }

    fn evaluate_expression(&mut self, expression: &TypedExpression) -> Result<Value, RuntimeError> {
        match expression {
            TypedExpression::Integer(s, ty) => {
                // Type is already resolved in the IR
                match ty {
                    Type::Signed(bits) => self.parse_integer(s, true, *bits),
                    Type::Unsigned(bits) => self.parse_integer(s, false, *bits),
                    _ => Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidOperation,
                        format!("Invalid type for integer literal: {}", ty),
                    )),
                }
            }
            TypedExpression::Float(s, ty) => {
                // Type is already resolved in the IR
                match ty {
                    Type::F32 => self.parse_float(s, 32),
                    Type::F64 => self.parse_float(s, 64),
                    _ => Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidOperation,
                        format!("Invalid type for float literal: {}", ty),
                    )),
                }
            }
            TypedExpression::Boolean(b) => Ok(Value::Boolean(*b)),
            TypedExpression::Void => Ok(Value::Void),
            TypedExpression::ArrayLiteral(elements, arr_type) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.evaluate_expression(elem)?);
                }

                // Extract element type from array type
                let element_type = match arr_type {
                    Type::Array(et, _) => *et.clone(),
                    _ => {
                        return Err(RuntimeError::new(
                            RuntimeErrorKind::InvalidOperation,
                            format!("Invalid array type: {}", arr_type),
                        ));
                    }
                };

                Ok(Value::Array {
                    elements: values,
                    element_type,
                })
            }
            TypedExpression::Identifier(name, ..) => self.environment.get(name),
            TypedExpression::Binary(binary) => self.evaluate_binary(binary),
            TypedExpression::Unary(unary) => self.evaluate_unary(unary),
            TypedExpression::Index(index_expr) => {
                let array_val = self.evaluate_expression(&index_expr.array)?;
                let index_val = self.evaluate_expression(&index_expr.index)?;

                let idx = self.get_array_idx(&index_val)?;

                // Index into array
                array_val.index(idx)
            }
            TypedExpression::Block(block) => self.evaluate_block(block),
            TypedExpression::If(if_expr) => self.evaluate_if(if_expr),
            TypedExpression::While(while_expr) => self.evaluate_while(while_expr),
            TypedExpression::Grouping(expr) => self.evaluate_expression(expr),
        }
    }

    fn parse_integer(&self, s: &str, signed: bool, bits: u8) -> Result<Value, RuntimeError> {
        let s = s.replace('_', "");

        if signed {
            let result = if s.starts_with("0x") || s.starts_with("0X") {
                i128::from_str_radix(&s[2..], 16)
            } else if s.starts_with("0b") || s.starts_with("0B") {
                i128::from_str_radix(&s[2..], 2)
            } else if s.starts_with("0o") || s.starts_with("0O") {
                i128::from_str_radix(&s[2..], 8)
            } else {
                s.parse::<i128>()
            };

            let value = result.map_err(|e| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidConversion,
                    format!("Invalid integer literal: {}", e),
                )
            })?;
            Value::new_signed(value, bits)
        } else {
            let result = if s.starts_with("0x") || s.starts_with("0X") {
                u128::from_str_radix(&s[2..], 16)
            } else if s.starts_with("0b") || s.starts_with("0B") {
                u128::from_str_radix(&s[2..], 2)
            } else if s.starts_with("0o") || s.starts_with("0O") {
                u128::from_str_radix(&s[2..], 8)
            } else {
                s.parse::<u128>()
            };

            let value = result.map_err(|e| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidConversion,
                    format!("Invalid integer literal: {}", e),
                )
            })?;
            Value::new_unsigned(value, bits)
        }
    }

    fn parse_float(&self, s: &str, bits: u8) -> Result<Value, RuntimeError> {
        let s = s.replace('_', "");

        let value = s.parse::<f64>().map_err(|e| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidConversion,
                format!("Invalid float literal: {}", e),
            )
        })?;

        match bits {
            32 => Ok(Value::Single(value as f32)),
            64 => Ok(Value::Double(value)),
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::InvalidOperation,
                format!("Invalid float bitwidth: {}", bits),
            )),
        }
    }

    fn evaluate_binary(&mut self, binary: &TypedBinaryExpr) -> Result<Value, RuntimeError> {
        if self.is_assignment_operator(&binary.operator) {
            return self.evaluate_assignment_binary(binary);
        }

        match binary.operator {
            BinaryOp::LogicalAnd => {
                let left = self.evaluate_expression(&binary.left)?;
                match left {
                    Value::Boolean(false) => Ok(Value::Boolean(false)),
                    Value::Boolean(true) => {
                        let right = self.evaluate_expression(&binary.right)?;
                        match right {
                            Value::Boolean(_) => Ok(right),
                            _ => Err(RuntimeError::new(
                                RuntimeErrorKind::InvalidOperation,
                                format!("Expected Boolean, found {}", right.type_name()),
                            )),
                        }
                    }
                    _ => Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidOperation,
                        format!("Expected Boolean, found {}", left.type_name()),
                    )),
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
                            _ => Err(RuntimeError::new(
                                RuntimeErrorKind::InvalidOperation,
                                format!("Expected Boolean, found {}", right.type_name()),
                            )),
                        }
                    }
                    _ => Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidOperation,
                        format!("Expected Boolean, found {}", left.type_name()),
                    )),
                }
            }
            _ => {
                let left = self.evaluate_expression(&binary.left)?;
                let right = self.evaluate_expression(&binary.right)?;
                self.apply_binary_operator(&binary.operator, &left, &right)
            }
        }
    }

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

    fn evaluate_assignment_binary(
        &mut self,
        binary: &TypedBinaryExpr,
    ) -> Result<Value, RuntimeError> {
        // Evaluate the right-hand side
        let right_value = self.evaluate_expression(&binary.right)?;

        match &binary.left {
            TypedExpression::Identifier(name, _ty) => {
                self.evaluate_identifier_assignment(name, &binary.operator, right_value)
            }
            TypedExpression::Index(index_expr) => {
                self.evaluate_index_assignment(index_expr, &binary.operator, right_value)
            }
            _ => Err(RuntimeError::new(
                RuntimeErrorKind::InvalidOperation,
                "Invalid assignment target".to_string(),
            )),
        }
    }

    fn evaluate_identifier_assignment(
        &mut self,
        name: &str,
        operator: &BinaryOp,
        right_value: Value,
    ) -> Result<Value, RuntimeError> {
        // Apply the assignment operator
        match operator {
            BinaryOp::Assign => {
                self.environment.assign(name, right_value.clone())?;
                Ok(right_value)
            }
            BinaryOp::PlusAssign => {
                let current = self.environment.get(name)?;
                let result = current.add(&right_value)?;
                self.environment.assign(name, result.clone())?;
                Ok(result)
            }
            BinaryOp::MinusAssign => {
                let current = self.environment.get(name)?;
                let result = current.subtract(&right_value)?;
                self.environment.assign(name, result.clone())?;
                Ok(result)
            }
            BinaryOp::MultAssign => {
                let current = self.environment.get(name)?;
                let result = current.multiply(&right_value)?;
                self.environment.assign(name, result.clone())?;
                Ok(result)
            }
            BinaryOp::DivAssign => {
                let current = self.environment.get(name)?;
                let result = current.divide(&right_value)?;
                self.environment.assign(name, result.clone())?;
                Ok(result)
            }
            BinaryOp::ModAssign => {
                let current = self.environment.get(name)?;
                let result = current.modulo(&right_value)?;
                self.environment.assign(name, result.clone())?;
                Ok(result)
            }
            BinaryOp::AndAssign => {
                let current = self.environment.get(name)?;
                let result = current.bitwise_and(&right_value)?;
                self.environment.assign(name, result.clone())?;
                Ok(result)
            }
            BinaryOp::OrAssign => {
                let current = self.environment.get(name)?;
                let result = current.bitwise_or(&right_value)?;
                self.environment.assign(name, result.clone())?;
                Ok(result)
            }
            BinaryOp::XorAssign => {
                let current = self.environment.get(name)?;
                let result = current.bitwise_xor(&right_value)?;
                self.environment.assign(name, result.clone())?;
                Ok(result)
            }
            BinaryOp::LShiftAssign => {
                let current = self.environment.get(name)?;
                let result = current.left_shift(&right_value)?;
                self.environment.assign(name, result.clone())?;
                Ok(result)
            }
            BinaryOp::RShiftAssign => {
                let current = self.environment.get(name)?;
                let result = current.right_shift(&right_value)?;
                self.environment.assign(name, result.clone())?;
                Ok(result)
            }
            _ => unreachable!("Non-assignment operator passed to evaluate_identifier_assignment"),
        }
    }

    fn evaluate_index_assignment(
        &mut self,
        index_expr: &TypedIndexExpr,
        operator: &BinaryOp,
        right_value: Value,
    ) -> Result<Value, RuntimeError> {
        // For indexed assignment like arr[0] = 42 or arr[i] += 10

        let idx_val = self.evaluate_expression(&index_expr.index)?;
        let index_val = self.get_array_idx(&idx_val)?;

        // Check if the array is an identifier (so we can write back)
        let array_name = match &index_expr.array {
            TypedExpression::Identifier(name, _) => Some(name.clone()),
            _ => None,
        };

        // Get the current array value
        let mut array = self.evaluate_expression(&index_expr.array)?;

        // Compute the new value to assign
        let new_value = match operator {
            BinaryOp::Assign => right_value,
            BinaryOp::PlusAssign => {
                let current = array.index(index_val)?;
                current.add(&right_value)?
            }
            BinaryOp::MinusAssign => {
                let current = array.index(index_val)?;
                current.subtract(&right_value)?
            }
            BinaryOp::MultAssign => {
                let current = array.index(index_val)?;
                current.multiply(&right_value)?
            }
            BinaryOp::DivAssign => {
                let current = array.index(index_val)?;
                current.divide(&right_value)?
            }
            BinaryOp::ModAssign => {
                let current = array.index(index_val)?;
                current.modulo(&right_value)?
            }
            BinaryOp::AndAssign => {
                let current = array.index(index_val)?;
                current.bitwise_and(&right_value)?
            }
            BinaryOp::OrAssign => {
                let current = array.index(index_val)?;
                current.bitwise_or(&right_value)?
            }
            BinaryOp::XorAssign => {
                let current = array.index(index_val)?;
                current.bitwise_xor(&right_value)?
            }
            BinaryOp::LShiftAssign => {
                let current = array.index(index_val)?;
                current.left_shift(&right_value)?
            }
            BinaryOp::RShiftAssign => {
                let current = array.index(index_val)?;
                current.right_shift(&right_value)?
            }
            _ => unreachable!("Non-assignment operator passed to evaluate_index_assignment"),
        };

        // Set the new value in the array
        array.set_index(index_val, new_value.clone())?;

        // If the array came from an identifier, write it back to the environment
        if let Some(name) = array_name {
            self.environment.assign(&name, array.clone())?;
        }
        Ok(array)
    }

    fn apply_binary_operator(
        &self,
        operator: &BinaryOp,
        left: &Value,
        right: &Value,
    ) -> Result<Value, RuntimeError> {
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

    fn evaluate_unary(&mut self, unary: &TypedUnaryExpr) -> Result<Value, RuntimeError> {
        // Type is already resolved in the typed IR
        let operand = self.evaluate_expression(&unary.operand)?;
        match unary.operator {
            UnaryOp::Plus => Ok(operand),
            UnaryOp::Minus => operand.negate(),
            UnaryOp::LogicalNot => operand.logical_not(),
            UnaryOp::BitwiseNot => operand.bitwise_not(),
        }
    }

    fn evaluate_block(&mut self, block: &TypedBlock) -> Result<Value, RuntimeError> {
        // Push a new scope for the block
        self.environment.push_scope();

        let mut result = Value::Void;

        // Execute statements until we hit a yield, which acts as a break
        for statement in &block.statements {
            match statement {
                TypedStatement::Yield(yield_stmt) => {
                    // Yield acts as a break - evaluate and return immediately
                    result = self.evaluate_expression(&yield_stmt.value)?;
                    break;
                }
                _ => {
                    // Other statements (declarations and expressions) don't contribute to block value
                    self.execute_statement(statement)?;
                }
            }
        }

        // Pop the block's scope
        self.environment.pop_scope();

        Ok(result)
    }

    fn evaluate_if(&mut self, if_expr: &TypedIfExpr) -> Result<Value, RuntimeError> {
        // Evaluate the condition
        let condition_value = self.evaluate_expression(&if_expr.condition)?;

        let condition_bool = match condition_value {
            Value::Boolean(b) => b,
            _ => {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidOperation,
                    format!(
                        "If condition must be boolean, found {}",
                        condition_value.type_name()
                    ),
                ));
            }
        };

        if condition_bool {
            // Execute then block
            return self.evaluate_typed_block(&if_expr.then_block);
        }

        // Check else-if branches
        for (else_if_condition, else_if_block) in &if_expr.else_ifs {
            let else_if_cond_value = self.evaluate_expression(else_if_condition)?;
            let else_if_bool = match else_if_cond_value {
                Value::Boolean(b) => b,
                _ => {
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidOperation,
                        format!(
                            "Else-if condition must be boolean, found {}",
                            else_if_cond_value.type_name()
                        ),
                    ));
                }
            };

            if else_if_bool {
                return self.evaluate_typed_block(else_if_block);
            }
        }

        // Execute else block if present, otherwise return void
        if let Some(else_block) = &if_expr.else_block {
            self.evaluate_typed_block(else_block)
        } else {
            Ok(Value::Void)
        }
    }

    fn evaluate_while(&mut self, while_expr: &TypedWhileExpr) -> Result<Value, RuntimeError> {
        loop {
            // Evaluate the condition
            let condition_value = self.evaluate_expression(&while_expr.condition)?;

            let condition_bool = match condition_value {
                Value::Boolean(b) => b,
                _ => {
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidOperation,
                        format!(
                            "While condition must be boolean, found {}",
                            condition_value.type_name()
                        ),
                    ));
                }
            };

            if !condition_bool {
                // Condition is false - execute else block if present
                if let Some(else_block) = &while_expr.else_block {
                    return self.evaluate_typed_block(else_block);
                } else {
                    return Ok(Value::Void);
                }
            }

            // Execute the body - check for breaks and top-level yields
            match self.evaluate_typed_block_in_loop(&while_expr.body)? {
                ControlFlow::Break(value) => {
                    // Break encountered - return that value
                    return Ok(value);
                }
                ControlFlow::None => {
                    // No break/yield, continue looping
                    continue;
                }
            }
        }
    }

    fn evaluate_typed_block(&mut self, block: &TypedBlock) -> Result<Value, RuntimeError> {
        // Push a new scope for the block
        self.environment.push_scope();

        let mut result = Value::Void;

        // Execute statements until we hit a yield, which acts as a break
        for statement in &block.statements {
            match statement {
                TypedStatement::Yield(yield_stmt) => {
                    // Yield acts as a break - evaluate and return immediately
                    result = self.evaluate_expression(&yield_stmt.value)?;
                    break;
                }
                _ => {
                    // Other statements (declarations and expressions) don't contribute to block value
                    self.execute_statement(statement)?;
                }
            }
        }

        // Pop the block's scope
        self.environment.pop_scope();

        Ok(result)
    }

    fn evaluate_typed_block_in_loop(
        &mut self,
        block: &TypedBlock,
    ) -> Result<ControlFlow, RuntimeError> {
        // Push a new scope for the block
        self.environment.push_scope();

        let mut control_flow = ControlFlow::None;

        // Execute statements, checking for top-level breaks and yields
        for statement in &block.statements {
            match statement {
                TypedStatement::Yield(yield_stmt) => {
                    // Top-level yield in loop body - acts as break
                    let value = self.evaluate_expression(&yield_stmt.value)?;
                    control_flow = ControlFlow::Break(value);
                    break;
                }
                TypedStatement::Break(break_stmt) => {
                    // Break statement - always breaks the loop (even from nested blocks)
                    let value = self.evaluate_expression(&break_stmt.value)?;
                    control_flow = ControlFlow::Break(value);
                    break;
                }
                TypedStatement::VariableDeclaration(var_decl) => {
                    self.execute_variable_declaration(var_decl)?;
                }
                TypedStatement::Expression(expr) => {
                    // Expressions might contain breaks in nested blocks
                    // We need to check if they triggered a break
                    match self.evaluate_expression_checking_breaks(expr)? {
                        ControlFlow::Break(value) => {
                            control_flow = ControlFlow::Break(value);
                            break;
                        }
                        ControlFlow::None => {}
                    }
                }
            }
        }

        // Pop the block's scope
        self.environment.pop_scope();

        Ok(control_flow)
    }

    fn evaluate_expression_checking_breaks(
        &mut self,
        expr: &TypedExpression,
    ) -> Result<ControlFlow, RuntimeError> {
        // Most expressions don't contain breaks, but if/while expressions can
        match expr {
            TypedExpression::If(if_expr) => self.evaluate_if_checking_breaks(if_expr),
            TypedExpression::While(while_expr) => {
                // While can contain breaks in its body
                // We need to execute the whole while loop, which handles breaks internally
                self.evaluate_while(while_expr)?;
                Ok(ControlFlow::None)
            }
            TypedExpression::Block(block) => {
                // Blocks can contain break statements
                self.evaluate_block_checking_breaks(block)
            }
            _ => {
                // Other expressions don't contain breaks
                self.evaluate_expression(expr)?;
                Ok(ControlFlow::None)
            }
        }
    }

    fn evaluate_if_checking_breaks(
        &mut self,
        if_expr: &TypedIfExpr,
    ) -> Result<ControlFlow, RuntimeError> {
        // Evaluate the condition
        let condition_value = self.evaluate_expression(&if_expr.condition)?;
        let condition_bool = match condition_value {
            Value::Boolean(b) => b,
            _ => {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidOperation,
                    format!(
                        "If condition must be boolean, found {}",
                        condition_value.type_name()
                    ),
                ));
            }
        };

        if condition_bool {
            // Execute then block and check for breaks
            return self.evaluate_block_checking_breaks(&if_expr.then_block);
        }

        // Check else-if branches
        for (else_if_condition, else_if_block) in &if_expr.else_ifs {
            let else_if_cond_value = self.evaluate_expression(else_if_condition)?;
            let else_if_bool = match else_if_cond_value {
                Value::Boolean(b) => b,
                _ => {
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidOperation,
                        format!(
                            "Else-if condition must be boolean, found {}",
                            else_if_cond_value.type_name()
                        ),
                    ));
                }
            };

            if else_if_bool {
                return self.evaluate_block_checking_breaks(else_if_block);
            }
        }

        // Execute else block if present
        if let Some(else_block) = &if_expr.else_block {
            self.evaluate_block_checking_breaks(else_block)
        } else {
            Ok(ControlFlow::None)
        }
    }

    fn evaluate_block_checking_breaks(
        &mut self,
        block: &TypedBlock,
    ) -> Result<ControlFlow, RuntimeError> {
        // Push a new scope for the block
        self.environment.push_scope();

        let mut control_flow = ControlFlow::None;

        // Execute statements, checking for breaks and yields
        for statement in &block.statements {
            match statement {
                TypedStatement::Yield(yield_stmt) => {
                    // Yield in a nested block doesn't break the loop
                    // Just evaluate and stop executing this block
                    self.evaluate_expression(&yield_stmt.value)?;
                    break;
                }
                TypedStatement::Break(break_stmt) => {
                    // Break statement propagates up to break the loop
                    let value = self.evaluate_expression(&break_stmt.value)?;
                    control_flow = ControlFlow::Break(value);
                    break;
                }
                TypedStatement::VariableDeclaration(var_decl) => {
                    self.execute_variable_declaration(var_decl)?;
                }
                TypedStatement::Expression(expr) => {
                    // Expressions might contain breaks in nested structures
                    match self.evaluate_expression_checking_breaks(expr)? {
                        ControlFlow::Break(value) => {
                            control_flow = ControlFlow::Break(value);
                            break;
                        }
                        ControlFlow::None => {}
                    }
                }
            }
        }

        // Pop the block's scope
        self.environment.pop_scope();

        Ok(control_flow)
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
    use crate::frontend::lexer::Lexer;
    use crate::frontend::parser::Parser;

    fn interpret(source: &str) -> Result<Value, RuntimeError> {
        use crate::analysis::type_checker::TypeChecker;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| {
            RuntimeError::new(RuntimeErrorKind::InvalidOperation, e.message.clone())
        })?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| {
            RuntimeError::new(RuntimeErrorKind::InvalidOperation, e.message.clone())
        })?;

        // Type check and get typed IR
        let mut type_checker = TypeChecker::new();
        let typed_program = type_checker.check_program(&program).map_err(|e| {
            RuntimeError::new(RuntimeErrorKind::InvalidOperation, e.message.clone())
        })?;

        // Interpret the typed IR
        let mut interpreter = Interpreter::new();
        interpreter.execute_program(&typed_program)
    }

    #[test]
    fn test_integer_literals() {
        assert_eq!(
            interpret("42;").unwrap(),
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
        assert_eq!(
            interpret("0xFF;").unwrap(),
            Value::SignedInt {
                value: 255,
                bits: 64
            }
        );
        assert_eq!(
            interpret("0b1010;").unwrap(),
            Value::SignedInt {
                value: 10,
                bits: 64
            }
        );
        assert_eq!(
            interpret("0o17;").unwrap(),
            Value::SignedInt {
                value: 15,
                bits: 64
            }
        );
        assert_eq!(
            interpret("1_000_000;").unwrap(),
            Value::SignedInt {
                value: 1_000_000,
                bits: 64
            }
        );
    }

    #[test]
    fn test_float_literals() {
        assert_eq!(interpret("3.14;").unwrap(), Value::Double(3.14));
        assert_eq!(interpret("1.5e2;").unwrap(), Value::Double(150.0));
    }

    #[test]
    fn test_boolean_literals() {
        assert_eq!(interpret("true;").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("false;").unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_void_literal() {
        assert_eq!(interpret("void;").unwrap(), Value::Void);
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(
            interpret("5 + 3;").unwrap(),
            Value::SignedInt { value: 8, bits: 64 }
        );
        assert_eq!(
            interpret("10 - 4;").unwrap(),
            Value::SignedInt { value: 6, bits: 64 }
        );
        assert_eq!(
            interpret("2 * 6;").unwrap(),
            Value::SignedInt {
                value: 12,
                bits: 64
            }
        );
        assert_eq!(
            interpret("15 / 3;").unwrap(),
            Value::SignedInt { value: 5, bits: 64 }
        );
        assert_eq!(
            interpret("10 % 3;").unwrap(),
            Value::SignedInt { value: 1, bits: 64 }
        );
    }

    #[test]
    fn test_arithmetic_precedence() {
        assert_eq!(
            interpret("2 + 3 * 4;").unwrap(),
            Value::SignedInt {
                value: 14,
                bits: 64
            }
        );
        assert_eq!(
            interpret("(2 + 3) * 4;").unwrap(),
            Value::SignedInt {
                value: 20,
                bits: 64
            }
        );
    }

    #[test]
    fn test_float_arithmetic() {
        assert_eq!(interpret("2.5 + 1.5;").unwrap(), Value::Double(4.0));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(interpret("5 < 10;").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("5 > 10;").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("5 <= 5;").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("5 >= 10;").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("5 == 5;").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("5 != 10;").unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_logical_operators() {
        assert_eq!(interpret("true && true;").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("true && false;").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("false || true;").unwrap(), Value::Boolean(true));
        assert_eq!(interpret("false || false;").unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_logical_short_circuit() {
        // Short-circuit evaluation should prevent the right side from being evaluated
        // Note: Type checker still validates both operands are boolean
        assert_eq!(interpret("false && false;").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("true || false;").unwrap(), Value::Boolean(true));

        // More meaningful short-circuit test: second operand depends on first
        // In a real scenario, this would prevent runtime errors
        let source = "let x = false;\nx && (5 > 10);";
        assert_eq!(interpret(source).unwrap(), Value::Boolean(false));

        let source2 = "let y = true;\ny || (5 < 3);";
        assert_eq!(interpret(source2).unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_bitwise_operators() {
        assert_eq!(
            interpret("5 & 3;").unwrap(),
            Value::SignedInt { value: 1, bits: 64 }
        );
        assert_eq!(
            interpret("5 | 3;").unwrap(),
            Value::SignedInt { value: 7, bits: 64 }
        );
        assert_eq!(
            interpret("5 ^ 3;").unwrap(),
            Value::SignedInt { value: 6, bits: 64 }
        );
        assert_eq!(
            interpret("~5;").unwrap(),
            Value::SignedInt {
                value: -6,
                bits: 64
            }
        );
        assert_eq!(
            interpret("2 << 3;").unwrap(),
            Value::SignedInt {
                value: 16,
                bits: 64
            }
        );
        assert_eq!(
            interpret("16 >> 2;").unwrap(),
            Value::SignedInt { value: 4, bits: 64 }
        );
    }

    #[test]
    fn test_unary_operators() {
        assert_eq!(
            interpret("-5;").unwrap(),
            Value::SignedInt {
                value: -5,
                bits: 64
            }
        );
        assert_eq!(
            interpret("+5;").unwrap(),
            Value::SignedInt { value: 5, bits: 64 }
        );
        assert_eq!(interpret("!true;").unwrap(), Value::Boolean(false));
        assert_eq!(interpret("!false;").unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_variable_declaration() {
        assert_eq!(
            interpret("let x = 42;\nx;").unwrap(),
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
        assert_eq!(
            interpret("fix y = 10;\ny;").unwrap(),
            Value::SignedInt {
                value: 10,
                bits: 64
            }
        );
    }

    #[test]
    fn test_variable_assignment() {
        assert_eq!(
            interpret("let x = 10;\nx = 20;\nx;").unwrap(),
            Value::SignedInt {
                value: 20,
                bits: 64
            }
        );
    }

    #[test]
    fn test_immutable_assignment_error() {
        let result = interpret("fix x = 10;\nx = 20;");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("immutable"));
    }

    #[test]
    fn test_undefined_variable_error() {
        let result = interpret("x;");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Undefined variable"));
    }

    #[test]
    fn test_block_expression() {
        let source = "{\n  let x = 5;\n  let y = 10;\n  <- x + y;\n}";
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
        let source = "let x = 1;\n{\n  let x = 2;\n}\nx;";
        assert_eq!(
            interpret(source).unwrap(),
            Value::SignedInt { value: 1, bits: 64 }
        );
    }

    #[test]
    fn test_nested_blocks() {
        let source = "let x = 1;\n{\nlet y = 2;\n{\nlet z = 3;\n<- x + y + z;\n}\n}";
        assert_eq!(interpret(source).unwrap(), Value::Void);
    }

    #[test]
    fn test_type_annotation() {
        assert_eq!(
            interpret("let x: I32 = 42;\nx;").unwrap(),
            Value::SignedInt {
                value: 42,
                bits: 32
            }
        );
        assert_eq!(
            interpret("let y: F32 = 3.14;\ny;").unwrap(),
            Value::Single(3.14)
        );
    }

    #[test]
    fn test_division_by_zero() {
        let result = interpret("10 / 0;");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Division by zero"));
    }

    #[test]
    fn test_complex_expression() {
        let source = "let a = 5;\nlet b = 10;\nlet c = a * b + 20;\nc;";
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
        let result = interpret("5 && true;");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Expected Boolean"));
        assert!(err.message.contains("I64"));

        let result2 = interpret("true && 5;");
        assert!(result2.is_err());
        let err2 = result2.unwrap_err();
        assert!(err2.message.contains("Expected Boolean"));
        assert!(err2.message.contains("I64"));
    }

    #[test]
    fn test_logical_or_requires_boolean() {
        let result = interpret("5 || false;");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Expected Boolean"));
        assert!(err.message.contains("I64"));

        let result2 = interpret("false || 5;");
        assert!(result2.is_err());
        let err2 = result2.unwrap_err();
        assert!(err2.message.contains("Expected Boolean"));
        assert!(err2.message.contains("I64"));
    }

    #[test]
    fn test_logical_not_requires_boolean() {
        let result = interpret("!5;");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Logical NOT requires boolean"));
        assert!(err.message.contains("I64"));

        let result2 = interpret("!0;");
        assert!(result2.is_err());
        assert!(
            result2
                .unwrap_err()
                .message
                .contains("Logical NOT requires boolean")
        );

        let result3 = interpret("!3.14;");
        assert!(result3.is_err());
        let err3 = result3.unwrap_err();
        assert!(err3.message.contains("Logical NOT requires boolean"));
        assert!(err3.message.contains("F64"));
    }

    #[test]
    fn test_unsigned_integer_type_annotation() {
        let source = "let x: U32 = 42;\nx;";
        assert_eq!(
            interpret(source).unwrap(),
            Value::UnsignedInt {
                value: 42,
                bits: 32
            }
        );

        let source2 = "let y: U64 = 100;\ny;";
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
        let source = "let x: U32 = 10;\nlet y: U32 = 20;\nx + y;";
        assert_eq!(
            interpret(source).unwrap(),
            Value::UnsignedInt {
                value: 30,
                bits: 32
            }
        );

        let source2 = "let a: U64 = 100;\nlet b: U64 = 50;\na - b;";
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
        let result = interpret("let x: U32 = -5;");
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Error is "Cannot negate U32" because the literal 5 is parsed as U32 due to
        // type context, and unsigned values cannot be negated
        assert!(err.message.contains("Cannot negate"));
    }

    #[test]
    fn test_unsigned_bitwise_operations() {
        let source = "let x: U32 = 5;\nlet y: U32 = 3;\nx & y;";
        assert_eq!(
            interpret(source).unwrap(),
            Value::UnsignedInt { value: 1, bits: 32 }
        );

        let source2 = "let a: U32 = 5;\nlet b: U32 = 3;\na | b;";
        assert_eq!(
            interpret(source2).unwrap(),
            Value::UnsignedInt { value: 7, bits: 32 }
        );
    }

    #[test]
    fn test_unsigned_comparison() {
        let source = "let x: U32 = 10;\nlet y: U32 = 20;\nx < y;";
        assert_eq!(interpret(source).unwrap(), Value::Boolean(true));

        let source2 = "let a: U64 = 100;\nlet b: U64 = 50;\na > b;";
        assert_eq!(interpret(source2).unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_mixed_signed_unsigned_not_allowed() {
        let result = interpret("let x: I32 = 10;\nlet y: U32 = 20;\nx + y;");
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Type checker now catches this, so error message is from type checker
        assert!(err.message.contains("Cannot") || err.message.contains("perform"));
    }

    #[test]
    fn test_f32_with_expression() {
        let source = "let f: F32 = 1.0 + 2.0;\nf;";
        assert_eq!(interpret(source).unwrap(), Value::Single(3.0));
    }

    #[test]
    fn test_yield_acts_as_break() {
        // First yield should be returned, subsequent statements should not execute
        let source = "{\n  let x = 10;\n  <- x;\n  let y = 20;\n  <- y;\n}";
        assert_eq!(
            interpret(source).unwrap(),
            Value::SignedInt {
                value: 10,
                bits: 64
            }
        );
    }

    #[test]
    fn test_yield_prevents_side_effects() {
        // Variable declaration after yield should not execute
        let source = "let a = 5;\n{\n  <- a;\n  let b = 10;\n}\nb;";
        let result = interpret(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Undefined variable"));
    }

    #[test]
    fn test_block_with_no_yield_evaluates_to_void() {
        // Variable declaration after yield should not execute
        let source = "{ let x = 10;\nlet y = 30; }";
        let result = interpret(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Void);
    }

    #[test]
    fn test_block_with_only_expressions_evaluates_to_void() {
        // Variable declaration after yield should not execute
        let source = "{ 3 + 2 * 10;\n15 << 2; }";
        let result = interpret(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Void);
    }

    #[test]
    fn test_inconsistent_yield_types_caught() {
        // Type checker should catch inconsistent yield types in a block
        let source = "{\n  let x = true;\n  <- 42;\n  <- x;\n}";
        let result = interpret(source);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Integer type expected")
        );
    }

    #[test]
    fn test_add_after_assignment() {
        let source = "let x: I32 = 10;\nx + 3;";
        let result = interpret(source);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::SignedInt {
                value: 13,
                bits: 32
            }
        );
    }

    // ============================================================================
    // Array Interpretation Tests
    // ============================================================================

    #[test]
    fn test_array_literal_evaluation() {
        let source = "[1, 2, 3];";
        let result = interpret(source).unwrap();
        match result {
            Value::Array {
                elements,
                element_type,
            } => {
                assert_eq!(elements.len(), 3);
                assert_eq!(element_type, Type::Signed(64));
                match &elements[0] {
                    Value::SignedInt { value, bits } => {
                        assert_eq!(*value, 1);
                        assert_eq!(*bits, 64);
                    }
                    _ => panic!("Expected SignedInt"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_array_with_explicit_type() {
        let source = "let arr: [I32, 3] = [10, 20, 30];\narr;";
        let result = interpret(source).unwrap();
        match result {
            Value::Array {
                elements,
                element_type,
            } => {
                assert_eq!(elements.len(), 3);
                assert_eq!(element_type, Type::Signed(32));
                match &elements[1] {
                    Value::SignedInt { value, bits } => {
                        assert_eq!(*value, 20);
                        assert_eq!(*bits, 32);
                    }
                    _ => panic!("Expected SignedInt"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_empty_array() {
        let source = "let arr: [I32, 0] = [];\narr;";
        let result = interpret(source).unwrap();
        match result {
            Value::Array {
                elements,
                element_type,
            } => {
                assert_eq!(elements.len(), 0);
                assert_eq!(element_type, Type::Signed(32));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_array_indexing_evaluation() {
        let source = "let arr = [10, 20, 30];\narr[1];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 20,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_indexing_first_element() {
        let source = "let arr = [5, 10, 15];\narr[0];";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 5, bits: 64 });
    }

    #[test]
    fn test_array_indexing_last_element() {
        let source = "let arr = [5, 10, 15, 20];\narr[3];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 20,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_indexing_with_variable() {
        let source = "let arr = [100, 200, 300];\nlet i = 2;\narr[i];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 300,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_indexing_with_expression() {
        let source = "let arr = [10, 20, 30, 40, 50];\nlet i = 1;\narr[i + 2];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 40,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_index_out_of_bounds() {
        let source = "let arr = [1, 2, 3];\narr[5];";
        let result = interpret(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("out of bounds"));
    }

    #[test]
    fn test_array_index_negative() {
        let source = "let arr = [1, 2, 3];\nlet i: I32 = -1;\narr[i];";
        let result = interpret(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("cannot be negative"));
    }

    #[test]
    fn test_nested_array_evaluation() {
        let source = "let matrix = [[1, 2], [3, 4]];\nmatrix;";
        let result = interpret(source).unwrap();
        match result {
            Value::Array {
                elements,
                element_type,
            } => {
                assert_eq!(elements.len(), 2);
                match element_type {
                    Type::Array(inner_type, size) => {
                        assert_eq!(*inner_type, Type::Signed(64));
                        assert_eq!(size, 2);
                    }
                    _ => panic!("Expected array element type"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_chained_array_indexing() {
        let source = "let matrix = [[1, 2], [3, 4]];\nmatrix[0][1];";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 2, bits: 64 });
    }

    #[test]
    fn test_chained_array_indexing_second_row() {
        let source = "let matrix = [[10, 20], [30, 40]];\nmatrix[1][0];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 30,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_element_assignment() {
        let source = "let arr = [1, 2, 3];\narr[1] = 42;\narr[1];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_element_assignment_first() {
        let source = "let arr = [10, 20, 30];\narr[0] = 99;\narr[0];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 99,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_compound_assignment_add() {
        let source = "let arr = [5, 10, 15];\narr[1] += 20;\narr[1];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 30,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_compound_assignment_subtract() {
        let source = "let arr = [100, 50, 25];\narr[0] -= 30;\narr[0];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 70,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_compound_assignment_multiply() {
        let source = "let arr = [2, 3, 4];\narr[1] *= 10;\narr[1];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 30,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_arithmetic_on_elements() {
        let source = "let arr = [10, 20, 30];\nlet sum = arr[0] + arr[1] + arr[2];\nsum;";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 60,
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_of_floats() {
        let source = "let arr: [F32, 3] = [1.5, 2.5, 3.5];\narr[1];";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::Single(2.5));
    }

    #[test]
    fn test_array_of_bools() {
        let source = "let arr = [true, false, true];\narr[2];";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_array_in_block() {
        let source = "{\n  let arr = [1, 2, 3];\n  <- arr[1];\n}";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 2, bits: 64 });
    }

    #[test]
    fn test_multiple_array_operations() {
        let source = "let arr = [5, 10, 15];\narr[0] += 5;\narr[1] *= 2;\narr[2] -= 5;\narr[0] + arr[1] + arr[2];";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 40, // (5+5) + (10*2) + (15-5) = 10 + 20 + 10
                bits: 64
            }
        );
    }

    #[test]
    fn test_array_assignment_out_of_bounds() {
        let source = "let arr = [1, 2, 3];\narr[5] = 42;";
        let result = interpret(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("out of bounds"));
    }

    #[test]
    fn test_index_array_literal() {
        let source = "let val = [1, 2, 3][0];";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 1, bits: 64 })
    }

    #[test]
    fn test_index_assignment_array_literal() {
        let source = "[1, 2, 3][0] -= 30;";
        let result = interpret(source).unwrap();
        assert_eq!(result.type_name(), "[I64, 3]")
    }

    // ============================================================================
    // If/Else Tests
    // ============================================================================

    #[test]
    fn test_if_then_true() {
        let source = "if true { <- 42; } else { <- 0; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
    }

    #[test]
    fn test_if_then_false() {
        let source = "if false { <- 42; } else { <- 0; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 0, bits: 64 });
    }

    #[test]
    fn test_if_else_if_first_branch() {
        let source = "if true { <- 1; } else if true { <- 2; } else { <- 3; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 1, bits: 64 });
    }

    #[test]
    fn test_if_else_if_second_branch() {
        let source = "if false { <- 1; } else if true { <- 2; } else { <- 3; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 2, bits: 64 });
    }

    #[test]
    fn test_if_else_if_else_branch() {
        let source = "if false { <- 1; } else if false { <- 2; } else { <- 3; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 3, bits: 64 });
    }

    #[test]
    fn test_if_with_expression_condition() {
        let source = "let x = 5;\nif x > 3 { <- 100; } else { <- 200; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 100,
                bits: 64
            }
        );
    }

    #[test]
    fn test_if_without_else_void() {
        let source = "if false { }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::Void);
    }

    #[test]
    fn test_if_nested() {
        let source = "if true { <- if false { <- 1; } else { <- 2; }; } else { <- 3; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 2, bits: 64 });
    }

    #[test]
    fn test_if_with_variable_assignment() {
        let source = "let x = if true { <- 42; } else { <- 0; };\n<- x;";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
    }

    // ============================================================================
    // While Loop Tests
    // ============================================================================

    #[test]
    fn test_while_false_no_execution() {
        let source = "while false { <- 42; } else { <- 0; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 0, bits: 64 });
    }

    #[test]
    fn test_while_true_with_yield_breaks() {
        let source = "while true { <- 42; } else { <- 0; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
    }

    #[test]
    fn test_while_loop_counter() {
        let source = "{ let i = 0;\nwhile i < 5 { i = i + 1; }\n<- i; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 5, bits: 64 });
    }

    #[test]
    fn test_while_loop_sum() {
        let source =
            "{ let sum = 0;\nlet i = 1;\nwhile i <= 5 { sum = sum + i;\ni = i + 1; }\n<- sum; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 15,
                bits: 64
            }
        );
    }

    #[test]
    fn test_while_with_conditional_yield() {
        let source = "{ let i = 0;\nlet result = 0;\nwhile i < 10 { i = i + 1;\nif i == 3 { result = i;\n<- void; } }\n<- result; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 3, bits: 64 });
    }

    #[test]
    fn test_while_nested_block_yield_doesnt_break() {
        let source = "{ let i = 0;\nwhile i < 3 { let x = { <- 10; };\ni = i + 1; }\n<- i; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 3, bits: 64 });
    }

    #[test]
    fn test_while_array_manipulation() {
        let source = "{ let arr = [0, 0, 0];\nlet i = 0;\nwhile i < 3 { arr[i] = i * 2;\ni = i + 1; }\n<- arr[2]; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 4, bits: 64 });
    }

    #[test]
    fn test_while_with_void_yield() {
        let source = "{ let i = 0;\nwhile i < 5 { i = i + 1;\nif i == 10 { <- void; } }\n<- i; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 5, bits: 64 });
    }

    // ============================================================================
    // Break and While...Else Tests
    // ============================================================================

    #[test]
    fn test_break_exits_loop() {
        let source = "while true { break 42; } else { <- 0; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
    }

    #[test]
    fn test_break_from_nested_block() {
        let source = "while true { { { break 99; } } } else { <- 0; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 99,
                bits: 64
            }
        );
    }

    #[test]
    fn test_break_from_nested_if() {
        let source = "let i = 0;\nwhile true { i = i + 1;\nif i > 3 { break i * 10; } else { } } else { <- 0; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 40,
                bits: 64
            }
        );
    }

    #[test]
    fn test_yield_in_nested_block_doesnt_break_loop() {
        let source = "{ let i = 0;\nwhile i < 3 { i = i + 1;\nlet x = { <- 100; }; }\n<- i; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 3, bits: 64 });
    }

    #[test]
    fn test_while_else_executes_on_false_condition() {
        let source = "{ let i = 0;\n<- while i < 3 { i = i + 1; } else { <- 42; }; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 42,
                bits: 64
            }
        );
    }

    #[test]
    fn test_while_else_skipped_on_break() {
        let source = "while true { break 100; } else { <- 200; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 100,
                bits: 64
            }
        );
    }

    #[test]
    fn test_while_else_skipped_on_yield() {
        let source = "while true { <- 100; } else { <- 200; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 100,
                bits: 64
            }
        );
    }

    #[test]
    fn test_while_else_with_initial_false_condition() {
        let source = "while false { <- 1; } else { <- 2; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 2, bits: 64 });
    }

    #[test]
    fn test_break_and_yield_coexist() {
        let source = r#"
        {
            let i = 0;
            <- while true {
                i = i + 1;
                if i == 3 {
                    <- void;
                } else { }
                if i == 7 {
                    break i;
                } else { }
            } else {
                <- 0;
            };
        }"#;
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 7, bits: 64 });
    }

    #[test]
    fn test_break_outside_loop_error() {
        let source = "break 42;";
        let result = interpret(source);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("can only be used inside loops")
        );
    }

    #[test]
    fn test_break_in_block_outside_loop_error() {
        let source = "{ break 10; }";
        let result = interpret(source);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("can only be used inside loops")
                || err.message.contains("only be used inside loops")
        );
    }

    #[test]
    fn test_nested_while_with_break_inner() {
        let source = "{ let outer = 0;\nwhile outer < 3 { outer = outer + 1;\nlet inner = 0;\nlet val = while inner < 5 { inner = inner + 1;\nif inner == 2 { break inner; } else { } } else { <- 0; }; }\n<- outer; }";
        let result = interpret(source).unwrap();
        assert_eq!(result, Value::SignedInt { value: 3, bits: 64 });
    }

    #[test]
    fn test_while_else_with_multiple_iterations() {
        let source = "let sum = 0;\nlet i = 1;\nwhile i <= 5 { sum = sum + i;\ni = i + 1; } else { <- sum; }";
        let result = interpret(source).unwrap();
        assert_eq!(
            result,
            Value::SignedInt {
                value: 15,
                bits: 64
            }
        );
    }
}
