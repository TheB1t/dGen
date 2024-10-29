use crate::transform::*;
use crate::dgen_ast;
use crate::sqf_ast;
use crate::boxable::*;

impl Transform<sqf_ast::Stmt> for dgen_ast::Stmt {
    fn transform(&self) -> sqf_ast::Stmt {
        match self {
            dgen_ast::Stmt::Expr(e) => sqf_ast::Stmt::Expr(e.transform()),
            dgen_ast::Stmt::VarDecl { typename:_, name, value } => sqf_ast::Stmt::VarDecl {
                name: name.clone(),
                value: value.as_ref().map(|expr| expr.transform()),
            },
            dgen_ast::Stmt::Assign { name, value } => sqf_ast::Stmt::Assign {
                name: name.clone(),
                value: value.transform(),
            },
            dgen_ast::Stmt::Block(v) => sqf_ast::Stmt::Block(
                v.iter().map(|s| s.transform()).collect(),
            ),
            dgen_ast::Stmt::ParamList(params) => sqf_ast::Stmt::ParamList(
                params.iter().map(|var_decl| {
                    match var_decl {
                        dgen_ast::Stmt::VarDecl { name, .. } => name.clone(),
                        _ => unreachable!()
                    }
                }).collect(),
            ),
            dgen_ast::Stmt::ExprList(v) => sqf_ast::Stmt::ExprList(
                v.iter().map(|e| e.transform()).collect(),
            ),
            dgen_ast::Stmt::FuncDef { return_type:_, name, params, body } => sqf_ast::Stmt::FuncDef {
                name: name.clone(),
                params: params.transform().into_box(),
                body: body.transform().into_box(),
            },
            dgen_ast::Stmt::FuncCall { name, args } => sqf_ast::Stmt::FuncCall {
                name: name.clone(),
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
            dgen_ast::Expr::Identifier(id)      => sqf_ast::Expr::Identifier(id.clone()),
            dgen_ast::Expr::UnaryOp { op, expr, is_postfix } => sqf_ast::Expr::UnaryOp {
                op: op.transform(),
                expr: expr.transform().into_box(),
                is_postfix: *is_postfix,
            },
            dgen_ast::Expr::BinaryOp { op, left, right } => sqf_ast::Expr::BinaryOp {
                op: op.transform(),
                left: left.transform().into_box(),
                right: right.transform().into_box(),
            },
            dgen_ast::Expr::Stmt(stmt)          => sqf_ast::Expr::Stmt(stmt.transform().into_box()),
        }
    }
}

impl Transform<sqf_ast::Operator> for dgen_ast::Operator {
    fn transform(&self) -> sqf_ast::Operator {
        match self {
            dgen_ast::Operator::Add => sqf_ast::Operator::Add,
            dgen_ast::Operator::Sub => sqf_ast::Operator::Sub,
            dgen_ast::Operator::Mul => sqf_ast::Operator::Mul,
            dgen_ast::Operator::Div => sqf_ast::Operator::Div,
            dgen_ast::Operator::Mod => sqf_ast::Operator::Mod,
            dgen_ast::Operator::Inc => sqf_ast::Operator::Inc,
            dgen_ast::Operator::Dec => sqf_ast::Operator::Dec,
            dgen_ast::Operator::Eq  => sqf_ast::Operator::Eq,
            dgen_ast::Operator::Neq => sqf_ast::Operator::Neq,
            dgen_ast::Operator::Lt  => sqf_ast::Operator::Lt,
            dgen_ast::Operator::Lte => sqf_ast::Operator::Lte,
            dgen_ast::Operator::Gt  => sqf_ast::Operator::Gt,
            dgen_ast::Operator::Gte => sqf_ast::Operator::Gte,
            dgen_ast::Operator::And => sqf_ast::Operator::And,
            dgen_ast::Operator::Or  => sqf_ast::Operator::Or,
            dgen_ast::Operator::Not => sqf_ast::Operator::Not,
            dgen_ast::Operator::Neg => sqf_ast::Operator::Neg,
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
