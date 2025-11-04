use crate::ast::{
    BinaryExpr, BinaryOp, Block, Expression, Program, Statement, Type, UnaryExpr, UnaryOp,
    VariableDeclaration,
};
use crate::typed_ast::{
    TypedBinaryExpr, TypedBlock, TypedExpression, TypedProgram, TypedStatement, TypedUnaryExpr,
    TypedVariableDeclaration, TypedYieldStatement,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TypeVar {
    Concrete(Type),
    IntVar(usize),
    FloatVar(usize),
}

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeVar::Concrete(t) => write!(f, "{}", t),
            TypeVar::IntVar(id) => write!(f, "?Int{}", id),
            TypeVar::FloatVar(id) => write!(f, "?Float{}", id),
        }
    }
}

type Constraint = (TypeVar, TypeVar);

type Substitution = HashMap<usize, TypeVar>;

pub struct TypeChecker {
    scopes: Vec<HashMap<String, TypeVar>>,
    next_var_id: usize,
    constraints: Vec<Constraint>,
    substitution: Substitution,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            scopes: vec![HashMap::new()],
            next_var_id: 0,
            constraints: Vec::new(),
            substitution: HashMap::new(),
        }
    }

    fn fresh_int_var(&mut self) -> TypeVar {
        let id = self.next_var_id;
        self.next_var_id += 1;
        TypeVar::IntVar(id)
    }

    fn fresh_float_var(&mut self) -> TypeVar {
        let id = self.next_var_id;
        self.next_var_id += 1;
        TypeVar::FloatVar(id)
    }

    fn add_constraint(&mut self, t1: TypeVar, t2: TypeVar) {
        self.constraints.push((t1, t2));
    }

    pub fn check_program(&mut self, program: &Program) -> Result<TypedProgram, String> {
        // First pass: generate constraints
        for statement in &program.statements {
            self.check_statement(statement)?;
        }

        // Second pass: solve constraints
        self.solve_constraints()?;

        // Third pass: convert to typed IR with resolved types
        let mut typed_statements = Vec::new();
        for statement in &program.statements {
            typed_statements.push(self.type_statement(statement)?);
        }

        Ok(TypedProgram {
            statements: typed_statements,
        })
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<TypeVar, String> {
        match statement {
            Statement::VariableDeclaration(var_decl) => self.check_variable_declaration(var_decl),
            Statement::Yield(yield_stmt) => self.infer_expression(&yield_stmt.value),
            Statement::Expression(expr) => self.infer_expression(expr),
        }
    }

    fn check_variable_declaration(
        &mut self,
        var_decl: &VariableDeclaration,
    ) -> Result<TypeVar, String> {
        // Infer the type of the initializer
        let inferred_type = self.infer_expression(&var_decl.initializer)?;

        // If there's a type annotation, constrain the inferred type to match it
        let var_type = if let Some(annotation) = &var_decl.type_annotation {
            let annotated_type = TypeVar::Concrete(annotation.clone());
            self.add_constraint(inferred_type.clone(), annotated_type.clone());
            annotated_type
        } else {
            inferred_type
        };

        // Check for redefinition in current scope
        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.contains_key(&var_decl.name) {
            return Err(format!(
                "Variable '{}' is already defined in this scope",
                var_decl.name
            ));
        }

        // Define the variable
        current_scope.insert(var_decl.name.clone(), var_type.clone());

        Ok(var_type)
    }

    fn infer_expression(&mut self, expression: &Expression) -> Result<TypeVar, String> {
        match expression {
            Expression::Integer(_) => Ok(self.fresh_int_var()),
            Expression::Float(_) => Ok(self.fresh_float_var()),
            Expression::Boolean(_) => Ok(TypeVar::Concrete(Type::Bool)),
            Expression::Void => Ok(TypeVar::Concrete(Type::Void)),
            Expression::Identifier(name) => self.lookup_variable(name),
            Expression::Binary(binary) => self.check_binary_expression(binary),
            Expression::Unary(unary) => self.check_unary_expression(unary),
            Expression::Block(block) => self.check_block(block),
            Expression::Grouping(expr) => self.infer_expression(expr),
        }
    }

    fn check_binary_expression(&mut self, binary: &BinaryExpr) -> Result<TypeVar, String> {
        use BinaryOp::*;

        let left_type = self.infer_expression(&binary.left)?;
        let right_type = self.infer_expression(&binary.right)?;

        match binary.operator {
            // Assignment operators
            Assign | PlusAssign | MinusAssign | MultAssign | DivAssign | ModAssign | AndAssign
            | OrAssign | XorAssign | LShiftAssign | RShiftAssign => {
                self.add_constraint(left_type.clone(), right_type);
                Ok(left_type)
            }

            // logical operators - both sides must be bool
            LogicalAnd | LogicalOr => {
                for side in vec![&left_type, &right_type] {
                    match side {
                        TypeVar::IntVar(_) => {
                            return Err("Expected Boolean operand, found I64".to_string());
                        }
                        TypeVar::FloatVar(_) => {
                            return Err("Expected Boolean operand, found F64".to_string());
                        }
                        TypeVar::Concrete(ty) if !matches!(ty, Type::Bool) => {
                            return Err(format!("Expected Boolean operand, found {}", ty));
                        }
                        _ => {}
                    }
                }

                let bool_type = TypeVar::Concrete(Type::Bool);

                self.add_constraint(left_type, bool_type.clone());
                self.add_constraint(right_type, bool_type.clone());
                Ok(bool_type)
            }

            // Comparison operators - operands must match, result is Bool
            Equal | NotEqual | LessThan | LessEqual | GreaterThan | GreaterEqual => {
                self.add_constraint(left_type, right_type);
                Ok(TypeVar::Concrete(Type::Bool))
            }

            // Arithmetic and bitwise operators - operands must match, result is same type
            Add | Subtract | Multiply | Divide | Modulo | BitwiseAnd | BitwiseOr | BitwiseXor => {
                self.add_constraint(left_type.clone(), right_type);
                Ok(left_type)
            }

            // logical operators - both sides must be integers (types don't need to match though)
            LeftShift | RightShift => {
                for side in vec![&left_type, &right_type] {
                    match side {
                        TypeVar::IntVar(_) => {}
                        TypeVar::Concrete(ty)
                            if matches!(ty, Type::Signed(_) | Type::Unsigned(_)) => {}
                        _ => return Err(format!("Expected Integer operand, found {}", side)),
                    }
                }
                Ok(left_type)
            }
        }
    }

    fn check_unary_expression(&mut self, unary: &UnaryExpr) -> Result<TypeVar, String> {
        let operand_type = self.infer_expression(&unary.operand)?;

        match unary.operator {
            UnaryOp::Plus | UnaryOp::Minus | UnaryOp::BitwiseNot => {
                // These operators preserve the operand type
                Ok(operand_type)
            }
            UnaryOp::LogicalNot => {
                // LogicalNot requires Bool and returns Bool
                match &operand_type {
                    TypeVar::IntVar(_) => {
                        return Err("Logical NOT requires boolean operand, found I64".to_string());
                    }
                    TypeVar::FloatVar(_) => {
                        return Err("Logical NOT requires boolean operand, found F64".to_string());
                    }
                    TypeVar::Concrete(ty) if !matches!(ty, Type::Bool) => {
                        return Err(format!(
                            "Logical NOT requires boolean operand, found {}",
                            ty
                        ));
                    }
                    _ => {}
                }
                self.add_constraint(operand_type, TypeVar::Concrete(Type::Bool));
                Ok(TypeVar::Concrete(Type::Bool))
            }
        }
    }

    fn check_block(&mut self, block: &Block) -> Result<TypeVar, String> {
        // Push a new scope
        self.scopes.push(HashMap::new());

        let mut block_type = TypeVar::Concrete(Type::Void);
        let mut yield_types: Vec<TypeVar> = Vec::new();

        // Check all statements and collect yield types
        // Note: Even though yield acts as a break at runtime, we still type-check
        // all statements to catch errors in unreachable code
        for statement in &block.statements {
            match statement {
                Statement::Yield(yield_stmt) => {
                    let yield_type = self.infer_expression(&yield_stmt.value)?;
                    yield_types.push(yield_type);
                }
                _ => {
                    self.check_statement(statement)?;
                }
            }
        }

        // All yields must have the same type
        // The unification phase will verify they can actually unify
        if !yield_types.is_empty() {
            block_type = yield_types[0].clone();
            for yield_type in yield_types.iter().skip(1) {
                self.add_constraint(block_type.clone(), yield_type.clone());
            }
        }

        // Pop the scope
        self.scopes.pop();

        Ok(block_type)
    }

    fn lookup_variable(&self, name: &str) -> Result<TypeVar, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Ok(ty.clone());
            }
        }
        Err(format!("Undefined variable '{}'", name))
    }

    fn solve_constraints(&mut self) -> Result<(), String> {
        for (t1, t2) in self.constraints.clone() {
            self.unify(t1, t2)?;
        }
        Ok(())
    }

    fn unify(&mut self, t1: TypeVar, t2: TypeVar) -> Result<(), String> {
        let t1 = self.apply_substitution(&t1);
        let t2 = self.apply_substitution(&t2);

        match (t1.clone(), t2.clone()) {
            // Same type variable - nothing to do
            (TypeVar::IntVar(v1), TypeVar::IntVar(v2)) if v1 == v2 => Ok(()),
            (TypeVar::FloatVar(v1), TypeVar::FloatVar(v2)) if v1 == v2 => Ok(()),

            // IntVar can only unify with integer types
            (TypeVar::IntVar(v), TypeVar::Concrete(ty))
            | (TypeVar::Concrete(ty), TypeVar::IntVar(v)) => {
                if matches!(ty, Type::Signed(_) | Type::Unsigned(_)) {
                    self.occurs_check_int(v, &TypeVar::Concrete(ty.clone()))?;
                    self.substitution.insert(v, TypeVar::Concrete(ty));
                    Ok(())
                } else {
                    Err(format!(
                        "Type mismatch: Integer type expected, found {}",
                        ty
                    ))
                }
            }

            // Two IntVars can unify
            (TypeVar::IntVar(v1), TypeVar::IntVar(v2)) => {
                self.substitution.insert(v1, TypeVar::IntVar(v2));
                Ok(())
            }

            // FloatVar can only unify with float types
            (TypeVar::FloatVar(v), TypeVar::Concrete(ty))
            | (TypeVar::Concrete(ty), TypeVar::FloatVar(v)) => {
                if matches!(ty, Type::F32 | Type::F64) {
                    self.occurs_check_float(v, &TypeVar::Concrete(ty.clone()))?;
                    self.substitution.insert(v, TypeVar::Concrete(ty));
                    Ok(())
                } else {
                    Err(format!("Type mismatch: Float type expected, found {}", ty))
                }
            }

            // Two FloatVars can unify
            (TypeVar::FloatVar(v1), TypeVar::FloatVar(v2)) => {
                self.substitution.insert(v1, TypeVar::FloatVar(v2));
                Ok(())
            }

            // IntVar and FloatVar cannot unify
            (TypeVar::IntVar(_), TypeVar::FloatVar(_))
            | (TypeVar::FloatVar(_), TypeVar::IntVar(_)) => {
                Err("Type mismatch: cannot unify Integer type with Float type".to_string())
            }

            // Unify concrete types
            (TypeVar::Concrete(ty1), TypeVar::Concrete(ty2)) => {
                if self.types_match(&ty1, &ty2) {
                    Ok(())
                } else {
                    // Check for specific error cases to provide better messages
                    match (&ty1, &ty2) {
                        (Type::Signed(_), Type::Unsigned(_))
                        | (Type::Unsigned(_), Type::Signed(_)) => Err(format!(
                            "Cannot perform operation on mixed signed/unsigned types: {} and {}",
                            ty1, ty2
                        )),
                        _ => Err(format!("Type mismatch: cannot unify {} with {}", ty1, ty2)),
                    }
                }
            }
        }
    }

    fn apply_substitution(&self, ty: &TypeVar) -> TypeVar {
        match ty {
            TypeVar::IntVar(v) | TypeVar::FloatVar(v) => {
                if let Some(substituted) = self.substitution.get(v) {
                    // Recursively apply substitution
                    self.apply_substitution(substituted)
                } else {
                    ty.clone()
                }
            }
            TypeVar::Concrete(_) => ty.clone(),
        }
    }

    fn occurs_check_int(&self, var: usize, ty: &TypeVar) -> Result<(), String> {
        match ty {
            TypeVar::IntVar(v) if *v == var => {
                Err("Occurs check failed: infinite type".to_string())
            }
            TypeVar::IntVar(v) | TypeVar::FloatVar(v) => {
                if let Some(substituted) = self.substitution.get(v) {
                    self.occurs_check_int(var, substituted)
                } else {
                    Ok(())
                }
            }
            TypeVar::Concrete(_) => Ok(()),
        }
    }

    fn occurs_check_float(&self, var: usize, ty: &TypeVar) -> Result<(), String> {
        match ty {
            TypeVar::FloatVar(v) if *v == var => {
                Err("Occurs check failed: infinite type".to_string())
            }
            TypeVar::IntVar(v) | TypeVar::FloatVar(v) => {
                if let Some(substituted) = self.substitution.get(v) {
                    self.occurs_check_float(var, substituted)
                } else {
                    Ok(())
                }
            }
            TypeVar::Concrete(_) => Ok(()),
        }
    }

    fn types_match(&self, ty1: &Type, ty2: &Type) -> bool {
        match (ty1, ty2) {
            (Type::Signed(b1), Type::Signed(b2)) => b1 == b2,
            (Type::Unsigned(b1), Type::Unsigned(b2)) => b1 == b2,
            (Type::F32, Type::F32) => true,
            (Type::F64, Type::F64) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Void, Type::Void) => true,
            _ => false,
        }
    }

    fn resolve_type_var(&self, type_var: &TypeVar) -> Result<Type, String> {
        let resolved = self.apply_substitution(type_var);
        match resolved {
            TypeVar::Concrete(ty) => Ok(ty),
            TypeVar::IntVar(_) => {
                // Default to i64 for unresolved integer type variables
                Ok(Type::Signed(64))
            }
            TypeVar::FloatVar(_) => {
                // Default to F64 for unresolved float type variables
                Ok(Type::F64)
            }
        }
    }

    fn type_statement(&mut self, statement: &Statement) -> Result<TypedStatement, String> {
        match statement {
            Statement::VariableDeclaration(var_decl) => Ok(TypedStatement::VariableDeclaration(
                self.type_variable_declaration(var_decl)?,
            )),
            Statement::Yield(yield_stmt) => {
                let typed_value = self.type_expression(&yield_stmt.value)?;
                Ok(TypedStatement::Yield(TypedYieldStatement {
                    value: typed_value,
                }))
            }
            Statement::Expression(expr) => {
                Ok(TypedStatement::Expression(self.type_expression(expr)?))
            }
        }
    }

    fn type_variable_declaration(
        &mut self,
        var_decl: &VariableDeclaration,
    ) -> Result<TypedVariableDeclaration, String> {
        // Pass the type annotation as expected type when converting initializer
        let typed_init = self
            .type_expression_with_hint(&var_decl.initializer, var_decl.type_annotation.as_ref())?;
        let var_type = if let Some(annotation) = &var_decl.type_annotation {
            annotation.clone()
        } else {
            typed_init.get_type()
        };

        Ok(TypedVariableDeclaration {
            mutable: var_decl.mutable,
            name: var_decl.name.clone(),
            var_type,
            initializer: typed_init,
        })
    }

    fn type_expression(&mut self, expression: &Expression) -> Result<TypedExpression, String> {
        self.type_expression_with_hint(expression, None)
    }

    fn resolve_numeric_literal_type(
        &mut self,
        expression: &Expression,
        hint: Option<&Type>,
    ) -> Result<Type, String> {
        if let Some(ty) = hint {
            Ok(ty.clone())
        } else {
            let type_var = self.infer_expression(expression)?;
            Ok(self.resolve_type_var(&type_var)?)
        }
    }

    fn type_expression_with_hint(
        &mut self,
        expression: &Expression,
        hint: Option<&Type>,
    ) -> Result<TypedExpression, String> {
        match expression {
            Expression::Integer(s) => Ok(TypedExpression::Integer(
                s.clone(),
                self.resolve_numeric_literal_type(expression, hint)?,
            )),
            Expression::Float(s) => Ok(TypedExpression::Float(
                s.clone(),
                self.resolve_numeric_literal_type(expression, hint)?,
            )),
            Expression::Boolean(b) => Ok(TypedExpression::Boolean(*b)),
            Expression::Void => Ok(TypedExpression::Void),
            Expression::Identifier(name) => {
                let type_var = self.lookup_variable(name)?;
                let resolved_type = self.resolve_type_var(&type_var)?;
                Ok(TypedExpression::Identifier(name.clone(), resolved_type))
            }
            Expression::Binary(binary) => {
                // For arithmetic operators, propagate hint to both operands
                // For comparison/logical operators, don't propagate
                let propagate_hint = matches!(
                    binary.operator,
                    BinaryOp::Add
                        | BinaryOp::Subtract
                        | BinaryOp::Multiply
                        | BinaryOp::Divide
                        | BinaryOp::Modulo
                        | BinaryOp::BitwiseAnd
                        | BinaryOp::BitwiseOr
                        | BinaryOp::BitwiseXor
                        | BinaryOp::LeftShift
                        | BinaryOp::RightShift
                );

                let typed_left = if propagate_hint {
                    self.type_expression_with_hint(&binary.left, hint)?
                } else {
                    self.type_expression(&binary.left)?
                };

                // For arithmetic operators, use left's type for right if no hint
                let left_type_for_right = typed_left.get_type();
                let right_hint = if propagate_hint {
                    hint.or(Some(&left_type_for_right))
                } else {
                    None
                };

                let typed_right = self.type_expression_with_hint(&binary.right, right_hint)?;

                // Determine result type based on operator
                let result_type = match binary.operator {
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
                    | BinaryOp::RShiftAssign => typed_left.get_type(),
                    BinaryOp::LogicalAnd
                    | BinaryOp::LogicalOr
                    | BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::LessThan
                    | BinaryOp::LessEqual
                    | BinaryOp::GreaterThan
                    | BinaryOp::GreaterEqual => Type::Bool,
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Modulo
                    | BinaryOp::BitwiseAnd
                    | BinaryOp::BitwiseOr
                    | BinaryOp::BitwiseXor
                    | BinaryOp::LeftShift
                    | BinaryOp::RightShift => typed_left.get_type(),
                };

                Ok(TypedExpression::Binary(Box::new(TypedBinaryExpr {
                    left: typed_left,
                    operator: binary.operator.clone(),
                    right: typed_right,
                    result_type,
                })))
            }
            Expression::Unary(unary) => {
                // Propagate hint for Plus, Minus, BitwiseNot; don't propagate for LogicalNot
                let operand_hint = match unary.operator {
                    UnaryOp::Plus | UnaryOp::Minus | UnaryOp::BitwiseNot => hint,
                    UnaryOp::LogicalNot => None,
                };

                let typed_operand = self.type_expression_with_hint(&unary.operand, operand_hint)?;
                let result_type = match unary.operator {
                    UnaryOp::Plus | UnaryOp::Minus | UnaryOp::BitwiseNot => {
                        typed_operand.get_type()
                    }
                    UnaryOp::LogicalNot => Type::Bool,
                };

                Ok(TypedExpression::Unary(Box::new(TypedUnaryExpr {
                    operator: unary.operator.clone(),
                    operand: typed_operand,
                    result_type,
                })))
            }
            Expression::Block(block) => {
                // Push scope for the block
                self.scopes.push(HashMap::new());

                let mut typed_statements = Vec::new();
                let mut block_type = Type::Void;

                for statement in &block.statements {
                    let typed_stmt = self.type_statement(statement)?;

                    // Track variable declarations in the scope
                    if let TypedStatement::VariableDeclaration(ref var_decl) = typed_stmt {
                        let current_scope = self.scopes.last_mut().unwrap();
                        current_scope.insert(
                            var_decl.name.clone(),
                            TypeVar::Concrete(var_decl.var_type.clone()),
                        );
                    }

                    // If it's a yield, use its type as the block type
                    if let TypedStatement::Yield(ref yield_stmt) = typed_stmt {
                        block_type = yield_stmt.value.get_type();
                        typed_statements.push(typed_stmt);
                        // Yield acts as break, so stop here
                        break;
                    }
                    typed_statements.push(typed_stmt);
                }

                // Pop scope
                self.scopes.pop();

                Ok(TypedExpression::Block(TypedBlock {
                    statements: typed_statements,
                    block_type,
                }))
            }
            Expression::Grouping(expr) => {
                let typed = self.type_expression_with_hint(expr, hint)?;
                Ok(TypedExpression::Grouping(Box::new(typed)))
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
