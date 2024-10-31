use crate::generic::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Any,
    Number,
    String,
    Boolean,
    Void,
    Object,
    Array(Box<Type>),
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
        is_postfix: bool
    },
    BinaryOp {
        op: Operator,
        left: Box<Expr>,
        right: Box<Expr>
    },
    FuncCall {
        name: String,
        args: Box<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    VarDecl {
        typename: Type,
        name: String,
        value: Option<Expr>
    },
    FuncDecl {
        return_type: Type,
        name: String,
        params: Box<Stmt>,
    },
    FuncDef {
        return_type: Type,
        name: String,
        params: Box<Stmt>,
        body: Box<Stmt>
    },
    Assign {
        name: String,
        value: Expr
    },
    TypeList(Vec<Type>),
    ParamList(Vec<Stmt>),
    ExprList(Vec<Expr>),
    Block(Vec<Stmt>),
    Program(Vec<Stmt>),
    Return(Expr)
}