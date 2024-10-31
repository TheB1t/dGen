use std::usize;

use crate::sqf_ast::*;

impl Expr {
    pub fn generate_sqf(&self, indent: usize) -> String {
        match self {
            Expr::Number(n)         => n.to_string(),
            Expr::Bool(b)           => b.to_string(),
            Expr::String(s)         => format!("\"{}\"", s),
            Expr::Identifier(id)    => id.clone(),
            Expr::FuncCall { name, args }           => format!("([{}] call {})", args.generate_sqf(indent), name),
            Expr::UnaryOp { op, expr, is_postfix }  => {
                if *is_postfix {
                    format!("{}{}", expr.generate_sqf(indent), op.to_string())
                } else {
                    format!("{}{}", op.to_string(), expr.generate_sqf(indent))
                }
            },
            Expr::BinaryOp { left, op, right }      => {
                format!("({} {} {})", left.generate_sqf(indent), op.to_string(), right.generate_sqf(indent))
            }
        }
    }
}

impl Stmt {
    pub fn generate_sqf(&self, indent: usize) -> String {
        let indent_str  = "    ".repeat(indent);
        let newline     = "\n";

        match self {
            Stmt::Expr(expr)                => format!("{}{}", indent_str, expr.generate_sqf(indent)),
            Stmt::VarDecl { name, value }   => {
                if value.is_none() {
                    format!("{}private {}", indent_str, name)
                } else {
                    let value_str = value.as_ref().unwrap().generate_sqf(indent);
                    format!("{}private {} = {}", indent_str, name, value_str)
                }
            }
            Stmt::Assign { name, value }    => {
                let value_str = value.generate_sqf(indent);
                format!("{}{} = {};", indent_str, name, value_str)
            }
            Stmt::FuncDef { name, params, body } => {
                format!(
                    "{}{} = {{\n{}\n{}\n{}}}",
                    indent_str,
                    name,
                    params.generate_sqf(indent + 1),
                    body.generate_sqf(indent),
                    "    ".repeat(indent)
                )
            }
            Stmt::ParamList(params)         => format!("{}params [\"{}\"];", indent_str, params.join("\", \"")),
            Stmt::ExprList(exprs)           => exprs.iter().map(|e| e.generate_sqf(indent)).collect::<Vec<_>>().join(", "),
            Stmt::Block(stmts)              => stmts.iter().map(|stmt| format!("{};", stmt.generate_sqf(indent + 1))).collect::<Vec<_>>().join("\n"),
            Stmt::Program(stmts)            => stmts.iter().map(|stmt| format!("{};", stmt.generate_sqf(indent))).collect::<Vec<_>>().join("\n"),
            Stmt::Return(expr)              => format!("{}{}", indent_str, expr.generate_sqf(indent)),
            _ => {
                println!("Warning: Unhandled statement ({:?}) type in generate_sqf", self);
                "".to_string()
            }
        }
    }
}