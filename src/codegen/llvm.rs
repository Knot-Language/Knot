use crate::ir::tac::*;
use std::collections::HashMap;

pub struct LlvmBackend;

impl LlvmBackend {
    pub fn generate(program: &TacProgram) -> String {
        let mut out = String::new();
        out.push_str("declare i32 @puts(i8*)\n");
        out.push_str("declare i32 @printf(i8*, ...)\n");
        out.push_str("@knot_exception = global i32 0\n");

        for (name, val) in &program.strings {
            let len = val.len() + 1;
            out.push_str(&format!("{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n", name, len, val));
        }
        out.push('\n');

        for class in &program.classes {
            emit_struct_type(&mut out, class);
        }

        out.push('\n');
        for func in &program.functions {
            emit_function(&mut out, func, program);
        }
        out
    }

    pub fn compile_to_exe(ll_path: &str, exe_path: &str) {
        let clang = std::env::var("KNOT_CLANG")
            .unwrap_or_else(|_| "clang".to_string());
        let status = std::process::Command::new(&clang)
            .arg(ll_path)
            .arg("-o")
            .arg(exe_path)
            .arg("-Wno-override-module")
            .status()
            .expect("failed to run clang");
        if !status.success() {
            panic!("clang failed");
        }
    }
}

fn emit_struct_type(out: &mut String, class: &ClassIr) {
    let mut field_types = Vec::new();
    for field in &class.fields {
        field_types.push(llvm_type(&field.ty));
    }
    out.push_str(&format!(
        "%{} = type {{ {} }}\n",
        class.name,
        field_types.join(", ")
    ));
}

fn llvm_type(ty: &crate::parser::ast::Type) -> &'static str {
    use crate::parser::ast::*;
    match ty {
        Type::Base(base) => match base {
            BaseType::I8 | BaseType::I16 | BaseType::I32 | BaseType::I64
            | BaseType::U8 | BaseType::U16 | BaseType::U32 | BaseType::U64 => "i32",
            BaseType::F32 | BaseType::F64 => "double",
            _ => "i32",
        },
        _ => "i32",
    }
}

fn is_class_method(func: &Function, program: &TacProgram) -> bool {
    func.name.contains("__") && program.classes.iter().any(|c| func.name.starts_with(&format!("{}__", c.name)))
}

fn emit_function(out: &mut String, func: &Function, program: &TacProgram) {
    let reg_types: HashMap<Reg, String> = HashMap::new();
    let is_method = is_class_method(func, program);

    let mut params = String::new();
    for i in 0..func.params {
        if i > 0 {
            params.push_str(", ");
        }
        if is_method && i == 0 {
            params.push_str("ptr");
        } else {
            params.push_str("i32");
        }
    }

    let needs_double = func.insts.iter().any(inst_has_f64);
    let ret_ty = if needs_double { "double" } else { "i32" };

    out.push_str(&format!("define {} @{}({}) {{\n", ret_ty, func.name, params));
    out.push_str("entry:\n");

    for i in 0..func.params {
        let pty = if is_method && i == 0 { "ptr" } else { "i32" };
        out.push_str(&format!("  %p{} = alloca {}\n", i, pty));
        out.push_str(&format!("  store {} %{}, ptr %p{}\n", pty, i, i));
    }

    let mut cur = 0;
    let mut ctx = Ctx { reg_types, program: Some(program) };
    let ret_ty_str = ret_ty.to_string();
    while cur < func.insts.len() {
        cur = emit_insts(out, func, cur, &mut ctx, &ret_ty_str);
    }

    out.push_str(&format!("  ret {} 0\n", ret_ty));
    out.push_str("}\n\n");
}

struct Ctx<'a> {
    reg_types: HashMap<Reg, String>,
    program: Option<&'a TacProgram>,
}

impl<'a> Ctx<'a> {
    fn set(&mut self, r: Reg, ty: &str) {
        self.reg_types.insert(r, ty.to_string());
    }

    fn get(&self, r: Reg) -> &str {
        self.reg_types.get(&r).map(|s| s.as_str()).unwrap_or("i32")
    }
}

