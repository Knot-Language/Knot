use crate::parser::ast::*;
use crate::ir::tac::*;
use std::collections::HashMap;

pub struct Lower {
    reg_counter: Reg,
    label_counter: usize,
    vars: HashMap<String, Reg>,
    var_types: HashMap<String, Type>,
    func: Function,
    temp_funcs: Vec<Function>,
    classes: Vec<ClassIr>,
    enums: Vec<EnumIr>,
    extern_funcs: Vec<ExternFuncIr>,
    extern_classes: Vec<ExternClassIr>,
    source_files: Vec<String>,
    break_labels: Vec<Label>,
    continue_labels: Vec<Label>,
    catch_label: Option<Label>,
    current_ret_ty: Option<Type>,
    current_class: Option<String>,
    string_counter: usize,
    strings: Vec<(String, String)>,
    func_params: HashMap<String, Vec<Param>>,
    all_class_members: HashMap<String, Vec<ClassMember>>,
    generic_funcs: HashMap<String, (Stmt, Vec<String>)>,
    monomorphized_funcs: HashMap<String, bool>,
    enum_variants: HashMap<String, Vec<String>>,
    last_lambda_name: Option<String>,
    lambda_bindings: HashMap<String, String>,
    std_path: Option<String>,
}

fn resolve_std_path() -> Option<String> {
    if let Ok(val) = std::env::var("KNOT_STD") {
        let p = std::path::Path::new(&val);
        if p.is_dir() {
            return Some(val);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        for ancestor in exe.ancestors().take(6) {
            let candidate = ancestor.join("std");
            if candidate.is_dir() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn translate_op_name(op: &str) -> String {
    match op {
        "<<" => "shl".into(),
        ">>" => "shr".into(),
        _ => op.replace("<<", "shl").replace(">>", "shr")
                .replace("==", "eq").replace("!=", "ne")
                .replace("<=", "le").replace(">=", "ge")
                .replace("+", "plus").replace("-", "minus")
                .replace("*", "mul").replace("/", "div")
                .replace("%", "mod").replace("&", "bitand")
                .replace("|", "bitor").replace("^", "bitxor")
                .replace("<", "lt").replace(">", "gt")
                .replace(" ", "_"),
    }
}

fn binop_to_op_str(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Eq => "==",
        BinOp::Neq => "!=",
        BinOp::Lt => "<",
        BinOp::Gt => ">",
        BinOp::Le => "<=",
        BinOp::Ge => ">=",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::Range | BinOp::NullCoalesce => "",
    }
}

struct SavedContext {
    prev_func: Function,
    prev_vars: HashMap<String, Reg>,
    prev_var_types: HashMap<String, Type>,
    prev_class: Option<String>,
    prev_ret: Option<Type>,
}

impl Lower {
    pub fn lower(program: &[Stmt]) -> TacProgram {
        let mut l = Lower {
            reg_counter: 0,
            label_counter: 0,
            vars: HashMap::new(),
            var_types: HashMap::new(),
            func: Function { name: String::new(), params: 0, insts: Vec::new(), ret_ty: None },
            temp_funcs: Vec::new(),
            classes: Vec::new(),
            enums: Vec::new(),
            extern_funcs: Vec::new(),
            extern_classes: Vec::new(),
            source_files: Vec::new(),
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
            catch_label: None,
            current_ret_ty: None,
            current_class: None,
            string_counter: 0,
            strings: Vec::new(),
            func_params: HashMap::new(),
            all_class_members: HashMap::new(),
            generic_funcs: HashMap::new(),
            monomorphized_funcs: HashMap::new(),
            enum_variants: HashMap::new(),
            last_lambda_name: None,
            lambda_bindings: HashMap::new(),
            std_path: resolve_std_path(),
        };

        // First pass: collect func params and class members
        for stmt in program {
            l.collect_info(stmt);
        }

        // Second pass: lower everything
        for stmt in program {
            l.lower_top_level(stmt);
        }

        TacProgram {
            functions: l.temp_funcs,
            classes: l.classes,
            enums: l.enums,
            extern_funcs: l.extern_funcs,
            extern_classes: l.extern_classes,
            strings: l.strings,
            source_files: l.source_files,
        }
    }

    fn collect_info(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FuncDef { name, params, generics, .. } => {
                self.func_params.insert(name.clone(), params.clone());
                if !generics.is_empty() {
                    self.generic_funcs.insert(name.clone(), (stmt.clone(), generics.clone()));
                }
            }
            Stmt::ClassDef { name, members, .. } => {
                self.all_class_members.insert(name.clone(), members.clone());
                for m in members {
                    match m {
                        ClassMember::New { params, .. } => {
                            self.func_params.insert(format!("{}__new", name), params.clone());
                        }
                        ClassMember::Operator { op, params, .. } => {
                            let op_name = translate_op_name(op);
                            self.func_params.insert(format!("{}__op_{}", name, op_name), params.clone());
                        }
                        ClassMember::Method { name: mname, params, .. } => {
                            self.func_params.insert(format!("{}__{}", name, mname), params.clone());
                        }
                        _ => {}
                    }
                }
            }
            Stmt::EnumDef { name, variants, .. } => {
                self.enum_variants.insert(name.clone(), variants.clone());
            }
            _ => {}
        }
    }

    // ── Mixin resolution ──────────────────────────────────

    fn resolve_mixins(&self, _class_name: &str, mixins: &[String], own_members: &[ClassMember]) -> Vec<ClassMember> {
        let mut result: Vec<ClassMember> = Vec::new();
        let own_names: Vec<String> = own_members.iter().filter_map(|m| member_name(m)).collect();

        for mixin_name in mixins {
            if let Some(source_members) = self.all_class_members.get(mixin_name) {
                for m in source_members {
                    if let Some(mn) = member_name(m) {
                        if !own_names.contains(&mn) && !result.iter().any(|r| member_name(r) == Some(mn.clone())) {
                            result.push(m.clone());
                        }
                    }
                }
            }
        }
        result.extend(own_members.iter().cloned());
        result
    }

    // ── Registers / Labels ─────────────────────────────────

    fn new_reg(&mut self) -> Reg { let r = self.reg_counter; self.reg_counter += 1; r }
    fn new_label(&mut self, prefix: &str) -> Label { let l = format!("{}_{}", prefix, self.label_counter); self.label_counter += 1; l }
    fn emit(&mut self, inst: TacInst) { self.func.insts.push(inst); }

    // ── String literals ───────────────────────────────────

    fn string_constant(&mut self, s: &str) -> String {
        let name = format!("@.str.{}", self.string_counter);
        self.string_counter += 1;
        let escaped = escape_llvm_string(s);
        self.strings.push((name.clone(), escaped));
        name
    }

    // ── Method lowering ────────────────────────────────────

    fn lower_method(&mut self, func_name: &str, has_this: bool, params: &[Param], ret_ty: &Option<Type>, body: &Block) {
        let prev = self.save_context(func_name, params.len() + if has_this { 1 } else { 0 });
        if let Some(cn) = func_name.split("__").next() { self.current_class = Some(cn.to_string()); }
        self.current_ret_ty = ret_ty.clone();
        self.reg_counter = 0;

        let mut pi = 0;
        if has_this { let r = self.new_reg(); self.vars.insert("this".to_string(), r); self.emit(TacInst::Param { dest: r, index: pi }); pi += 1; }
        for p in params { let r = self.new_reg(); self.vars.insert(p.name.clone(), r); self.emit(TacInst::Param { dest: r, index: pi }); pi += 1; }

        self.lower_block(body);
        if ret_ty.is_none() || matches!(ret_ty, Some(Type::Base(BaseType::Void))) { self.emit(TacInst::Ret(None)); }
        self.restore_context(prev);
    }

    fn lower_constructor(&mut self, func_name: &str, class_name: &str, params: &[Param], body: &Block) {
        let prev = self.save_context(func_name, params.len());
        self.current_class = Some(class_name.to_string());
        self.current_ret_ty = None;
        self.reg_counter = 0;

        for (i, p) in params.iter().enumerate() { let r = self.new_reg(); self.vars.insert(p.name.clone(), r); self.emit(TacInst::Param { dest: r, index: i }); }
        let this_reg = self.new_reg(); self.vars.insert("this".to_string(), this_reg);
        self.emit(TacInst::Alloc { dest: this_reg, ty: Type::Named(class_name.to_string()) });
        self.lower_block(body);
        self.emit(TacInst::Ret(Some(Operand::Reg(this_reg))));
        self.restore_context(prev);
    }

    fn lower_destructor(&mut self, func_name: &str, class_name: &str, body: &Block) {
        let prev = self.save_context(func_name, 1);
        self.current_class = Some(class_name.to_string());
        self.current_ret_ty = None;
        self.reg_counter = 0;
        let this_reg = self.new_reg(); self.vars.insert("this".to_string(), this_reg); self.emit(TacInst::Param { dest: this_reg, index: 0 });
        self.lower_block(body);
        self.emit(TacInst::Free { src: this_reg });
        self.emit(TacInst::Ret(None));
        self.restore_context(prev);
    }

    fn save_context(&mut self, fname: &str, nparams: usize) -> SavedContext {
        let prev_func = std::mem::replace(&mut self.func, Function { name: fname.to_string(), params: nparams, insts: Vec::new(), ret_ty: None });
        let prev_vars = std::mem::take(&mut self.vars);
        let prev_var_types = std::mem::take(&mut self.var_types);
        let prev_class = self.current_class.clone();
        let prev_ret = self.current_ret_ty.clone();
        SavedContext { prev_func, prev_vars, prev_var_types, prev_class, prev_ret }
    }

    fn restore_context(&mut self, ctx: SavedContext) {
        let mut finished = std::mem::replace(&mut self.func, ctx.prev_func);
        finished.ret_ty = self.current_ret_ty.clone();
        self.temp_funcs.push(finished);
        self.vars = ctx.prev_vars;
        self.var_types = ctx.prev_var_types;
        self.current_class = ctx.prev_class;
        self.current_ret_ty = ctx.prev_ret;
    }

    // ── Top level ──────────────────────────────────────────

    fn lower_top_level(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FuncDef { name, generics, params, ret_ty, body, .. } => {
                if generics.is_empty() {
                    let prev = self.save_context(name, params.len());
                    self.current_ret_ty = ret_ty.clone();
                    self.reg_counter = 0;
                    for (i, p) in params.iter().enumerate() {
                        let r = self.new_reg(); self.vars.insert(p.name.clone(), r);
                        self.emit(TacInst::Param { dest: r, index: i });
                    }
                    self.lower_block(body);
                    if ret_ty.is_none() || matches!(ret_ty, Some(Type::Base(BaseType::Void))) { self.emit(TacInst::Ret(None)); }
                    self.restore_context(prev);
                }
                // Generic funcs are stored in collect_info, monomorphized on first call
            }
            Stmt::ClassDef { name, mixins, abstract_class, members, .. } => {
                let mixin_names: Vec<String> = mixins.iter().map(|(n, _)| n.clone()).collect();
                let resolved = self.resolve_mixins(name, &mixin_names, members);
                let mut class = ClassIr { name: name.clone(), generics: vec![], mixins: mixin_names, abstract_class: *abstract_class, fields: Vec::new() };
                let prev_class = self.current_class.clone();
                self.current_class = Some(name.clone());

                for member in &resolved {
                    match member {
                        ClassMember::Field { name: fname, ty, private, .. } => {
                            class.fields.push(FieldIr { name: fname.clone(), ty: ty.clone().unwrap_or(Type::Base(BaseType::Void)), default: None, private: *private });
                        }
                        ClassMember::Method { name: mname, params, ret_ty, body, .. } => {
                            self.lower_method(&format!("{}__{}", name, mname), true, params, ret_ty, body);
                        }
                        ClassMember::StaticMethod { name: mname, params, ret_ty, body, .. } => {
                            self.lower_method(&format!("{}_static__{}", name, mname), false, params, ret_ty, body);
                        }
                        ClassMember::Operator { op, params, ret_ty, body, .. } => {
                            let op_name = translate_op_name(&op);
                            self.lower_method(&format!("{}__op_{}", name, op_name), true, params, ret_ty, body);
                        }
                        ClassMember::Wrap { name: wname, params, body } => {
                            self.lower_method(&format!("{}__wrap_{}", name, wname), true, params, &None, body);
                        }
                        ClassMember::New { params, body, .. } => {
                            self.lower_constructor(&format!("{}__new", name), name, params, body);
                        }
                        ClassMember::Delete { body, .. } => {
                            self.lower_destructor(&format!("{}__delete", name), name, body);
                        }
                        ClassMember::AbstractMethod { .. } | ClassMember::Mixin(_) => {}
                    }
                }
                self.current_class = prev_class;
                self.classes.push(class);
            }
            Stmt::EnumDef { name, generics, variants } => {
                self.enums.push(EnumIr { name: name.clone(), generics: generics.clone(), variants: variants.clone() });
            }
            Stmt::Import { path, .. } => {
                self.lower_import(path);
            }
            Stmt::ExternFunc { name, params, ret_ty } => {
                let param_tys: Vec<Type> = params.iter().map(|p| p.ty.clone().unwrap_or(Type::Base(BaseType::I32))).collect();
                self.extern_funcs.push(ExternFuncIr { name: name.clone(), param_tys, ret_ty: ret_ty.clone() });
            }
            Stmt::ExternClass { name, fields } => {
                self.extern_classes.push(ExternClassIr { name: name.clone(), fields: fields.clone() });
            }
            _ => {}
        }
    }

    fn lower_import(&mut self, path: &str) {
        if path.contains("..") {
            eprintln!("warning: import path '{}' contains '..', skipping (path traversal blocked)", path);
            return;
        }
        let full_path = if let Some(rest) = path.strip_prefix("std/") {
            match &self.std_path {
                Some(std_dir) => format!("{}/{}", std_dir, rest),
                None => {
                    eprintln!("warning: std library not found (set KNOT_STD env var) — cannot resolve import '{}'", path);
                    return;
                }
            }
        } else {
            path.to_string()
        };
        let canon = if full_path.contains('.') || full_path.contains('/') || full_path.contains('\\') {
            match std::fs::canonicalize(&full_path) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(e) => {
                    eprintln!("warning: cannot resolve import '{}': {}", full_path, e);
                    return;
                }
            }
        } else {
            let file_with_ext = format!("{}.knot", full_path);
            match std::fs::canonicalize(&file_with_ext) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => {
                    eprintln!("warning: cannot resolve bare import '{}' (tried '{}.knot')", full_path, full_path);
                    return;
                }
            }
        };
        // Auto-detect companion .c file (same path, .c extension)
        let c_file = canon.replace(".knot", ".c");
        if std::path::Path::new(&c_file).exists() && !self.source_files.contains(&c_file) {
            self.source_files.push(c_file);
        }
        match std::fs::read_to_string(&canon) {
            Ok(source) => {
                let mut parser = crate::parser::Parser::new(&source);
                let stmts = parser.parse_program();
                for stmt in &stmts {
                    self.lower_top_level(stmt);
                }
            }
            Err(e) => {
                eprintln!("warning: failed to read import '{}': {}", path, e);
            }
        }
    }

    // ── Statements ─────────────────────────────────────────

    fn lower_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => { self.lower_expr(expr); }
            Stmt::Return(expr) => {
                let val = expr.as_ref().map(|e| self.lower_expr(e)).unwrap_or(Operand::Imm(0));
                self.emit(TacInst::Ret(Some(val)));
            }
            Stmt::If { cond, then_block, else_block } => self.lower_if(cond, then_block, else_block),
            Stmt::While { cond, body } => self.lower_while(cond, body),
            Stmt::Block(block) => self.lower_block(block),
            Stmt::For { var, iter, body } => self.lower_for(var, iter, body),
            Stmt::Break(n) => {
                let levels = n.unwrap_or(1) as usize;
                if levels <= self.break_labels.len() {
                    let idx = self.break_labels.len() - levels;
                    self.emit(TacInst::Jmp(self.break_labels[idx].clone()));
                }
            }
            Stmt::Continue(n) => {
                let levels = n.unwrap_or(1) as usize;
                if levels <= self.continue_labels.len() {
                    let idx = self.continue_labels.len() - levels;
                    self.emit(TacInst::Jmp(self.continue_labels[idx].clone()));
                }
            }
            Stmt::Throw(expr) => {
                let val = self.lower_expr(expr);
                let label = self.catch_label.clone().unwrap_or_else(|| "_no_handler".to_string());
                self.emit(TacInst::Throw { value: val, catch_label: label });
            }
            Stmt::TryCatch { try_block, catches } => self.lower_try_catch(try_block, catches),
            Stmt::Assert { expr, message } => {
                let cond = self.lower_expr(expr);
                let skip_label = self.new_label("assert_ok");
                self.emit(TacInst::JmpIf { cond: Operand::Not(Box::new(cond.clone())), label: skip_label.clone() });
                let msg = message.clone().unwrap_or_else(|| "assertion failed".to_string());
                let str_name = self.string_constant(&msg);
                let msg_reg = self.new_reg();
                self.emit(TacInst::LoadStrConst { dest: msg_reg, name: str_name });
                self.emit(TacInst::Throw { value: Operand::Reg(msg_reg), catch_label: "_assert_fail".to_string() });
                self.emit(TacInst::Label(skip_label));
            }
            Stmt::Match { expr, branches } => {
                let val = self.lower_expr(expr);
                let end_label = self.new_label("match_end");
                let mut else_label: Option<Label> = None;
                for (i, branch) in branches.iter().enumerate() {
                    let next_label = if i + 1 < branches.len() { self.new_label("match_next") } else { end_label.clone() };
                    if is_else_pattern(&branch.pattern) {
                        else_label = Some(next_label);
                        break;
                    }
                    let pat = self.lower_expr(&branch.pattern);
                    let cmp = self.new_reg();
                    self.emit(TacInst::CmpEq { dest: cmp, lhs: val.clone(), rhs: pat });
                    self.emit(TacInst::JmpIf { cond: Operand::Not(Box::new(Operand::Reg(cmp))), label: next_label.clone() });
                    self.lower_block(&branch.body);
                    self.emit(TacInst::Jmp(end_label.clone()));
                    self.emit(TacInst::Label(next_label));
                }
                if let Some(el) = else_label {
                    self.emit(TacInst::Label(el));
                    if let Some(else_branch) = branches.iter().find(|b| is_else_pattern(&b.pattern)) {
                        self.lower_block(&else_branch.body);
                    }
                }
                self.emit(TacInst::Label(end_label));
            }
            _ => {}
        }
    }

    fn lower_block(&mut self, block: &Block) {
        for stmt in &block.stmts { self.lower_stmt(stmt); }
    }

    fn lower_if(&mut self, cond: &Expr, then_block: &Block, else_block: &Option<Box<Stmt>>) {
        let cond_val = self.lower_expr(cond);
        let else_label = self.new_label("else");
        let end_label = self.new_label("endif");
        self.emit(TacInst::JmpIf { cond: Operand::Not(Box::new(cond_val)), label: else_label.clone() });
        self.lower_block(then_block);
        self.emit(TacInst::Jmp(end_label.clone()));
        self.emit(TacInst::Label(else_label));
        if let Some(es) = else_block { self.lower_stmt(es); }
        self.emit(TacInst::Label(end_label));
    }

    fn lower_while(&mut self, cond: &Expr, body: &Block) {
        let loop_label = self.new_label("loop");
        let cont_label = self.new_label("loop_cont");
        let end_label = self.new_label("endloop");
        self.break_labels.push(end_label.clone());
        self.continue_labels.push(cont_label.clone());
        self.emit(TacInst::Label(loop_label.clone()));
        let cond_val = self.lower_expr(cond);
        self.emit(TacInst::JmpIf { cond: Operand::Not(Box::new(cond_val)), label: end_label.clone() });
        self.lower_block(body);
        self.emit(TacInst::Label(cont_label));
        self.emit(TacInst::Jmp(loop_label));
        self.emit(TacInst::Label(end_label));
        self.break_labels.pop();
        self.continue_labels.pop();
    }

    fn lower_for(&mut self, var: &str, iter: &Expr, body: &Block) {
        let loop_label = self.new_label("forloop");
        let cont_label = self.new_label("for_cont");
        let end_label = self.new_label("forend");
        self.break_labels.push(end_label.clone());
        self.continue_labels.push(cont_label.clone());

        match iter {
            Expr::Binary { op: BinOp::Range, left, right, .. } => {
                let start = self.lower_expr(left);
                let end = self.lower_expr(right);
                let ir = self.new_reg(); self.emit(TacInst::Mov { dest: ir, src: start });
                let mr = self.new_reg(); self.emit(TacInst::Mov { dest: mr, src: end });
                self.lower_for_loop(var, &loop_label, &cont_label, &end_label, ir, mr, body, None);
            }
            Expr::Array(items, _) => {
                let arr = self.new_reg();
                let count = Operand::Imm(items.len() as i64);
                self.emit(TacInst::AllocArray { dest: arr, count });
                for (i, item) in items.iter().enumerate() {
                    let val = self.lower_expr(item);
                    let ptr = self.new_reg();
                    self.emit(TacInst::GetElemPtr { dest: ptr, obj: arr, index: Operand::Imm(i as i64) });
                    self.emit(TacInst::Store { addr: ptr, src: val });
                }
                let ir = self.new_reg(); self.emit(TacInst::Mov { dest: ir, src: Operand::Imm(0) });
                let mr = self.new_reg(); self.emit(TacInst::Mov { dest: mr, src: Operand::Imm(items.len() as i64) });
                self.lower_for_loop(var, &loop_label, &cont_label, &end_label, ir, mr, body, Some(arr));
            }
            _ => {
                let val = self.lower_expr(iter);
                let ir = self.new_reg(); self.emit(TacInst::Mov { dest: ir, src: Operand::Imm(0) });
                let mr = self.new_reg(); self.emit(TacInst::Mov { dest: mr, src: val });
                self.lower_for_loop(var, &loop_label, &cont_label, &end_label, ir, mr, body, None);
            }
        }
        self.break_labels.pop();
        self.continue_labels.pop();
    }

    fn lower_for_loop(&mut self, var: &str, loop_label: &Label, cont_label: &Label, end_label: &Label, index_reg: Reg, max_reg: Reg, body: &Block, arr_reg: Option<Reg>) {
        self.emit(TacInst::Label(loop_label.clone()));
        let cond_reg = self.new_reg();
        self.emit(TacInst::CmpGe { dest: cond_reg, lhs: Operand::Reg(index_reg), rhs: Operand::Reg(max_reg) });
        self.emit(TacInst::JmpIf { cond: Operand::Reg(cond_reg), label: end_label.clone() });

        let var_reg = self.new_reg();
        if let Some(arr) = arr_reg {
            let elem_ptr = self.new_reg();
            self.emit(TacInst::GetElemPtr { dest: elem_ptr, obj: arr, index: Operand::Reg(index_reg) });
            self.emit(TacInst::Load { dest: var_reg, addr: elem_ptr });
        } else {
            self.emit(TacInst::Mov { dest: var_reg, src: Operand::Reg(index_reg) });
        }
        self.vars.insert(var.to_string(), var_reg);
        self.lower_block(body);
        self.emit(TacInst::Label(cont_label.clone()));
        let one = self.new_reg(); self.emit(TacInst::Mov { dest: one, src: Operand::Imm(1) });
        self.emit(TacInst::Add { dest: index_reg, lhs: Operand::Reg(index_reg), rhs: Operand::Reg(one) });
        self.emit(TacInst::Jmp(loop_label.clone()));
        self.emit(TacInst::Label(end_label.clone()));
    }

    fn lower_try_catch(&mut self, try_block: &Block, catches: &[CatchClause]) {
        let first_catch = self.new_label("catch");
        let end_label = self.new_label("try_end");
        let prev_catch = self.catch_label.clone();
        self.catch_label = Some(first_catch.clone());

        self.lower_block(try_block);
        self.catch_label = prev_catch;
        self.emit(TacInst::Jmp(end_label.clone()));

        let mut next_labels: Vec<Label> = Vec::new();
        for i in 0..catches.len() {
            let lbl = if i == 0 { first_catch.clone() } else { self.new_label("catch") };
            next_labels.push(lbl);
        }
        next_labels.push(end_label.clone());

        for (i, catch) in catches.iter().enumerate() {
            let exc_reg = self.new_reg();
            self.emit(TacInst::Label(next_labels[i].clone()));
            self.emit(TacInst::CatchEntry(exc_reg));

            if let Some(var) = &catch.var {
                self.vars.insert(var.clone(), exc_reg);
            }
            self.lower_block(&catch.body);
            self.emit(TacInst::Jmp(end_label.clone()));
        }

        self.emit(TacInst::Label(end_label));
    }

    // ── Expressions ────────────────────────────────────────

    fn lower_expr(&mut self, expr: &Expr) -> Operand {
        match expr {
            Expr::Int(v, _) => Operand::Imm(*v),
            Expr::Float(v, _) => Operand::F64(*v),
            Expr::Bool(b, _) => Operand::Bool(*b),
            Expr::Null(_) => Operand::NullSentinel,
            Expr::String(s, _) => {
                let name = self.string_constant(s);
                let r = self.new_reg();
                self.emit(TacInst::LoadStrConst { dest: r, name });
                self.var_types.insert(format!("_reg_{}", r), Type::Array(Box::new(Type::Base(BaseType::Char))));
                Operand::Reg(r)
            }
            Expr::Char(v, _) => Operand::Imm(*v as i64),
            Expr::Ident(name, _) => {
                if let Some(&reg) = self.vars.get(name) { Operand::Reg(reg) }
                else { let r = self.new_reg(); self.vars.insert(name.clone(), r); Operand::Reg(r) }
            }
            Expr::Binary { op, left, right, .. } => self.lower_binary(op, left, right),
            Expr::Unary { op, expr, .. } => self.lower_unary(op, expr),
            Expr::Call { callee, args, .. } => self.lower_call(callee, args),
            Expr::Assign { target, value, .. } => self.lower_assign(target, value),
            Expr::Access { obj, field, .. } => self.lower_access(obj, field),
            Expr::PostfixOp { op, target, .. } => {
                let old_val = self.lower_expr(target);
                let old_reg = self.new_reg(); self.emit(TacInst::Mov { dest: old_reg, src: old_val.clone() });
                let one = Operand::Imm(1);
                let dest = self.new_reg();
                let inst = match op { BinOp::Add => TacInst::Add { dest, lhs: old_val, rhs: one }, BinOp::Sub => TacInst::Sub { dest, lhs: old_val, rhs: one }, _ => unreachable!() };
                self.emit(inst);
                let target_reg = match target.as_ref() { Expr::Ident(name, _) => *self.vars.get(name).unwrap_or(&0), _ => dest };
                self.emit(TacInst::Mov { dest: target_reg, src: Operand::Reg(dest) });
                Operand::Reg(old_reg)
            }
            Expr::Cast { expr, ty, forced: _, .. } => {
                let val = self.lower_expr(expr);
                let dest = self.new_reg();
                let target_ty = ty;
                match (self.infer_type_of(&val), target_ty) {
                    (Type::Base(lhs), Type::Base(rhs)) => {
                        let lhs_is_float = matches!(lhs, BaseType::F32 | BaseType::F64);
                        let rhs_is_float = matches!(rhs, BaseType::F32 | BaseType::F64);
                        if lhs_is_float && !rhs_is_float {
                            self.emit(TacInst::FloatToInt { dest, src: val });
                        } else if !lhs_is_float && rhs_is_float {
                            self.emit(TacInst::IntToFloat { dest, src: val });
                        } else {
                            self.emit(TacInst::Mov { dest, src: val });
                        }
                    }
                    _ => {
                        self.emit(TacInst::Mov { dest, src: val });
                    }
                }
                self.var_types.insert(format!("_cast_{}", dest), target_ty.clone());
                Operand::Reg(dest)
            }
            Expr::IfExpr { cond, then_block, else_block, .. } => {
                let cond_val = self.lower_expr(cond);
                let else_label = self.new_label("ifexpr_else");
                let end_label = self.new_label("ifexpr_end");
                let result = self.new_reg();
                self.emit(TacInst::JmpIf { cond: Operand::Not(Box::new(cond_val)), label: else_label.clone() });
                let then_val = self.lower_block_expr(then_block);
                self.emit(TacInst::Mov { dest: result, src: then_val });
                self.emit(TacInst::Jmp(end_label.clone()));
                self.emit(TacInst::Label(else_label));
                if let Some(eb) = else_block {
                    let else_val = self.lower_block_expr(eb);
                    self.emit(TacInst::Mov { dest: result, src: else_val });
                }
                self.emit(TacInst::Label(end_label));
                Operand::Reg(result)
            }
            Expr::Lambda { params, body, .. } => self.lower_lambda(params, body),
            Expr::MatchExpr { expr, branches, .. } => {
                let val = self.lower_expr(expr);
                let result = self.new_reg();
                let end_label = self.new_label("matchexpr_end");
                for (i, branch) in branches.iter().enumerate() {
                    let next_label = if i + 1 < branches.len() { self.new_label("matchexpr_next") } else { end_label.clone() };
                    if is_else_pattern(&branch.pattern) { continue; }
                    let pat = self.lower_expr(&branch.pattern);
                    let cmp = self.new_reg();
                    self.emit(TacInst::CmpEq { dest: cmp, lhs: val.clone(), rhs: pat });
                    self.emit(TacInst::JmpIf { cond: Operand::Not(Box::new(Operand::Reg(cmp))), label: next_label.clone() });
                    let body_val = self.lower_block_expr(&branch.body);
                    self.emit(TacInst::Mov { dest: result, src: body_val });
                    self.emit(TacInst::Jmp(end_label.clone()));
                    self.emit(TacInst::Label(next_label));
                }
                if let Some(else_branch) = branches.iter().find(|b| is_else_pattern(&b.pattern)) {
                    let else_val = self.lower_block_expr(&else_branch.body);
                    self.emit(TacInst::Mov { dest: result, src: else_val });
                }
                self.emit(TacInst::Label(end_label));
                Operand::Reg(result)
            }
            Expr::Array(items, _) => {
                let arr = self.new_reg();
                let count = Operand::Imm(items.len() as i64);
                self.emit(TacInst::AllocArray { dest: arr, count });
                for (i, item) in items.iter().enumerate() {
                    let val = self.lower_expr(item);
                    let ptr = self.new_reg();
                    self.emit(TacInst::GetElemPtr { dest: ptr, obj: arr, index: Operand::Imm(i as i64) });
                    self.emit(TacInst::Store { addr: ptr, src: val });
                }
                Operand::Reg(arr)
            }
            Expr::Dict(entries, _) => {
                let map = self.new_reg();
                let _count = Operand::Imm(entries.len() as i64);
                self.emit(TacInst::AllocArray { dest: map, count: Operand::Imm((entries.len() * 2) as i64) });
                for (i, (k, v)) in entries.iter().enumerate() {
                    let key_val = self.lower_expr(k);
                    let val_val = self.lower_expr(v);
                    let key_ptr = self.new_reg();
                    self.emit(TacInst::GetElemPtr { dest: key_ptr, obj: map, index: Operand::Imm((i * 2) as i64) });
                    self.emit(TacInst::Store { addr: key_ptr, src: key_val });
                    let val_ptr = self.new_reg();
                    self.emit(TacInst::GetElemPtr { dest: val_ptr, obj: map, index: Operand::Imm((i * 2 + 1) as i64) });
                    self.emit(TacInst::Store { addr: val_ptr, src: val_val });
                }
                Operand::Reg(map)
            }
            Expr::Index { obj, index, .. } => {
                let obj_reg = self.lower_expr(obj);
                let idx_op = self.lower_expr(index);
                let obj_r = match &obj_reg { Operand::Reg(r) => *r, _ => 0 };
                let elem_ptr = self.new_reg();
                self.emit(TacInst::GetElemPtr { dest: elem_ptr, obj: obj_r, index: idx_op });
                let result = self.new_reg();
                self.emit(TacInst::Load { dest: result, addr: elem_ptr });
                Operand::Reg(result)
            }
        }
    }

    fn lower_block_expr(&mut self, block: &Block) -> Operand {
        let mut last = Operand::Imm(0);
        for stmt in &block.stmts {
            if let Stmt::Expr(e) = stmt { last = self.lower_expr(e); }
            else if let Stmt::Return(Some(e)) = stmt { return self.lower_expr(e); }
            else { self.lower_stmt(stmt); }
        }
        last
    }

    fn lower_binary(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Operand {
        if let Some(result) = self.try_operator_call(op, left, right) {
            return result;
        }

        let lhs = self.lower_expr(left);
        let rhs = self.lower_expr(right);

        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod
            | BinOp::Shl | BinOp::Shr | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
                let dest = self.new_reg();
                let inst = match op {
                    BinOp::Add => TacInst::Add { dest, lhs, rhs },
                    BinOp::Sub => TacInst::Sub { dest, lhs, rhs },
                    BinOp::Mul => TacInst::Mul { dest, lhs, rhs },
                    BinOp::Div => TacInst::Div { dest, lhs, rhs },
                    BinOp::Mod => TacInst::Mod { dest, lhs, rhs },
                    BinOp::Shl => TacInst::Shl { dest, lhs, rhs },
                    BinOp::Shr => TacInst::Shr { dest, lhs, rhs },
                    BinOp::BitAnd => TacInst::BitAnd { dest, lhs, rhs },
                    BinOp::BitOr => TacInst::BitOr { dest, lhs, rhs },
                    BinOp::BitXor => TacInst::BitXor { dest, lhs, rhs },
                    _ => unreachable!(),
                };
                self.emit(inst);
                Operand::Reg(dest)
            }
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                let dest = self.new_reg();
                let inst = match op {
                    BinOp::Eq => TacInst::CmpEq { dest, lhs, rhs },
                    BinOp::Neq => TacInst::CmpNe { dest, lhs, rhs },
                    BinOp::Lt => TacInst::CmpLt { dest, lhs, rhs },
                    BinOp::Gt => TacInst::CmpGt { dest, lhs, rhs },
                    BinOp::Le => TacInst::CmpLe { dest, lhs, rhs },
                    BinOp::Ge => TacInst::CmpGe { dest, lhs, rhs },
                    _ => unreachable!(),
                };
                self.emit(inst);
                Operand::Reg(dest)
            }
            BinOp::And | BinOp::Or => {
                let dest = self.new_reg();
                let end_label = self.new_label("logic_end");
                self.emit(TacInst::Mov { dest, src: lhs.clone() });
                if matches!(op, BinOp::And) {
                    self.emit(TacInst::JmpIf { cond: Operand::Not(Box::new(lhs)), label: end_label.clone() });
                } else {
                    self.emit(TacInst::JmpIf { cond: lhs, label: end_label.clone() });
                }
                self.emit(TacInst::Mov { dest, src: rhs });
                self.emit(TacInst::Label(end_label));
                Operand::Reg(dest)
            }
            BinOp::NullCoalesce => {
                let dest = self.new_reg();
                let end_label = self.new_label("coalesce_end");
                self.emit(TacInst::Mov { dest, src: lhs.clone() });
                let cmp = self.new_reg();
                self.emit(TacInst::CmpEq { dest: cmp, lhs: lhs.clone(), rhs: Operand::NullSentinel });
                self.emit(TacInst::JmpIf { cond: Operand::Not(Box::new(Operand::Reg(cmp))), label: end_label.clone() });
                self.emit(TacInst::Mov { dest, src: rhs });
                self.emit(TacInst::Label(end_label));
                Operand::Reg(dest)
            }
            BinOp::Range => {
                let dest = self.new_reg();
                self.emit(TacInst::Mov { dest, src: rhs });
                Operand::Reg(dest)
            }
        }
    }

    fn try_operator_call(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Option<Operand> {
        let op_str = binop_to_op_str(op);
        if op_str.is_empty() { return None; }
        let class_name = match left {
            Expr::Ident(name, _) => match self.var_types.get(name)? {
                Type::Named(cn) => cn.clone(),
                _ => return None,
            },
            _ => return None,
        };
        let op_name = translate_op_name(op_str);
        let method_name = format!("{}__op_{}", class_name, op_name);
        self.func_params.get(&method_name)?;
        let lhs = self.lower_expr(left);
        let rhs = self.lower_expr(right);
        let dest = self.new_reg();
        self.emit(TacInst::Call { dest: Some(dest), name: method_name, args: vec![lhs, rhs] });
        Some(Operand::Reg(dest))
    }

    fn constructor_class_name(&self, expr: &Expr) -> Option<String> {
        if let Expr::Call { callee, .. } = expr {
            if let Expr::Access { obj, field, .. } = callee.as_ref() {
                if field == "new" {
                    if let Expr::Ident(name, _) = obj.as_ref() {
                        if self.all_class_members.contains_key(name) {
                            return Some(name.clone());
                        }
                    }
                }
            }
        }
        None
    }

    fn lower_unary(&mut self, op: &UnaryOp, expr: &Expr) -> Operand {
        let src = self.lower_expr(expr);
        let dest = self.new_reg();
        match op {
            UnaryOp::Neg => self.emit(TacInst::Neg { dest, src }),
            UnaryOp::Not => self.emit(TacInst::Not { dest, src }),
            UnaryOp::BitNot => self.emit(TacInst::Not { dest, src }),
        }
        Operand::Reg(dest)
    }

    fn lower_call(&mut self, callee: &Expr, args: &[Expr]) -> Operand {
        let (name, this_arg) = match callee {
            Expr::Ident(s, _) => {
                let resolved = if let Some(lambda_name) = self.lambda_bindings.get(s) {
                    lambda_name.clone()
                } else {
                    s.clone()
                };
                (resolved, None)
            }
            Expr::Access { obj, field, .. } => {
                if let Expr::Ident(class_name, _) = obj.as_ref() {
                    let method_name = format!("{}__{}", class_name, field);
                    let this = if field == "new" { None } else { Some(self.lower_expr(obj)) };
                    (method_name, this)
                } else {
                    let method_name = format!("__{}", field);
                    let obj_val = self.lower_expr(obj);
                    (method_name, Some(obj_val))
                }
            }
            _ => return Operand::Imm(0),
        };

        // Check monomorphization: if this is a generic function call
        let resolved_name = self.resolve_generic_call(&name, args);

        let mut arg_ops = Vec::new();
        if let Some(this) = this_arg { arg_ops.push(this); }

        let params_clone = self.func_params.get(&resolved_name).cloned();
        if let Some(func_params) = params_clone {
            for (i, param) in func_params.iter().enumerate() {
                if i < args.len() {
                    arg_ops.push(self.lower_expr(&args[i]));
                } else if let Some(default) = &param.default {
                    arg_ops.push(self.lower_expr(default));
                } else {
                    arg_ops.push(Operand::Imm(0));
                }
            }
        } else {
            for arg in args { arg_ops.push(self.lower_expr(arg)); }
        }

        let dest = self.new_reg();
        self.emit(TacInst::Call { dest: Some(dest), name: resolved_name, args: arg_ops });
        Operand::Reg(dest)
    }

    fn resolve_generic_call(&mut self, name: &str, args: &[Expr]) -> String {
        let generics: Vec<String> = match self.generic_funcs.get(name) {
            Some((_, g)) => g.clone(),
            None => return name.to_string(),
        };
        let inferred = infer_type_from_args(args, &generics);
        if let Some(ty) = inferred {
            let mono_name = format!("{}_{}", name, sanitize_type_name(&ty));
            if !self.monomorphized_funcs.contains_key(&mono_name) {
                self.monomorphize(name, &mono_name, &generics, &ty);
            }
            return mono_name;
        }
        name.to_string()
    }

    fn monomorphize(&mut self, generic_name: &str, mono_name: &str, generics: &[String], concrete_ty: &Type) {
        self.monomorphized_funcs.insert(mono_name.to_string(), true);

        if let Some((stmt, _)) = self.generic_funcs.get(generic_name) {
            let mut mono_stmt = stmt.clone();
            for g in generics {
                substitute_type_param(&mut mono_stmt, g, concrete_ty);
            }
            if let Stmt::FuncDef { name: _, params, ret_ty, body, .. } = &mono_stmt {
                let mono_name_copy = mono_name.to_string();
                let params_copy = params.clone();
                let ret_ty_copy = ret_ty.clone();
                let body_copy = body.clone();
                self.func_params.insert(mono_name_copy.clone(), params_copy.clone());
                let prev = self.save_context(&mono_name_copy, params_copy.len());
                self.current_ret_ty = ret_ty_copy.clone();
                self.reg_counter = 0;
                for (i, p) in params_copy.iter().enumerate() {
                    let r = self.new_reg(); self.vars.insert(p.name.clone(), r);
                    self.emit(TacInst::Param { dest: r, index: i });
                }
                self.lower_block(&body_copy);
                if ret_ty_copy.is_none() || matches!(ret_ty_copy, Some(Type::Base(BaseType::Void))) { self.emit(TacInst::Ret(None)); }
                self.restore_context(prev);
            }
        }
    }

    fn lower_assign(&mut self, target: &Expr, value: &Expr) -> Operand {
        let val = self.lower_expr(value);

        if let Expr::Ident(name, _) = target {
            if let Some(class_name) = self.constructor_class_name(value) {
                self.var_types.insert(name.clone(), Type::Named(class_name));
            }
        }

        let (dest_reg, _target_name) = match target {
            Expr::Ident(name, _) => {
                if matches!(value, Expr::Lambda { .. }) {
                    if let Some(lambda_name) = self.last_lambda_name.take() {
                        self.lambda_bindings.insert(name.clone(), lambda_name);
                    }
                }
                let r = if let Some(&r) = self.vars.get(name) { r }
                else { let r = self.new_reg(); self.vars.insert(name.clone(), r); r };
                (r, Some(name.clone()))
            }
            Expr::Access { obj, field, .. } => {
                let obj_reg = self.lower_expr(obj);
                let class_name = self.current_class.clone().unwrap_or_default();
                let field_ptr = self.new_reg();
                self.emit(TacInst::GetFieldPtr { dest: field_ptr, obj: self.reg_from_op(&obj_reg), class: class_name, field: field.clone() });
                self.emit(TacInst::Store { addr: field_ptr, src: val.clone() });
                return val;
            }
            _ => (self.new_reg(), None),
        };

        self.emit(TacInst::Mov { dest: dest_reg, src: val });
        Operand::Reg(dest_reg)
    }

    fn infer_type_of(&self, op: &Operand) -> Type {
        match op {
            Operand::F64(_) => Type::Base(BaseType::F64),
            Operand::Bool(_) => Type::Base(BaseType::Bool),
            Operand::Reg(r) => {
                if let Some(ty) = self.var_types.get(&format!("_cast_{}", r)) {
                    ty.clone()
                } else {
                    Type::Base(BaseType::I32)
                }
            }
            _ => Type::Base(BaseType::I32),
        }
    }

    fn lower_access(&mut self, obj: &Expr, field: &str) -> Operand {
        if let Expr::Ident(name, _) = obj {
            if let Some(variants) = self.enum_variants.get(name) {
                if let Some(idx) = variants.iter().position(|v| v == field) {
                    return Operand::Imm(idx as i64);
                }
            }
        }
        let class_name = self.current_class.clone().unwrap_or_default();
        match obj {
            Expr::Ident(name, _) if name == "this" => {
                let this_reg = self.vars.get("this").copied().unwrap_or(0);
                let field_ptr = self.new_reg();
                self.emit(TacInst::GetFieldPtr { dest: field_ptr, obj: this_reg, class: class_name, field: field.to_string() });
                let result = self.new_reg();
                self.emit(TacInst::Load { dest: result, addr: field_ptr });
                Operand::Reg(result)
            }
            Expr::Ident(var_name, _) => {
                let obj_class = self.resolve_object_class(var_name);
                let obj_reg = self.reg_from_name(var_name);
                let field_ptr = self.new_reg();
                self.emit(TacInst::GetFieldPtr { dest: field_ptr, obj: obj_reg, class: obj_class, field: field.to_string() });
                let result = self.new_reg();
                self.emit(TacInst::Load { dest: result, addr: field_ptr });
                Operand::Reg(result)
            }
            Expr::Access { obj: inner_obj, field: inner_field, .. } => {
                let inner_val = self.lower_access(inner_obj, inner_field);
                let inner_reg = self.reg_from_op(&inner_val);
                let inner_class = self.resolve_object_class_from_access(inner_obj, inner_field);
                let field_ptr = self.new_reg();
                self.emit(TacInst::GetFieldPtr { dest: field_ptr, obj: inner_reg, class: inner_class, field: field.to_string() });
                let result = self.new_reg();
                self.emit(TacInst::Load { dest: result, addr: field_ptr });
                Operand::Reg(result)
            }
            _ => {
                let obj_op = self.lower_expr(obj);
                obj_op
            }
        }
    }

    fn resolve_object_class(&self, var_name: &str) -> String {
        if let Some(ty) = self.var_types.get(var_name) {
            if let Type::Named(class) = ty {
                return class.clone();
            }
        }
        self.current_class.clone().unwrap_or_default()
    }

    fn resolve_object_class_from_access(&self, _obj: &Expr, _field: &str) -> String {
        self.current_class.clone().unwrap_or_default()
    }

    fn reg_from_name(&self, name: &str) -> Reg {
        self.vars.get(name).copied().unwrap_or(0)
    }

    fn lower_lambda(&mut self, params: &[Param], body: &Block) -> Operand {
        let captured = self.collect_captures(body, params);
        let lambda_name = format!("__lambda_{}", self.label_counter);
        self.last_lambda_name = Some(lambda_name.clone());
        let total_params = params.len() + captured.len();
        let prev = self.save_context(&lambda_name, total_params);
        self.current_ret_ty = None;
        self.reg_counter = 0;

        let mut pi = 0;
        for p in params.iter() {
            let r = self.new_reg(); self.vars.insert(p.name.clone(), r);
            self.emit(TacInst::Param { dest: r, index: pi }); pi += 1;
        }
        for (name, _) in &captured {
            let r = self.new_reg(); self.vars.insert(name.clone(), r);
            self.emit(TacInst::Param { dest: r, index: pi }); pi += 1;
        }

        let result = self.lower_block_expr(body);
        self.emit(TacInst::Ret(Some(result)));
        self.restore_context(prev);

        self.func_params.insert(lambda_name.clone(), params.to_vec());
        Operand::Imm(1)
    }

    fn collect_captures(&self, body: &Block, params: &[Param]) -> Vec<(String, Reg)> {
        let param_names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
        let local_names = self.collect_local_names(body);
        let mut captured = Vec::new();
        self.collect_captures_in_block(body, &param_names, &local_names, &mut captured);
        captured
    }

    fn collect_local_names(&self, block: &Block) -> Vec<String> {
        let mut names = Vec::new();
        for stmt in &block.stmts {
            match stmt {
                Stmt::Expr(Expr::Assign { target, .. }) => {
                    if let Expr::Ident(name, _) = target.as_ref() {
                        names.push(name.clone());
                    }
                }
                Stmt::For { var, .. } => {
                    names.push(var.clone());
                }
                Stmt::TryCatch { catches, .. } => {
                    for c in catches {
                        if let Some(v) = &c.var { names.push(v.clone()); }
                    }
                }
                Stmt::Block(b) => {
                    names.extend(self.collect_local_names(b));
                }
                _ => {}
            }
        }
        names
    }

    fn collect_captures_in_block(&self, block: &Block, param_names: &[&str], local_names: &[String], out: &mut Vec<(String, Reg)>) {
        for stmt in &block.stmts {
            self.collect_captures_in_stmt(stmt, param_names, local_names, out);
        }
    }

    fn collect_captures_in_stmt(&self, stmt: &Stmt, param_names: &[&str], local_names: &[String], out: &mut Vec<(String, Reg)>) {
        match stmt {
            Stmt::Expr(e) => self.collect_captures_in_expr(e, param_names, local_names, out),
            Stmt::Return(Some(e)) => self.collect_captures_in_expr(e, param_names, local_names, out),
            Stmt::If { cond, then_block, else_block } => {
                self.collect_captures_in_expr(cond, param_names, local_names, out);
                self.collect_captures_in_block(then_block, param_names, local_names, out);
                if let Some(es) = else_block {
                    self.collect_captures_in_stmt(es, param_names, local_names, out);
                }
            }
            Stmt::While { cond, body } => {
                self.collect_captures_in_expr(cond, param_names, local_names, out);
                self.collect_captures_in_block(body, param_names, local_names, out);
            }
            Stmt::For { iter, body, .. } => {
                self.collect_captures_in_expr(iter, param_names, local_names, out);
                self.collect_captures_in_block(body, param_names, local_names, out);
            }
            Stmt::Block(b) => self.collect_captures_in_block(b, param_names, local_names, out),
            _ => {}
        }
    }

    fn collect_captures_in_expr(&self, expr: &Expr, param_names: &[&str], local_names: &[String], out: &mut Vec<(String, Reg)>) {
        match expr {
            Expr::Ident(name, _) => {
                if !param_names.contains(&name.as_str()) && !local_names.contains(name) {
                    if let Some(&reg) = self.vars.get(name) {
                        if !out.iter().any(|(n, _)| n == name) {
                            out.push((name.clone(), reg));
                        }
                    }
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_captures_in_expr(left, param_names, local_names, out);
                self.collect_captures_in_expr(right, param_names, local_names, out);
            }
            Expr::Unary { expr: e, .. } => {
                self.collect_captures_in_expr(e, param_names, local_names, out);
            }
            Expr::Call { callee, args, .. } => {
                self.collect_captures_in_expr(callee, param_names, local_names, out);
                for a in args { self.collect_captures_in_expr(a, param_names, local_names, out); }
            }
            Expr::Index { obj, index, .. } => {
                self.collect_captures_in_expr(obj, param_names, local_names, out);
                self.collect_captures_in_expr(index, param_names, local_names, out);
            }
            Expr::Access { obj, .. } => {
                self.collect_captures_in_expr(obj, param_names, local_names, out);
            }
            Expr::Assign { target, value, .. } => {
                self.collect_captures_in_expr(value, param_names, local_names, out);
                if !matches!(target.as_ref(), Expr::Ident(..)) {
                    self.collect_captures_in_expr(target, param_names, local_names, out);
                }
            }
            Expr::IfExpr { cond, then_block, else_block, .. } => {
                self.collect_captures_in_expr(cond, param_names, local_names, out);
                self.collect_captures_in_block(then_block, param_names, local_names, out);
                if let Some(eb) = else_block {
                    self.collect_captures_in_block(eb, param_names, local_names, out);
                }
            }
            Expr::MatchExpr { expr: e, branches, .. } => {
                self.collect_captures_in_expr(e, param_names, local_names, out);
                for b in branches {
                    self.collect_captures_in_expr(&b.pattern, param_names, local_names, out);
                    self.collect_captures_in_block(&b.body, param_names, local_names, out);
                }
            }
            _ => {}
        }
    }

    fn reg_from_op(&self, op: &Operand) -> Reg {
        match op { Operand::Reg(r) => *r, _ => 0 }
    }
}

fn member_name(m: &ClassMember) -> Option<String> {
    match m {
        ClassMember::Field { name, .. } => Some(name.clone()),
        ClassMember::Method { name, .. } => Some(name.clone()),
        ClassMember::StaticMethod { name, .. } => Some(format!("static_{}", name)),
        ClassMember::Operator { op, .. } => Some(format!("op_{}", op)),
        ClassMember::Wrap { name, .. } => Some(format!("wrap_{}", name)),
        ClassMember::New { .. } => Some("new".to_string()),
        ClassMember::Delete { .. } => Some("delete".to_string()),
        ClassMember::AbstractMethod { name, .. } => Some(name.clone()),
        ClassMember::Mixin(_) => None,
    }
}

fn is_else_pattern(expr: &Expr) -> bool {
    matches!(expr, Expr::Ident(name, _) if name == "else")
}

fn escape_llvm_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'\\' => out.push_str("\\5C"),
            b'"' => out.push_str("\\22"),
            b'\n' => out.push_str("\\0A"),
            b'\r' => out.push_str("\\0D"),
            b'\t' => out.push_str("\\09"),
            0x10..=0x7E => out.push(byte as char),
            _ => out.push_str(&format!("\\{:02X}", byte)),
        }
    }
    out
}

