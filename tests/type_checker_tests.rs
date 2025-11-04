use clause::lexer::Lexer;
use clause::parser::Parser;
use clause::type_checker::TypeChecker;

fn type_check(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.message)?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.message)?;

    let mut type_checker = TypeChecker::new();
    type_checker.check_program(&program)?;
    Ok(())
}

fn assert_type_checks(source: &str) {
    match type_check(source) {
        Ok(_) => {}
        Err(e) => panic!("Expected type checking to succeed but got error: {}", e),
    }
}

fn assert_type_error(source: &str, expected_error_fragment: &str) {
    match type_check(source) {
        Ok(_) => panic!("Expected type checking to fail but it succeeded"),
        Err(e) => {
            if !e.contains(expected_error_fragment) {
                panic!(
                    "Expected error containing '{}' but got: {}",
                    expected_error_fragment, e
                );
            }
        }
    }
}

// ============================================================================
// Integer Literal Type Inference Tests
// ============================================================================

#[test]
fn test_integer_literal_defaults_to_i64() {
    // Without type annotation, integer literals should be inferred as i64
    assert_type_checks("let x = 42");
}

#[test]
fn test_integer_literal_with_i32_annotation() {
    assert_type_checks("let x: I32 = 42");
}

#[test]
fn test_integer_literal_with_u32_annotation() {
    assert_type_checks("let x: U32 = 42");
}

#[test]
fn test_integer_literal_adapts_to_context() {
    // The literal 3 should adapt to i32 because of x's type
    assert_type_checks("let x: I32 = 10\nx + 3");
}

#[test]
fn test_chained_integer_operations() {
    // All literals should unify to the same type
    assert_type_checks("let x: I16 = 5\nx + 10 + 15");
}

// ============================================================================
// Float Literal Type Inference Tests
// ============================================================================

#[test]
fn test_float_literal_defaults_to_f64() {
    assert_type_checks("let x = 3.14");
}

#[test]
fn test_float_literal_with_f32_annotation() {
    assert_type_checks("let x: F32 = 3.14");
}

#[test]
fn test_float_literal_adapts_to_context() {
    assert_type_checks("let x: F32 = 1.0\nx + 2.0");
}

// ============================================================================
// Type Mismatch Tests
// ============================================================================

#[test]
fn test_integer_and_float_cannot_mix() {
    assert_type_error("let x = 42\nlet y = 3.14\nx + y", "cannot unify");
}

#[test]
fn test_bool_and_integer_cannot_mix() {
    assert_type_error("let x = true\nlet y = 42\nx + y", "Type mismatch");
}

#[test]
fn test_mixed_signed_unsigned() {
    assert_type_error(
        "let x: I32 = 10\nlet y: U32 = 20\nx + y",
        "mixed signed/unsigned",
    );
}

#[test]
fn test_wrong_type_annotation() {
    // Can't assign a boolean to an integer variable
    assert_type_error("let x: I32 = true", "Type mismatch");
}

// ============================================================================
// Logical Operator Tests
// ============================================================================

#[test]
fn test_logical_and_requires_bool() {
    assert_type_checks("true && false");
}

#[test]
fn test_logical_and_rejects_integer() {
    assert_type_error("5 && true", "Expected Boolean");
}

#[test]
fn test_logical_or_requires_bool() {
    assert_type_checks("true || false");
}

#[test]
fn test_logical_or_rejects_integer() {
    assert_type_error("false || 10", "Expected Boolean");
}

#[test]
fn test_logical_not_requires_bool() {
    assert_type_checks("!true");
}

#[test]
fn test_logical_not_rejects_integer() {
    assert_type_error("!5", "Logical NOT requires boolean");
}

#[test]
fn test_logical_not_rejects_float() {
    assert_type_error("!3.14", "Logical NOT requires boolean");
}

// ============================================================================
// Comparison Operator Tests
// ============================================================================

#[test]
fn test_comparison_operators_return_bool() {
    assert_type_checks("let x = 5 < 10");
    assert_type_checks("let x = 5 <= 10");
    assert_type_checks("let x = 5 > 10");
    assert_type_checks("let x = 5 >= 10");
    assert_type_checks("let x = 5 == 10");
    assert_type_checks("let x = 5 != 10");
}

#[test]
fn test_comparison_operands_must_match() {
    // Both operands should unify to the same type
    assert_type_checks("let x: I32 = 5\nx < 10");
}

#[test]
fn test_comparison_result_is_bool() {
    assert_type_checks("let result = 5 < 10\nresult && true");
}