fn inst_has_f64(inst: &TacInst) -> bool {
    match inst {
        TacInst::Mov { src, .. }
        | TacInst::Neg { src, .. }
        | TacInst::Not { src, .. }
        | TacInst::Ret(Some(src)) => op_is_f64(src),
        TacInst::Add { lhs, rhs, .. }
        | TacInst::Sub { lhs, rhs, .. }
        | TacInst::Mul { lhs, rhs, .. }
        | TacInst::Div { lhs, rhs, .. }
        | TacInst::Mod { lhs, rhs, .. }
        | TacInst::CmpEq { lhs, rhs, .. }
        | TacInst::CmpNe { lhs, rhs, .. }
        | TacInst::CmpLt { lhs, rhs, .. }
        | TacInst::CmpGt { lhs, rhs, .. }
        | TacInst::CmpLe { lhs, rhs, .. }
        | TacInst::CmpGe { lhs, rhs, .. } => op_is_f64(lhs) || op_is_f64(rhs),
        _ => false,
    }
}

fn op_is_f64(op: &Operand) -> bool {
    matches!(op, Operand::F64(_))
}

fn emit_insts(
    out: &mut String,
    func: &Function,
    start: usize,
    ctx: &mut Ctx,
    ret_ty: &str,
) -> usize {
    let mut pos = start;

    while pos < func.insts.len() {
        let inst = &func.insts[pos];

        match inst {
            TacInst::Param { dest, index } => {
                let pty = if *index == 0 && is_class_method(func, ctx.program.unwrap()) { "ptr" } else { "i32" };
                ctx.set(*dest, pty);
                out.push_str(&format!("  %r{} = load {}, ptr %p{}\n", dest, pty, index));
                pos += 1;
                return pos;
            }

            TacInst::Label(name) => {
                out.push_str(&format!("{}:\n", name));
                pos += 1;
                return pos;
            }

            TacInst::Ret(val) => {
                let op_ty = match val {
                    Some(op) => op_ty(op, ctx),
                    None => "i32".to_string(),
                };
                let v = fmt_op(val.as_ref().unwrap_or(&Operand::Imm(0)), ctx);
                if op_ty != ret_ty {
                    if ret_ty == "double" {
                        out.push_str(&format!("  %ret_conv_{} = sitofp i32 {} to double\n", pos, v));
                        out.push_str(&format!("  ret double %ret_conv_{}\n", pos));
                    } else {
                        out.push_str(&format!("  %ret_conv_{} = fptosi double {} to i32\n", pos, v));
                        out.push_str(&format!("  ret i32 %ret_conv_{}\n", pos));
                    }
                } else {
                    out.push_str(&format!("  ret {} {}\n", ret_ty, v));
                }
                pos += 1;
                return pos;
            }

            TacInst::Jmp(label) => {
                out.push_str(&format!("  br label %{}\n", label));
                pos += 1;
                return pos;
            }

            TacInst::JmpIf { cond, label } => {
                let negated = matches!(cond, Operand::Not(_));
                let inner = if let Operand::Not(i) = cond { i.as_ref() } else { cond };
                let c = fmt_op(inner, ctx);
                let ty = op_ty(inner, ctx);
                let next_is_label = pos + 1 < func.insts.len() && matches!(func.insts[pos + 1], TacInst::Label(_));

                let (cmp_op, zero) = if negated {
                    (if ty == "double" { "fcmp oeq" } else { "icmp eq" }, if ty == "double" { "0.0" } else { "0" })
                } else {
                    (if ty == "double" { "fcmp one" } else { "icmp ne" }, if ty == "double" { "0.0" } else { "0" })
                };

                if next_is_label {
                    let next_label = match &func.insts[pos + 1] {
                        TacInst::Label(l) => l.clone(),
                        _ => unreachable!(),
                    };
                    out.push_str(&format!("  %cmp_{} = {} {} {}, {}\n", pos, cmp_op, ty, c, zero));
                    out.push_str(&format!("  br i1 %cmp_{}, label %{}, label %{}\n", pos, label, next_label));
                } else {
                    let merge = format!("merge_{}", pos);
                    out.push_str(&format!("  %cmp_{} = {} {} {}, {}\n", pos, cmp_op, ty, c, zero));
                    out.push_str(&format!("  br i1 %cmp_{}, label %{}, label %{}\n", pos, label, merge));
                    out.push_str(&format!("{}:\n", merge));
                }
                pos += 1;
                return pos;
            }

            TacInst::Mov { dest, src } => {
                let ty = val_ty_src_reg(src, ctx);
                let s = fmt_op(src, ctx);
                let op = if ty == "double" { "fadd" } else { "add" };
                let zero = if ty == "double" { "0.0" } else { "0" };
                out.push_str(&format!("  %r{} = {} {} {}, {}\n", dest, op, ty, s, zero));
                ctx.set(*dest, &ty);
                pos += 1;
            }

            TacInst::Add { dest, lhs, rhs } => { arith(out, dest, lhs, rhs, ctx, "add", "fadd"); pos += 1; }
            TacInst::Sub { dest, lhs, rhs } => { arith(out, dest, lhs, rhs, ctx, "sub", "fsub"); pos += 1; }
            TacInst::Mul { dest, lhs, rhs } => { arith(out, dest, lhs, rhs, ctx, "mul", "fmul"); pos += 1; }
            TacInst::Div { dest, lhs, rhs } => { arith(out, dest, lhs, rhs, ctx, "sdiv", "fdiv"); pos += 1; }
            TacInst::Mod { dest, lhs, rhs } => { arith(out, dest, lhs, rhs, ctx, "srem", "frem"); pos += 1; }
            TacInst::Neg { dest, src } => {
                let ty = op_ty(src, ctx);
                let s = fmt_op(src, ctx);
                if ty == "double" {
                    out.push_str(&format!("  %r{} = fsub {} 0.0, {}\n", dest, ty, s));
                } else {
                    out.push_str(&format!("  %r{} = sub {} 0, {}\n", dest, ty, s));
                }
                ctx.set(*dest, &ty);
                pos += 1;
            }
            TacInst::Not { dest, src } => {
                let s = fmt_op(src, ctx);
                out.push_str(&format!("  %r{} = xor i32 {}, 1\n", dest, s));
                ctx.set(*dest, "i32");
                pos += 1;
            }
            TacInst::Shl { dest, lhs, rhs } => { arith(out, dest, lhs, rhs, ctx, "shl", "shl"); pos += 1; }
            TacInst::Shr { dest, lhs, rhs } => { arith(out, dest, lhs, rhs, ctx, "ashr", "ashr"); pos += 1; }
            TacInst::BitAnd { dest, lhs, rhs } => { arith(out, dest, lhs, rhs, ctx, "and", "and"); pos += 1; }
            TacInst::BitOr { dest, lhs, rhs } => { arith(out, dest, lhs, rhs, ctx, "or", "or"); pos += 1; }
            TacInst::BitXor { dest, lhs, rhs } => { arith(out, dest, lhs, rhs, ctx, "xor", "xor"); pos += 1; }
            TacInst::CmpEq { dest, lhs, rhs } => { emit_cmp(out, *dest, lhs, rhs, "eq", ctx); ctx.set(*dest, "i32"); pos += 1; }
            TacInst::CmpNe { dest, lhs, rhs } => { emit_cmp(out, *dest, lhs, rhs, "ne", ctx); ctx.set(*dest, "i32"); pos += 1; }
            TacInst::CmpLt { dest, lhs, rhs } => { emit_cmp(out, *dest, lhs, rhs, "slt", ctx); ctx.set(*dest, "i32"); pos += 1; }
            TacInst::CmpGt { dest, lhs, rhs } => { emit_cmp(out, *dest, lhs, rhs, "sgt", ctx); ctx.set(*dest, "i32"); pos += 1; }
            TacInst::CmpLe { dest, lhs, rhs } => { emit_cmp(out, *dest, lhs, rhs, "sle", ctx); ctx.set(*dest, "i32"); pos += 1; }
            TacInst::CmpGe { dest, lhs, rhs } => { emit_cmp(out, *dest, lhs, rhs, "sge", ctx); ctx.set(*dest, "i32"); pos += 1; }

            TacInst::GetFieldPtr { dest, obj, class, field } => {
                let struct_ty = format!("%{}", class);
                let field_idx = ctx.program
                    .and_then(|p| p.classes.iter().find(|c| c.name == *class))
                    .and_then(|c| c.fields.iter().position(|f| f.name == *field))
                    .unwrap_or(0);
                let s = fmt_op(&Operand::Reg(*obj), ctx);
                out.push_str(&format!(
                    "  %r{} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}\n",
                    dest, struct_ty, s, field_idx
                ));
                ctx.set(*dest, "ptr");
                pos += 1;
            }

            TacInst::Load { dest, addr } => {
                let addr_str = fmt_op(&Operand::Reg(*addr), ctx);
                out.push_str(&format!("  %r{} = load i32, ptr {}\n", dest, addr_str));
                ctx.set(*dest, "i32");
                pos += 1;
            }

            TacInst::Store { addr, src } => {
                let addr_str = fmt_op(&Operand::Reg(*addr), ctx);
                let src_str = fmt_op(src, ctx);
                out.push_str(&format!("  store i32 {}, ptr {}\n", src_str, addr_str));
                pos += 1;
            }

            TacInst::Alloc { dest, ty } => {
                let ll_ty = llvm_type(ty);
                out.push_str(&format!("  %r{} = alloca {}\n", dest, ll_ty));
                ctx.set(*dest, "ptr");
                pos += 1;
            }

            TacInst::Free { .. } => {
                pos += 1;
            }

            TacInst::CatchEntry(dest) => {
                out.push_str(&format!("  %r{} = load i32, ptr @knot_exception\n", dest));
                out.push_str("  store i32 0, ptr @knot_exception\n");
                ctx.set(*dest, "i32");
                pos += 1;
            }

            TacInst::Throw { value, catch_label } => {
                let v = fmt_op(value, ctx);
                out.push_str(&format!("  store i32 {}, ptr @knot_exception\n", v));
                out.push_str(&format!("  br label %{}\n", catch_label));
                pos += 1;
                return pos;
            }

            TacInst::LoadStrConst { dest, name } => {
                let len = ctx.program
                    .and_then(|p| p.strings.iter().find(|(n, _)| n == name))
                    .map(|(_, v)| v.len() + 1)
                    .unwrap_or(1);
                out.push_str(&format!("  %r{} = getelementptr inbounds [{} x i8], ptr {}, i32 0, i32 0\n", dest, len, name));
                ctx.set(*dest, "ptr");
                pos += 1;
            }

            TacInst::Call { dest, name, args } => {
                let mut arg_strs = Vec::new();
                for a in args {
                    let ty = op_ty(a, ctx);
                    let v = fmt_op(a, ctx);
                    arg_strs.push(format!("{} {}", ty, v));
                }
                if let Some(d) = dest {
                    ctx.set(*d, "i32");
                    out.push_str(&format!("  %r{} = call i32 @{}({})\n", d, name, arg_strs.join(", ")));
                } else {
                    out.push_str(&format!("  call i32 @{}({})\n", name, arg_strs.join(", ")));
                }
                pos += 1;
            }
        }
    }
    pos
}

