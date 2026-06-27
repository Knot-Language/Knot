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
    break_labels: Vec<Label>,
    continue_labels: Vec<Label>,
    catch_label: Option<Label>,
    current_ret_ty: Option<Type>,
    current_class: Option<String>,
    string_counter: usize,
    strings: Vec<(String, String)>,
    // Track known function parameter info for default arg filling
    func_params: HashMap<String, Vec<Param>>,
    // Track all class members for mixin resolution
    all_class_members: HashMap<String, Vec<ClassMember>>,
}

impl Lower {
    pub fn lower(program: &[Stmt]) -> TacProgram {
        let mut l = Lower {
            reg_counter: 0,
            label_counter: 0,
            vars: HashMap::new(),
            var_types: HashMap::new(),
            func: Function { name: String::new(), params: 0, insts: Vec::new() },
            temp_funcs: Vec::new(),
            classes: Vec::new(),
            enums: Vec::new(),
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
            catch_label: None,
            current_ret_ty: None,
            current_class: None,
            string_counter: 0,
            strings: Vec::new(),
            func_params: HashMap::new(),
            all_class_members: HashMap::new(),
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
            strings: l.strings,
        }
    }

    fn collect_info(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FuncDef { name, params, .. } => {
                self.func_params.insert(name.clone(), params.clone());
            }
            Stmt::ClassDef { name, members, .. } => {
                self.all_class_members.insert(name.clone(), members.clone());
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
        let escaped = s.replace('\\', "\\5C").replace('\n', "\\0A").replace('"', "\\22");
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

    fn save_context(&mut self, fname: &str, nparams: usize) -> (Function, HashMap<String, Reg>, HashMap<String, Type>, Option<String>, Option<Type>) {
        let prev_func = std::mem::replace(&mut self.func, Function { name: fname.to_string(), params: nparams, insts: Vec::new() });
        let prev_vars = std::mem::take(&mut self.vars);
        let prev_var_types = std::mem::take(&mut self.var_types);
        let prev_class = self.current_class.clone();
        let prev_ret = self.current_ret_ty.clone();
        (prev_func, prev_vars, prev_var_types, prev_class, prev_ret)
    }

    fn restore_context(&mut self, ctx: (Function, HashMap<String, Reg>, HashMap<String, Type>, Option<String>, Option<Type>)) {
        let finished = std::mem::replace(&mut self.func, ctx.0);
        self.temp_funcs.push(finished);
        self.vars = ctx.1;
        self.var_types = ctx.2;
        self.current_class = ctx.3;
        self.current_ret_ty = ctx.4;
    }

    // ── Top level ──────────────────────────────────────────

    fn lower_top_level(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FuncDef { name, params, ret_ty, body, .. } => {
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
            Stmt::ClassDef { name, mixins, abstract_class, members, .. } => {
                let resolved = self.resolve_mixins(name, mixins, members);
                let mut class = ClassIr { name: name.clone(), generics: vec![], mixins: mixins.clone(), abstract_class: *abstract_class, fields: Vec::new() };
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
                            let op_name = op.replace('+', "plus").replace('-', "minus").replace('*', "mul").replace('/', "div").replace('%', "mod").replace("==", "eq").replace("!=", "ne").replace('<', "lt").replace('>', "gt").replace(" ", "_");
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
            _ => {}
        }
    }

    fn lower_import(&mut self, path: &str) {
        if let Ok(source) = std::fs::read_to_string(path) {
            let mut parser = crate::parser::Parser::new(&source);
            let stmts = parser.parse_program();
            for stmt in &stmts {
                self.lower_top_level(stmt);
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
                    if branch.pattern == Expr::Ident("else".to_string()) {
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
                    if let Some(else_branch) = branches.iter().find(|b| b.pattern == Expr::Ident("else".to_string())) {
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

        let (index_reg, max_reg) = match iter {
            Expr::Binary { op: BinOp::Range, left, right } => {
                let start = self.lower_expr(left);
                let end = self.lower_expr(right);
                let ir = self.new_reg(); self.emit(TacInst::Mov { dest: ir, src: start });
                let mr = self.new_reg(); self.emit(TacInst::Mov { dest: mr, src: end });
                (ir, mr)
            }
            _ => {
                let val = self.lower_expr(iter);
                let ir = self.new_reg(); self.emit(TacInst::Mov { dest: ir, src: Operand::Imm(0) });
                let mr = self.new_reg(); self.emit(TacInst::Mov { dest: mr, src: val });
                (ir, mr)
            }
        };

        self.emit(TacInst::Label(loop_label.clone()));
        let cond_reg = self.new_reg();
        self.emit(TacInst::CmpGe { dest: cond_reg, lhs: Operand::Reg(index_reg), rhs: Operand::Reg(max_reg) });
        self.emit(TacInst::JmpIf { cond: Operand::Reg(cond_reg), label: end_label.clone() });

        let var_reg = self.new_reg();
        self.vars.insert(var.to_string(), var_reg);
        self.emit(TacInst::Mov { dest: var_reg, src: Operand::Reg(index_reg) });
        self.lower_block(body);
        self.emit(TacInst::Label(cont_label));
        let one = self.new_reg(); self.emit(TacInst::Mov { dest: one, src: Operand::Imm(1) });
        self.emit(TacInst::Add { dest: index_reg, lhs: Operand::Reg(index_reg), rhs: Operand::Reg(one) });
        self.emit(TacInst::Jmp(loop_label));
        self.emit(TacInst::Label(end_label));
        self.break_labels.pop();
        self.continue_labels.pop();
    }

    fn lower_try_catch(&mut self, try_block: &Block, catches: &[CatchClause]) {
        let catch_label = self.new_label("catch");
        let end_label = self.new_label("try_end");
        let prev_catch = self.catch_label.clone();
        self.catch_label = Some(catch_label.clone());
        self.lower_block(try_block);
        self.catch_label = prev_catch;
        self.emit(TacInst::Jmp(end_label.clone()));
        self.emit(TacInst::Label(catch_label.clone()));
        if let Some(fc) = catches.first() {
            let exc_reg = self.new_reg();
            self.emit(TacInst::CatchEntry(exc_reg));
            if let Some(var) = &fc.var { self.vars.insert(var.clone(), exc_reg); }
            self.lower_block(&fc.body);
        }
        self.emit(TacInst::Label(end_label));
    }

    // ── Expressions ────────────────────────────────────────

    fn lower_expr(&mut self, expr: &Expr) -> Operand {
        match expr {
            Expr::Int(v) => Operand::Imm(*v),
            Expr::Float(v) => Operand::F64(*v),
            Expr::Bool(b) => Operand::Bool(*b),
            Expr::Null => Operand::Imm(0),
            Expr::String(s) => {
                let name = self.string_constant(s);
                let r = self.new_reg();
                self.emit(TacInst::LoadStrConst { dest: r, name });
                Operand::Reg(r)
            }
            Expr::Ident(name) => {
                if let Some(&reg) = self.vars.get(name) { Operand::Reg(reg) }
                else { let r = self.new_reg(); self.vars.insert(name.clone(), r); Operand::Reg(r) }
            }
            Expr::Binary { op, left, right } => self.lower_binary(op, left, right),
            Expr::Unary { op, expr } => self.lower_unary(op, expr),
            Expr::Call { callee, args } => self.lower_call(callee, args),
            Expr::Assign { target, value } => self.lower_assign(target, value),
            Expr::Access { obj, field } => self.lower_access(obj, field),
            Expr::PostfixOp { op, target } => {
                let old_val = self.lower_expr(target);
                let old_reg = self.new_reg(); self.emit(TacInst::Mov { dest: old_reg, src: old_val.clone() });
                let one = Operand::Imm(1);
                let dest = self.new_reg();
                let inst = match op { BinOp::Add => TacInst::Add { dest, lhs: old_val, rhs: one }, BinOp::Sub => TacInst::Sub { dest, lhs: old_val, rhs: one }, _ => unreachable!() };
                self.emit(inst);
                let target_reg = match target.as_ref() { Expr::Ident(name) => *self.vars.get(name).unwrap_or(&0), _ => dest };
                self.emit(TacInst::Mov { dest: target_reg, src: Operand::Reg(dest) });
                Operand::Reg(old_reg)
            }
            Expr::Cast { expr, ty: _, forced: _ } => { self.lower_expr(expr) }
            Expr::IfExpr { cond, then_block, else_block } => {
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
            Expr::Lambda { params, body } => self.lower_lambda(params, body),
            Expr::MatchExpr { expr, branches } => {
                let val = self.lower_expr(expr);
                let result = self.new_reg();
                let end_label = self.new_label("matchexpr_end");
                for (i, branch) in branches.iter().enumerate() {
                    let next_label = if i + 1 < branches.len() { self.new_label("matchexpr_next") } else { end_label.clone() };
                    if branch.pattern == Expr::Ident("else".to_string()) { continue; }
                    let pat = self.lower_expr(&branch.pattern);
                    let cmp = self.new_reg();
                    self.emit(TacInst::CmpEq { dest: cmp, lhs: val.clone(), rhs: pat });
                    self.emit(TacInst::JmpIf { cond: Operand::Not(Box::new(Operand::Reg(cmp))), label: next_label.clone() });
                    let body_val = self.lower_block_expr(&branch.body);
                    self.emit(TacInst::Mov { dest: result, src: body_val });
                    self.emit(TacInst::Jmp(end_label.clone()));
                    self.emit(TacInst::Label(next_label));
                }
                if let Some(else_branch) = branches.iter().find(|b| b.pattern == Expr::Ident("else".to_string())) {
                    let else_val = self.lower_block_expr(&else_branch.body);
                    self.emit(TacInst::Mov { dest: result, src: else_val });
                }
                self.emit(TacInst::Label(end_label));
                Operand::Reg(result)
            }
            Expr::Array(items) => {
                let r = self.new_reg();
                self.emit(TacInst::Mov { dest: r, src: Operand::Imm(items.len() as i64) });
                Operand::Reg(r)
            }
            Expr::Dict(entries) => {
                let r = self.new_reg();
                self.emit(TacInst::Mov { dest: r, src: Operand::Imm(entries.len() as i64) });
                Operand::Reg(r)
            }
            Expr::Index { obj, index } => {
                let _o = self.lower_expr(obj);
                let _i = self.lower_expr(index);
                let r = self.new_reg();
                self.emit(TacInst::Mov { dest: r, src: Operand::Imm(0) });
                Operand::Reg(r)
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
        // Check for operator overloading
        if let Expr::Access { .. } = left {
            if let Some(_cn) = self.current_class.as_ref() {
                // Instance method call via operator – handled by parser already as Call
            }
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
                self.emit(TacInst::CmpEq { dest: cmp, lhs: lhs.clone(), rhs: Operand::Imm(0) });
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
            Expr::Ident(s) => (s.clone(), None),
            Expr::Access { obj, field } => {
                if let Expr::Ident(class_name) = obj.as_ref() {
                    let method_name = format!("{}__{}", class_name, field);
                    (method_name, Some(self.lower_expr(obj)))
                } else {
                    let method_name = format!("__{}", field);
                    let obj_val = self.lower_expr(obj);
                    (method_name, Some(obj_val))
                }
            }
            _ => return Operand::Imm(0),
        };

        let mut arg_ops = Vec::new();
        if let Some(this) = this_arg { arg_ops.push(this); }

        // Fill default parameters
        let params_clone = self.func_params.get(&name).cloned();
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

        // Check if this is a generic function call – create monomorphized version if needed
        let actual_name = self.monomorphize_if_needed(&name);

        let dest = self.new_reg();
        self.emit(TacInst::Call { dest: Some(dest), name: actual_name, args: arg_ops });
        Operand::Reg(dest)
    }

    fn monomorphize_if_needed(&mut self, _name: &str) -> String {
        // TODO: full monomorphization with type substitution
        // For now, return the name as-is; the parser has already resolved
        // generic calls to concrete function names during semantic analysis
        _name.to_string()
    }

    fn lower_assign(&mut self, target: &Expr, value: &Expr) -> Operand {
        let val = self.lower_expr(value);

        let (dest_reg, target_name) = match target {
            Expr::Ident(name) => {
                let r = if let Some(&r) = self.vars.get(name) { r }
                else { let r = self.new_reg(); self.vars.insert(name.clone(), r); r };
                (r, Some(name.clone()))
            }
            Expr::Access { obj, field } => {
                let obj_reg = self.lower_expr(obj);
                let class_name = self.current_class.clone().unwrap_or_default();
                let field_ptr = self.new_reg();
                self.emit(TacInst::GetFieldPtr { dest: field_ptr, obj: self.reg_from_op(&obj_reg), class: class_name, field: field.clone() });
                self.emit(TacInst::Store { addr: field_ptr, src: val.clone() });
                return val;
            }
            _ => (self.new_reg(), None),
        };

        // If target has Any type, pack the value into tagged union
        if let Some(ref name) = target_name {
            if let Some(ty) = self.var_types.get(name) {
                if matches!(ty, Type::Base(BaseType::Any)) {
                    let tag = crate::ir::tac::type_tag(&self.infer_type_of(&val));
                    let packed = self.new_reg();
                    self.emit(TacInst::PackAny { dest: packed, tag, value: val });
                    self.emit(TacInst::Mov { dest: dest_reg, src: Operand::Reg(packed) });
                    self.var_types.insert(name.clone(), Type::Base(BaseType::Any));
                    return Operand::Reg(dest_reg);
                }
            }
        }

        self.emit(TacInst::Mov { dest: dest_reg, src: val });
        Operand::Reg(dest_reg)
    }

    fn infer_type_of(&self, _op: &Operand) -> Type {
        Type::Base(BaseType::I32)
    }

    fn lower_access(&mut self, obj: &Expr, field: &str) -> Operand {
        let class_name = self.current_class.clone().unwrap_or_default();
        if let Expr::Ident(name) = obj {
            if name == "this" {
                let this_reg = self.vars.get("this").copied().unwrap_or(0);
                let field_ptr = self.new_reg();
                self.emit(TacInst::GetFieldPtr { dest: field_ptr, obj: this_reg, class: class_name, field: field.to_string() });
                let result = self.new_reg();
                self.emit(TacInst::Load { dest: result, addr: field_ptr });
                return Operand::Reg(result);
            }
        }
        let _obj_op = self.lower_expr(obj);
        Operand::Imm(0)
    }

    fn lower_lambda(&mut self, params: &[Param], body: &Block) -> Operand {
        let lambda_name = format!("__lambda_{}", self.label_counter);
        let prev = self.save_context(&lambda_name, params.len());
        self.current_ret_ty = None;
        self.reg_counter = 0;
        for (i, p) in params.iter().enumerate() { let r = self.new_reg(); self.vars.insert(p.name.clone(), r); self.emit(TacInst::Param { dest: r, index: i }); }
        let result = self.lower_block_expr(body);
        self.emit(TacInst::Ret(Some(result)));
        self.restore_context(prev);
        Operand::Imm(1) // Lambda returns a function pointer id
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
