use crate::parser::ast::*;
use crate::parser::symbol::SymbolTable;
use crate::error::Span;

pub fn analyze_expr(
    expr: &Expr,
    symbols: &mut SymbolTable,
    errors: &mut Vec<(String, Span)>,
) -> Option<Type> {
    let _span = expr_span(expr);
    match expr {
        Expr::Int(..) => Some(Type::Base(BaseType::I32)),
        Expr::Float(..) => Some(Type::Base(BaseType::F64)),
        Expr::String(..) => Some(Type::Base(BaseType::String)),
        Expr::Bool(..) => Some(Type::Base(BaseType::Bool)),
        Expr::Null(_) => Some(Type::Base(BaseType::Null)),
        Expr::Ident(name, s) => match symbols.lookup(name) {
            Some(info) => info.ty.clone(),
            None => {
                errors.push((format!("undefined variable: {}", name), *s));
                None
            }
        },
        Expr::Binary { op, left, right, span: s } => check_binary(op, left, right, symbols, errors, *s),
        Expr::Unary { op, expr, span: s } => check_unary(op, expr, symbols, errors, *s),
        Expr::Call { callee, args, span: s } => check_call(callee, args, symbols, errors, *s),
        Expr::Index { obj, index, span: s } => check_index(obj, index, symbols, errors, *s),
        Expr::Access { obj, field, span: s } => check_access(obj, field, symbols, errors, *s),
        Expr::Assign { target, value, span: s } => check_assign(target, value, symbols, errors, *s),
        Expr::IfExpr { cond, then_block, else_block, span: s } => check_if_expr(cond, then_block, else_block, symbols, errors, *s),
        Expr::Array(elems, _s) => check_array(elems, symbols, errors),
        Expr::Dict(entries, _s) => check_dict(entries, symbols, errors),
        Expr::Cast { expr: e, ty, forced: _, span: _s } => {
            analyze_expr(e, symbols, errors);
            Some(ty.clone())
        }
        Expr::Lambda { params, body, span: _s } => check_lambda(params, body, symbols, errors),
        Expr::MatchExpr { expr: e, branches, span: _s } => check_match_expr(e, branches, symbols, errors),
        Expr::PostfixOp { op: _, target, span: _s } => {
            analyze_expr(target, symbols, errors)
        }
    }
}

pub fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Int(_, s) => *s,
        Expr::Float(_, s) => *s,
        Expr::String(_, s) => *s,
        Expr::Bool(_, s) => *s,
        Expr::Null(s) => *s,
        Expr::Ident(_, s) => *s,
        Expr::Binary { span: s, .. } => *s,
        Expr::Unary { span: s, .. } => *s,
        Expr::Call { span: s, .. } => *s,
        Expr::Index { span: s, .. } => *s,
        Expr::Access { span: s, .. } => *s,
        Expr::Assign { span: s, .. } => *s,
        Expr::IfExpr { span: s, .. } => *s,
        Expr::Array(_, s) => *s,
        Expr::Dict(_, s) => *s,
        Expr::Cast { span: s, .. } => *s,
        Expr::Lambda { span: s, .. } => *s,
        Expr::MatchExpr { span: s, .. } => *s,
        Expr::PostfixOp { span: s, .. } => *s,
    }
}

