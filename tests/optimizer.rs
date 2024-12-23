use std::f64::INFINITY;

use dgen::parser::*;
use dgen::optimizer::*;
use dgen::dgen_ast::*;

fn test_optimizer_generic(expr: &str, expected: Expr) {
    let parsed = parse(expr.to_string()).unwrap();
    let optimized = optimize(parsed);

    match optimized {
        Stmt::Block(v) => {
            assert_eq!(v.get(0), Some(&Stmt::Expr(expected)), "Failed optimization: {:?}", expr);
        },
        _ => assert!(true, "Failed optimization: {:?}", expr)
    }
}

#[test]
fn test_optimizer_addition() {
    test_optimizer_generic("2 + 3;", Expr::Number(5.0));
}

#[test]
fn test_optimizer_subtraction() {
    test_optimizer_generic("5 - 3;", Expr::Number(2.0));
}

#[test]
fn test_optimizer_multiplication() {
    test_optimizer_generic("4 * 2;", Expr::Number(8.0));
}

#[test]
fn test_optimizer_division() {
    test_optimizer_generic("10 / 2;", Expr::Number(5.0));
    test_optimizer_generic("10 / 0;", Expr::Number(INFINITY));
}

#[test]
fn test_optimizer_negation() {
    test_optimizer_generic("-5;", Expr::Number(-5.0));
    test_optimizer_generic("!!true;", Expr::Bool(true));
    test_optimizer_generic("!!false;", Expr::Bool(false));
}

#[test]
fn test_optimizer_string_concatenation() {
    test_optimizer_generic("\"Hello\" + \"World\";", Expr::String("HelloWorld".to_string()));
}

#[test]
fn test_optimizer_complex_expression() {
    test_optimizer_generic("((2 + 3) * 4 - 5) / 3;", Expr::Number(5.0));
    test_optimizer_generic("3 + 2 * (1 + 4) / 5 - 6;", Expr::Number(-1.0));
    test_optimizer_generic("-(2 + 3 * 4);", Expr::Number(-14.0));
}

#[test]
fn test_optimizer_nested_logical_expression() {
    test_optimizer_generic("((true && false) || !false);", Expr::Bool(true));
    test_optimizer_generic("(true || false) && (false || true);", Expr::Bool(true));
}

#[test]
fn test_optimizer_mixed_string_and_number_addition() {
    test_optimizer_generic("\"Number: \" + 5;", Expr::String("Number: 5".to_string()));
}

#[test]
fn test_optimizer_boolean_multiplication() {
    test_optimizer_generic("true && false;", Expr::Bool(false));
    test_optimizer_generic("true && true;", Expr::Bool(true));
}

#[test]
fn test_optimizer_boolean_negation_and_comparison() {
    test_optimizer_generic("!(3 > 2);", Expr::Bool(false));
}

#[test]
fn test_optimizer_multiple_operations() {
    test_optimizer_generic("((3 + 2) * 2 >= 10) || (false && true);", Expr::Bool(true));
}