// ============================================================================
// Arithmetic Operator Tests
// ============================================================================

#[test]
fn test_arithmetic_preserves_type() {
    assert_type_checks("let x: I32 = 5\nlet y = x + 10\nlet z: I32 = y");
}

#[test]
fn test_bitwise_operators() {
    assert_type_checks("let x: I32 = 5 & 3");
    assert_type_checks("let x: I32 = 5 | 3");
    assert_type_checks("let x: I32 = 5 ^ 3");
}

#[test]
fn test_shift_operators() {
    assert_type_checks("let x: I32 = 5 << 2");
    assert_type_checks("let x: I32 = 20 >> 2");
}

#[test]
fn test_shift_requires_right_integer() {
    assert_type_error("let x = 255 << void", "");
}

#[test]
fn test_shift_requires_left_integer() {
    assert_type_error("let x = true >> 2", "");
}

#[test]
fn test_shift_allows_different_integer_types() {
    assert_type_checks("let x: I64 = 3\nlet y: U64 = 2\nx << y");
}

// ============================================================================
// Unary Operator Tests
// ============================================================================

#[test]
fn test_unary_plus() {
    assert_type_checks("let x: I32 = +42");
}

#[test]
fn test_unary_minus() {
    assert_type_checks("let x: I32 = -42");
}

#[test]
fn test_bitwise_not() {
    assert_type_checks("let x: I32 = ~42");
}

#[test]
fn test_unary_operators_preserve_type() {
    assert_type_checks("let x: I16 = 10\nlet y: I16 = -x");
}

// ============================================================================
// Variable Tests
// ============================================================================

#[test]
fn test_undefined_variable() {
    assert_type_error("x + 5", "Undefined variable");
}

#[test]
fn test_variable_redefinition_error() {
    assert_type_error("let x = 5\nlet x = 10", "already defined");
}

#[test]
fn test_variable_usage() {
    assert_type_checks("let x = 42\nlet y = x + 10");
}

// ============================================================================
// Block and Scoping Tests
// ============================================================================

#[test]
fn test_block_with_yield() {
    assert_type_checks("{\n  let x = 5\n  <- x\n}");
}

#[test]
fn test_block_without_yield_is_void() {
    assert_type_checks("{\n  let x = 5\n}");
}

#[test]
fn test_nested_blocks() {
    assert_type_checks("{\n  let x = 5\n  {\n    let y = 10\n    <- x + y\n  }\n}");
}

#[test]
fn test_block_scoping() {
    // x defined in block should not be visible outside
    assert_type_error("{\n  let x = 5\n}\nx", "Undefined variable");
}

#[test]
fn test_shadowing_in_nested_scope() {
    // This should work - inner x shadows outer x
    assert_type_checks("let x = 5\n{\n  let x = 10\n}");
}

#[test]
fn test_inconsistent_yield_types() {
    // The error happens during unification: IntVar cannot unify with Bool
    assert_type_error("{\n  <- 42\n  <- true\n}", "Type mismatch");
}

#[test]
fn test_all_yields_must_match() {
    // All yields in a block must have the same type
    // The error happens during unification: IntVar cannot unify with FloatVar
    assert_type_error(
        "{\n  let x = 10\n  <- x\n  let y = 3.14\n  <- y\n}",
        "cannot unify",
    );
}

// ============================================================================
// Assignment Operator Tests
// ============================================================================

#[test]
fn test_simple_assignment() {
    assert_type_checks("let x = 5\nx = 10");
}

#[test]
fn test_compound_assignment() {
    assert_type_checks("let x = 5\nx += 10");
    assert_type_checks("let x = 5\nx -= 10");
    assert_type_checks("let x = 5\nx *= 10");
    assert_type_checks("let x = 5\nx /= 10");
}

#[test]
fn test_assignment_type_must_match() {
    assert_type_error("let x: I32 = 5\nx = 3.14", "Type mismatch");
}

// ============================================================================
// Complex Expression Tests
// ============================================================================

#[test]
fn test_complex_arithmetic_expression() {
    assert_type_checks("let x: I32 = (5 + 10) * 2 - 3");
}

#[test]
fn test_complex_logical_expression() {
    assert_type_checks("let x = (5 < 10) && (20 > 15)");
}

#[test]
fn test_mixed_operations() {
    assert_type_checks("let x: I32 = 5\nlet y = (x + 10) > 20");
}

#[test]
fn test_grouping() {
    assert_type_checks("let x: I32 = (5 + 10) * (2 - 1)");
}

