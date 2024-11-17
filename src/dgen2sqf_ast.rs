use crate::transform::*;
use crate::generic::*;
use crate::dgen_ast;
use crate::sqf_ast;
use crate::boxable::*;

impl Transform<Vec<sqf_ast::Stmt>> for Vec<dgen_ast::Stmt> {
    fn transform(&self) -> Vec<sqf_ast::Stmt> {
        self.iter().map(|s| s.transform()).collect()
    }
}

impl Transform<Vec<sqf_ast::Expr>> for Vec<dgen_ast::Expr> {
    fn transform(&self) -> Vec<sqf_ast::Expr> {
        self.iter().map(|s| s.transform()).collect()
    }
}

impl Transform<Vec<(sqf_ast::Type, String)>> for Vec<(dgen_ast::Type, String)> {
    fn transform(&self) -> Vec<(sqf_ast::Type, String)> {
        self.iter().map(|s| (s.0.transform(), format!("_{}", s.1))).collect()
    }
}

impl Transform<Option<sqf_ast::Type>> for Option<dgen_ast::Type> {
    fn transform(&self) -> Option<sqf_ast::Type> {
        self.as_ref().map(|t| t.transform())
    }
}

impl Transform<Option<sqf_ast::Expr>> for Option<dgen_ast::Expr> {
    fn transform(&self) -> Option<sqf_ast::Expr> {
        self.as_ref().map(|e| e.transform())
    }
}

impl Transform<Option<Box<sqf_ast::Stmt>>> for Option<Box<dgen_ast::Stmt>> {
    fn transform(&self) -> Option<Box<sqf_ast::Stmt>> {
        self.as_ref().map(|s| s.transform())
    }
}

impl Transform<Box<sqf_ast::Stmt>> for Box<dgen_ast::Stmt> {
    fn transform(&self) -> Box<sqf_ast::Stmt> {
        self.as_ref().transform().wrap()
    }
}

impl Transform<Box<sqf_ast::Expr>> for Box<dgen_ast::Expr> {
    fn transform(&self) -> Box<sqf_ast::Expr> {
        self.as_ref().transform().wrap()
    }
}

impl Transform<sqf_ast::Stmt> for dgen_ast::Stmt {
    fn transform(&self) -> sqf_ast::Stmt {
        match self {
            dgen_ast::Stmt::Expr(e)                                     => sqf_ast::Stmt::Expr(e.transform()),
            dgen_ast::Stmt::VarDecl(_, name, value)                     => sqf_ast::Stmt::VarDecl(format!("_{}", name), value.transform()),
            dgen_ast::Stmt::Assign(name, value)                         => sqf_ast::Stmt::Assign(format!("_{}", name), value.transform()),
            dgen_ast::Stmt::Block(v)                                    => sqf_ast::Stmt::Block(v.transform()),
            dgen_ast::Stmt::Program(v)                                  => sqf_ast::Stmt::Program(v.transform()),
            dgen_ast::Stmt::FuncDef(_, name, params, body)              => sqf_ast::Stmt::FuncDef(format!("_{}", name), params.transform(), body.transform()),
            dgen_ast::Stmt::Return(e)                                   => sqf_ast::Stmt::Return(e.transform()),
            dgen_ast::Stmt::Break                                       => sqf_ast::Stmt::Break,
            dgen_ast::Stmt::Continue                                    => sqf_ast::Stmt::Continue,
            dgen_ast::Stmt::If(condition, if_block, else_block)         => sqf_ast::Stmt::If(condition.transform(), if_block.transform(), else_block.transform()),
            dgen_ast::Stmt::For(init, condition, step, block)           => sqf_ast::Stmt::For(init.transform(), condition.transform(), step.transform(), block.transform()),
            dgen_ast::Stmt::While(condition, block)                     => sqf_ast::Stmt::While(condition.transform(), block.transform()),
            dgen_ast::Stmt::FuncDecl { .. }                             => sqf_ast::Stmt::Dummy, // SQF doesn't support function declarations
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
            dgen_ast::Expr::Number(n)                       => sqf_ast::Expr::Number(*n),
            dgen_ast::Expr::Bool(b)                         => sqf_ast::Expr::Bool(*b),
            dgen_ast::Expr::String(s)                       => sqf_ast::Expr::String(s.clone()),
            dgen_ast::Expr::Identifier(id)                  => sqf_ast::Expr::Identifier(format!("_{}", id)),
            dgen_ast::Expr::UnaryOp(op, expr, is_postfix)   => {
                match op {
                    Operator::Inc => sqf_ast::Expr::BinaryOp(Operator::Add, expr.transform(), sqf_ast::Expr::Number(1.0).wrap()),
                    Operator::Dec => sqf_ast::Expr::BinaryOp(Operator::Sub, expr.transform(), sqf_ast::Expr::Number(1.0).wrap()),
                    _ => sqf_ast::Expr::UnaryOp(op.clone(), expr.transform(), *is_postfix)
                }
            },
            dgen_ast::Expr::BinaryOp(op, left, right)       => sqf_ast::Expr::BinaryOp(op.clone(), left.transform(), right.transform()),
            dgen_ast::Expr::FuncCall(name, args)            => sqf_ast::Expr::FuncCall(format!("_{}", name), args.transform()),
            dgen_ast::Expr::Array(v)                        => sqf_ast::Expr::Array(v.transform()),
            dgen_ast::Expr::ArrayAccess(array, index)       => sqf_ast::Expr::ArrayAccess(format!("_{}", array), index.transform())
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
            dgen_ast::Type::Array(t)    => sqf_ast::Type::Array(t.transform().wrap()),
            dgen_ast::Type::Object      => sqf_ast::Type::Object,
            dgen_ast::Type::Void        => sqf_ast::Type::Void,
        }
    }
}
