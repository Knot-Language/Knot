use crate::parser::ast::*;
use crate::parser::symbol::SymbolTable;

pub fn analyze_expr(
    expr: &Expr,
    symbols: &mut SymbolTable,
    errors: &mut Vec<String>,
) -> Option<Type> {
    match expr {
        Expr::Int(_) => Some(Type::Base(BaseType::I32)),
        Expr::Float(_) => Some(Type::Base(BaseType::F64)),
        Expr::String(_) => Some(Type::Base(BaseType::String)),
        Expr::Bool(_) => Some(Type::Base(BaseType::Bool)),
        Expr::Null => Some(Type::Base(BaseType::Null)),
        Expr::Ident(name) => match symbols.lookup(name) {
            Some(info) => info.ty.clone(),
            None => {
                errors.push(format!("undefined variable: {}", name));
                None
            }
        },
        Expr::Binary { op, left, right } => check_binary(op, left, right, symbols, errors),
        Expr::Unary { op, expr } => check_unary(op, expr, symbols, errors),
        Expr::Call { callee, args } => check_call(callee, args, symbols, errors),
        Expr::Index { obj, index } => check_index(obj, index, symbols, errors),
        Expr::Access { obj, field } => check_access(obj, field, symbols, errors),
        Expr::Assign { target, value } => check_assign(target, value, symbols, errors),
        Expr::IfExpr {
            cond,
            then_block,
            else_block,
        } => check_if_expr(cond, then_block, else_block, symbols, errors),
        Expr::Array(_) | Expr::Dict(_) | Expr::Cast { .. } | Expr::Lambda { .. } | Expr::MatchExpr { .. }
        | Expr::PostfixOp { .. } => None,
    }
}

fn check_binary(
    op: &BinOp,
    left: &Expr,
    right: &Expr,
    symbols: &mut SymbolTable,
    errors: &mut Vec<String>,
) -> Option<Type> {
    let lt = analyze_expr(left, symbols, errors);
    let rt = analyze_expr(right, symbols, errors);

    match (lt, rt) {
        (Some(l), Some(r)) => match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                if !is_numeric(&l) {
                    errors.push(format!("left operand of {:?} must be numeric, got {:?}", op, l));
                }
                if !is_numeric(&r) {
                    errors.push(format!("right operand of {:?} must be numeric, got {:?}", op, r));
                }
                Some(wider_type(&l, &r))
            }
            BinOp::Shl | BinOp::Shr | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
                if !is_integer(&l) {
                    errors.push(format!(
                        "left operand of {:?} must be integer, got {:?}",
                        op, l
                    ));
                }
                if !is_integer(&r) {
                    errors.push(format!(
                        "right operand of {:?} must be integer, got {:?}",
                        op, r
                    ));
                }
                Some(Type::Base(BaseType::I32))
            }
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                Some(Type::Base(BaseType::Bool))
            }
            BinOp::And | BinOp::Or => {
                if !matches!(&l, Type::Base(BaseType::Bool)) {
                    errors.push(format!(
                        "left operand of &&/|| must be Bool, got {:?}",
                        l
                    ));
                }
                if !matches!(&r, Type::Base(BaseType::Bool)) {
                    errors.push(format!(
                        "right operand of &&/|| must be Bool, got {:?}",
                        r
                    ));
                }
                Some(Type::Base(BaseType::Bool))
            }
            BinOp::Range => Some(Type::Base(BaseType::I32)),
            BinOp::NullCoalesce => {
                let inner = if let Type::Nullable(t) = &l {
                    t.as_ref().clone()
                } else {
                    l
                };
                Some(inner)
            }
        },
        _ => None,
    }
}

fn check_unary(
    op: &UnaryOp,
    expr: &Expr,
    symbols: &mut SymbolTable,
    errors: &mut Vec<String>,
) -> Option<Type> {
    let ty = analyze_expr(expr, symbols, errors);
    match op {
        UnaryOp::Neg => {
            if let Some(ref t) = ty {
                if !is_numeric(t) {
                    errors.push(format!("negation requires numeric type, got {:?}", t));
                }
            }
            ty
        }
        UnaryOp::Not => {
            if let Some(ref t) = ty {
                if !matches!(t, Type::Base(BaseType::Bool)) {
                    errors.push(format!("! requires Bool, got {:?}", t));
                }
            }
            Some(Type::Base(BaseType::Bool))
        }
        UnaryOp::BitNot => {
            if let Some(ref t) = ty {
                if !is_integer(t) {
                    errors.push(format!("~ requires integer type, got {:?}", t));
                }
            }
            Some(Type::Base(BaseType::I32))
        }
    }
}

fn check_call(
    callee: &Expr,
    args: &[Expr],
    symbols: &mut SymbolTable,
    errors: &mut Vec<String>,
) -> Option<Type> {
    let name = match callee {
        Expr::Ident(s) => Some(s.clone()),
        Expr::Access { obj, field } => {
            if let Expr::Ident(class_name) = obj.as_ref() {
                Some(format!("{}__{}", class_name, field))
            } else {
                None
            }
        }
        _ => None,
    };
    for arg in args {
        analyze_expr(arg, symbols, errors);
    }
    if let Some(n) = name {
        let fn_ty = symbols.lookup(&n).and_then(|info| info.ty.clone());
        if fn_ty.is_some() {
            return fn_ty;
        }
    }
    None
}