fn arith(out: &mut String, dest: &Reg, lhs: &Operand, rhs: &Operand, ctx: &mut Ctx, int_op: &str, float_op: &str) {
    let ty = bin_ty(lhs, rhs, ctx);
    let op = if ty == "double" { float_op } else { int_op };
    let l = fmt_op(lhs, ctx);
    let r = fmt_op(rhs, ctx);
    out.push_str(&format!("  %r{} = {} {} {}, {}\n", dest, op, ty, l, r));
    ctx.set(*dest, &ty);
}

fn emit_cmp(out: &mut String, dest: Reg, lhs: &Operand, rhs: &Operand, int_op: &str, ctx: &Ctx) {
    let ty = bin_ty(lhs, rhs, ctx);
    let (prefix, op_str) = if ty == "double" {
        let fop = match int_op { "eq"|"ne" => int_op, "slt" => "olt", "sgt" => "ogt", "sle" => "ole", "sge" => "oge", _ => int_op };
        ("fcmp", fop)
    } else { ("icmp", int_op) };
    let l = fmt_op(lhs, ctx);
    let r = fmt_op(rhs, ctx);
    out.push_str(&format!("  %r{}_i1 = {} {} {} {}, {}\n", dest, prefix, op_str, ty, l, r));
    out.push_str(&format!("  %r{} = zext i1 %r{}_i1 to i32\n", dest, dest));
}

fn op_ty(op: &Operand, ctx: &Ctx) -> String {
    match op {
        Operand::F64(_) => "double".into(),
        Operand::Not(inner) => op_ty(inner, ctx),
        Operand::Reg(r) => ctx.get(*r).into(),
        _ => "i32".into(),
    }
}

fn bin_ty(lhs: &Operand, rhs: &Operand, ctx: &Ctx) -> String {
    let lt = op_ty(lhs, ctx);
    if lt == "double" { return "double".into(); }
    let rt = op_ty(rhs, ctx);
    if rt == "double" { "double".into() } else { "i32".into() }
}

fn val_ty_src_reg(src: &Operand, ctx: &Ctx) -> String { op_ty(src, ctx) }

fn fmt_op(op: &Operand, ctx: &Ctx) -> String {
    match op {
        Operand::Reg(r) => format!("%r{}", r),
        Operand::Imm(v) => format!("{}", v),
        Operand::F64(v) => if *v == v.trunc() && v.is_finite() { format!("{:.1}", v) } else { format!("{}", v) },
        Operand::Bool(b) => (if *b { 1 } else { 0 }).to_string(),
        Operand::Label(l) => format!("%{}", l),
        Operand::Not(inner) => fmt_op(inner, ctx),
    }
}
