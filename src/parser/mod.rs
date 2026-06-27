pub mod ast;
pub mod symbol;

use crate::error::{DiagnosticBag, Span};
use crate::lexer::lexer::tokenize;
use crate::lexer::token::Token;
use crate::parser::ast::*;
use crate::parser::symbol::SymbolTable;

const MAX_PARSE_DEPTH: usize = 256;

pub struct Parser {
    tokens: Vec<Token>,
    positions: Vec<(usize, usize)>,
    pos: usize,
    pub symbols: SymbolTable,
    pub diagnostics: DiagnosticBag,
    depth: usize,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let (tokens, positions) = tokenize(source);
        Parser {
            tokens,
            positions,
            pos: 0,
            symbols: SymbolTable::new(),
            diagnostics: DiagnosticBag::new(source),
            depth: 0,
        }
    }

    fn enter_depth(&mut self) -> bool {
        if self.depth >= MAX_PARSE_DEPTH {
            self.error("maximum nesting depth exceeded".to_string());
            return false;
        }
        self.depth += 1;
        true
    }

    fn leave_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    fn current_span(&self) -> Span {
        let (line, col) = self.positions
            .get(self.pos.min(self.positions.len().saturating_sub(1)))
            .copied()
            .unwrap_or((1, 1));
        Span::new(line, col)
    }

    fn error(&mut self, msg: String) {
        self.diagnostics.error(msg, self.current_span());
    }

    fn warn(&mut self, msg: String) {
        self.diagnostics.warn(msg, self.current_span());
    }

    fn err_expected(&mut self, expected: &str, got: &Token) {
        self.error(format!(
            "expected {}, got {}",
            expected,
            token_desc(got)
        ));
    }

    fn peek(&self) -> &Token {
        if self.pos >= self.tokens.len() {
            // Return a static EOF reference; safe because tokens always ends with EOF
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[self.pos]
        }
    }

    fn advance(&mut self) -> Token {
        if self.pos >= self.tokens.len() {
            return Token::Eof;
        }
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }

    fn is_op(&self, op: &str) -> bool {
        matches!(self.peek(), Token::Operator(s) if s == op)
    }

    fn is_kw(&self, kw: &str) -> bool {
        matches!(self.peek(), Token::Keyword(s) if s == kw)
    }

    fn is_newline(&self) -> bool {
        matches!(self.peek(), Token::NewLine)
    }

    fn at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn skip_newlines(&mut self) {
        while self.is_newline() {
            self.advance();
        }
    }

    fn expect_newline_or_end(&mut self) {
        if self.is_newline() {
            self.advance();
            return;
        }
        if self.at_end() || self.is_op("}") {
            return;
        }
        let tok = self.peek().clone();
        self.err_expected("newline or end of block", &tok);
        self.sync_to_stmt_end();
    }

    fn expect_operator(&mut self, op: &str) {
        let tok = self.peek().clone();
        match &tok {
            Token::Operator(s) if s == op => {
                self.advance();
            }
            _ => {
                self.err_expected(&format!("'{}'", op), &tok);
            }
        }
    }

    fn expect_keyword(&mut self, kw: &str) {
        let tok = self.peek().clone();
        match &tok {
            Token::Keyword(s) if s == kw => {
                self.advance();
            }
            _ => {
                self.err_expected(&format!("'{}'", kw), &tok);
            }
        }
    }

    fn sync_to_stmt_end(&mut self) {
        while !self.at_end() && !self.is_newline() && !self.is_op("}") {
            self.advance();
        }
    }

    #[allow(dead_code)]
    fn sync_to_block_end(&mut self) {
        let mut depth = 0;
        while !self.at_end() {
            if self.is_op("{") {
                depth += 1;
            } else if self.is_op("}") {
                if depth == 0 {
                    return;
                }
                depth -= 1;
            }
            self.advance();
        }
    }

    // 鈹€鈹€ Program 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.at_end() {
            stmts.push(self.parse_top_level());
            self.skip_newlines();
        }
        stmts
    }

    fn parse_top_level(&mut self) -> Stmt {
        if self.is_op("@") {
            self.parse_wrap_annotation()
        } else if self.is_kw("func") {
            self.parse_func_def()
        } else if self.is_kw("class") || self.is_kw("abstract") {
            self.parse_class_def()
        } else if self.is_kw("enum") {
            self.parse_enum_def()
        } else if self.is_kw("import") {
            self.parse_import()
        } else if self.is_kw("wrap") {
            self.parse_top_level_wrap()
        } else if self.is_kw("try") {
            Stmt::TryCatch {
                try_block: Block { stmts: vec![] },
                catches: vec![],
            }
        } else {
            self.parse_stmt()
        }
    }

    // 鈹€鈹€ Statements 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    fn parse_stmt(&mut self) -> Stmt {
        if self.is_kw("return") {
            self.parse_return()
        } else if self.is_kw("if") {
            self.parse_if()
        } else if self.is_kw("while") {
            self.parse_while()
        } else if self.is_kw("for") {
            self.parse_for()
        } else if self.is_kw("break") {
            self.parse_break()
        } else if self.is_kw("continue") {
            self.parse_continue()
        } else if self.is_kw("match") {
            self.parse_match()
        } else if self.is_kw("throw") {
            self.parse_throw()
        } else if self.is_kw("try") {
            self.parse_try_catch()
        } else if self.is_kw("assert") {
            self.parse_assert()
        } else if self.is_op("{") {
            self.parse_block()
        } else {
            self.parse_expr_stmt()
        }
    }

    fn parse_break(&mut self) -> Stmt {
        self.advance();
        let n = self.parse_ntimes();
        self.expect_newline_or_end();
        Stmt::Break(n)
    }

    fn parse_continue(&mut self) -> Stmt {
        self.advance();
        let n = self.parse_ntimes();
        self.expect_newline_or_end();
        Stmt::Continue(n)
    }

    fn parse_ntimes(&mut self) -> Option<u32> {
        let v = match self.peek() {
            Token::Number(crate::lexer::num::Num::I32(v)) => {
                if *v <= 0 {
                    self.warn(format!("break/continue level must be a positive integer, got {}", v));
                    None
                } else {
                    Some(*v as u32)
                }
            }
            _ => return None,
        };
        self.advance();
        v
    }

    fn parse_match(&mut self) -> Stmt {
        self.advance();
        let expr = self.parse_expr();
        let branches = self.parse_match_body();
        Stmt::Match { expr, branches }
    }

    fn parse_match_body(&mut self) -> Vec<MatchBranch> {
        self.expect_operator("{");
        let mut branches = Vec::new();
        self.skip_newlines();
        while !self.at_end() && !self.is_op("}") {
            if !self.is_newline() {
                branches.push(self.parse_match_branch());
            }
            self.skip_newlines();
        }
        self.expect_operator("}");
        branches
    }

    fn parse_match_branch(&mut self) -> MatchBranch {
        let pattern = if self.is_kw("else") {
            self.advance();
            Expr::Ident("else".to_string(), self.current_span())
        } else {
            self.parse_expr()
        };
        self.expect_operator("=>");
        let body = if self.is_op("{") {
            match self.parse_block() {
                Stmt::Block(b) => b,
                _ => Block { stmts: vec![] },
            }
        } else {
            let e = self.parse_expr();
            self.expect_newline_or_end();
            Block { stmts: vec![Stmt::Expr(e)] }
        };
        MatchBranch { pattern, body }
    }

    fn parse_throw(&mut self) -> Stmt {
        self.advance();
        let expr = self.parse_expr();
        self.expect_newline_or_end();
        Stmt::Throw(expr)
    }

    fn parse_try_catch(&mut self) -> Stmt {
        self.advance();
        let try_block = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block after 'try'".to_string());
                Block { stmts: vec![] }
            }
        };
        let mut catches = Vec::new();
        while self.is_kw("catch") {
            self.advance();
            let var = if matches!(self.peek(), Token::Identifier(_)) {
                let v = self.parse_ident();
                if self.is_op(":") {
                    self.advance();
                    let ty = self.parse_type();
                    Some((v, Some(ty)))
                } else {
                    Some((v, None))
                }
            } else {
                None
            };
            let body = match self.parse_block() {
                Stmt::Block(b) => b,
                _ => {
                    self.error("expected block after 'catch'".to_string());
                    Block { stmts: vec![] }
                }
            };
            if let Some((v, ty)) = var {
                catches.push(CatchClause {
                    var: Some(v),
                    ty,
                    body,
                });
            } else {
                catches.push(CatchClause { var: None, ty: None, body });
            }
        }
        Stmt::TryCatch { try_block, catches }
    }

    fn parse_assert(&mut self) -> Stmt {
        self.advance();
        let expr = self.parse_expr();
        let message = if self.is_op(",") {
            self.advance();
            match self.advance() {
                Token::String(s) => Some(s.clone()),
                t => {
                    self.err_expected("string message", &t);
                    None
                }
            }
        } else {
            None
        };
        self.expect_newline_or_end();
        Stmt::Assert { expr, message }
    }

    fn parse_expr_stmt(&mut self) -> Stmt {
        let expr = self.parse_expr();
        self.expect_newline_or_end();
        Stmt::Expr(expr)
    }

    fn parse_block(&mut self) -> Stmt {
        self.expect_operator("{");
        if !self.enter_depth() {
            self.sync_to_block_end();
            return Stmt::Block(Block { stmts: vec![] });
        }
        self.symbols.push_scope();
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.at_end() && !self.is_op("}") {
            if !self.is_newline() {
                stmts.push(self.parse_stmt());
            }
            self.skip_newlines();
        }
        self.symbols.pop_scope();
        self.leave_depth();
        self.expect_operator("}");
        Stmt::Block(Block { stmts })
    }

    fn parse_return(&mut self) -> Stmt {
        self.advance();
        if self.is_newline() || self.at_end() || self.is_op("}") {
            self.expect_newline_or_end();
            Stmt::Return(None)
        } else {
            let expr = self.parse_expr();
            self.expect_newline_or_end();
            Stmt::Return(Some(expr))
        }
    }

    fn parse_if(&mut self) -> Stmt {
        self.advance();
        let cond = self.parse_expr();
        let then_block = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block after if condition".to_string());
                Block { stmts: vec![] }
            }
        };

        let else_block = if self.is_kw("else") {
            self.advance();
            self.skip_newlines();
            if self.is_kw("if") {
                if self.enter_depth() {
                    let result = Some(Box::new(self.parse_if()));
                    self.leave_depth();
                    result
                } else {
                    None
                }
            } else {
                match self.parse_block() {
                    Stmt::Block(b) => Some(Box::new(Stmt::Block(b))),
                    _ => {
                        self.error("expected block after else".to_string());
                        None
                    }
                }
            }
        } else {
            None
        };

        Stmt::If {
            cond,
            then_block,
            else_block,
        }
    }

    fn parse_while(&mut self) -> Stmt {
        self.advance();
        let cond = self.parse_expr();
        let body = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block after while condition".to_string());
                Block { stmts: vec![] }
            }
        };
        Stmt::While { cond, body }
    }

    fn parse_for(&mut self) -> Stmt {
        self.advance();
        let var = match self.advance() {
            Token::Identifier(s) => s.clone(),
            t => {
                self.err_expected("identifier after 'for'", &t);
                "_".to_string()
            }
        };
        self.expect_keyword("in");
        let iter = self.parse_expr();
        let body = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block after for".to_string());
                Block { stmts: vec![] }
            }
        };
        Stmt::For { var, iter, body }
    }

    // 鈹€鈹€ Class Definition 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    fn parse_class_def(&mut self) -> Stmt {
        let is_abstract = if self.is_kw("abstract") {
            self.advance();
            true
        } else {
            false
        };
        self.expect_keyword("class");
        let name = self.parse_ident();
        let generics = self.parse_generic_params();
        let (members, mixins) = self.parse_class_body();
        Stmt::ClassDef {
            name,
            abstract_class: is_abstract,
            generics,
            mixins,
            members,
        }
    }

    fn parse_class_body(&mut self) -> (Vec<ClassMember>, Vec<(String, Span)>) {
        self.expect_operator("{");
        let mut members = Vec::new();
        let mut mixins = Vec::new();
        self.skip_newlines();
        while !self.at_end() && !self.is_op("}") {
            if !self.is_newline() {
                if self.is_kw("mixin") {
                    let span = self.current_span();
                    self.advance();
                    mixins.push((self.parse_ident(), span));
                } else {
                    members.push(self.parse_class_member());
                }
            }
            self.skip_newlines();
        }
        self.expect_operator("}");
        (members, mixins)
    }

    fn parse_class_member(&mut self) -> ClassMember {
        let is_private = if self.is_kw("private") {
            self.advance();
            true
        } else {
            false
        };

        if self.is_kw("abstract") {
            self.advance();
            self.expect_keyword("func");
            let name = self.parse_ident();
            let params = self.parse_params();
            let ret_ty = if self.is_op("->") {
                self.advance();
                Some(self.parse_type())
            } else {
                None
            };
            self.expect_newline_or_end();
            return ClassMember::AbstractMethod { name, params, ret_ty };
        }

        if self.is_kw("func") {
            self.advance();
            if self.is_kw("new") {
                self.advance();
                self.parse_constructor(is_private)
            } else if self.is_kw("delete") {
                self.advance();
                self.parse_destructor(is_private)
            } else {
                self.parse_method(is_private, false)
            }
        } else if self.is_kw("static") {
            self.advance();
            if self.is_kw("func") {
                self.advance();
                if self.is_kw("new") {
                    self.warn("'static' is redundant for constructor 'new'".to_string());
                    self.advance();
                    self.parse_constructor(is_private)
                } else if self.is_kw("delete") {
                    self.warn("'static' is redundant for destructor 'delete'".to_string());
                    self.advance();
                    self.parse_destructor(is_private)
                } else {
                    self.parse_method(is_private, true)
                }
            } else {
                let tok = self.peek().clone();
                self.err_expected("'func' after 'static'", &tok);
                self.sync_to_stmt_end();
                ClassMember::Field {
                    private: false,
                    name: "_".to_string(),
                    ty: None,
                    default: None,
                }
            }
        } else if self.is_kw("operator") {
            self.parse_operator_def(is_private)
        } else if self.is_kw("wrap") {
            self.parse_wrap_def()
        } else {
            self.parse_field(is_private)
        }
    }

    fn parse_constructor(&mut self, private: bool) -> ClassMember {
        let params = self.parse_params();
        if self.is_op("->") {
            self.warn("return type is redundant for constructor 'new'".to_string());
            self.advance();
            let _ = self.parse_type();
        }
        let body = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block for constructor body".to_string());
                Block { stmts: vec![] }
            }
        };
        ClassMember::New { private, params, body }
    }

    fn parse_destructor(&mut self, private: bool) -> ClassMember {
        let _ = self.parse_params();
        if self.is_op("->") {
            self.warn("destructor 'delete' cannot have return type".to_string());
            self.advance();
            let _ = self.parse_type();
        }
        let body = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block for destructor body".to_string());
                Block { stmts: vec![] }
            }
        };
        ClassMember::Delete { private, body }
    }

    fn parse_field(&mut self, private: bool) -> ClassMember {
        let name = self.parse_ident();
        let ty = if self.is_op(":") {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };
        let default = if self.is_op("=") {
            self.advance();
            Some(self.parse_expr())
        } else {
            None
        };
        self.expect_newline_or_end();
        ClassMember::Field {
            private,
            name,
            ty,
            default,
        }
    }

    fn parse_method(&mut self, private: bool, is_static: bool) -> ClassMember {
        let name = self.parse_ident();
        let generics = self.parse_generic_params();
        let params = self.parse_params();
        let ret_ty = if self.is_op("->") {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };
        let body = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block for method body".to_string());
                Block { stmts: vec![] }
            }
        };
        if is_static {
            ClassMember::StaticMethod {
                private,
                name,
                generics,
                params,
                ret_ty,
                body,
            }
        } else {
            ClassMember::Method {
                private,
                name,
                generics,
                params,
                ret_ty,
                body,
            }
        }
    }

    fn parse_operator_def(&mut self, _private: bool) -> ClassMember {
        self.advance();
        let op = match self.advance() {
            Token::Operator(s) => s.clone(),
            Token::Keyword(s) if s == "as" => {
                let ident = self.parse_ident();
                format!("as {}", ident)
            }
            t => {
                self.err_expected("operator symbol", &t);
                "+".to_string()
            }
        };
        let generics = self.parse_generic_params();
        let params = self.parse_params();
        let ret_ty = if self.is_op("->") {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };
        let body = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block for operator body".to_string());
                Block { stmts: vec![] }
            }
        };
        ClassMember::Operator {
            op,
            generics,
            params,
            ret_ty,
            body,
        }
    }

    fn parse_wrap_annotation(&mut self) -> Stmt {
        self.advance();
        let wrap_expr = self.parse_expr();
        let (callee_ident, mut extra_args) = match &wrap_expr {
            Expr::Call { callee, args, .. } => (callee.clone(), args.clone()),
            other => (Box::new(other.clone()), vec![]),
        };

        let stmt = if self.is_kw("func") {
            self.parse_func_def()
        } else {
            self.parse_stmt()
        };

        match stmt {
            Stmt::FuncDef { name, params, body, .. } => {
                let mut args = vec![Expr::Lambda { params, body, span: self.current_span() }];
                args.append(&mut extra_args);
                Stmt::Expr(Expr::Assign {
                    target: Box::new(Expr::Ident(name, self.current_span())),
                    value: Box::new(Expr::Call { callee: callee_ident, args, span: self.current_span() }),
                    span: self.current_span(),
                })
            }
            Stmt::Expr(Expr::Assign { target, value, .. }) => {
                let mut args = vec![*value];
                args.append(&mut extra_args);
                Stmt::Expr(Expr::Assign {
                    target,
                    value: Box::new(Expr::Call { callee: callee_ident, args, span: self.current_span() }),
                    span: self.current_span(),
                })
            }
            _other => {
                self.error("expected function or variable declaration after @wrap".to_string());
                Stmt::Expr(Expr::Null(self.current_span()))
            }
        }
    }

    fn parse_wrap_def(&mut self) -> ClassMember {
        self.advance();
        let name = self.parse_ident();
        let params = self.parse_params();
        let body = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block for wrap body".to_string());
                Block { stmts: vec![] }
            }
        };
        ClassMember::Wrap { name, params, body }
    }

    fn parse_top_level_wrap(&mut self) -> Stmt {
        self.advance();
        let name = self.parse_ident();
        let params = self.parse_params();
        let body = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block for wrap body".to_string());
                Block { stmts: vec![] }
            }
        };
        Stmt::WrapDef { name, params, body }
    }

    // 鈹€鈹€ Enum Definition 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    fn parse_enum_def(&mut self) -> Stmt {
        self.advance();
        let name = self.parse_ident();
        let generics = self.parse_generic_params();
        self.expect_operator("{");
        let mut variants = Vec::new();
        self.skip_newlines();
        while !self.at_end() && !self.is_op("}") {
            if !self.is_newline() {
                variants.push(self.parse_ident());
            }
            self.skip_newlines();
        }
        self.expect_operator("}");
        Stmt::EnumDef {
            name,
            generics,
            variants,
        }
    }

    // 鈹€鈹€ Import 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    fn parse_import(&mut self) -> Stmt {
        self.advance();
        let path = match self.advance() {
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            t => {
                self.err_expected("import path", &t);
                "".to_string()
            }
        };
        let alias = if self.is_kw("as") {
            self.advance();
            Some(self.parse_ident())
        } else {
            None
        };
        self.expect_newline_or_end();
        Stmt::Import { path, alias }
    }

    // 鈹€鈹€ Helpers 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    fn parse_ident(&mut self) -> String {
        match self.advance() {
            Token::Identifier(s) => s.clone(),
            t => {
                self.err_expected("identifier", &t);
                "_".to_string()
            }
        }
    }

    fn parse_generic_params(&mut self) -> Vec<String> {
        if self.is_op("[") {
            self.advance();
            let params = self.parse_ident_list();
            self.expect_operator("]");
            params
        } else {
            Vec::new()
        }
    }

    fn parse_ident_list(&mut self) -> Vec<String> {
        let mut list = Vec::new();
        loop {
            list.push(self.parse_ident());
            if self.is_op(",") {
                self.advance();
            } else {
                break;
            }
        }
        list
    }

    // 鈹€鈹€ Function Definition 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    fn parse_func_def(&mut self) -> Stmt {
        self.advance();
        let name = match self.advance() {
            Token::Identifier(s) => s.clone(),
            t => {
                self.err_expected("function name", &t);
                "_".to_string()
            }
        };

        self.symbols.declare(name.clone(), None);
        self.symbols.push_scope();

        let generics = self.parse_generic_params();
        let params = self.parse_params();

        let ret_ty = if self.is_op("->") {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };

        let body = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block for function body".to_string());
                Block { stmts: vec![] }
            }
        };

        self.symbols.pop_scope();
        Stmt::FuncDef {
            name,
            generics,
            params,
            ret_ty,
            body,
        }
    }

    fn parse_params(&mut self) -> Vec<Param> {
        self.expect_operator("(");
        let mut params = Vec::new();

        if self.is_op(")") {
            self.advance();
            return params;
        }

        loop {
            let is_args = if self.is_kw("args") {
                self.advance();
                true
            } else {
                false
            };

            let is_kwargs = if self.is_kw("kwargs") {
                self.advance();
                true
            } else {
                false
            };

            let name = match self.advance() {
                Token::Identifier(s) => s.clone(),
                t => {
                    self.err_expected("parameter name", &t);
                    "_".to_string()
                }
            };

            let ty = if self.is_op(":") {
                self.advance();
                Some(self.parse_type())
            } else {
                None
            };

            let default = if self.is_op("=") {
                self.advance();
                Some(self.parse_expr())
            } else {
                None
            };

            self.symbols.declare(name.clone(), ty.clone());
            params.push(Param { name, ty, default, is_args, is_kwargs });

            if self.is_op(",") {
                self.advance();
            } else if self.is_op(")") {
                self.advance();
                break;
            } else {
                self.err_expected("',' or ')'", &self.peek().clone());
                break;
            }
        }

        params
    }

    // 鈹€鈹€ Types 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    fn parse_type(&mut self) -> Type {
        let ty = match self.advance() {
            Token::Identifier(s) => match s.as_str() {
                "I8" => Type::Base(BaseType::I8),
                "I16" => Type::Base(BaseType::I16),
                "I32" => Type::Base(BaseType::I32),
                "I64" => Type::Base(BaseType::I64),
                "U8" => Type::Base(BaseType::U8),
                "U16" => Type::Base(BaseType::U16),
                "U32" => Type::Base(BaseType::U32),
                "U64" => Type::Base(BaseType::U64),
                "F32" => Type::Base(BaseType::F32),
                "F64" => Type::Base(BaseType::F64),
                "String" => Type::Base(BaseType::String),
                "Bool" => Type::Base(BaseType::Bool),
                "Null" => Type::Base(BaseType::Null),
                "Void" => Type::Base(BaseType::Void),
                _ => Type::Named(s.clone()),
            },
            t => {
                self.err_expected("type name", &t);
                Type::Base(BaseType::Void)
            }
        };

        if self.is_op("?") {
            self.advance();
            Type::Nullable(Box::new(ty))
        } else {
            ty
        }
    }

    // 鈹€鈹€ Expressions (Pratt) 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(1)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Expr {
        let mut left = self.parse_prefix();

        loop {
            let op_bp = self.infix_bp();
            if op_bp < min_bp {
                break;
            }

            if self.is_op("=") {
                self.advance();
                let right = self.parse_expr_bp(op_bp);
                left = Expr::Assign {
                    target: Box::new(left),
                    value: Box::new(right),
                    span: self.current_span(),
                };
            } else if self.is_compound_assign() {
                left = self.parse_compound_assign(left, op_bp);
            } else if self.is_op("++") || self.is_op("--") {
                let s = match self.advance() {
                    Token::Operator(s) => s.clone(),
                    _ => unreachable!(),
                };
                let binop = if s == "++" { BinOp::Add } else { BinOp::Sub };
                left = Expr::PostfixOp { op: binop, target: Box::new(left), span: self.current_span() };
            } else if self.is_op(".") {
                self.advance();
                let field = match self.advance() {
                    Token::Identifier(s) => s.clone(),
                    t => {
                        self.err_expected("field name after '.'", &t);
                        "_".to_string()
                    }
                };
                left = Expr::Access {
                    obj: Box::new(left),
                    field,
                    span: self.current_span(),
                };
            } else if self.is_op("::") {
                self.advance();
                let field = match self.advance() {
                    Token::Identifier(s) => s.clone(),
                    t => {
                        self.err_expected("field name after '::'", &t);
                        "_".to_string()
                    }
                };
                if self.is_op("(") {
                    self.advance();
                    let args = self.parse_args();
                    left = Expr::Call {
                        callee: Box::new(Expr::Access {
                            obj: Box::new(left),
                            field,
                            span: self.current_span(),
                        }),
                        args,
                        span: self.current_span(),
                    };
                } else {
                    left = Expr::Access {
                        obj: Box::new(left),
                        field,
                        span: self.current_span(),
                    };
                }
            } else if self.is_op("(") {
                self.advance();
                let args = self.parse_args();
                left = Expr::Call {
                    callee: Box::new(left),
                    args,
                    span: self.current_span(),
                };
            } else if self.is_op("[") {
                self.advance();
                let index = self.parse_expr();
                self.expect_operator("]");
                left = Expr::Index {
                    obj: Box::new(left),
                    index: Box::new(index),
                    span: self.current_span(),
                };
            } else if self.is_kw("as") || self.is_kw("as!") {
                let forced = self.is_kw("as!");
                self.advance();
                let ty = self.parse_type();
                left = Expr::Cast {
                    expr: Box::new(left),
                    ty,
                    forced,
                    span: self.current_span(),
                };
            } else {
                let op = self.parse_binary_op();
                let right = self.parse_expr_bp(op_bp);
                left = Expr::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                    span: self.current_span(),
                };
            }
        }

        left
    }

    fn parse_binary_op(&mut self) -> BinOp {
        match self.advance() {
            Token::Operator(s) => match s.as_str() {
                "+" => BinOp::Add,
                "-" => BinOp::Sub,
                "*" => BinOp::Mul,
                "/" => BinOp::Div,
                "%" => BinOp::Mod,
                "==" => BinOp::Eq,
                "!=" => BinOp::Neq,
                "<" => BinOp::Lt,
                ">" => BinOp::Gt,
                "<=" => BinOp::Le,
                ">=" => BinOp::Ge,
                "&&" => BinOp::And,
                "||" => BinOp::Or,
                "<<" => BinOp::Shl,
                ">>" => BinOp::Shr,
                "&" => BinOp::BitAnd,
                "|" => BinOp::BitOr,
                "^" => BinOp::BitXor,
                ".." => BinOp::Range,
                "??" => BinOp::NullCoalesce,
                _ => {
                    self.error(format!("unknown binary operator: {}", s));
                    BinOp::Add
                }
            },
            t => {
                self.err_expected("binary operator", &t);
                BinOp::Add
            }
        }
    }

    fn parse_prefix(&mut self) -> Expr {
        match self.peek().clone() {
            Token::Operator(s) if s == "@" => {
                self.advance();
                let expr = self.parse_expr_bp(90);
                match expr {
                    Expr::Call { callee, mut args, .. } => {
                        match callee.as_ref() {
                            Expr::Access { obj, .. } => {
                                args.insert(0, obj.as_ref().clone());
                            }
                            _ => {
                                args.insert(0, callee.as_ref().clone());
                            }
                        }
                        Expr::Call { callee, args, span: self.current_span() }
                    }
                    other => {
                        Expr::Call {
                            callee: Box::new(other.clone()),
                            args: vec![other],
                            span: self.current_span(),
                        }
                    }
                }
            }
            Token::Operator(s) if s == "-" || s == "!" || s == "~" || s == "++" || s == "--" => {
                if s == "++" || s == "--" {
                    let op = if s == "++" { BinOp::Add } else { BinOp::Sub };
                    self.advance();
                    let target = self.parse_expr_bp(85);
                    let one = Expr::Int(1, self.current_span());
                    Expr::Assign {
                        target: Box::new(target.clone()),
                        value: Box::new(Expr::Binary {
                            op,
                            left: Box::new(target),
                            right: Box::new(one),
                            span: self.current_span(),
                        }),
                        span: self.current_span(),
                    }
                } else {
                    let op = match s.as_str() {
                        "-" => UnaryOp::Neg,
                        "!" => UnaryOp::Not,
                        "~" => UnaryOp::BitNot,
                        _ => unreachable!(),
                    };
                    self.advance();
                    let expr = self.parse_expr_bp(85);
                    Expr::Unary {
                        op,
                        expr: Box::new(expr),
                        span: self.current_span(),
                    }
                }
            }
            Token::Number(n) => {
                self.advance();
                let span = self.current_span();
                match n {
                    crate::lexer::num::Num::F32(v) => Expr::Float(v as f64, span),
                    crate::lexer::num::Num::F64(v) => Expr::Float(v, span),
                    crate::lexer::num::Num::I8(v) => Expr::Int(v as i64, span),
                    crate::lexer::num::Num::I16(v) => Expr::Int(v as i64, span),
                    crate::lexer::num::Num::I32(v) => Expr::Int(v as i64, span),
                    crate::lexer::num::Num::I64(v) => Expr::Int(v, span),
                    crate::lexer::num::Num::U8(v) => Expr::Int(v as i64, span),
                    crate::lexer::num::Num::U16(v) => Expr::Int(v as i64, span),
                    crate::lexer::num::Num::U32(v) => Expr::Int(v as i64, span),
                    crate::lexer::num::Num::U64(v) => Expr::Int(v as i64, span),
                }
            }
            Token::String(s) => {
                self.advance();
                Expr::String(s.clone(), self.current_span())
            }
            Token::Keyword(ref s) if s == "true" => {
                self.advance();
                Expr::Bool(true, self.current_span())
            }
            Token::Keyword(ref s) if s == "false" => {
                self.advance();
                Expr::Bool(false, self.current_span())
            }
            Token::Keyword(ref s) if s == "null" => {
                self.advance();
                Expr::Null(self.current_span())
            }
            Token::Identifier(s) => {
                self.advance();
                Expr::Ident(s.clone(), self.current_span())
            }
            Token::Operator(s) if s == "(" => {
                self.advance();
                if self.is_op(")") {
                    self.advance();
                    if self.is_op("->") {
                        self.advance();
                        let body = if self.is_op("{") {
                            match self.parse_block() {
                                Stmt::Block(b) => b,
                                _ => Block { stmts: vec![] },
                            }
                        } else {
                            let e = self.parse_expr();
                            Block { stmts: vec![Stmt::Return(Some(e))] }
                        };
                        return Expr::Lambda { params: vec![], body, span: self.current_span() };
                    }
                    return Expr::Null(self.current_span());
                }
                if self.try_parse_lambda() {
                    return self.parse_lambda_body();
                }
                let expr = self.parse_expr();
                self.expect_operator(")");
                expr
            }
            Token::Operator(s) if s == "[" => self.parse_array(),
            Token::Operator(s) if s == "{" => self.parse_dict(),
            Token::Keyword(ref s) if s == "if" => self.parse_if_expr(),
            Token::Keyword(ref s) if s == "match" => self.parse_match_expr(),
            t => {
                self.error(format!(
                    "unexpected token in expression: {}",
                    token_desc(&t)
                ));
                self.advance();
                Expr::Null(self.current_span())
            }
        }
    }

    fn try_parse_lambda(&mut self) -> bool {
        let saved = self.pos;
        while !self.at_end() {
            match self.peek() {
                Token::Operator(s) if s == ")" => {
                    self.advance();
                    let is_lambda = self.is_op("->");
                    self.pos = saved;
                    return is_lambda;
                }
                Token::Identifier(_) => {
                    self.advance();
                    if self.is_op(":") {
                        self.advance();
                        let _ = self.parse_type();
                    }
                    if self.is_op(",") {
                        self.advance();
                    }
                }
                _ => {
                    self.pos = saved;
                    return false;
                }
            }
        }
        self.pos = saved;
        false
    }

    fn parse_lambda_body(&mut self) -> Expr {
        let params = self.parse_params();
        self.expect_operator("->");
        let body = if self.is_op("{") {
            match self.parse_block() {
                Stmt::Block(b) => b,
                _ => Block { stmts: vec![] },
            }
        } else {
            let e = self.parse_expr();
            Block { stmts: vec![Stmt::Return(Some(e))] }
        };
        Expr::Lambda { params, body, span: self.current_span() }
    }

    fn parse_array(&mut self) -> Expr {
        self.advance();
        let mut items = Vec::new();
        self.skip_newlines();
        if !self.is_op("]") {
            loop {
                items.push(self.parse_expr());
                if self.is_op(",") {
                    self.advance();
                } else if self.is_op("]") {
                    break;
                } else {
                    self.err_expected("',' or ']'", &self.peek().clone());
                    break;
                }
            }
        }
        self.expect_operator("]");
        Expr::Array(items, self.current_span())
    }

    fn parse_dict(&mut self) -> Expr {
        self.advance();
        let mut entries = Vec::new();
        self.skip_newlines();
        if !self.is_op("}") {
            loop {
                let key = self.parse_expr();
                self.expect_operator(":");
                let val = self.parse_expr();
                entries.push((key, val));
                if self.is_op(",") {
                    self.advance();
                } else if self.is_op("}") {
                    break;
                } else {
                    self.err_expected("',' or '}'", &self.peek().clone());
                    break;
                }
            }
        }
        self.expect_operator("}");
        Expr::Dict(entries, self.current_span())
    }

    fn parse_match_expr(&mut self) -> Expr {
        self.advance();
        let expr = self.parse_expr();
        let branches = self.parse_match_body();
        Expr::MatchExpr {
            expr: Box::new(expr),
            branches,
            span: self.current_span(),
        }
    }

    fn parse_if_expr(&mut self) -> Expr {
        self.advance();
        let cond = self.parse_expr();
        let then_block = match self.parse_block() {
            Stmt::Block(b) => b,
            _ => {
                self.error("expected block after if condition".to_string());
                Block { stmts: vec![] }
            }
        };

        let else_block = if self.is_kw("else") {
            self.advance();
            match self.parse_block() {
                Stmt::Block(b) => Some(b),
                _ => {
                    self.error("expected block after else".to_string());
                    None
                }
            }
        } else {
            None
        };

        Expr::IfExpr {
            cond: Box::new(cond),
            then_block,
            else_block,
            span: self.current_span(),
        }
    }

    fn parse_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        if self.is_op(")") {
            self.advance();
            return args;
        }

        loop {
            args.push(self.parse_expr());
            if self.is_op(",") {
                self.advance();
            } else if self.is_op(")") {
                self.advance();
                break;
            } else {
                self.err_expected("',' or ')'", &self.peek().clone());
                break;
            }
        }
        args
    }

    // 鈹€鈹€ Pratt Helpers 鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€鈹€

    fn is_compound_assign(&self) -> bool {
        matches!(self.peek(), Token::Operator(s) if matches!(s.as_str(), "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^="))
    }

    fn parse_compound_assign(&mut self, left: Expr, op_bp: u8) -> Expr {
        let op_str = match self.advance() {
            Token::Operator(s) => s.clone(),
            _ => unreachable!(),
        };
        let right = self.parse_expr_bp(op_bp);
        let binop = match op_str.as_str() {
            "+=" => BinOp::Add, "-=" => BinOp::Sub, "*=" => BinOp::Mul,
            "/=" => BinOp::Div, "%=" => BinOp::Mod, "&=" => BinOp::BitAnd,
            "|=" => BinOp::BitOr, "^=" => BinOp::BitXor,
            _ => unreachable!(),
        };
        Expr::Assign {
            target: Box::new(left.clone()),
            value: Box::new(Expr::Binary { op: binop, left: Box::new(left), right: Box::new(right), span: self.current_span() }),
            span: self.current_span(),
        }
    }

    fn infix_bp(&self) -> u8 {
        match self.peek() {
            Token::Operator(s) => match s.as_str() {
                "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" => 10,
                "||" => 20,
                "|" => 25,
                "^" => 27,
                "&" => 29,
                "<<" | ">>" => 35,
                "&&" => 30,
                "==" | "!=" => 40,
                "<" | ">" | "<=" | ">=" => 50,
                ".." => 55,
                "??" => 60,
                "+" | "-" => 70,
                "*" | "/" | "%" => 80,
                "." | "::" | "(" | "[" | "++" | "--" => 90,
                _ => 0,
            },
            Token::Keyword(s) if s == "as" || s == "as!" => 65,
            _ => 0,
        }
    }
}

fn token_desc(token: &Token) -> String {
    match token {
        Token::Number(n) => format!("number {:?}", n),
        Token::String(s) => format!("\"{}\"", s),
        Token::Identifier(s) => format!("identifier '{}'", s),
        Token::Keyword(s) => format!("keyword '{}'", s),
        Token::Operator(s) => format!("'{}'", s),
        Token::NewLine => "newline".to_string(),
        Token::Error(s) => format!("error: {}", s),
        Token::Eof => "end of file".to_string(),
    }
}
