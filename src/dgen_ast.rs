use clap::builder::Str;

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
    Array(Vec<Expr>),
    ArrayAccess(String, Box<Expr>),
    Identifier(String),
    UnaryOp(Operator, Box<Expr>, bool),
    BinaryOp(Operator, Box<Expr>, Box<Expr>),
    FuncCall(String, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    VarDecl(Type, String, Option<Expr>),
    ArrayDecl(Type, String, Option<Expr>),
    FuncDecl(Type, String, Vec<Type>),
    FuncDef(Type, String, Vec<(Type, String)>, Box<Stmt>),
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