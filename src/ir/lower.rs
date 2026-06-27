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
    break_label: Option<Label>,
    catch_label: Option<Label>,
    current_ret_ty: Option<Type>,
    current_class: Option<String>,
}

impl Lower {
    pub fn lower(program: &[Stmt]) -> TacProgram {
        let mut l = Lower {
            reg_counter: 0,
            label_counter: 0,
            vars: HashMap::new(),
            var_types: HashMap::new(),
            func: Function {
                name: String::new(),
                params: 0,
                insts: Vec::new(),
            },
            temp_funcs: Vec::new(),
            classes: Vec::new(),
            enums: Vec::new(),
            break_label: None,
            catch_label: None,
            current_ret_ty: None,
            current_class: None,
        };

        for stmt in program {
            l.lower_top_level(stmt);
        }

        TacProgram {
            functions: l.temp_funcs,
            classes: l.classes,
            enums: l.enums,
        }
    }

    fn new_reg(&mut self) -> Reg {
        let r = self.reg_counter;
        self.reg_counter += 1;
        r
    }

    fn new_label(&mut self, prefix: &str) -> Label {
        let l = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        l
    }

    fn lower_method(
        &mut self,
        func_name: &str,
        has_this: bool,
        params: &[Param],
        ret_ty: &Option<Type>,
        body: &Block,
    ) {
        let prev_func = std::mem::replace(
            &mut self.func,
            Function {
                name: func_name.to_string(),
                params: params.len() + if has_this { 1 } else { 0 },
                insts: Vec::new(),
            },
        );
        let prev_vars = std::mem::take(&mut self.vars);
        let prev_var_types = std::mem::take(&mut self.var_types);
        let prev_class = self.current_class.clone();
        if let Some(class_name) = func_name.split("__").next() {
            self.current_class = Some(class_name.to_string());
        }
        let prev_ret = self.current_ret_ty.clone();
        self.current_ret_ty = ret_ty.clone();
        self.reg_counter = 0;

        let mut param_idx = 0;
        if has_this {
            let r = self.new_reg();
            self.vars.insert("this".to_string(), r);
            self.emit(TacInst::Param { dest: r, index: param_idx });
            param_idx += 1;
        }

        for param in params {
            let r = self.new_reg();
            self.vars.insert(param.name.clone(), r);
            self.emit(TacInst::Param { dest: r, index: param_idx });
            param_idx += 1;
        }

        self.lower_block(body);

        if ret_ty.is_none() || matches!(ret_ty, Some(Type::Base(BaseType::Void))) {
            self.emit(TacInst::Ret(None));
        }

        let finished_func = std::mem::replace(&mut self.func, prev_func);
        self.temp_funcs.push(finished_func);
        self.vars = prev_vars;
        self.var_types = prev_var_types;
        self.current_class = prev_class;
        self.current_ret_ty = prev_ret;
    }

    fn lower_constructor(
        &mut self,
        func_name: &str,
        class_name: &str,
        params: &[Param],
        body: &Block,
    ) {
        let prev_func = std::mem::replace(
            &mut self.func,
            Function {
                name: func_name.to_string(),
                params: params.len(),
                insts: Vec::new(),
            },
        );
        let prev_vars = std::mem::take(&mut self.vars);
        let prev_var_types = std::mem::take(&mut self.var_types);
        let prev_class = self.current_class.clone();
        self.current_class = Some(class_name.to_string());
        let prev_ret = self.current_ret_ty.clone();
        self.current_ret_ty = None;
        self.reg_counter = 0;

        for (i, param) in params.iter().enumerate() {
            let r = self.new_reg();
            self.vars.insert(param.name.clone(), r);
            self.emit(TacInst::Param { dest: r, index: i });
        }

        let this_reg = self.new_reg();
        self.vars.insert("this".to_string(), this_reg);
        self.emit(TacInst::Alloc { dest: this_reg, ty: Type::Named(class_name.to_string()) });

        self.lower_block(body);
        self.emit(TacInst::Ret(Some(Operand::Reg(this_reg))));

        let finished_func = std::mem::replace(&mut self.func, prev_func);
        self.temp_funcs.push(finished_func);
        self.vars = prev_vars;
        self.var_types = prev_var_types;
        self.current_class = prev_class;
        self.current_ret_ty = prev_ret;
    }

