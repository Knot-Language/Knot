use knot::error::Span;
use knot::parser::ast::*;
use knot::parser::symbol::SymbolTable;
use knot::semantic::check;
use std::collections::HashMap;

fn check_expr(expr: &Expr, symbols: &mut SymbolTable) -> Option<Type> {
    let mut errors = Vec::new();
    check::analyze_expr(expr, symbols, &mut errors, &HashMap::new())
}

#[test]
fn literal_types() {
    let mut sym = SymbolTable::new();
    assert_eq!(check_expr(&Expr::Int(42, Span::new(1, 1)), &mut sym), Some(Type::Base(BaseType::I32)));
    assert_eq!(check_expr(&Expr::Float(3.14, Span::new(1, 1)), &mut sym), Some(Type::Base(BaseType::F64)));
    assert_eq!(check_expr(&Expr::String("hello".into(), Span::new(1, 1)), &mut sym), Some(Type::Array(Box::new(Type::Base(BaseType::Char)))));
    assert_eq!(check_expr(&Expr::Bool(true, Span::new(1, 1)), &mut sym), Some(Type::Base(BaseType::Bool)));
    assert_eq!(check_expr(&Expr::Null(Span::new(1, 1)), &mut sym), Some(Type::Base(BaseType::Null)));
}

#[test]
fn array_infers_element_type() {
    let mut sym = SymbolTable::new();
    let arr = Expr::Array(vec![
        Expr::Int(1, Span::new(1, 1)),
        Expr::Int(2, Span::new(1, 1)),
    ], Span::new(1, 1));
    assert_eq!(check_expr(&arr, &mut sym), Some(Type::Array(Box::new(Type::Base(BaseType::I32)))));
}

#[test]
fn empty_array_infers_void() {
    let mut sym = SymbolTable::new();
    let arr = Expr::Array(vec![], Span::new(1, 1));
    assert_eq!(check_expr(&arr, &mut sym), Some(Type::Array(Box::new(Type::Base(BaseType::Void)))));
}

#[test]
fn cast_returns_target_type() {
    let mut sym = SymbolTable::new();
    let cast = Expr::Cast {
        expr: Box::new(Expr::Int(42, Span::new(1, 1))),
        ty: Type::Base(BaseType::I64),
        forced: false,
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&cast, &mut sym), Some(Type::Base(BaseType::I64)));
}

#[test]
fn cast_as_bang_returns_target_type() {
    let mut sym = SymbolTable::new();
    let cast = Expr::Cast {
        expr: Box::new(Expr::Int(42, Span::new(1, 1))),
        ty: Type::Base(BaseType::F64),
        forced: true,
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&cast, &mut sym), Some(Type::Base(BaseType::F64)));
}