fn check_binary(
    op: &BinOp,
    left: &Expr,
    right: &Expr,
    symbols: &mut SymbolTable,
    errors: &mut Vec<(String, Span)>,
    _span: Span,
) -> Option<Type> {
    let lt = analyze_expr(left, symbols, errors);
    let rt = analyze_expr(right, symbols, errors);

    match (lt, rt) {
        (Some(l), Some(r)) => match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                if !is_numeric(&l) {
                    errors.push((format!("left operand of {:?} must be numeric, got {:?}", op, l), expr_span(left)));
                }
                if !is_numeric(&r) {
                    errors.push((format!("right operand of {:?} must be numeric, got {:?}", op, r), expr_span(right)));
                }
                Some(wider_type(&l, &r))
            }
            BinOp::Shl | BinOp::Shr | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
                if !is_integer(&l) {
                    errors.push((format!("left operand of {:?} must be integer, got {:?}", op, l), expr_span(left)));
                }
                if !is_integer(&r) {
                    errors.push((format!("right operand of {:?} must be integer, got {:?}", op, r), expr_span(right)));
                }
                Some(Type::Base(BaseType::I32))
            }
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                Some(Type::Base(BaseType::Bool))
            }
            BinOp::And | BinOp::Or => {
                if !matches!(&l, Type::Base(BaseType::Bool)) {
                    errors.push((format!("left operand of &&/|| must be Bool, got {:?}", l), expr_span(left)));
                }
                if !matches!(&r, Type::Base(BaseType::Bool)) {
                    errors.push((format!("right operand of &&/|| must be Bool, got {:?}", r), expr_span(right)));
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
    errors: &mut Vec<(String, Span)>,
    span: Span,
) -> Option<Type> {
    let ty = analyze_expr(expr, symbols, errors);
    match op {
        UnaryOp::Neg => {
            if let Some(ref t) = ty {
                if !is_numeric(t) {
                    errors.push((format!("negation requires numeric type, got {:?}", t), span));
                }
            }
            ty
        }
        UnaryOp::Not => {
            if let Some(ref t) = ty {
                if !matches!(t, Type::Base(BaseType::Bool)) {
                    errors.push((format!("! requires Bool, got {:?}", t), span));
                }
            }
            Some(Type::Base(BaseType::Bool))
        }
        UnaryOp::BitNot => {
            if let Some(ref t) = ty {
                if !is_integer(t) {
                    errors.push((format!("~ requires integer type, got {:?}", t), span));
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
    errors: &mut Vec<(String, Span)>,
    _span: Span,
) -> Option<Type> {
    let name = match callee {
        Expr::Ident(s, _) => Some(s.clone()),
        Expr::Access { obj, field, .. } => {
            if let Expr::Ident(class_name, _) = obj.as_ref() {
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
    errors: &mut Vec<(String, Span)>,
    _span: Span,
) -> Option<Type> {
    let obj_ty = analyze_expr(obj, symbols, errors);
    let idx_ty = analyze_expr(index, symbols, errors);
    if let Some(ref t) = idx_ty {
        if !is_integer(t) && !matches!(t, Type::Base(BaseType::Any)) {
            errors.push((format!("index must be integer, got {:?}", t), expr_span(index)));
        }
    }
    match &obj_ty {
        Some(Type::Array(elem_ty)) => Some(*elem_ty.clone()),
        Some(Type::Map(_kty, vty)) => Some(*vty.clone()),
        _ => Some(Type::Base(BaseType::I32)),
    }
}

fn check_access(
    obj: &Expr,
    field: &str,
    symbols: &mut SymbolTable,
    errors: &mut Vec<(String, Span)>,
    _span: Span,
) -> Option<Type> {
    analyze_expr(obj, symbols, errors);
    match obj {
        Expr::Ident(class_name, _) => {
            let member_key = format!("{}__{}", class_name, field);
            symbols.lookup(&member_key).and_then(|info| info.ty.clone())
        }
        Expr::Access { obj: inner, field: inner_field, .. } => {
            if let Expr::Ident(class_name, _) = inner.as_ref() {
                let member_key = format!("{}__{}__{}", class_name, inner_field, field);
                symbols.lookup(&member_key).and_then(|info| info.ty.clone())
            } else {
                Some(Type::Base(BaseType::Void))
            }
        }
        _ => Some(Type::Base(BaseType::Void)),
    }
}

fn check_assign(
    target: &Expr,
    value: &Expr,
    symbols: &mut SymbolTable,
    errors: &mut Vec<(String, Span)>,
    span: Span,
) -> Option<Type> {
    let val_ty = analyze_expr(value, symbols, errors);
    match target {
        Expr::Ident(name, s) => {
            let symbol = symbols.lookup(name).cloned();
            match symbol {
                Some(info) => {
                    if let (Some(expected), Some(actual)) = (&info.ty, &val_ty) {
                        if !types_compatible(expected, actual) {
                            errors.push((format!(
                                "type mismatch: cannot assign {:?} to {}: {:?}",
                                actual, name, expected
                            ), *s));
                        }
                    }
                }
                None => {
                    let ty = val_ty.clone().unwrap_or(Type::Base(BaseType::Void));
                    symbols.declare(name.clone(), Some(ty));
                }
            }
        }
        _ => {
            errors.push(("assignment target must be an identifier".into(), span));
        }
    }
    val_ty
}

fn check_if_expr(
    cond: &Expr,
    then_block: &Block,
    else_block: &Option<Block>,
    symbols: &mut SymbolTable,
    errors: &mut Vec<(String, Span)>,
    span: Span,
) -> Option<Type> {
    let cond_ty = analyze_expr(cond, symbols, errors);
    if let Some(ty) = &cond_ty {
        if !matches!(ty, Type::Base(BaseType::Bool)) {
            errors.push((format!("if condition must be Bool, got {:?}", ty), span));
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
    errors: &mut Vec<(String, Span)>,
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

// ── Array / Dict / Lambda / MatchExpr ─────────────────

fn check_array(
    elems: &[Expr],
    symbols: &mut SymbolTable,
    errors: &mut Vec<(String, Span)>,
) -> Option<Type> {
    let mut elem_ty = None;
    for e in elems {
        let ty = analyze_expr(e, symbols, errors);
        if elem_ty.is_none() {
            elem_ty = ty;
        }
    }
    Some(Type::Array(Box::new(elem_ty.unwrap_or(Type::Base(BaseType::Void)))))
}

fn check_dict(
    entries: &[(Expr, Expr)],
    symbols: &mut SymbolTable,
    errors: &mut Vec<(String, Span)>,
) -> Option<Type> {
    let mut key_ty = None;
    let mut val_ty = None;
    for (k, v) in entries {
        let kt = analyze_expr(k, symbols, errors);
        let vt = analyze_expr(v, symbols, errors);
        if key_ty.is_none() { key_ty = kt; }
        if val_ty.is_none() { val_ty = vt; }
    }
    Some(Type::Map(
        Box::new(key_ty.unwrap_or(Type::Base(BaseType::Void))),
        Box::new(val_ty.unwrap_or(Type::Base(BaseType::Void))),
    ))
}

fn check_lambda(
    params: &[Param],
    body: &Block,
    symbols: &mut SymbolTable,
    errors: &mut Vec<(String, Span)>,
) -> Option<Type> {
    symbols.push_scope();
    for p in params {
        let ty = p.ty.clone().unwrap_or(Type::Base(BaseType::Void));
        symbols.declare(p.name.clone(), Some(ty));
    }
    let _body_ty = {
        let mut last = None;
        for stmt in &body.stmts {
            last = analyze_expr_stmt(stmt, symbols, errors);
        }
        last
    };
    symbols.pop_scope();
    Some(Type::Named("__lambda".to_string()))
}

fn check_match_expr(
    expr: &Expr,
    branches: &[MatchBranch],
    symbols: &mut SymbolTable,
    errors: &mut Vec<(String, Span)>,
) -> Option<Type> {
    analyze_expr(expr, symbols, errors);
    let mut result_ty = None;
    for branch in branches {
        symbols.push_scope();
        analyze_expr(&branch.pattern, symbols, errors);
        let mut last = None;
        for stmt in &branch.body.stmts {
            last = analyze_expr_stmt(stmt, symbols, errors);
        }
        symbols.pop_scope();
        if result_ty.is_none() { result_ty = last; }
    }
    result_ty
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
    if matches!(a, Type::Base(BaseType::Any)) || matches!(b, Type::Base(BaseType::Any)) {
        return Type::Base(BaseType::Any);
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