    fn lower_destructor(
        &mut self,
        func_name: &str,
        class_name: &str,
        body: &Block,
    ) {
        let prev_func = std::mem::replace(
            &mut self.func,
            Function {
                name: func_name.to_string(),
                params: 1,
                insts: Vec::new(),
            },
        );
        let prev_vars = std::mem::take(&mut self.vars);
        let prev_var_types = std::mem::take(&mut self.var_types);
        let prev_class = self.current_class.clone();
        self.current_class = Some(class_name.to_string());
        let prev_ret = self.current_ret_ty.clone();
        self.current_ret_ty = None;
        self.reg_counter = 0;

        let this_reg = self.new_reg();
        self.vars.insert("this".to_string(), this_reg);
        self.emit(TacInst::Param { dest: this_reg, index: 0 });

        self.lower_block(body);
        self.emit(TacInst::Free { src: this_reg });
        self.emit(TacInst::Ret(None));

        let finished_func = std::mem::replace(&mut self.func, prev_func);
        self.temp_funcs.push(finished_func);
        self.vars = prev_vars;
        self.var_types = prev_var_types;
        self.current_class = prev_class;
        self.current_ret_ty = prev_ret;
    }

    fn lower_access(&mut self, obj: &Expr, field: &str) -> Operand {
        let _obj_op = self.lower_expr(obj);
        let class_name = match &self.current_class {
            Some(c) => c.clone(),
            None => return Operand::Imm(0),
        };
        let this_reg = self.vars.get("this").copied().unwrap_or(0);
        if let Expr::Ident(name) = obj {
            if name == "this" {
                let field_ptr = self.new_reg();
                self.emit(TacInst::GetFieldPtr {
                    dest: field_ptr,
                    obj: this_reg,
                    class: class_name,
                    field: field.to_string(),
                });
                let result = self.new_reg();
                self.emit(TacInst::Load { dest: result, addr: field_ptr });
                return Operand::Reg(result);
            }
        }
        Operand::Imm(0)
    }

    fn emit(&mut self, inst: TacInst) {
        self.func.insts.push(inst);
    }

