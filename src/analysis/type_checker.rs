use crate::frontend::ast::{
    BinaryExpr, BinaryOp, Block, BreakStatement, Expression, Program, Statement, Type, UnaryExpr, UnaryOp,
    VariableDeclaration,
};
use crate::analysis::typed_ast::{
    TypedBinaryExpr, TypedBlock, TypedBreakStatement, TypedExpression, TypedIfExpr, TypedIndexExpr,
    TypedProgram, TypedStatement, TypedUnaryExpr, TypedVariableDeclaration, TypedWhileExpr,
    TypedYieldStatement,
};
use crate::error::{TypeError, TypeErrorKind};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TypeVar {
    Concrete(Type),
    IntVar(usize),
    FloatVar(usize),
    ArrayVar(usize, Option<Box<TypeVar>>, usize),
}

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeVar::Concrete(t) => write!(f, "{}", t),
            TypeVar::IntVar(id) => write!(f, "?Int{}", id),
            TypeVar::FloatVar(id) => write!(f, "?Float{}", id),
            TypeVar::ArrayVar(id, _, _) => write!(f, "?Array{}", id),
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
    in_loop: bool, // Track if we're currently inside a loop
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            scopes: vec![HashMap::new()],
            next_var_id: 0,
            constraints: Vec::new(),
            substitution: HashMap::new(),
            in_loop: false,
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

    fn fresh_array_var(&mut self, ty: Option<TypeVar>, size: usize) -> TypeVar {
        let id = self.next_var_id;
        self.next_var_id += 1;
        TypeVar::ArrayVar(id, match ty {
            Some(t) => Some(Box::new(t)),
            None => None,
        }, size)
    }

    fn add_constraint(&mut self, t1: TypeVar, t2: TypeVar) {
        self.constraints.push((t1, t2));
    }

    pub fn check_program(&mut self, program: &Program) -> Result<TypedProgram, TypeError> {
        // Clear constraints and substitutions from previous calls
        // but preserve scopes so variables persist in REPL
        self.constraints.clear();
        self.substitution.clear();

        // Save the current global scope for rollback on error (important for REPL)
        // If type checking fails, we don't want partially-declared variables in scope
        let saved_global_scope = self.scopes[0].clone();

        let result = (|| -> Result<TypedProgram, TypeError> {
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
        })();

        // If any pass failed, restore the global scope
        if result.is_err() {
            self.scopes[0] = saved_global_scope;
        }

        result
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<TypeVar, TypeError> {
        match statement {
            Statement::VariableDeclaration(var_decl) => self.check_variable_declaration(var_decl),
            Statement::Yield(yield_stmt) => self.infer_expression(&yield_stmt.value),
            Statement::Break(break_stmt) => {
                if !self.in_loop {
                    return Err(TypeError::new(
                        TypeErrorKind::InvalidOperation,
                        "break statement can only be used inside loops".to_string(),
                    ));
                }
                self.infer_expression(&break_stmt.value)
            }
            Statement::Expression(expr) => self.infer_expression(expr),
        }
    }

    fn check_variable_declaration(
        &mut self,
        var_decl: &VariableDeclaration,
    ) -> Result<TypeVar, TypeError> {
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
            return Err(TypeError::new(
                TypeErrorKind::InvalidOperation,
                format!(
                    "Variable '{}' is already defined in this scope",
                    var_decl.name
                ),
            ));
        }

        // Define the variable
        current_scope.insert(var_decl.name.clone(), var_type.clone());

        Ok(var_type)
    }

    fn infer_expression(&mut self, expression: &Expression) -> Result<TypeVar, TypeError> {
        match expression {
            Expression::Integer(_) => Ok(self.fresh_int_var()),
            Expression::Float(_) => Ok(self.fresh_float_var()),
            Expression::Boolean(_) => Ok(TypeVar::Concrete(Type::Bool)),
            Expression::Void => Ok(TypeVar::Concrete(Type::Void)),
            Expression::ArrayLiteral(elements) => self.check_array_literal(elements),
            Expression::Identifier(name) => self.lookup_variable(name),
            Expression::Binary(binary) => self.check_binary_expression(binary),
            Expression::Unary(unary) => self.check_unary_expression(unary),
            Expression::Index(index_expr) => self.check_index_expression(index_expr),
            Expression::Block(block) => self.check_block(block),
            Expression::If(if_expr) => self.check_if_expression(if_expr),
            Expression::While(while_expr) => self.check_while_expression(while_expr),
            Expression::Grouping(expr) => self.infer_expression(expr),
        }
    }

    fn check_binary_expression(&mut self, binary: &BinaryExpr) -> Result<TypeVar, TypeError> {
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
                            return Err(TypeError::new(TypeErrorKind::TypeMismatch, "Expected Boolean operand, found I64".to_string()));
                        }
                        TypeVar::FloatVar(_) => {
                            return Err(TypeError::new(TypeErrorKind::TypeMismatch, "Expected Boolean operand, found F64".to_string()));
                        }
                        TypeVar::Concrete(ty) if !matches!(ty, Type::Bool) => {
                            return Err(TypeError::new(
                                TypeErrorKind::TypeMismatch,
                                format!("Expected Boolean operand, found {}", ty),
                            ));
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
                        _ => return Err(TypeError::new(
                            TypeErrorKind::TypeMismatch,
                            format!("Expected Integer operand, found {}", side),
                        )),
                    }
                }
                Ok(left_type)
            }
        }
    }

    fn check_unary_expression(&mut self, unary: &UnaryExpr) -> Result<TypeVar, TypeError> {
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
                        return Err(TypeError::new(TypeErrorKind::TypeMismatch, "Logical NOT requires boolean operand, found I64".to_string()));
                    }
                    TypeVar::FloatVar(_) => {
                        return Err(TypeError::new(TypeErrorKind::TypeMismatch, "Logical NOT requires boolean operand, found F64".to_string()));
                    }
                    TypeVar::Concrete(ty) if !matches!(ty, Type::Bool) => {
                        return Err(TypeError::new(
                            TypeErrorKind::TypeMismatch,
                            format!("Logical NOT requires boolean operand, found {}", ty),
                        ));
                    }
                    _ => {}
                }
                self.add_constraint(operand_type, TypeVar::Concrete(Type::Bool));
                Ok(TypeVar::Concrete(Type::Bool))
            }
        }
    }

    fn check_array_literal(&mut self, elements: &[Expression]) -> Result<TypeVar, TypeError> {
        // Infer type of first element
        let first_type = if !elements.is_empty() {
            let first = self.infer_expression(&elements[0])?;
            // All subsequent elements must have the same type
            for element in elements.iter().skip(1) {
                let elem_type = self.infer_expression(element)?;
                self.add_constraint(first.clone(), elem_type);
            }
            Some(first)
        } else {
            // Empty array - element type will need to come from context/annotation
            None
        };

        // Return array type with inferred element type and size
        Ok(self.fresh_array_var(first_type, elements.len()))
    }

    fn check_index_expression(&mut self, index_expr: &crate::frontend::ast::IndexExpr) -> Result<TypeVar, TypeError> {
        let array_type = self.infer_expression(&index_expr.array)?;
        let index_type = self.infer_expression(&index_expr.index)?;

        // Index must be an integer type (any integer type is fine)
        match &index_type {
            TypeVar::IntVar(_) => {}, // OK
            TypeVar::Concrete(Type::Signed(_)) | TypeVar::Concrete(Type::Unsigned(_)) => {}, // OK
            TypeVar::FloatVar(_) => {
                return Err(TypeError::new(TypeErrorKind::TypeMismatch, "Array index must be an integer, found F64".to_string()));
            }
            TypeVar::Concrete(ty) => {
                return Err(TypeError::new(
                    TypeErrorKind::TypeMismatch,
                    format!("Array index must be an integer, found {}", ty),
                ));
            }
            TypeVar::ArrayVar(..) => {
                return Err(TypeError::new(TypeErrorKind::TypeMismatch, "Array index must be an integer, found Array".to_string()));
            }
        }

        // Extract element type from array type
        match array_type {
            TypeVar::Concrete(Type::Array(elem_type, _)) => Ok(TypeVar::Concrete(*elem_type)),
            TypeVar::ArrayVar(_, Some(elem_type), _) => Ok(*elem_type),
            TypeVar::ArrayVar(_, None, _) => {
                Err(TypeError::new(TypeErrorKind::InvalidOperation, "Cannot infer element type of array without type annotation".to_string()))
            }
            _ => Err(TypeError::new(
                TypeErrorKind::InvalidOperation,
                "Cannot index into non-array type".to_string(),
            )),
        }
    }

    fn check_block(&mut self, block: &Block) -> Result<TypeVar, TypeError> {
        // Push a new scope
        self.scopes.push(HashMap::new());

        let mut block_type = TypeVar::Concrete(Type::Void);
        let mut yield_types: Vec<TypeVar> = Vec::new();

        // Check all statements and collect yield types
        // Note: Even though yield acts as a break at runtime, we still type-check
        // all statements to catch errors in unreachable code
        // Break statements do NOT contribute to block type - they exit the loop, not the block
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

    fn check_if_expression(&mut self, if_expr: &crate::frontend::ast::IfExpr) -> Result<TypeVar, TypeError> {
        // Check condition must be boolean
        let condition_type = self.infer_expression(&if_expr.condition)?;
        self.add_constraint(condition_type, TypeVar::Concrete(Type::Bool));

        // Check then block
        let then_type = self.check_block(&if_expr.then_block)?;

        // Check all else-if branches
        let mut branch_types = vec![then_type.clone()];
        for (else_if_condition, else_if_block) in &if_expr.else_ifs {
            let else_if_cond_type = self.infer_expression(else_if_condition)?;
            self.add_constraint(else_if_cond_type, TypeVar::Concrete(Type::Bool));
            let else_if_type = self.check_block(else_if_block)?;
            branch_types.push(else_if_type);
        }

        // Check else block if present
        let result_type = if let Some(else_block) = &if_expr.else_block {
            let else_type = self.check_block(else_block)?;
            branch_types.push(else_type);

            // All branches must have the same type
            let unified_type = branch_types[0].clone();
            for branch_type in branch_types.iter().skip(1) {
                self.add_constraint(unified_type.clone(), branch_type.clone());
            }
            unified_type
        } else {
            // No else branch - check if then block has a yield
            // If it does, we require an else branch
            match &then_type {
                TypeVar::Concrete(Type::Void) => {
                    // All branches must be void if there's no else
                    for branch_type in branch_types.iter() {
                        self.add_constraint(branch_type.clone(), TypeVar::Concrete(Type::Void));
                    }
                    TypeVar::Concrete(Type::Void)
                }
                _ => {
                    return Err(TypeError::new(TypeErrorKind::InvalidOperation, "If expression with non-void branch requires an else clause".to_string()));
                }
            }
        };

        Ok(result_type)
    }

    fn check_while_expression(&mut self, while_expr: &crate::frontend::ast::WhileExpr) -> Result<TypeVar, TypeError> {
        // Check condition must be boolean
        let condition_type = self.infer_expression(&while_expr.condition)?;
        self.add_constraint(condition_type, TypeVar::Concrete(Type::Bool));

        // Find all break statements in the body (excluding nested loops)
        let breaks = self.find_all_breaks(&while_expr.body);

        // Set loop context and check body block
        let old_in_loop = self.in_loop;
        self.in_loop = true;

        // Check all statements in the body (for general type checking)
        // But we'll handle break types separately
        let yield_body_type = self.check_block(&while_expr.body)?;

        self.in_loop = old_in_loop;

        // Collect break types (only if we're in a loop context)
        let mut break_types: Vec<TypeVar> = Vec::new();
        for break_stmt in breaks {
            let break_type = self.infer_expression(&break_stmt.value)?;
            break_types.push(break_type);
        }

        // Determine the body type based on breaks
        let body_type = if !break_types.is_empty() {
            let first_break_type = break_types[0].clone();
            for break_type in break_types.iter().skip(1) {
                self.add_constraint(first_break_type.clone(), break_type.clone());
            }
            first_break_type
        } else {
            // If no breaks, use the yield type (these are constrained to be the same)
            yield_body_type.clone()
        };

        // Make sure that break statements match any yields
        match yield_body_type {
            TypeVar::Concrete(Type::Void) => {}
            _ => self.add_constraint(yield_body_type, body_type.clone())
        }

        // Check else block if present
        let result_type = if let Some(else_block) = &while_expr.else_block {
            let else_type = self.check_block(else_block)?;

            // If body has breaks (non-void), they must match the else type
            // If body is void (no breaks), the loop returns the else type
            match &body_type {
                TypeVar::Concrete(Type::Void) => {
                    // No breaks in body - loop returns else type
                    else_type
                }
                _ => {
                    // Body has breaks - they must match else type
                    self.add_constraint(body_type.clone(), else_type.clone());
                    body_type
                }
            }
        } else {
            // No else block - body must be void (no breaks)
            match &body_type {
                TypeVar::Concrete(Type::Void) => TypeVar::Concrete(Type::Void),
                _ => {
                    return Err(TypeError::new(TypeErrorKind::InvalidOperation, "While loop with non-void break requires an else clause".to_string()));
                }
            }
        };

        Ok(result_type)
    }

    fn find_all_breaks<'a>(&self, block: &'a Block) -> Vec<&'a BreakStatement> {
        let mut breaks = Vec::new();
        self.collect_breaks_from_block(block, &mut breaks);
        breaks
    }

    fn collect_breaks_from_block<'a>(&self, block: &'a Block, breaks: &mut Vec<&'a BreakStatement>) {
        for statement in &block.statements {
            self.collect_breaks_from_statement(statement, breaks);
        }
    }

    fn collect_breaks_from_statement<'a>(&self, statement: &'a Statement, breaks: &mut Vec<&'a BreakStatement>) {
        match statement {
            Statement::Break(break_stmt) => {
                breaks.push(break_stmt);
            }
            Statement::Expression(expr) => {
                self.collect_breaks_from_expression(expr, breaks);
            }
            Statement::VariableDeclaration(var_decl) => {
                self.collect_breaks_from_expression(&var_decl.initializer, breaks);
            }
            Statement::Yield(_) => {
                // Yields don't contain breaks
            }
        }
    }

    fn collect_breaks_from_expression<'a>(&self, expression: &'a Expression, breaks: &mut Vec<&'a BreakStatement>) {
        match expression {
            Expression::Block(block) => {
                self.collect_breaks_from_block(block, breaks);
            }
            Expression::If(if_expr) => {
                // Check condition for breaks
                self.collect_breaks_from_expression(&if_expr.condition, breaks);
                // Check then block
                self.collect_breaks_from_block(&if_expr.then_block, breaks);
                // Check else-if branches
                for (else_if_cond, else_if_block) in &if_expr.else_ifs {
                    self.collect_breaks_from_expression(else_if_cond, breaks);
                    self.collect_breaks_from_block(else_if_block, breaks);
                }
                // Check else block
                if let Some(else_block) = &if_expr.else_block {
                    self.collect_breaks_from_block(else_block, breaks);
                }
            }
            Expression::While(_) => {
                // Stop here - don't recurse into nested loops
                // Breaks inside nested loops belong to those loops, not this one
            }
            Expression::Binary(binary) => {
                self.collect_breaks_from_expression(&binary.left, breaks);
                self.collect_breaks_from_expression(&binary.right, breaks);
            }
            Expression::Unary(unary) => {
                self.collect_breaks_from_expression(&unary.operand, breaks);
            }
            Expression::Index(index_expr) => {
                self.collect_breaks_from_expression(&index_expr.array, breaks);
                self.collect_breaks_from_expression(&index_expr.index, breaks);
            }
            Expression::ArrayLiteral(elements) => {
                for elem in elements {
                    self.collect_breaks_from_expression(elem, breaks);
                }
            }
            Expression::Grouping(expr) => {
                self.collect_breaks_from_expression(expr, breaks);
            }
            // Literals and identifiers don't contain breaks
            Expression::Integer(_) | Expression::Float(_) | Expression::Boolean(_)
            | Expression::Void | Expression::Identifier(_) => {}
        }
    }

    fn lookup_variable(&self, name: &str) -> Result<TypeVar, TypeError> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Ok(ty.clone());
            }
        }
        Err(TypeError::new(
            TypeErrorKind::UndefinedVariable,
            format!("Undefined variable '{}'", name),
        ))
    }

    fn solve_constraints(&mut self) -> Result<(), TypeError> {
        for (t1, t2) in self.constraints.clone() {
            self.unify(t1, t2)?;
        }
        Ok(())
    }

    fn unify(&mut self, t1: TypeVar, t2: TypeVar) -> Result<(), TypeError> {
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
                    Err(TypeError::new(
                        TypeErrorKind::TypeMismatch,
                        format!("Type mismatch: Integer type expected, found {}", ty),
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
                    Err(TypeError::new(
                        TypeErrorKind::TypeMismatch,
                        format!("Type mismatch: Float type expected, found {}", ty),
                    ))
                }
            }

            // Two FloatVars can unify
            (TypeVar::FloatVar(v1), TypeVar::FloatVar(v2)) => {
                self.substitution.insert(v1, TypeVar::FloatVar(v2));
                Ok(())
            }

            // ArrayVar can only unify with Array types
            (TypeVar::ArrayVar(v, vty, len), TypeVar::Concrete(ty))
            | (TypeVar::Concrete(ty), TypeVar::ArrayVar(v, vty, len)) => {
                match &ty {
                    Type::Array(elem_ty, tlen) => {
                        // Arrays can unify only if their lengths are the same
                        if len != *tlen {
                            return Err(TypeError::new(
                                TypeErrorKind::ArraySizeMismatch,
                                format!("Array size mismatch {} != {}", len, tlen),
                            ));
                        }
                        // Arrays can unify only if their element types can unify
                        if let Some(vty_box) = vty {
                            self.unify(*vty_box, TypeVar::Concrete(*elem_ty.clone()))?;
                        }
                        self.occurs_check_array(v, &TypeVar::Concrete(ty.clone()))?;
                        self.substitution.insert(v, TypeVar::Concrete(ty));
                        Ok(())
                    }
                    _ => Err(TypeError::new(
                        TypeErrorKind::TypeMismatch,
                        format!("Type mismatch: Array type expected, found {}", ty),
                    ))
                }
            }

            (TypeVar::ArrayVar(v1, lty, llen), TypeVar::ArrayVar(v2, rty, rlen)) => {
                // Arrays can unify only if their lengths are the same
                if llen != rlen {
                    return Err(TypeError::new(
                        TypeErrorKind::ArraySizeMismatch,
                        format!("Array size mismatch {} != {}", llen, rlen),
                    ));
                }
                // Arrays can unify only if their element types can unify
                match (lty, &rty) {
                    (Some(lt), Some(rt)) => {
                        self.unify(*lt, *rt.clone())?;
                    }
                    (_, _) => {}
                };
                self.substitution.insert(v1, TypeVar::ArrayVar(v2, rty, rlen));
                Ok(())
            }

            // IntVar and FloatVar cannot unify
            (TypeVar::IntVar(_), TypeVar::FloatVar(_))
            | (TypeVar::FloatVar(_), TypeVar::IntVar(_)) => {
                Err(TypeError::new(TypeErrorKind::IncompatibleTypes, "Type mismatch: cannot unify Integer type with Float type".to_string()))
            }

            // Unify concrete types
            (TypeVar::Concrete(ty1), TypeVar::Concrete(ty2)) => {
                if self.types_match(&ty1, &ty2) {
                    Ok(())
                } else {
                    // Check for specific error cases to provide better messages
                    match (&ty1, &ty2) {
                        (Type::Signed(_), Type::Unsigned(_))
                        | (Type::Unsigned(_), Type::Signed(_)) => Err(TypeError::new(
                            TypeErrorKind::IncompatibleTypes,
                            format!("Cannot perform operation on mixed signed/unsigned types: {} and {}", ty1, ty2),
                        )),
                        _ => Err(TypeError::new(
                            TypeErrorKind::TypeMismatch,
                            format!("Type mismatch: cannot unify {} with {}", ty1, ty2),
                        )),
                    }
                }
            }
            (_, _) => Err(TypeError::new(
                TypeErrorKind::TypeMismatch,
                format!("Type mismatch: cannot unify {} with {}", t1, t2),
            )),
        }
    }

    fn apply_substitution(&self, ty: &TypeVar) -> TypeVar {
        match ty {
            TypeVar::IntVar(v) | TypeVar::FloatVar(v) | TypeVar::ArrayVar(v, ..) => {
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

    fn occurs_check_int(&self, var: usize, ty: &TypeVar) -> Result<(), TypeError> {
        match ty {
            TypeVar::IntVar(v) if *v == var => {
                Err(TypeError::new(
                    TypeErrorKind::ConstraintSolvingFailure,
                    "Occurs check failed: infinite type".to_string(),
                ))
            }
            TypeVar::IntVar(v) | TypeVar::FloatVar(v) | TypeVar::ArrayVar(v, ..) => {
                if let Some(substituted) = self.substitution.get(v) {
                    self.occurs_check_int(var, substituted)
                } else {
                    Ok(())
                }
            }
            TypeVar::Concrete(_) => Ok(()),
        }
    }

    fn occurs_check_float(&self, var: usize, ty: &TypeVar) -> Result<(), TypeError> {
        match ty {
            TypeVar::FloatVar(v) if *v == var => {
                Err(TypeError::new(
                    TypeErrorKind::ConstraintSolvingFailure,
                    "Occurs check failed: infinite type".to_string(),
                ))
            }
            TypeVar::IntVar(v) | TypeVar::FloatVar(v) | TypeVar::ArrayVar(v, ..) => {
                if let Some(substituted) = self.substitution.get(v) {
                    self.occurs_check_float(var, substituted)
                } else {
                    Ok(())
                }
            }
            TypeVar::Concrete(_) => Ok(()),
        }
    }

    fn occurs_check_array(&self, var: usize, ty: &TypeVar) -> Result<(), TypeError> {
        match ty {
            TypeVar::ArrayVar(v, ..) if *v == var => {
                Err(TypeError::new(
                    TypeErrorKind::ConstraintSolvingFailure,
                    "Occurs check failed: infinite type".to_string(),
                ))
            }
            TypeVar::IntVar(v) | TypeVar::FloatVar(v) | TypeVar::ArrayVar(v, ..) => {
                if let Some(substituted) = self.substitution.get(v) {
                    self.occurs_check_array(var, substituted)
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

    fn resolve_type_var(&self, type_var: &TypeVar) -> Result<Type, TypeError> {
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
            },
            TypeVar::ArrayVar(_, ty, len) => {
                match ty {
                    Some(var) => Ok(Type::Array(Box::new(self.resolve_type_var(&*var)?), len)),
                    None => Err(TypeError::new(TypeErrorKind::InvalidOperation, "Unable to resolve empty array type".to_string())),
                }
            }
        }
    }

    fn type_statement(&mut self, statement: &Statement) -> Result<TypedStatement, TypeError> {
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
            Statement::Break(break_stmt) => {
                let typed_value = self.type_expression(&break_stmt.value)?;
                Ok(TypedStatement::Break(TypedBreakStatement {
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
    ) -> Result<TypedVariableDeclaration, TypeError> {
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

    fn type_expression(&mut self, expression: &Expression) -> Result<TypedExpression, TypeError> {
        self.type_expression_with_hint(expression, None)
    }

    fn resolve_numeric_literal_type(
        &mut self,
        expression: &Expression,
        hint: Option<&Type>,
    ) -> Result<Type, TypeError> {
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
    ) -> Result<TypedExpression, TypeError> {
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
            Expression::ArrayLiteral(elements) => {
                if let Some(ty) = hint {
                    match &ty {
                        Type::Array(t, _len) => {
                            let mut typed_elements = Vec::new();
                            for element in elements {
                                typed_elements.push(self.type_expression_with_hint(element, Some(t))?);
                            }
                            Ok(TypedExpression::ArrayLiteral(typed_elements, ty.clone()))
                        }
                        _ => unreachable!()
                    }
                } else {
                    if elements.is_empty() {
                        Err(TypeError::new(TypeErrorKind::InvalidOperation, "Cannot infer type of empty array literal without type annotation".to_string()))
                    } else {
                        let type_var = self.infer_expression(expression)?;
                        let resolved_type = self.resolve_type_var(&type_var)?;
                        match &resolved_type {
                            Type::Array(ty, _) => {
                                let mut typed_elements = Vec::new();
                                for element in elements {
                                    typed_elements.push(self.type_expression_with_hint(element, Some(&*ty))?);
                                }
                                Ok(TypedExpression::ArrayLiteral(typed_elements, resolved_type))
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
            Expression::Index(index_expr) => {
                let typed_array = self.type_expression(&index_expr.array)?;
                let typed_index = self.type_expression(&index_expr.index)?;

                // Extract element type from array type
                let element_type = match typed_array.get_type() {
                    Type::Array(elem_type, _) => *elem_type,
                    other => {
                        return Err(TypeError::new(
                            TypeErrorKind::InvalidOperation,
                            format!("Cannot index into non-array type {}", other),
                        ));
                    }
                };

                Ok(TypedExpression::Index(Box::new(
                    TypedIndexExpr {
                        array: typed_array,
                        index: typed_index,
                        element_type,
                    },
                )))
            }
            Expression::Block(block) => {
                let typed_block = self.type_block(block)?;
                Ok(TypedExpression::Block(typed_block))
            }
            Expression::If(if_expr) => self.type_if_expression(if_expr),
            Expression::While(while_expr) => self.type_while_expression(while_expr),
            Expression::Grouping(expr) => {
                let typed = self.type_expression_with_hint(expr, hint)?;
                Ok(TypedExpression::Grouping(Box::new(typed)))
            }
        }
    }

    fn type_if_expression(&mut self, if_expr: &crate::frontend::ast::IfExpr) -> Result<TypedExpression, TypeError> {
        // Type the condition
        let typed_condition = self.type_expression(&if_expr.condition)?;

        // Type the then block
        let typed_then_block = self.type_block(&if_expr.then_block)?;

        // Type else-if branches
        let mut typed_else_ifs = Vec::new();
        for (else_if_condition, else_if_block) in &if_expr.else_ifs {
            let typed_else_if_condition = self.type_expression(else_if_condition)?;
            let typed_else_if_block = self.type_block(else_if_block)?;
            typed_else_ifs.push((typed_else_if_condition, typed_else_if_block));
        }

        // Type else block if present
        let typed_else_block = if let Some(else_block) = &if_expr.else_block {
            Some(self.type_block(else_block)?)
        } else {
            None
        };

        // Determine result type (we already did type checking, so we can look at the first branch)
        let result_type = typed_then_block.block_type.clone();

        Ok(TypedExpression::If(Box::new(TypedIfExpr {
            condition: typed_condition,
            then_block: typed_then_block,
            else_ifs: typed_else_ifs,
            else_block: typed_else_block,
            result_type,
        })))
    }

    fn type_while_expression(&mut self, while_expr: &crate::frontend::ast::WhileExpr) -> Result<TypedExpression, TypeError> {
        // Type the condition
        let typed_condition = self.type_expression(&while_expr.condition)?;

        // Find all breaks in the body (excluding nested loops)
        let breaks = self.find_all_breaks(&while_expr.body);

        // Set loop context and type the body
        let old_in_loop = self.in_loop;
        self.in_loop = true;
        let typed_body = self.type_block(&while_expr.body)?;
        self.in_loop = old_in_loop;

        // Type the else block if present
        let typed_else_block = if let Some(else_block) = &while_expr.else_block {
            Some(self.type_block(else_block)?)
        } else {
            None
        };

        // Compute result type based on breaks (not the body's type)
        let result_type = if !breaks.is_empty() {
            // Type the first break to get the result type
            let first_break_type = self.type_expression(&breaks[0].value)?.get_type();
            first_break_type
        } else if let Some(ref else_block_typed) = typed_else_block {
            // No breaks, use else block type
            else_block_typed.block_type.clone()
        } else {
            // No breaks and no else block
            Type::Void
        };

        Ok(TypedExpression::While(Box::new(TypedWhileExpr {
            condition: typed_condition,
            body: typed_body,
            else_block: typed_else_block,
            result_type,
        })))
    }

    fn type_block(&mut self, block: &Block) -> Result<TypedBlock, TypeError> {
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
                // Yield acts as break from block, so stop here
                break;
            }

            // Break statements do NOT contribute to block type
            // They break out of the loop, not the block
            // The loop's type is determined separately by analyzing breaks
            typed_statements.push(typed_stmt);
        }

        // Pop scope
        self.scopes.pop();

        Ok(TypedBlock {
            statements: typed_statements,
            block_type,
        })
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
