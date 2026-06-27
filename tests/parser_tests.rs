use knot::parser::ast::*;
use knot::parser::Parser;

fn parse(source: &str) -> Vec<Stmt> {
    Parser::new(source).parse_program()
}

// ── Basics ─────────────────────────────────────────

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

// ── Expressions ─────────────────────────────────────

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
fn function_call() {
    let stmts = parse("func f() { print(42) }");
    assert_eq!(stmts.len(), 1);
}

// ── Type Parsing ────────────────────────────────────

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

#[test]
fn type_array_of_i32() {
    let stmts = parse("func f(a: Array[I32]) {}");
    match &stmts[0] {
        Stmt::FuncDef { params, .. } => {
            assert_eq!(params[0].ty, Some(Type::Array(Box::new(Type::Base(BaseType::I32)))));
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn type_map_of_string_to_i32() {
    let stmts = parse("func f(a: Map[String, I32]) {}");
    match &stmts[0] {
        Stmt::FuncDef { params, .. } => {
            assert_eq!(
                params[0].ty,
                Some(Type::Map(Box::new(Type::Base(BaseType::String)), Box::new(Type::Base(BaseType::I32))))
            );
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn type_nullable_array() {
    let stmts = parse("func f(a: Array[I32]?) {}");
    match &stmts[0] {
        Stmt::FuncDef { params, .. } => {
            assert_eq!(
                params[0].ty,
                Some(Type::Nullable(Box::new(Type::Array(Box::new(Type::Base(BaseType::I32))))))
            );
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn all_base_types_parsed() {
    let stmts = parse("func f(a: I8, b: I16, c: I32, d: I64, e: U8, f: U16, g: U32, h: U64, i: F32, j: F64, k: String, l: Bool, m: Null, n: Void) {}");
    match &stmts[0] {
        Stmt::FuncDef { params, .. } => {
            assert_eq!(params.len(), 14);
            assert_eq!(params[0].ty, Some(Type::Base(BaseType::I8)));
            assert_eq!(params[1].ty, Some(Type::Base(BaseType::I16)));
            assert_eq!(params[2].ty, Some(Type::Base(BaseType::I32)));
            assert_eq!(params[3].ty, Some(Type::Base(BaseType::I64)));
            assert_eq!(params[4].ty, Some(Type::Base(BaseType::U8)));
            assert_eq!(params[5].ty, Some(Type::Base(BaseType::U16)));
            assert_eq!(params[6].ty, Some(Type::Base(BaseType::U32)));
            assert_eq!(params[7].ty, Some(Type::Base(BaseType::U64)));
            assert_eq!(params[8].ty, Some(Type::Base(BaseType::F32)));
            assert_eq!(params[9].ty, Some(Type::Base(BaseType::F64)));
            assert_eq!(params[10].ty, Some(Type::Base(BaseType::String)));
            assert_eq!(params[11].ty, Some(Type::Base(BaseType::Bool)));
            assert_eq!(params[12].ty, Some(Type::Base(BaseType::Null)));
            assert_eq!(params[13].ty, Some(Type::Base(BaseType::Void)));
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn type_named_class() {
    let stmts = parse("func f(a: Point) {}");
    match &stmts[0] {
        Stmt::FuncDef { params, .. } => {
            assert_eq!(params[0].ty, Some(Type::Named("Point".to_string())));
        }
        _ => panic!("expected FuncDef"),
    }
}

// ── Generics ────────────────────────────────────────

#[test]
fn generic_function_single_param() {
    let stmts = parse("func identity[T](x: T) -> T { return x }");
    match &stmts[0] {
        Stmt::FuncDef { name, generics, params, ret_ty, .. } => {
            assert_eq!(name, "identity");
            assert_eq!(generics, &["T"]);
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "x");
            assert_eq!(params[0].ty, Some(Type::Named("T".to_string())));
            assert_eq!(*ret_ty, Some(Type::Named("T".to_string())));
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn generic_function_multi_param() {
    let stmts = parse("func pair[K, V](key: K, val: V) -> Map[K, V] { return {key: val} }");
    match &stmts[0] {
        Stmt::FuncDef { name, generics, params, .. } => {
            assert_eq!(name, "pair");
            assert_eq!(generics, &["K", "V"]);
            assert_eq!(params.len(), 2);
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn generic_class() {
    let stmts = parse("class Box[T] {\n    value: T\n}");
    match &stmts[0] {
        Stmt::ClassDef { name, generics, .. } => {
            assert_eq!(name, "Box");
            assert_eq!(generics, &["T"]);
        }
        _ => panic!("expected ClassDef"),
    }
}

#[test]
fn generic_enum() {
    let stmts = parse("enum Option[T] {\n    Some\n    None\n}");
    match &stmts[0] {
        Stmt::EnumDef { name, generics, .. } => {
            assert_eq!(name, "Option");
            assert_eq!(generics, &["T"]);
        }
        _ => panic!("expected EnumDef"),
    }
}

#[test]
fn generic_method_in_class() {
    let stmts = parse("class Util {\n    static func map[T, U](arr: Array[T], f: (T) -> U) -> Array[U] {\n    }\n}");
    assert_eq!(stmts.len(), 1);
}

// ── Control Flow ────────────────────────────────────

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
fn if_else_if_statement() {
    let stmts = parse("func f() { if x > 0 { return 1 } else if x == 0 { return 0 } else { return -1 } }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn if_expression() {
    let stmts = parse("func f() { x = if a > 0 { 1 } else { 0 } }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::IfExpr { .. }));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
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
fn for_loop_over_array() {
    let stmts = parse("func f() { for item in [1, 2, 3] { print(item) } }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn break_with_level() {
    let stmts = parse("func f() { while true { break } }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::While { body: while_body, .. } => {
                assert!(while_body.stmts.iter().any(|s| matches!(s, Stmt::Break(None))));
            }
            _ => panic!("expected While"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn continue_simple() {
    let stmts = parse("func f() { while true { continue } }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::While { body: while_body, .. } => {
                assert!(while_body.stmts.iter().any(|s| matches!(s, Stmt::Continue(None))));
            }
            _ => panic!("expected While"),
        },
        _ => panic!("expected FuncDef"),
    }
}

// ── Match ───────────────────────────────────────────

#[test]
fn match_statement() {
    let stmts = parse("func f() { match x { 1 => { print(\"one\") } 2 => { print(\"two\") } else => { print(\"other\") } } }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => {
            assert!(body.stmts.iter().any(|s| matches!(s, Stmt::Match { .. })));
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn match_expression() {
    let stmts = parse("func f() { label = match x { 1 => \"one\" 2 => \"two\" else => \"other\" } }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::MatchExpr { .. }));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

// ── Lambda ─────────────────────────────────────────

#[test]
fn lambda_simple() {
    let stmts = parse("func f() { double = (x: I32) -> x * 2 }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::Lambda { .. }));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn lambda_block_body() {
    let stmts = parse("func f() { run = () -> { print(\"hello\") } }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn lambda_no_params() {
    let stmts = parse("func f() { greet = () -> \"hello\" }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                match value.as_ref() {
                    Expr::Lambda { params, .. } => {
                        assert!(params.is_empty());
                    }
                    _ => panic!("expected Lambda"),
                }
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

// ── Array / Dict Literals ───────────────────────────

#[test]
fn array_literal() {
    let stmts = parse("func f() { x = [1, 2, 3] }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::Array(..)));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn empty_array() {
    let stmts = parse("func f() { x = [] }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                match value.as_ref() {
                    Expr::Array(elems, _) => assert!(elems.is_empty()),
                    _ => panic!("expected Array"),
                }
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn dict_literal() {
    let stmts = parse("func f() { x = {\"name\": \"Knot\", \"year\": 2026} }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::Dict(..)));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn empty_dict() {
    let stmts = parse("func f() { x = {} }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                match value.as_ref() {
                    Expr::Dict(entries, _) => assert!(entries.is_empty()),
                    _ => panic!("expected Dict"),
                }
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn index_expression() {
    let stmts = parse("func f() { x = arr[0] }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::Index { .. }));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn access_expression() {
    let stmts = parse("func f() { x = obj.field }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::Access { .. }));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

// ── Operators ──────────────────────────────────────

#[test]
fn compound_assign() {
    let stmts = parse("func f() { x += 1\n x -= 1\n x *= 2\n x /= 2\n x %= 2 }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn null_coalesce() {
    let stmts = parse("func f() { x = a ?? 0 }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::Binary { op: BinOp::NullCoalesce, .. }));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn postfix_increment() {
    let stmts = parse("func f() { x++\n x-- }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => {
            assert!(body.stmts.iter().any(|s| matches!(s, Stmt::Expr(Expr::PostfixOp { op: BinOp::Add, .. }))));
            assert!(body.stmts.iter().any(|s| matches!(s, Stmt::Expr(Expr::PostfixOp { op: BinOp::Sub, .. }))));
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn bitwise_operators() {
    let stmts = parse("func f() { x = a | b\n y = a & b\n z = a ^ b\n w = a << 1\n v = a >> 1 }");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn range_operator() {
    let stmts = parse("func f() { x = 0..10 }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                assert!(matches!(value.as_ref(), Expr::Binary { op: BinOp::Range, .. }));
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn cast_as() {
    let stmts = parse("func f() { x = value as I32 }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                match value.as_ref() {
                    Expr::Cast { ty, forced, .. } => {
                        assert_eq!(*ty, Type::Base(BaseType::I32));
                        assert!(!forced);
                    }
                    _ => panic!("expected Cast"),
                }
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn cast_as_bang() {
    let stmts = parse("func f() { x = value as! I64 }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::Expr(Expr::Assign { value, .. }) => {
                match value.as_ref() {
                    Expr::Cast { ty, forced, .. } => {
                        assert_eq!(*ty, Type::Base(BaseType::I64));
                        assert!(forced);
                    }
                    _ => panic!("expected Cast"),
                }
            }
            _ => panic!("expected Assign"),
        },
        _ => panic!("expected FuncDef"),
    }
}

// ── Class / OOP ────────────────────────────────────

#[test]
fn class_def_with_field() {
    let stmts = parse("class Point {\n    x: I32\n    y: I32 = 0\n}");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::ClassDef { name, members, .. } => {
            assert_eq!(name, "Point");
            assert_eq!(members.len(), 2);
        }
        _ => panic!("expected ClassDef"),
    }
}

#[test]
fn class_with_constructor() {
    let stmts = parse("class Point {\n    x: I32\n    func new(x: I32) { this.x = x }\n}");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn class_with_static_method() {
    let stmts = parse("class Math {\n    static func abs(x: I32) -> I32 { return if x >= 0 { x } else { -x } }\n}");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn class_with_operator() {
    let stmts = parse("class Vector {\n    operator +(other: Vector) -> Vector {\n    }\n}");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn abstract_class() {
    let stmts = parse("abstract class Shape {\n    abstract func area() -> F64\n}");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::ClassDef { abstract_class, .. } => {
            assert!(abstract_class);
        }
        _ => panic!("expected ClassDef"),
    }
}

#[test]
fn class_with_mixin() {
    let stmts = parse("class C {\n    mixin A\n    func f() {}\n}");
    match &stmts[0] {
        Stmt::ClassDef { mixins, .. } => {
            assert_eq!(mixins.len(), 1);
            assert_eq!(mixins[0].0, "A");
        }
        _ => panic!("expected ClassDef"),
    }
}

#[test]
fn class_with_private_field() {
    let stmts = parse("class Secret {\n    private code: I32\n}");
    assert_eq!(stmts.len(), 1);
}

// ── Enum ────────────────────────────────────────────

#[test]
fn enum_def() {
    let stmts = parse("enum Color {\n    Red\n    Green\n    Blue\n}");
    match &stmts[0] {
        Stmt::EnumDef { name, variants, .. } => {
            assert_eq!(name, "Color");
            assert_eq!(variants, &["Red", "Green", "Blue"]);
        }
        _ => panic!("expected EnumDef"),
    }
}

// ── Exception Handling ─────────────────────────────

#[test]
fn try_catch_basic() {
    let stmts = parse("func f() { try { risky() } catch e { print(e) } }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => {
            assert!(body.stmts.iter().any(|s| matches!(s, Stmt::TryCatch { .. })));
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn try_catch_with_type_filter() {
    let stmts = parse("func f() { try { mightFail() } catch e: String { print(e) } catch e: I32 { print(e) } }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => match &body.stmts[0] {
            Stmt::TryCatch { catches, .. } => {
                assert_eq!(catches.len(), 2);
                assert_eq!(catches[0].var, Some("e".to_string()));
                assert_eq!(catches[0].ty, Some(Type::Base(BaseType::String)));
                assert_eq!(catches[1].ty, Some(Type::Base(BaseType::I32)));
            }
            _ => panic!("expected TryCatch"),
        },
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn throw_expression() {
    let stmts = parse("func f() { throw \"error\" }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => {
            assert!(body.stmts.iter().any(|s| matches!(s, Stmt::Throw(..))));
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn assert_with_message() {
    let stmts = parse("func f() { assert x > 0, \"x must be positive\" }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => {
            assert!(body.stmts.iter().any(|s| matches!(s, Stmt::Assert { .. })));
        }
        _ => panic!("expected FuncDef"),
    }
}

#[test]
fn assert_without_message() {
    let stmts = parse("func f() { assert x > 0 }");
    assert_eq!(stmts.len(), 1);
}

// ── Import ──────────────────────────────────────────

#[test]
fn import_string_path() {
    let stmts = parse("import \"utils.knot\"");
    match &stmts[0] {
        Stmt::Import { path, alias } => {
            assert_eq!(path, "utils.knot");
            assert!(alias.is_none());
        }
        _ => panic!("expected Import"),
    }
}

#[test]
fn import_with_alias() {
    let stmts = parse("import \"math.knot\" as math");
    match &stmts[0] {
        Stmt::Import { path, alias } => {
            assert_eq!(path, "math.knot");
            assert_eq!(alias.as_deref(), Some("math"));
        }
        _ => panic!("expected Import"),
    }
}

// ── Wrap / Decorator ───────────────────────────────

#[test]
fn top_level_wrap() {
    let stmts = parse("wrap debug(input) {\n    return input\n}\n\n@debug\nfunc hello() { return \"world\" }");
    assert!(stmts.iter().any(|s| matches!(s, Stmt::WrapDef { name, .. } if name == "debug")));
}

#[test]
fn class_level_wrap() {
    let stmts = parse("class Range {\n    wrap binarySearch(check) {\n    }\n}");
    assert_eq!(stmts.len(), 1);
}

// ── Default Parameters ─────────────────────────────

#[test]
fn function_with_default_params() {
    let stmts = parse("func greet(name: String = \"world\", times: I32 = 1) {}");
    match &stmts[0] {
        Stmt::FuncDef { params, .. } => {
            assert_eq!(params.len(), 2);
            assert!(params[0].default.is_some());
            assert!(params[1].default.is_some());
        }
        _ => panic!("expected FuncDef"),
    }
}

// ── Destructor ─────────────────────────────────────

#[test]
fn destructor_in_class() {
    let stmts = parse("class Resource {\n    func delete() { cleanup() }\n}");
    assert_eq!(stmts.len(), 1);
}

// ── Nested Try/Catch ───────────────────────────────

#[test]
fn nested_try_catch() {
    let stmts = parse("func f() { try { try { throw \"inner\" } catch e { print(e) } throw \"outer\" } catch e { print(e) } }");
    match &stmts[0] {
        Stmt::FuncDef { body, .. } => {
            let has_try = body.stmts.iter().any(|s| matches!(s, Stmt::TryCatch { .. }));
            assert!(has_try);
        }
        _ => panic!("expected FuncDef"),
    }
}