fn infer_type_from_args(args: &[Expr], _generics: &[String]) -> Option<Type> {
    // Infer generic type from the first argument
    match args.first()? {
        Expr::Int(..) => Some(Type::Base(BaseType::I32)),
        Expr::Float(..) => Some(Type::Base(BaseType::F64)),
        Expr::String(..) => Some(Type::Array(Box::new(Type::Base(BaseType::Char)))),
        Expr::Char(..) => Some(Type::Base(BaseType::Char)),
        Expr::Bool(..) => Some(Type::Base(BaseType::Bool)),
        Expr::Null(_) => Some(Type::Base(BaseType::Null)),
        Expr::Ident(..) => Some(Type::Base(BaseType::I32)),
        Expr::Array(items, _) if !items.is_empty() => Some(Type::Array(Box::new(Type::Base(BaseType::I32)))),
        _ => Some(Type::Base(BaseType::I32)),
    }
}

fn sanitize_type_name(ty: &Type) -> String {
    match ty {
        Type::Base(b) => format!("{:?}", b),
        Type::Nullable(inner) => format!("Nullable{:?}", inner),
        Type::Named(n) => n.clone(),
        Type::Array(inner) => format!("Array_{}", sanitize_type_name(inner)),
        Type::Map(k, v) => format!("Map_{}_{}", sanitize_type_name(k), sanitize_type_name(v)),
        Type::Pointer(inner) => format!("Ptr_{}", sanitize_type_name(inner)),
    }
}