// ============================================================================
// Type Annotation Tests
// ============================================================================

#[test]
fn test_explicit_type_annotations() {
    assert_type_checks("let x: I8 = 42");
    assert_type_checks("let x: I16 = 42");
    assert_type_checks("let x: I32 = 42");
    assert_type_checks("let x: I64 = 42");
    assert_type_checks("let x: U8 = 42");
    assert_type_checks("let x: U16 = 42");
    assert_type_checks("let x: U32 = 42");
    assert_type_checks("let x: U64 = 42");
}

#[test]
fn test_float_type_annotations() {
    assert_type_checks("let x: F32 = 3.14");
    assert_type_checks("let x: F64 = 3.14");
}

#[test]
fn test_bool_type_annotation() {
    assert_type_checks("let x: Bool = true");
}

// ============================================================================
// Edge Cases and Error Conditions
// ============================================================================

#[test]
fn test_void_expression() {
    assert_type_checks("let x = void");
}

#[test]
fn test_empty_block() {
    assert_type_checks("{}");
}

#[test]
fn test_multiple_statements() {
    assert_type_checks("let x = 5\nlet y = 10\nlet z = x + y");
}

#[test]
fn test_type_propagation_through_operations() {
    // The 3 should infer to i8 because of x's type
    assert_type_checks("let x: I8 = 5\nlet y = x + 3\nlet z: I8 = y");
}

// ============================================================================
// Unification Algorithm Tests
// ============================================================================

#[test]
fn test_transitive_unification() {
    // x + y forces x and y to be same type
    // x = 5 makes x an integer
    // Therefore y must also be an integer
    assert_type_checks("let y = 10\nlet x = 5\nx + y");
}

#[test]
fn test_bidirectional_type_inference() {
    // Type flows both ways: from annotation to literal and literal to usage
    assert_type_checks("let x: I32 = 10\nlet y = x\nlet z: I32 = y");
}

#[test]
fn test_inference_after_explicit_type_declaration() {
    assert_type_checks("let x: I16 = 15\nlet y = x")
}

#[test]
fn test_multiple_constraints_on_same_variable() {
    // x appears in multiple contexts, all constraints must be consistent
    assert_type_checks("let x = 10\nlet a = x + 5\nlet b = x * 2");
}

#[test]
fn test_type_error_in_deeply_nested_expression() {
    assert_type_error("let x = (5 + (10 * (true && false)))", "Type mismatch");
}

// ============================================================================
// Yield Statement Tests
// ============================================================================

#[test]
fn test_yield_type_matches_annotation() {
    assert_type_checks("let x: I32 = {\n  <- 42\n}");
}

#[test]
fn test_yield_type_mismatch() {
    assert_type_error("let x: I32 = {\n  <- true\n}", "Type mismatch");
}

#[test]
fn test_yield_acts_as_break_for_type_checking() {
    // Code after yield should still be type-checked even though unreachable
    assert_type_error("{\n  <- 42\n  let x: I32 = true\n}", "Type mismatch");
}

// ============================================================================
// Fix vs Let Tests
// ============================================================================

#[test]
fn test_fix_declaration() {
    assert_type_checks("fix x = 42");
}

#[test]
fn test_let_declaration() {
    assert_type_checks("let x = 42");
}

// ============================================================================
// Integration Tests - Real World Scenarios
// ============================================================================

#[test]
fn test_factorial_like_computation() {
    assert_type_checks(
        "let n: I32 = 5\n\
         let result = n * (n - 1) * (n - 2)",
    );
}

#[test]
fn test_conditional_expression_simulation() {
    assert_type_checks(
        "let x = 10\n\
         let y = 20\n\
         let max = {\n\
           <- x\n\
         }",
    );
}

#[test]
fn test_accumulator_pattern() {
    assert_type_checks(
        "let acc: I32 = 0\n\
         acc += 10\n\
         acc += 20\n\
         acc += 30",
    );
}

#[test]
fn test_bitwise_flag_manipulation() {
    assert_type_checks(
        "let flags: U32 = 0\n\
         flags |= 1\n\
         flags |= 4\n\
         let has_flag = (flags & 1) != 0",
    );
}

// ============================================================================
// Array Type Checking Tests
// ============================================================================

#[test]
fn test_array_literal_with_inferred_type() {
    // Array type should be inferred from elements
    assert_type_checks("let arr = [1, 2, 3]");
}

#[test]
fn test_array_literal_with_explicit_type() {
    // Array with explicit type annotation
    assert_type_checks("let arr: [I32, 3] = [1, 2, 3]");
}