#[test]
fn postfix_op_returns_target_type() {
    let mut sym = SymbolTable::new();
    sym.declare("x".to_string(), Some(Type::Base(BaseType::I32)));
    let postfix = Expr::PostfixOp {
        op: BinOp::Add,
        target: Box::new(Expr::Ident("x".to_string(), Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&postfix, &mut sym), Some(Type::Base(BaseType::I32)));
}

#[test]
fn ident_lookup() {
    let mut sym = SymbolTable::new();
    sym.declare("x".to_string(), Some(Type::Base(BaseType::I32)));
    let ident = Expr::Ident("x".to_string(), Span::new(1, 1));
    assert_eq!(check_expr(&ident, &mut sym), Some(Type::Base(BaseType::I32)));
}

#[test]
fn ident_undefined_gives_error() {
    let mut sym = SymbolTable::new();
    let mut errors = Vec::new();
    let ident = Expr::Ident("unknown".to_string(), Span::new(1, 1));
    let ty = check::analyze_expr(&ident, &mut sym, &mut errors, &HashMap::new());
    assert_eq!(ty, None);
    assert!(!errors.is_empty());
    assert!(errors[0].0.contains("undefined variable"));
}

#[test]
fn binary_add_numeric() {
    let mut sym = SymbolTable::new();
    let expr = Expr::Binary {
        op: BinOp::Add,
        left: Box::new(Expr::Int(1, Span::new(1, 1))),
        right: Box::new(Expr::Int(2, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::I32)));
}

#[test]
fn binary_add_float_returns_wider() {
    let mut sym = SymbolTable::new();
    let expr = Expr::Binary {
        op: BinOp::Add,
        left: Box::new(Expr::Int(1, Span::new(1, 1))),
        right: Box::new(Expr::Float(2.0, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::F64)));
}

#[test]
fn binary_eq_returns_bool() {
    let mut sym = SymbolTable::new();
    let expr = Expr::Binary {
        op: BinOp::Eq,
        left: Box::new(Expr::Int(1, Span::new(1, 1))),
        right: Box::new(Expr::Int(2, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::Bool)));
}

#[test]
fn binary_and_requires_bool() {
    let mut sym = SymbolTable::new();
    let mut errors = Vec::new();
    let expr = Expr::Binary {
        op: BinOp::And,
        left: Box::new(Expr::Int(1, Span::new(1, 1))),
        right: Box::new(Expr::Bool(true, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    let _ = check::analyze_expr(&expr, &mut sym, &mut errors, &HashMap::new());
    assert!(!errors.is_empty());
    assert!(errors[0].0.contains("must be Bool"));
}

#[test]
fn binary_or_requires_bool() {
    let mut sym = SymbolTable::new();
    let mut errors = Vec::new();
    let expr = Expr::Binary {
        op: BinOp::Or,
        left: Box::new(Expr::Bool(true, Span::new(1, 1))),
        right: Box::new(Expr::Int(1, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    let _ = check::analyze_expr(&expr, &mut sym, &mut errors, &HashMap::new());
    assert!(!errors.is_empty());
    assert!(errors[0].0.contains("must be Bool"));
}

#[test]
fn binary_non_numeric_left_errors() {
    let mut sym = SymbolTable::new();
    let mut errors = Vec::new();
    let expr = Expr::Binary {
        op: BinOp::Add,
        left: Box::new(Expr::Bool(true, Span::new(1, 1))),
        right: Box::new(Expr::Int(1, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    let _ = check::analyze_expr(&expr, &mut sym, &mut errors, &HashMap::new());
    assert!(!errors.is_empty());
    assert!(errors[0].0.contains("must be numeric"));
}

#[test]
fn unary_neg_numeric() {
    let mut sym = SymbolTable::new();
    let expr = Expr::Unary {
        op: UnaryOp::Neg,
        expr: Box::new(Expr::Int(42, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::I32)));
}

#[test]
fn unary_not_returns_bool() {
    let mut sym = SymbolTable::new();
    let expr = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(Expr::Bool(true, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::Bool)));
}

#[test]
fn unary_not_requires_bool() {
    let mut sym = SymbolTable::new();
    let mut errors = Vec::new();
    let expr = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(Expr::Int(1, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    let _ = check::analyze_expr(&expr, &mut sym, &mut errors, &HashMap::new());
    assert!(!errors.is_empty());
    assert!(errors[0].0.contains("requires Bool"));
}

#[test]
fn index_requires_integer_index() {
    let mut sym = SymbolTable::new();
    sym.declare("arr".to_string(), Some(Type::Array(Box::new(Type::Base(BaseType::I32)))));
    let mut errors = Vec::new();
    let expr = Expr::Index {
        obj: Box::new(Expr::Ident("arr".to_string(), Span::new(1, 1))),
        index: Box::new(Expr::String("bad".into(), Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    let _ = check::analyze_expr(&expr, &mut sym, &mut errors, &HashMap::new());
    assert!(!errors.is_empty());
    assert!(errors[0].0.contains("must be integer"));
}

#[test]
fn index_on_array_returns_element_type() {
    let mut sym = SymbolTable::new();
    sym.declare("arr".to_string(), Some(Type::Array(Box::new(Type::Base(BaseType::Char)))));
    let expr = Expr::Index {
        obj: Box::new(Expr::Ident("arr".to_string(), Span::new(1, 1))),
        index: Box::new(Expr::Int(0, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::Char)));
}

#[test]
fn assign_declares_variable() {
    let mut sym = SymbolTable::new();
    let expr = Expr::Assign {
        target: Box::new(Expr::Ident("x".to_string(), Span::new(1, 1))),
        value: Box::new(Expr::Int(42, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::I32)));
    assert!(sym.lookup("x").is_some());
}

#[test]
fn assign_type_mismatch_errors() {
    let mut sym = SymbolTable::new();
    sym.declare("x".to_string(), Some(Type::Base(BaseType::I32)));
    let mut errors = Vec::new();
    let expr = Expr::Assign {
        target: Box::new(Expr::Ident("x".to_string(), Span::new(1, 1))),
        value: Box::new(Expr::Bool(true, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    let _ = check::analyze_expr(&expr, &mut sym, &mut errors, &HashMap::new());
    assert!(!errors.is_empty());
    assert!(errors[0].0.contains("type mismatch"));
}

#[test]
fn assign_numeric_compatible_no_error() {
    let mut sym = SymbolTable::new();
    sym.declare("x".to_string(), Some(Type::Base(BaseType::F64)));
    let mut errors = Vec::new();
    let expr = Expr::Assign {
        target: Box::new(Expr::Ident("x".to_string(), Span::new(1, 1))),
        value: Box::new(Expr::Int(42, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    let ty = check::analyze_expr(&expr, &mut sym, &mut errors, &HashMap::new());
    assert_eq!(ty, Some(Type::Base(BaseType::I32)));
    assert!(errors.is_empty());
}

#[test]
fn null_coalesce_unwraps_nullable() {
    let mut sym = SymbolTable::new();
    sym.declare("maybe".to_string(), Some(Type::Nullable(Box::new(Type::Base(BaseType::I32)))));
    let expr = Expr::Binary {
        op: BinOp::NullCoalesce,
        left: Box::new(Expr::Ident("maybe".to_string(), Span::new(1, 1))),
        right: Box::new(Expr::Int(0, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::I32)));
}

#[test]
fn null_compatible_with_nullable() {
    assert!(check::types_compatible(
        &Type::Base(BaseType::Null),
        &Type::Nullable(Box::new(Type::Base(BaseType::I32)))
    ));
    assert!(!check::types_compatible(
        &Type::Base(BaseType::Null),
        &Type::Base(BaseType::I32)
    ));
}

#[test]
fn identical_types_compatible() {
    assert!(check::types_compatible(&Type::Base(BaseType::I32), &Type::Base(BaseType::I32)));
    assert!(check::types_compatible(&Type::Base(BaseType::Char), &Type::Base(BaseType::Char)));
}

#[test]
fn numeric_types_mutually_compatible() {
    assert!(check::types_compatible(&Type::Base(BaseType::I32), &Type::Base(BaseType::F64)));
    assert!(check::types_compatible(&Type::Base(BaseType::U8), &Type::Base(BaseType::I64)));
}

#[test]
fn range_returns_i32() {
    let mut sym = SymbolTable::new();
    let expr = Expr::Binary {
        op: BinOp::Range,
        left: Box::new(Expr::Int(0, Span::new(1, 1))),
        right: Box::new(Expr::Int(10, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::I32)));
}

#[test]
fn lambda_infers_type() {
    let mut sym = SymbolTable::new();
    let lambda = Expr::Lambda {
        params: vec![Param {
            name: "x".to_string(),
            ty: Some(Type::Base(BaseType::I32)),
            default: None,
            is_args: false,
            is_kwargs: false,
        }],
        body: Block { stmts: vec![
            Stmt::Return(Some(Expr::Ident("x".to_string(), Span::new(1, 1)))),
        ]},
        span: Span::new(1, 1),
    };
    let ty = check_expr(&lambda, &mut sym);
    assert_eq!(ty, Some(Type::Named("__lambda".to_string())));
}

#[test]
fn match_expr_infers_branch_type() {
    let mut sym = SymbolTable::new();
    let match_expr = Expr::MatchExpr {
        expr: Box::new(Expr::Int(1, Span::new(1, 1))),
        branches: vec![
            MatchBranch {
                pattern: Expr::Int(1, Span::new(1, 1)),
                body: Block { stmts: vec![
                    Stmt::Expr(Expr::String("one".into(), Span::new(1, 1))),
                ]},
            },
            MatchBranch {
                pattern: Expr::Ident("else".to_string(), Span::new(1, 1)),
                body: Block { stmts: vec![
                    Stmt::Expr(Expr::String("other".into(), Span::new(1, 1))),
                ]},
            },
        ],
        span: Span::new(1, 1),
    };
    let ty = check_expr(&match_expr, &mut sym);
    assert_eq!(ty, Some(Type::Array(Box::new(Type::Base(BaseType::Char)))));
}

#[test]
fn bitwise_requires_integer() {
    let mut sym = SymbolTable::new();
    let mut errors = Vec::new();
    let expr = Expr::Binary {
        op: BinOp::BitAnd,
        left: Box::new(Expr::Float(1.0, Span::new(1, 1))),
        right: Box::new(Expr::Int(2, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    let _ = check::analyze_expr(&expr, &mut sym, &mut errors, &HashMap::new());
    assert!(!errors.is_empty());
    assert!(errors[0].0.contains("must be integer"));
}

#[test]
fn shift_returns_i32() {
    let mut sym = SymbolTable::new();
    let expr = Expr::Binary {
        op: BinOp::Shl,
        left: Box::new(Expr::Int(1, Span::new(1, 1))),
        right: Box::new(Expr::Int(2, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::I32)));
}

#[test]
fn bit_not_returns_i32() {
    let mut sym = SymbolTable::new();
    let expr = Expr::Unary {
        op: UnaryOp::BitNot,
        expr: Box::new(Expr::Int(1, Span::new(1, 1))),
        span: Span::new(1, 1),
    };
    assert_eq!(check_expr(&expr, &mut sym), Some(Type::Base(BaseType::I32)));
}
