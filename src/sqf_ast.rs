use crate::generic::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Any,
    Array(Box<Type>),
    Boolean,
    Color,
    Code,
    Config,
    Control,
    DiaryRecord,
    Display,
    Date,
    EditorObject,
    Group,
    HashMap,
    HashMapKey,
    Location,
    Namespace,
    NaN,
    Number,
    Nothing,
    Object,
    ScriptHandle,
    Side,
    String,
    StructuredText,
    Task,
    Team,
    TeamMember,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Bool(bool),
    String(String),
    Array(Vec<Expr>),
    ArrayAccess(String, Box<Expr>),
    Identifier(String),
    UnaryOp(Operator, Box<Expr>, bool),
    BinaryOp(Operator, Box<Expr>, Box<Expr>),
    FuncCall(String, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Dummy,
    Expr(Expr),
    VarDecl(String, Option<Expr>),
    FuncDef(String, Vec<(Type, String)>, Box<Stmt>),
    Assign(String, Expr),
    Block(Vec<Stmt>),
    Program(Vec<Stmt>),
    Return(Expr),
    Break,
    Continue,
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    For(Box<Stmt>, Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
}