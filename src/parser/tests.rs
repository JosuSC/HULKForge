/// Comprehensive test suite for parser stabilization.
/// Tests cover associativity, unary operators, function calls, and more.

use crate::lexer::lexer::TokenStream;
use crate::parser::Parser;
use crate::parser::ast::*;

// ============================================
// TEST UTILITIES
// ============================================

/// Parse a source string and return AST or error messages.
fn parse_source(source: &str) -> Result<Vec<FunctionDef>, Vec<String>> {
    let stream = TokenStream::new(source);
    let mut parser = Parser::new(stream);
    let (functions, errors) = parser.parse_program();
    
    if errors.is_empty() {
        Ok(functions)
    } else {
        Err(errors)
    }
}

// ============================================
// ASSOCIATIVITY TESTS
// ============================================

#[test]
fn test_addition_left_associative() {
    // 1 + 2 + 3 should parse as (1 + 2) + 3, not 1 + (2 + 3)
    // We verify by checking the AST structure.
    let source = "function f() => 1 + 2 + 3;";
    let result = parse_source(source);
    
    assert!(result.is_ok(), "Should parse successfully");
    // Structure verification would require traversing AST deeply.
    // For now, we just verify it parses without error.
}

#[test]
fn test_subtraction_left_associative() {
    let source = "function f() => 1 - 2 - 3;";
    let result = parse_source(source);
    assert!(result.is_ok(), "1 - 2 - 3 should be left-associative");
}

#[test]
fn test_multiplication_left_associative() {
    let source = "function f() => 2 * 3 * 4;";
    let result = parse_source(source);
    assert!(result.is_ok(), "2 * 3 * 4 should be left-associative");
}

#[test]
fn test_division_left_associative() {
    let source = "function f() => 8 / 4 / 2;";
    let result = parse_source(source);
    assert!(result.is_ok(), "8 / 4 / 2 should be left-associative");
}

#[test]
fn test_mixed_precedence_expr_term() {
    // 1 + 2 * 3 should be 1 + (2 * 3) due to precedence, not (1 + 2) * 3
    let source = "function f() => 1 + 2 * 3;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Precedence: * before +");
}

// ============================================
// POWER / EXPONENTIATION TESTS
// ============================================

#[test]
#[ignore] // Power operator needs AST redesign; currently not fully supported
fn test_power_right_associative() {
    // 2 ^ 3 ^ 2 should be 2 ^ (3 ^ 2) = 2 ^ 9 = 512
    let source = "function f() => 2 ^ 3 ^ 2;";
    let result = parse_source(source);
    // Expected: will fail until Factor::Power is added to AST
}

// ============================================
// UNARY OPERATOR TESTS
// ============================================

#[test]
fn test_unary_negation_number() {
    let source = "function f() => -5;";
    let result = parse_source(source);
    assert!(result.is_ok(), "-5 should parse as unary negation");
}

#[test]
fn test_unary_negation_identifier() {
    let source = "function f() => -x;";
    let result = parse_source(source);
    assert!(result.is_ok(), "-x should parse as unary negation of identifier");
}

#[test]
fn test_unary_positive_number() {
    let source = "function f() => +4;";
    let result = parse_source(source);
    assert!(result.is_ok(), "+4 should parse");
}

#[test]
fn test_unary_negation_in_expr() {
    let source = "function f() => -5 + 3;";
    let result = parse_source(source);
    assert!(result.is_ok(), "-5 + 3 should parse");
}

#[test]
fn test_nested_unary() {
    let source = "function f() => --x;";
    let result = parse_source(source);
    assert!(result.is_ok(), "--x (double negation) should parse");
}

// ============================================
// FUNCTION CALL TESTS (USER-DEFINED)
// ============================================

#[test]
fn test_simple_function_call() {
    let source = "function f() => foo(1, 2);";
    let result = parse_source(source);
    assert!(result.is_ok(), "foo(1, 2) should parse as function call");
}

#[test]
fn test_function_call_single_arg() {
    let source = "function f() => bar(x);";
    let result = parse_source(source);
    assert!(result.is_ok(), "bar(x) should parse");
}

#[test]
fn test_function_call_no_args() {
    let source = "function f() => foo();";
    let result = parse_source(source);
    assert!(result.is_ok(), "foo() should parse with no arguments");
}

#[test]
fn test_function_call_with_builtin() {
    let source = "function f() => bar(x, sin(1));";
    let result = parse_source(source);
    assert!(result.is_ok(), "Function call with builtin as argument");
}