fn substitute_type_param(stmt: &mut Stmt, from: &str, to: &Type) {
    match stmt {
        Stmt::FuncDef { name, generics, params, ret_ty, body } => {
            generics.retain(|g| g != from);
            for p in params.iter_mut() {
                if let Some(ty) = &mut p.ty { substitute_type(ty, from, to); }
            }
            if let Some(ty) = ret_ty { substitute_type(ty, from, to); }
            substitute_in_block(body, from, to);
            if !name.contains('_') {
                *name = format!("{}_{}", name, sanitize_type_name(to));
            }
        }
        _ => {}
    }
}

fn substitute_in_block(block: &mut Block, from: &str, to: &Type) {
    for stmt in block.stmts.iter_mut() {
        substitute_in_stmt(stmt, from, to);
    }
}

fn substitute_in_stmt(stmt: &mut Stmt, from: &str, to: &Type) {
    match stmt {
        Stmt::Expr(e) => substitute_in_expr(e, from, to),
        Stmt::Return(Some(e)) => substitute_in_expr(e, from, to),
        Stmt::If { cond, then_block, else_block } => {
            substitute_in_expr(cond, from, to);
            substitute_in_block(then_block, from, to);
            if let Some(es) = else_block { substitute_in_stmt(es, from, to); }
        }
        Stmt::While { cond, body } => {
            substitute_in_expr(cond, from, to);
            substitute_in_block(body, from, to);
        }
        Stmt::Block(b) => substitute_in_block(b, from, to),
        _ => {}
    }
}

