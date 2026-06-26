pub mod check;

use crate::parser::ast::*;
use crate::parser::symbol::SymbolTable;

pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    errors: Vec<String>,
    return_type: Option<Type>,
}

impl SemanticAnalyzer {
    pub fn analyze(program: &[Stmt]) -> Result<SymbolTable, Vec<String>> {
        let mut sa = SemanticAnalyzer {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            return_type: None,
        };
        for stmt in program {
            sa.analyze_stmt(stmt);
        }
        sa.check_entry_point(program);
        if sa.errors.is_empty() {
            Ok(sa.symbols)
        } else {
            Err(sa.errors)
        }
    }

    fn error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    fn type_mismatch(&mut self, expected: &Type, actual: &Type, context: &str) {
        self.error(format!(
            "type mismatch in {}: expected {:?}, got {:?}",
            context, expected, actual
        ));
    }

    fn check_entry_point(&mut self, _program: &[Stmt]) {}

    // ── Statements ────────────────────────────────────────

    fn analyze_stmt(&mut self, stmt: &Stmt) -> Option<Type> {
        match stmt {
            Stmt::FuncDef {
                name,
                params,
                ret_ty,
                body,
            } => {
                self.analyze_func_def(name, params, ret_ty, body);
                None
            }
            Stmt::Return(expr) => self.analyze_return(expr),
            Stmt::If {
                cond,
                then_block,
                else_block,
            } => {
                self.analyze_if(cond, then_block, else_block);
                None
            }
            Stmt::While { cond, body } => {
                self.analyze_while(cond, body);
                None
            }
            Stmt::For { var, iter, body } => {
                self.analyze_for(var, iter, body);
                None
            }
            Stmt::Break(_) | Stmt::Continue(_) => None,
            Stmt::ClassDef { name, members, .. } => {
                self.analyze_class_def(name, members);
                None
            }
            Stmt::EnumDef { name, .. } => {
                self.symbols.declare(name.clone(), None, false);
                None
            }
            Stmt::Import { .. } => None,
            Stmt::Expr(expr) => {
                self.analyze_expr(expr);
                None
            }
            Stmt::Block(block) => self.analyze_block(block),
            Stmt::Match { .. } | Stmt::TryCatch { .. } | Stmt::Throw(_) | Stmt::Assert { .. } | Stmt::WrapDef { .. } => None,
        }
    }

    fn analyze_block(&mut self, block: &Block) -> Option<Type> {
        self.symbols.push_scope();
        let mut last_ty = None;
        for stmt in &block.stmts {
            last_ty = self.analyze_stmt(stmt);
        }
        self.symbols.pop_scope();
        last_ty
    }

    fn analyze_func_def(
        &mut self,
        name: &str,
        params: &[Param],
        ret_ty: &Option<Type>,
        body: &Block,
    ) {
        let prev_ret = self.return_type.clone();
        self.return_type = ret_ty.clone();

        self.symbols.declare(name.to_string(), None, false);
        self.symbols.push_scope();

        for param in params {
            let ty = param.ty.clone().unwrap_or(Type::Base(BaseType::Void));
            self.symbols.declare(param.name.clone(), Some(ty), false);
        }

        self.analyze_block(body);

        self.symbols.pop_scope();
        self.return_type = prev_ret;
    }

    fn analyze_return(&mut self, expr: &Option<Expr>) -> Option<Type> {
        let actual = match expr {
            Some(e) => self.analyze_expr(e),
            None => Some(Type::Base(BaseType::Void)),
        };

        let expected_ret = self.return_type.clone();
        if let (Some(expected), Some(actual)) = (&expected_ret, &actual) {
            if !check::types_compatible(expected, actual) {
                self.type_mismatch(expected, actual, "return");
            }
        }

        actual
    }

    fn analyze_if(
        &mut self,
        cond: &Expr,
        then_block: &Block,
        else_block: &Option<Box<Stmt>>,
    ) {
        let cond_ty = self.analyze_expr(cond);
        if let Some(ty) = &cond_ty {
            if !matches!(ty, Type::Base(BaseType::Bool)) {
                self.type_mismatch(&Type::Base(BaseType::Bool), ty, "if condition");
            }
        }

        self.symbols.push_scope();
        for stmt in &then_block.stmts {
            self.analyze_stmt(stmt);
        }
        self.symbols.pop_scope();

        if let Some(else_stmt) = else_block {
            self.symbols.push_scope();
            self.analyze_stmt(else_stmt);
            self.symbols.pop_scope();
        }
    }

    fn analyze_while(&mut self, cond: &Expr, body: &Block) {
        let cond_ty = self.analyze_expr(cond);
        if let Some(ty) = &cond_ty {
            if !matches!(ty, Type::Base(BaseType::Bool)) {
                self.type_mismatch(&Type::Base(BaseType::Bool), ty, "while condition");
            }
        }

        self.symbols.push_scope();
        for stmt in &body.stmts {
            self.analyze_stmt(stmt);
        }
        self.symbols.pop_scope();
    }

    fn analyze_for(&mut self, var: &str, iter: &Expr, body: &Block) {
        self.symbols.push_scope();
        self.symbols
            .declare(var.to_string(), Some(Type::Base(BaseType::I32)), false);

        let _iter_ty = self.analyze_expr(iter);

        for stmt in &body.stmts {
            self.analyze_stmt(stmt);
        }
        self.symbols.pop_scope();
    }

    fn analyze_class_def(&mut self, name: &str, _members: &[ClassMember]) {
        self.symbols.declare(name.to_string(), None, false);
    }

    // ── Expressions (delegated to check.rs) ──────────────

    fn analyze_expr(&mut self, expr: &Expr) -> Option<Type> {
        check::analyze_expr(expr, &mut self.symbols, &mut self.errors)
    }
}
