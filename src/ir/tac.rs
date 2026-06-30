use crate::parser::ast::Type;

pub type Reg = usize;
pub type Label = String;

#[derive(Debug, Clone)]
pub enum Operand {
    Reg(Reg),
    Imm(i64),
    F64(f64),
    Bool(bool),
    Label(Label),
    Not(Box<Operand>),
    NullSentinel,
}

#[derive(Debug, Clone)]
pub enum TacInst {
    Label(Label),
    Alloc { dest: Reg, ty: Type },
    Free { src: Reg },
    Mov { dest: Reg, src: Operand },
    Add { dest: Reg, lhs: Operand, rhs: Operand },
    Sub { dest: Reg, lhs: Operand, rhs: Operand },
    Mul { dest: Reg, lhs: Operand, rhs: Operand },
    Div { dest: Reg, lhs: Operand, rhs: Operand },
    Mod { dest: Reg, lhs: Operand, rhs: Operand },
    Neg { dest: Reg, src: Operand },
    Not { dest: Reg, src: Operand },
    BitNot { dest: Reg, src: Operand },
    Shl { dest: Reg, lhs: Operand, rhs: Operand },
    Shr { dest: Reg, lhs: Operand, rhs: Operand },
    BitAnd { dest: Reg, lhs: Operand, rhs: Operand },
    BitOr { dest: Reg, lhs: Operand, rhs: Operand },
    BitXor { dest: Reg, lhs: Operand, rhs: Operand },
    CmpEq { dest: Reg, lhs: Operand, rhs: Operand },
    CmpNe { dest: Reg, lhs: Operand, rhs: Operand },
    CmpLt { dest: Reg, lhs: Operand, rhs: Operand },
    CmpGt { dest: Reg, lhs: Operand, rhs: Operand },
    CmpLe { dest: Reg, lhs: Operand, rhs: Operand },
    CmpGe { dest: Reg, lhs: Operand, rhs: Operand },
    Jmp(Label),
    JmpIf { cond: Operand, label: Label },
    Call { dest: Option<Reg>, name: String, args: Vec<Operand> },
    Ret(Option<Operand>),
    Param { dest: Reg, index: usize },
    GetFieldPtr { dest: Reg, obj: Reg, class: String, field: String },
    Load { dest: Reg, addr: Reg },
    Store { addr: Reg, src: Operand },
    Throw { value: Operand, catch_label: Label },
    CatchEntry(Reg),
    LoadStrConst { dest: Reg, name: String },
    IntToFloat { dest: Reg, src: Operand },
    FloatToInt { dest: Reg, src: Operand },
    AllocArray { dest: Reg, count: Operand, elem_ty: Type },
    GetElemPtr { dest: Reg, obj: Reg, index: Operand, elem_ty: Type },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: usize,
    pub param_types: Vec<Type>,
    pub insts: Vec<TacInst>,
    pub ret_ty: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct FieldIr {
    pub name: String,
    pub ty: Type,
    pub default: Option<Operand>,
    pub private: bool,
}

#[derive(Debug, Clone)]
pub struct WrapIr {
    pub name: String,
    pub params: usize,
    pub body: Vec<TacInst>,
}

#[derive(Debug, Clone)]
pub struct OperatorIr {
    pub op: String,
    pub func: Function,
}

#[derive(Debug, Clone)]
pub struct ClassIr {
    pub name: String,
    pub generics: Vec<String>,
    pub mixins: Vec<String>,
    pub abstract_class: bool,
    pub fields: Vec<FieldIr>,
}

#[derive(Debug, Clone)]
pub struct EnumIr {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExternFuncIr {
    pub name: String,
    pub param_tys: Vec<Type>,
    pub ret_ty: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct ExternClassIr {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
pub struct TacProgram {
    pub functions: Vec<Function>,
    pub classes: Vec<ClassIr>,
    pub enums: Vec<EnumIr>,
    pub extern_funcs: Vec<ExternFuncIr>,
    pub extern_classes: Vec<ExternClassIr>,
    pub strings: Vec<(String, String)>,
    pub source_files: Vec<String>,
}
