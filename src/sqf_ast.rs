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
    Identifier(String),
    UnaryOp {
        op: Operator,
        expr: Box<Expr>,
        is_postfix: bool,
    },
    BinaryOp {
        op: Operator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FuncCall {
        name: String,
        args: Box<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Dummy,
    Expr(Expr),
    VarDecl {
        name: String,
        value: Option<Expr>,
    },
    FuncDef {
        name: String,
        params: Box<Stmt>,
        body: Box<Stmt>,
    },
    Assign {
        name: String,
        value: Expr,
    },
    ParamList(Vec<String>),
    ExprList(Vec<Expr>),
    Block(Vec<Stmt>),
    Program(Vec<Stmt>),
    Return(Expr),
    Break,
    Continue,
    If {
        condition: Expr,
        if_block: Box<Stmt>,
        else_block: Option<Box<Stmt>>,
    },
    For {
        init: Box<Stmt>,
        condition: Expr,
        step: Box<Stmt>,
        block: Box<Stmt>,
    },
    While {
        condition: Expr,
        block: Box<Stmt>,
    },
}