fn substitute_in_expr(expr: &mut Expr, from: &str, to: &Type) {
    match expr {
        Expr::Binary { left, right, .. } => {
            substitute_in_expr(left, from, to);
            substitute_in_expr(right, from, to);
        }
        Expr::Unary { expr: e, .. } => substitute_in_expr(e, from, to),
        Expr::Call { callee, args, .. } => {
            substitute_in_expr(callee, from, to);
            for a in args { substitute_in_expr(a, from, to); }
        }
        Expr::Assign { target: _, value, .. } => {
            substitute_in_expr(value, from, to);
        }
        Expr::Cast { expr: e, ty, .. } => {
            substitute_in_expr(e, from, to);
            if let Type::Named(n) = ty {
                if n == from { *ty = to.clone(); }
            }
        }
        Expr::Access { obj, .. } => { substitute_in_expr(obj, from, to); }
        Expr::Index { obj, index, .. } => {
            substitute_in_expr(obj, from, to);
            substitute_in_expr(index, from, to);
        }
        _ => {}
    }
}

fn substitute_type(ty: &mut Type, from: &str, to: &Type) {
    match ty {
        Type::Named(n) if n == from => { *ty = to.clone(); }
        Type::Nullable(inner) => substitute_type(inner, from, to),
        Type::Array(inner) => substitute_type(inner, from, to),
        Type::Map(k, v) => {
            substitute_type(k, from, to);
            substitute_type(v, from, to);
        }
        Type::Pointer(inner) => substitute_type(inner, from, to),
        _ => {}
    }
}