    fn lower_top_level(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FuncDef {
                name,
                params,
                ret_ty,
                body,
                ..
            } => {
                let prev_func = std::mem::replace(
                    &mut self.func,
                    Function {
                        name: name.clone(),
                        params: params.len(),
                        insts: Vec::new(),
                    },
                );
                let prev_vars = std::mem::take(&mut self.vars);
                let prev_var_types = std::mem::take(&mut self.var_types);
                let prev_ret = self.current_ret_ty.clone();
                self.current_ret_ty = ret_ty.clone();
                self.reg_counter = 0;

                for (i, param) in params.iter().enumerate() {
                    let r = self.new_reg();
                    self.vars.insert(param.name.clone(), r);
                    self.emit(TacInst::Param { dest: r, index: i });
                }

                self.lower_block(body);

                if ret_ty.is_none() || matches!(ret_ty, Some(Type::Base(BaseType::Void))) {
                    self.emit(TacInst::Ret(None));
                }

                let finished_func = std::mem::replace(&mut self.func, prev_func);
                self.temp_funcs.push(finished_func);
                self.vars = prev_vars;
                self.var_types = prev_var_types;
                self.current_ret_ty = prev_ret;
            }
            Stmt::ClassDef {
                name,
                generics,
                mixins,
                abstract_class,
                members,
            } => {
                let mut class = ClassIr {
                    name: name.clone(),
                    generics: generics.clone(),
                    mixins: mixins.clone(),
                    abstract_class: *abstract_class,
                    fields: Vec::new(),
                };

                for member in members {
                    match member {
                        ClassMember::Field { name: fname, ty, default: _, private } => {
                            class.fields.push(FieldIr {
                                name: fname.clone(),
                                ty: ty.clone().unwrap_or(Type::Base(BaseType::Void)),
                                default: None,
                                private: *private,
                            });
                        }
                        ClassMember::Method { name: mname, params, ret_ty, body, .. } => {
                            let func_name = format!("{}__{}", name, mname);
                            self.lower_method(&func_name, true, params, ret_ty, body);
                        }
                        ClassMember::StaticMethod { name: mname, params, ret_ty, body, .. } => {
                            let func_name = format!("{}_static__{}", name, mname);
                            self.lower_method(&func_name, false, params, ret_ty, body);
                        }
                        ClassMember::Operator { op, params, ret_ty, body, .. } => {
                            let op_name = op.replace('+', "plus").replace('-', "minus")
                                .replace('*', "mul").replace('/', "div")
                                .replace('%', "mod").replace("==", "eq")
                                .replace("!=", "ne").replace('<', "lt")
                                .replace('>', "gt").replace(" ", "_");
                            let func_name = format!("{}__op_{}", name, op_name);
                            self.lower_method(&func_name, true, params, ret_ty, body);
                        }
                        ClassMember::Wrap { name: wname, params, body } => {
                            let func_name = format!("{}__wrap_{}", name, wname);
                            self.lower_method(&func_name, true, params, &None, body);
                        }
                        ClassMember::New { params, body, .. } => {
                            let func_name = format!("{}__new", name);
                            self.lower_constructor(&func_name, name, params, body);
                        }
                        ClassMember::Delete { body, .. } => {
                            let func_name = format!("{}__delete", name);
                            self.lower_destructor(&func_name, name, body);
                        }
                        ClassMember::Mixin(_) | ClassMember::AbstractMethod { .. } => {}
                    }
                }

                self.classes.push(class);
            }
            Stmt::EnumDef {
                name,
                generics,
                variants,
            } => {
                self.enums.push(EnumIr {
                    name: name.clone(),
                    generics: generics.clone(),
                    variants: variants.clone(),
                });
            }
            Stmt::Import { .. } | Stmt::Expr(_) | Stmt::Return(_) | Stmt::If { .. }
            | Stmt::While { .. } | Stmt::For { .. } | Stmt::Break(_) | Stmt::Continue(_)
            | Stmt::Block(_) | Stmt::Match { .. } | Stmt::TryCatch { .. }
            | Stmt::Throw(_) | Stmt::Assert { .. } | Stmt::WrapDef { .. } => {}
        }
    }

    fn lower_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.lower_expr(expr);
            }
            Stmt::Return(expr) => {
                let val = expr
                    .as_ref()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Operand::Imm(0));
                self.emit(TacInst::Ret(Some(val)));
            }
            Stmt::If {
                cond,
                then_block,
                else_block,
            } => self.lower_if(cond, then_block, else_block),
            Stmt::While { cond, body } => self.lower_while(cond, body),
            Stmt::Block(block) => self.lower_block(block),
            Stmt::For { var, iter, body } => self.lower_for(var, iter, body),
            Stmt::Break(_) => {
                if let Some(label) = &self.break_label {
                    self.emit(TacInst::Jmp(label.clone()));
                }
            }
            Stmt::Continue(_) => {}
            Stmt::Throw(expr) => {
                let val = self.lower_expr(expr);
                let label = self
                    .catch_label
                    .clone()
                    .unwrap_or_else(|| "_no_handler".to_string());
                self.emit(TacInst::Throw {
                    value: val,
                    catch_label: label,
                });
            }
            Stmt::TryCatch { try_block, catches } => self.lower_try_catch(try_block, catches),
            Stmt::FuncDef { .. } | Stmt::ClassDef { .. } | Stmt::EnumDef { .. } | Stmt::Import { .. }
            | Stmt::Match { .. } | Stmt::Assert { .. }
            | Stmt::WrapDef { .. } => {}
        }
    }

    fn lower_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.lower_stmt(stmt);
        }
    }

    fn lower_if(
        &mut self,
        cond: &Expr,
        then_block: &Block,
        else_block: &Option<Box<Stmt>>,
    ) {
        let cond_val = self.lower_expr(cond);
        let else_label = self.new_label("else");
        let end_label = self.new_label("endif");

        self.emit(TacInst::JmpIf {
            cond: Operand::Not(Box::new(cond_val)),
            label: else_label.clone(),
        });

        self.lower_block(then_block);
        self.emit(TacInst::Jmp(end_label.clone()));

        self.emit(TacInst::Label(else_label));
        if let Some(else_stmt) = else_block {
            self.lower_stmt(else_stmt);
        }

        self.emit(TacInst::Label(end_label));
    }

    fn lower_while(&mut self, cond: &Expr, body: &Block) {
        let loop_label = self.new_label("loop");
        let end_label = self.new_label("endloop");
        let prev_break = self.break_label.clone();

        self.break_label = Some(end_label.clone());

        self.emit(TacInst::Label(loop_label.clone()));
        let cond_val = self.lower_expr(cond);
        self.emit(TacInst::JmpIf {
            cond: Operand::Not(Box::new(cond_val)),
            label: end_label.clone(),
        });

        self.lower_block(body);
        self.emit(TacInst::Jmp(loop_label));

        self.emit(TacInst::Label(end_label));
        self.break_label = prev_break;
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

        if let Some(first_catch) = catches.first() {
            let exc_reg = self.new_reg();
            self.emit(TacInst::CatchEntry(exc_reg));
            if let Some(var) = &first_catch.var {
                self.vars.insert(var.clone(), exc_reg);
            }
            self.lower_block(&first_catch.body);
        }

        self.emit(TacInst::Label(end_label));
    }

    fn lower_for(&mut self, var: &str, iter: &Expr, body: &Block) {
        let loop_label = self.new_label("forloop");
        let end_label = self.new_label("forend");
        let prev_break = self.break_label.clone();
        self.break_label = Some(end_label.clone());

        let start_val = match iter {
            Expr::Binary { op: BinOp::Range, left, right } => {
                let start = self.lower_expr(left);
                let end = self.lower_expr(right);

                let index_reg = self.new_reg();
                self.emit(TacInst::Mov {
                    dest: index_reg,
                    src: start,
                });

                let max_reg = self.new_reg();
                self.emit(TacInst::Mov {
                    dest: max_reg,
                    src: end,
                });

                (index_reg, max_reg)
            }
            _ => {
                let val = self.lower_expr(iter);
                let index_reg = self.new_reg();
                self.emit(TacInst::Mov {
                    dest: index_reg,
                    src: Operand::Imm(0),
                });
                let max_reg = self.new_reg();
                self.emit(TacInst::Mov {
                    dest: max_reg,
                    src: val,
                });
                (index_reg, max_reg)
            }
        };

        self.emit(TacInst::Label(loop_label.clone()));

        let cond_reg = self.new_reg();
        self.emit(TacInst::CmpGe {
            dest: cond_reg,
            lhs: Operand::Reg(start_val.0),
            rhs: Operand::Reg(start_val.1),
        });
        self.emit(TacInst::JmpIf {
            cond: Operand::Reg(cond_reg),
            label: end_label.clone(),
        });

        let var_reg = self.new_reg();
        self.vars.insert(var.to_string(), var_reg);
        self.emit(TacInst::Mov {
            dest: var_reg,
            src: Operand::Reg(start_val.0),
        });

        self.lower_block(body);

        let one_reg = self.new_reg();
        self.emit(TacInst::Mov {
            dest: one_reg,
            src: Operand::Imm(1),
        });
        self.emit(TacInst::Add {
            dest: start_val.0,
            lhs: Operand::Reg(start_val.0),
            rhs: Operand::Reg(one_reg),
        });
        self.emit(TacInst::Jmp(loop_label));

        self.emit(TacInst::Label(end_label));
        self.break_label = prev_break;
    }

    fn lower_expr(&mut self, expr: &Expr) -> Operand {
        match expr {
            Expr::Int(v) => Operand::Imm(*v),
            Expr::Float(v) => {
                Operand::F64(*v)
            }
            Expr::Bool(b) => Operand::Bool(*b),
            Expr::Null => Operand::Imm(0),
            Expr::String(s) => {
                let r = self.new_reg();
                self.emit(TacInst::Mov {
                    dest: r,
                    src: Operand::Imm(0),
                });
                let _ = s;
                Operand::Reg(r)
            }
            Expr::Ident(name) => {
                if let Some(&reg) = self.vars.get(name) {
                    Operand::Reg(reg)
                } else {
                    let r = self.new_reg();
                    self.vars.insert(name.clone(), r);
                    Operand::Reg(r)
                }
            }
            Expr::Binary { op, left, right } => self.lower_binary(op, left, right),
            Expr::Unary { op, expr } => self.lower_unary(op, expr),
            Expr::Call { callee, args } => self.lower_call(callee, args),
            Expr::Assign { target, value } => self.lower_assign(target, value),
            Expr::Access { obj, field } => self.lower_access(obj, field),
            Expr::PostfixOp { op, target } => {
                // x++ → temp = x; x = x + 1; return temp
                let old_val = self.lower_expr(target);  // old value lives in register
                let old_reg = self.new_reg();
                self.emit(TacInst::Mov { dest: old_reg, src: old_val.clone() });

                let one = Operand::Imm(1);
                let dest = self.new_reg();
                let inst = match op {
                    BinOp::Add => TacInst::Add { dest, lhs: old_val, rhs: one },
                    BinOp::Sub => TacInst::Sub { dest, lhs: old_val, rhs: one },
                    _ => unreachable!(),
                };
                self.emit(inst);

                // Store result back
                let target_reg = match target.as_ref() {
                    Expr::Ident(name) => *self.vars.get(name).unwrap_or(&0),
                    _ => dest,
                };
                self.emit(TacInst::Mov { dest: target_reg, src: Operand::Reg(dest) });
                Operand::Reg(old_reg)  // return old value
            }
            _ => Operand::Imm(0),
        }
    }

    fn lower_binary(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Operand {
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
                let _and_label = self.new_label("and");
                let end_label = self.new_label("logic_end");

                self.emit(TacInst::Mov { dest, src: lhs.clone() });

                if matches!(op, BinOp::And) {
                    self.emit(TacInst::JmpIf {
                        cond: Operand::Not(Box::new(lhs)),
                        label: end_label.clone(),
                    });
                } else {
                    self.emit(TacInst::JmpIf {
                        cond: lhs,
                        label: end_label.clone(),
                    });
                }

                self.emit(TacInst::Mov { dest, src: rhs });
                self.emit(TacInst::Label(end_label));
                Operand::Reg(dest)
            }
            BinOp::Range | BinOp::NullCoalesce => Operand::Imm(0),
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
        if let Some(this) = this_arg {
            arg_ops.push(this);
        }
        for arg in args {
            arg_ops.push(self.lower_expr(arg));
        }

        let dest = self.new_reg();
        self.emit(TacInst::Call {
            dest: Some(dest),
            name,
            args: arg_ops,
        });
        Operand::Reg(dest)
    }

    fn lower_assign(&mut self, target: &Expr, value: &Expr) -> Operand {
        let val = self.lower_expr(value);
        let dest_reg = match target {
            Expr::Ident(name) => {
                if let Some(&r) = self.vars.get(name) {
                    r
                } else {
                    let r = self.new_reg();
                    self.vars.insert(name.clone(), r);
                    r
                }
            }
            _ => self.new_reg(),
        };

        self.emit(TacInst::Mov {
            dest: dest_reg,
            src: val,
        });
        Operand::Reg(dest_reg)
    }
}
