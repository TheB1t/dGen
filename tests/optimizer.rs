use std::f64::INFINITY;

use dgen::parser::*;
use dgen::optimizer::*;
use dgen::dgen_ast::*;

fn test_optimizer_generic(expr: &str, expected: Expr) {
    let parsed = parse(expr.to_string());
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
}


#[test]
fn test_optimizer_string_concatenation() {
    test_optimizer_generic("\"Hello\" + \"World\";", Expr::String("HelloWorld".to_string()));
}

#[test]
fn test_optimizer_complex_expression() {
    test_optimizer_generic("((2 + 3) * 4 - 5) / 3;", Expr::Number(5.0));
}

#[test]
fn test_optimizer_nested_logical_expression() {
    test_optimizer_generic("((true && false) || !false);", Expr::Bool(true));
}