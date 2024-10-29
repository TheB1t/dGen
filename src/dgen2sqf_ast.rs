use crate::transform::*;
use crate::dgen_ast;
use crate::sqf_ast;
use crate::boxable::*;

impl Transform<sqf_ast::Stmt> for dgen_ast::Stmt {
    fn transform(&self) -> sqf_ast::Stmt {
        match self {
            dgen_ast::Stmt::Expr(e) => sqf_ast::Stmt::Expr(e.transform()),
            dgen_ast::Stmt::VarDecl { typename:_, name, value } => sqf_ast::Stmt::VarDecl {
                name: format!("_{}", name),
                value: value.as_ref().map(|expr| expr.transform()),
            },
            dgen_ast::Stmt::Assign { name, value } => sqf_ast::Stmt::Assign {
                name: format!("_{}", name),
                value: value.transform(),
            },
            dgen_ast::Stmt::Block(v) => sqf_ast::Stmt::Block(
                v.iter().map(|s| s.transform()).collect(),
            ),
            dgen_ast::Stmt::ParamList(params) => sqf_ast::Stmt::ParamList(
                params.iter().map(|var_decl| {
                    match var_decl {
                        dgen_ast::Stmt::VarDecl { name, .. } => format!("_{}", name),
                        _ => unreachable!()
                    }
                }).collect(),
            ),
            dgen_ast::Stmt::ExprList(v) => sqf_ast::Stmt::ExprList(
                v.iter().map(|e| e.transform()).collect(),
            ),
            dgen_ast::Stmt::FuncDef { return_type:_, name, params, body } => sqf_ast::Stmt::FuncDef {
                name: format!("_{}", name),
                params: params.transform().into_box(),
                body: body.transform().into_box(),
            },
            dgen_ast::Stmt::FuncCall { name, args } => sqf_ast::Stmt::FuncCall {
                name: format!("_{}", name),
                args: args.transform().into_box(),
            },
            dgen_ast::Stmt::Return(e)               => sqf_ast::Stmt::Return(e.transform()),
            dgen_ast::Stmt::FuncDecl { .. }         => sqf_ast::Stmt::Dummy, // SQF doesn't support function declarations
            _ => {
                println!("Can't convert {:?} to sqf_ast::Stmt", self);
                sqf_ast::Stmt::Dummy
            }
        }
    }
}

impl Transform<sqf_ast::Expr> for dgen_ast::Expr {
    fn transform(&self) -> sqf_ast::Expr {
        match self {
            dgen_ast::Expr::Number(n)           => sqf_ast::Expr::Number(*n),
            dgen_ast::Expr::Bool(b)             => sqf_ast::Expr::Bool(*b),
            dgen_ast::Expr::String(s)           => sqf_ast::Expr::String(s.clone()),
            dgen_ast::Expr::Identifier(id)      => sqf_ast::Expr::Identifier(format!("_{}", id)),
            dgen_ast::Expr::UnaryOp { op, expr, is_postfix } => {
                match op.transform() {
                    Option::Some(o) => sqf_ast::Expr::UnaryOp {
                        op: o,
                        expr: expr.transform().into_box(),
                        is_postfix: *is_postfix,
                    },
                    Option::None => match op {
                        dgen_ast::Operator::Inc => sqf_ast::Expr::BinaryOp {
                            op: sqf_ast::Operator::Add,
                            left: expr.transform().into_box(),
                            right: sqf_ast::Expr::Number(1.0).into_box(),
                        },
                        dgen_ast::Operator::Dec => sqf_ast::Expr::BinaryOp {
                            op: sqf_ast::Operator::Sub,
                            left: expr.transform().into_box(),
                            right: sqf_ast::Expr::Number(1.0).into_box(),
                        },
                        _ => {
                            println!("Can't convert {:?} to sqf_ast::Expr", self);
                            unreachable!()
                        }
                    }
                }
            },
            dgen_ast::Expr::BinaryOp { op, left, right } => {
                match op.transform() {
                    Option::Some(o) => sqf_ast::Expr::BinaryOp {
                        op: o,
                        left: left.transform().into_box(),
                        right: right.transform().into_box(),
                    },
                    Option::None => {
                        println!("Can't convert {:?} to sqf_ast::Expr", self);
                        unreachable!()
                    }
                }
            },
            dgen_ast::Expr::Stmt(stmt)          => sqf_ast::Expr::Stmt(stmt.transform().into_box()),
        }
    }
}

impl Transform<Option<sqf_ast::Operator>> for dgen_ast::Operator {
    fn transform(&self) -> Option<sqf_ast::Operator> {
        match self {
            dgen_ast::Operator::Add => Some(sqf_ast::Operator::Add),
            dgen_ast::Operator::Sub => Some(sqf_ast::Operator::Sub),
            dgen_ast::Operator::Mul => Some(sqf_ast::Operator::Mul),
            dgen_ast::Operator::Div => Some(sqf_ast::Operator::Div),
            dgen_ast::Operator::Mod => Some(sqf_ast::Operator::Mod),
            dgen_ast::Operator::Eq  => Some(sqf_ast::Operator::Eq),
            dgen_ast::Operator::Neq => Some(sqf_ast::Operator::Neq),
            dgen_ast::Operator::Lt  => Some(sqf_ast::Operator::Lt),
            dgen_ast::Operator::Lte => Some(sqf_ast::Operator::Lte),
            dgen_ast::Operator::Gt  => Some(sqf_ast::Operator::Gt),
            dgen_ast::Operator::Gte => Some(sqf_ast::Operator::Gte),
            dgen_ast::Operator::And => Some(sqf_ast::Operator::And),
            dgen_ast::Operator::Or  => Some(sqf_ast::Operator::Or),
            dgen_ast::Operator::Not => Some(sqf_ast::Operator::Not),
            dgen_ast::Operator::Neg => Some(sqf_ast::Operator::Neg),
            _ => None
        }
    }
}

impl Transform<sqf_ast::Type> for dgen_ast::Type {
    fn transform(&self) -> sqf_ast::Type {
        match self {
            dgen_ast::Type::Any         => sqf_ast::Type::Any,
            dgen_ast::Type::Boolean     => sqf_ast::Type::Boolean,
            dgen_ast::Type::Number      => sqf_ast::Type::Number,
            dgen_ast::Type::String      => sqf_ast::Type::String,
            dgen_ast::Type::Array(t)    => sqf_ast::Type::Array(t.transform().into_box()),
            dgen_ast::Type::Object      => sqf_ast::Type::Object,
            dgen_ast::Type::Void        => sqf_ast::Type::Void,
        }
    }
}