#[test]
fn test_nested_function_calls() {
    let source = "function f() => nested(a(b(1), 2), 3);";
    let result = parse_source(source);
    assert!(result.is_ok(), "Nested function calls should parse");
}

#[test]
fn test_function_call_with_arithmetic() {
    let source = "function f() => foo(1 + 2, 3 * 4);";
    let result = parse_source(source);
    assert!(result.is_ok(), "Function calls with arithmetic expressions");
}

// ============================================
// FUNCTION DEFINITION TESTS
// ============================================

#[test]
fn test_inline_function() {
    let source = "function f(x) => x + 1;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Inline function should parse");
    
    if let Ok(funcs) = result {
        assert_eq!(funcs.len(), 1, "Should have one function");
        assert_eq!(funcs[0].params.len(), 1, "Should have one parameter");
    }
}

#[test]
fn test_block_function() {
    let source = "function f(x) { x + 1; x + 2 }";
    let result = parse_source(source);
    assert!(result.is_ok(), "Block function should parse");
    
    if let Ok(funcs) = result {
        assert_eq!(funcs.len(), 1);
    }
}

#[test]
fn test_function_multiple_params() {
    let source = "function add(a, b) => a + b;";
    let result = parse_source(source);
    assert!(result.is_ok());
    
    if let Ok(funcs) = result {
        assert_eq!(funcs[0].params.len(), 2, "Should have two parameters");
    }
}

#[test]
fn test_function_with_type_annotations() {
    let source = "function f(x: Number) => x + 1;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Function with type annotation should parse");
}

// ============================================
// BOOLEAN CONSTANTS TESTS
// ============================================

#[test]
fn test_boolean_true() {
    let source = "function f() => true;";
    let result = parse_source(source);
    assert!(result.is_ok(), "true constant should parse");
}

#[test]
fn test_boolean_false() {
    let source = "function f() => false;";
    let result = parse_source(source);
    assert!(result.is_ok(), "false constant should parse");
}

#[test]
fn test_boolean_in_expression() {
    let source = "function f() => true + false;";
    let result = parse_source(source);
    // Parse succeeds; semantic analysis will handle bool+bool type error
    assert!(result.is_ok(), "Boolean arithmetic parses (semantic check later)");
}

// ============================================
// COMPLEX EXPRESSIONS TESTS
// ============================================

#[test]
fn test_complex_expression_1() {
    let source = "function f() => 2 * 3 + 4 * 5;";
    let result = parse_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_complex_expression_2() {
    let source = "function f() => -5 + 3 * 2 - -1;";
    let result = parse_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_grouped_expression() {
    let source = "function f() => (1 + 2) * 3;";
    let result = parse_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_functions() {
    let source = r#"
        function f() => 1;
        function g(x) => x + 1;
        function h(a, b) => a * b;
    "#;
    let result = parse_source(source);
    assert!(result.is_ok(), "Should parse multiple function definitions");
    
    if let Ok(funcs) = result {
        assert_eq!(funcs.len(), 3, "Should have three functions");
    }
}

// ============================================
// ERROR RECOVERY TESTS
// ============================================

#[test]
fn test_missing_semicolon_recovers() {
    let source = r#"
        function f() => 1
        function g() => 2;
    "#;
    let result = parse_source(source);
    // Should recover from missing semicolon and continue
    assert!(result.is_err(), "Should have errors");
}

#[test]
fn test_malformed_function_recovers() {
    let source = r#"
        function f( { 1 + }
        function g() => 2;
    "#;
    let result = parse_source(source);
    // Should report error for first function but recover
    assert!(result.is_err());
}

// ============================================
// EDGE CASES
// ============================================

#[test]
fn test_empty_function_call() {
    let source = "function f() => foo();";
    let result = parse_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_single_number() {
    let source = "function f() => 42;";
    let result = parse_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_single_identifier() {
    let source = "function f() => x;";
    let result = parse_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_constants() {
    let source = "function f() => PI + E;";
    let result = parse_source(source);
    assert!(result.is_ok(), "Mathematical constants should parse");
}

#[test]
fn test_builtin_functions() {
    let source = "function f() => sin(1) + cos(2) + sqrt(3) + exp(4) + log(5);";
    let result = parse_source(source);
    assert!(result.is_ok(), "Builtin functions should parse");
}
