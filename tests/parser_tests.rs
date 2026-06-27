use knot::parser::ast::*;
use knot::parser::Parser;

fn parse(source: &str) -> Vec<Stmt> {
    Parser::new(source).parse_program()
}

#[test]
fn empty_program() {
    assert_eq!(parse(""), vec![]);
    assert_eq!(parse("\n\n"), vec![]);
}

#[test]
fn simple_func_def() {
    let stmts = parse("func main() { x = 42\n return x }");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::FuncDef { name, .. } if name == "main"));
}

#[test]
fn func_with_return_type() {
    let stmts = parse("func add(a: I32, b: I32) -> I32 { return a + b }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::FuncDef { name, params, ret_ty, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert_eq!(params[1].name, "b");
            assert_eq!(*ret_ty, Some(Type::Base(BaseType::I32)));
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn binary_expressions() {
    let stmts = parse("func f() { x = 1 + 2 * 3 }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::Binary { op: BinOp::Add, .. }));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn prec_mult_before_add() {
    let stmts = parse("func f() { x = 1 + 2 * 3 }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => match value.as_ref() {
                Expr::Binary { op: BinOp::Add, left, right, .. } => {
                    assert!(matches!(left.as_ref(), Expr::Int(1, _)));
                    assert!(matches!(right.as_ref(), Expr::Binary { op: BinOp::Mul, .. }));
                }
                _ => panic!("expected Add"),
            },
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn if_statement() {
    let stmts = parse("func f() { if x > 0 { return 1 } }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn if_else_statement() {
    let stmts = parse("func f() { if x > 0 { return 1 } else { return 0 } }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn while_loop() {
    let stmts = parse("func f() { while x > 0 { x = x - 1 } }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn for_loop() {
    let stmts = parse("func f() { for i in 0..10 { x = x + 1 } }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn function_call() {
    let stmts = parse("func f() { print(42) }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn unary_neg() {
    let stmts = parse("func f() { x = -1 + 2 }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::Binary { op: BinOp::Add, left, .. }
                    if matches!(left.as_ref(), Expr::Unary { op: UnaryOp::Neg, .. })));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn comparison_chain() {
    let stmts = parse("func f() { x = a == b && c > d }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::Binary { op: BinOp::And, .. }));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn string_literal() {
    let stmts = parse("func f() { x = \"hello\" }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::String(s, _) if s == "hello"));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn bool_and_null() {
    let stmts = parse("func f() { x = true\n y = false\n z = null }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn nullable_type() {
    let stmts = parse("func f(a: I32?) {}");
    match &stmts[0] {
        Stmt::FuncDef { params, .. } => {
            assert_eq!(params[0].ty, Some(Type::Nullable(Box::new(Type::Base(BaseType::I32)))));
        }
        _ => panic!("expected FuncDef"),
    }
}