#[test]
fn test_array_literal_elements_must_match() {
    // All elements must have the same type
    assert_type_checks("let arr: [I32, 3] = [1, 2, 3]");
}

#[test]
fn test_empty_array_requires_type_annotation() {
    // Empty arrays cannot infer type
    assert_type_error("let arr = []", "Cannot infer type of empty array");
}

#[test]
fn test_empty_array_with_type_annotation() {
    // Empty array with explicit type annotation should work
    assert_type_checks("let arr: [I32, 0] = []");
}

#[test]
fn test_array_size_mismatch() {
    // Size in type annotation must match number of elements
    assert_type_error("let arr: [I32, 5] = [1, 2, 3]", "Array size mismatch");
}

#[test]
fn test_array_element_type_mismatch() {
    // Elements must match the annotated element type
    assert_type_error("let arr: [I32, 3] = [1, 2, 3.14]", "cannot unify");
}

#[test]
fn test_array_mixed_element_types() {
    // Mixed integer and float types should fail
    assert_type_error("let arr = [1, 2.5, 3]", "cannot unify");
}

#[test]
fn test_array_mixed_integer_types() {
    // Mixed integer types with explicit annotations
    assert_type_error(
        "let x: I32 = 1\nlet y: I64 = 2\nlet arr = [x, y]",
        "Type mismatch",
    );
}

#[test]
fn test_array_indexing() {
    // Basic array indexing
    assert_type_checks("let arr = [1, 2, 3]\narr[0]");
}

#[test]
fn test_array_indexing_with_variable() {
    // Indexing with variable index
    assert_type_checks("let arr = [1, 2, 3]\nlet i = 1\narr[i]");
}

#[test]
fn test_array_indexing_returns_element_type() {
    // Index result should have element type
    assert_type_checks("let arr: [I32, 3] = [1, 2, 3]\nlet x: I32 = arr[0]");
}

#[test]
fn test_array_index_must_be_integer() {
    // Index must be an integer type
    assert_type_error("let arr = [1, 2, 3]\narr[1.5]", "Array index must be an integer");
}

#[test]
fn test_array_index_bool_error() {
    // Index cannot be bool
    assert_type_error("let arr = [1, 2, 3]\narr[true]", "Array index must be an integer");
}

#[test]
fn test_chained_array_indexing() {
    // Indexing into nested arrays
    assert_type_checks("let matrix: [[I32, 2], 2] = [[1, 2], [3, 4]]\nmatrix[0][1]");
}

#[test]
fn test_array_assignment() {
    // Assigning to array element
    assert_type_checks("let arr = [1, 2, 3]\narr[0] = 42");
}

#[test]
fn test_array_assignment_type_must_match() {
    // Assignment value must match element type
    assert_type_error("let arr: [I32, 3] = [1, 2, 3]\narr[0] = 3.14", "Type mismatch");
}

#[test]
fn test_array_compound_assignment() {
    // Compound assignment to array element
    assert_type_checks("let arr = [1, 2, 3]\narr[0] += 10");
}

#[test]
fn test_cannot_index_non_array() {
    // Cannot index into non-array types
    assert_type_error("let x = 42\nx[0]", "Cannot index");
}

#[test]
fn test_nested_arrays() {
    // Arrays of arrays
    assert_type_checks("let matrix = [[1, 2], [3, 4]]");
}

#[test]
fn test_nested_array_explicit_type() {
    // Nested arrays with explicit type
    assert_type_checks("let matrix: [[I32, 2], 2] = [[1, 2], [3, 4]]");
}

#[test]
fn test_array_of_floats() {
    // Array of floating-point numbers
    assert_type_checks("let arr: [F32, 3] = [1.0, 2.0, 3.0]");
}

#[test]
fn test_array_of_bools() {
    // Array of booleans
    assert_type_checks("let arr = [true, false, true]");
}

#[test]
fn test_array_in_block() {
    // Arrays work inside blocks
    assert_type_checks("{\n  let arr = [1, 2, 3]\n  <- arr[1]\n}");
}

#[test]
fn test_array_arithmetic_on_elements() {
    // Arithmetic on array elements
    assert_type_checks("let arr = [10, 20, 30]\nlet sum = arr[0] + arr[1]");
}

#[test]
fn test_array_type_propagation() {
    // Type propagates from array annotation to elements
    assert_type_checks("let arr: [I16, 3] = [1, 2, 3]\nlet x: I16 = arr[0]");
}
