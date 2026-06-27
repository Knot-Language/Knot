use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    FuncDef {
        name: String,
        generics: Vec<String>,
        params: Vec<Param>,
        ret_ty: Option<Type>,
        body: Block,
    },
    Return(Option<Expr>),
    If {
        cond: Expr,
        then_block: Block,
        else_block: Option<Box<Stmt>>,
    },
    While {
        cond: Expr,
        body: Block,
    },
    For {
        var: String,
        iter: Expr,
        body: Block,
    },
    Break(Option<u32>),
    Continue(Option<u32>),
    Expr(Expr),
    Block(Block),
    ClassDef {
        name: String,
        abstract_class: bool,
        generics: Vec<String>,
        mixins: Vec<String>,
        members: Vec<ClassMember>,
    },
    EnumDef {
        name: String,
        generics: Vec<String>,
        variants: Vec<String>,
    },
    Import {
        path: String,
        alias: Option<String>,
    },
    Match {
        expr: Expr,
        branches: Vec<MatchBranch>,
    },
    TryCatch {
        try_block: Block,
        catches: Vec<CatchClause>,
    },
    Throw(Expr),
    Assert {
        expr: Expr,
        message: Option<String>,
    },
    WrapDef {
        name: String,
        params: Vec<Param>,
        body: Block,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchBranch {
    pub pattern: Expr,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub var: Option<String>,
    pub ty: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    Field {
        private: bool,
        name: String,
        ty: Option<Type>,
        default: Option<Expr>,
    },
    Method {
        private: bool,
        name: String,
        generics: Vec<String>,
        params: Vec<Param>,
        ret_ty: Option<Type>,
        body: Block,
    },
    StaticMethod {
        private: bool,
        name: String,
        generics: Vec<String>,
        params: Vec<Param>,
        ret_ty: Option<Type>,
        body: Block,
    },
    Operator {
        op: String,
        generics: Vec<String>,
        params: Vec<Param>,
        ret_ty: Option<Type>,
        body: Block,
    },
    Mixin(String),
    Wrap {
        name: String,
        params: Vec<Param>,
        body: Block,
    },
    AbstractMethod {
        name: String,
        params: Vec<Param>,
        ret_ty: Option<Type>,
    },
    /// Constructor — implicitly returns the class type, `static` is redundant
    New {
        private: bool,
        params: Vec<Param>,
        body: Block,
    },
    /// Destructor — no params, no return type, `static` is redundant
    Delete {
        private: bool,
        body: Block,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
    pub default: Option<Expr>,
    pub is_args: bool,
    pub is_kwargs: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64, Span),
    Float(f64, Span),
    String(String, Span),
    Bool(bool, Span),
    Null(Span),
    Ident(String, Span),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Index {
        obj: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Access {
        obj: Box<Expr>,
        field: String,
        span: Span,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    IfExpr {
        cond: Box<Expr>,
        then_block: Block,
        else_block: Option<Block>,
        span: Span,
    },
    Array(Vec<Expr>, Span),
    Dict(Vec<(Expr, Expr)>, Span),
    Cast {
        expr: Box<Expr>,
        ty: Type,
        forced: bool,
        span: Span,
    },
    Lambda {
        params: Vec<Param>,
        body: Block,
        span: Span,
    },
    MatchExpr {
        expr: Box<Expr>,
        branches: Vec<MatchBranch>,
        span: Span,
    },
    PostfixOp {
        op: BinOp,
        target: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
    Range,
    NullCoalesce,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Base(BaseType),
    Nullable(Box<Type>),
    Named(String),
    Array(Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaseType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    Bool,
    Null,
    Void,
    Any,
}