fn check_index(
    obj: &Expr,
    index: &Expr,
    symbols: &mut SymbolTable,
    errors: &mut Vec<String>,
) -> Option<Type> {
    let _obj_ty = analyze_expr(obj, symbols, errors);
    let idx_ty = analyze_expr(index, symbols, errors);
    if let Some(ref t) = idx_ty {
        if !matches!(t, Type::Base(BaseType::I32)) {
            errors.push(format!("index must be I32, got {:?}", t));
        }
    }
    Some(Type::Base(BaseType::I32))
}

fn check_access(
    obj: &Expr,
    _field: &str,
    symbols: &mut SymbolTable,
    errors: &mut Vec<String>,
) -> Option<Type> {
    let _obj_ty = analyze_expr(obj, symbols, errors);
    Some(Type::Base(BaseType::Void))
}

fn check_assign(
    target: &Expr,
    value: &Expr,
    symbols: &mut SymbolTable,
    errors: &mut Vec<String>,
) -> Option<Type> {
    let val_ty = analyze_expr(value, symbols, errors);
    match target {
        Expr::Ident(name) => {
            let symbol = symbols.lookup(name).cloned();
            match symbol {
                Some(info) => {
                    if let (Some(expected), Some(actual)) = (&info.ty, &val_ty) {
                        if !types_compatible(expected, actual) {
                            if info.mutable {
                                symbols.update_type(name, actual.clone());
                            } else {
                                errors.push(format!(
                                    "type mismatch: cannot assign {:?} to {}: {:?}",
                                    actual, name, expected
                                ));
                            }
                        }
                    }
                }
                None => {
                    let ty = val_ty.clone().unwrap_or(Type::Base(BaseType::Void));
                    symbols.declare(name.clone(), Some(ty), false);
                }
            }
        }
        _ => {
            errors.push("assignment target must be an identifier".to_string());
        }
    }
    val_ty
}

fn check_if_expr(
    cond: &Expr,
    then_block: &Block,
    else_block: &Option<Block>,
    symbols: &mut SymbolTable,
    errors: &mut Vec<String>,
) -> Option<Type> {
    let cond_ty = analyze_expr(cond, symbols, errors);
    if let Some(ty) = &cond_ty {
        if !matches!(ty, Type::Base(BaseType::Bool)) {
            errors.push(format!("if condition must be Bool, got {:?}", ty));
        }
    }
    symbols.push_scope();
    let then_ty = {
        let mut last = None;
        for stmt in &then_block.stmts {
            last = analyze_expr_stmt(stmt, symbols, errors);
        }
        last
    };
    symbols.pop_scope();

    if let Some(else_blk) = else_block {
        symbols.push_scope();
        let else_ty = {
            let mut last = None;
            for stmt in &else_blk.stmts {
                last = analyze_expr_stmt(stmt, symbols, errors);
            }
            last
        };
        symbols.pop_scope();
        then_ty.or(else_ty)
    } else {
        then_ty
    }
}

fn analyze_expr_stmt(
    stmt: &Stmt,
    symbols: &mut SymbolTable,
    errors: &mut Vec<String>,
) -> Option<Type> {
    match stmt {
        Stmt::Expr(e) => analyze_expr(e, symbols, errors),
        Stmt::Return(Some(e)) => analyze_expr(e, symbols, errors),
        Stmt::Block(b) => {
            symbols.push_scope();
            let mut last = None;
            for s in &b.stmts {
                last = analyze_expr_stmt(s, symbols, errors);
            }
            symbols.pop_scope();
            last
        }
        _ => None,
    }
}

// ── Type Helpers ───────────────────────────────────────

fn is_integer(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Base(BaseType::I8)
            | Type::Base(BaseType::I16)
            | Type::Base(BaseType::I32)
            | Type::Base(BaseType::I64)
            | Type::Base(BaseType::U8)
            | Type::Base(BaseType::U16)
            | Type::Base(BaseType::U32)
            | Type::Base(BaseType::U64)
    )
}

fn is_numeric(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Base(BaseType::I8)
            | Type::Base(BaseType::I16)
            | Type::Base(BaseType::I32)
            | Type::Base(BaseType::I64)
            | Type::Base(BaseType::U8)
            | Type::Base(BaseType::U16)
            | Type::Base(BaseType::U32)
            | Type::Base(BaseType::U64)
            | Type::Base(BaseType::F32)
            | Type::Base(BaseType::F64)
    )
}

pub fn types_compatible(expected: &Type, actual: &Type) -> bool {
    if expected == actual {
        return true;
    }
    if matches!(expected, Type::Base(BaseType::Any)) || matches!(actual, Type::Base(BaseType::Any)) {
        return true;
    }
    if is_numeric(expected) && is_numeric(actual) {
        return true;
    }
    matches!(
        (expected, actual),
        (Type::Base(BaseType::Null), Type::Nullable(_))
    )
}

fn wider_type(a: &Type, b: &Type) -> Type {
    if a == b {
        return a.clone();
    }
    match (a, b) {
        (Type::Base(BaseType::F64), _) | (_, Type::Base(BaseType::F64)) => Type::Base(BaseType::F64),
        (Type::Base(BaseType::F32), _) | (_, Type::Base(BaseType::F32)) => Type::Base(BaseType::F32),
        (Type::Base(BaseType::I64), _) | (_, Type::Base(BaseType::I64)) => Type::Base(BaseType::I64),
        (Type::Base(BaseType::U64), _) | (_, Type::Base(BaseType::U64)) => Type::Base(BaseType::U64),
        (Type::Base(BaseType::I32), _) | (_, Type::Base(BaseType::I32)) => Type::Base(BaseType::I32),
        (Type::Base(BaseType::U32), _) | (_, Type::Base(BaseType::U32)) => Type::Base(BaseType::U32),
        _ => a.clone(),
    }
}
