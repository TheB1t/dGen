use std::usize;

use crate::sqf_ast::*;

impl Expr {
    pub fn generate_sqf(&self, indent: usize, minify: bool) -> String {
        match self {
            Expr::Number(n)         => n.to_string(),
            Expr::Bool(b)           => b.to_string(),
            Expr::String(s)         => format!("\"{}\"", s),
            Expr::Identifier(id)    => id.clone(),
            Expr::FuncCall { name, args }           => format!("([{}] call {})", args.generate_sqf(indent, minify), name),
            Expr::UnaryOp { op, expr, is_postfix }  => {
                if *is_postfix {
                    format!("{}{}", expr.generate_sqf(indent, minify), op.to_string())
                } else {
                    format!("{}{}", op.to_string(), expr.generate_sqf(indent, minify))
                }
            },
            Expr::BinaryOp { left, op, right }      => {
                format!("({}{}{})", left.generate_sqf(indent, minify), op.to_string(), right.generate_sqf(indent, minify))
            }
        }
    }
}

impl Stmt {
    pub fn generate_sqf(&self, indent: usize, minify: bool) -> String {
        let mut indent_str  = "    ".repeat(indent);
        let mut indent_str2 = "    ".repeat(indent + 1);

        if minify {
            indent_str.clear();
            indent_str2.clear();
        }

        let res = match self {
            Stmt::Expr(expr)                => format!("{}{}", indent_str, expr.generate_sqf(indent, minify)),
            Stmt::VarDecl { name, value }   => {
                if value.is_none() {
                    format!("{}private {}", indent_str, name)
                } else {
                    let value_str = value.as_ref().unwrap().generate_sqf(indent, minify);
                    format!("{}private {}={}", indent_str, name, value_str)
                }
            }
            Stmt::Assign { name, value }    => {
                let value_str = value.generate_sqf(indent, minify);
                format!("{}{}={}", indent_str, name, value_str)
            }
            Stmt::FuncDef { name, params, body } => {
                format!(
                    "{}{}={{\n{}\n{}scopeName \"__func__\";\n{}\n{}}}",
                    indent_str,
                    name,
                    params.generate_sqf(indent + 1, minify),
                    indent_str2,
                    body.generate_sqf(indent, minify),
                    indent_str
                )
            }
            Stmt::ParamList(params)         => format!("{}params[\"{}\"];", indent_str, params.join("\",\"")),
            Stmt::ExprList(exprs)           => exprs.iter().map(|e| e.generate_sqf(indent, minify)).collect::<Vec<_>>().join(","),
            Stmt::Block(stmts)              => stmts.iter().map(|stmt| format!("{};", stmt.generate_sqf(indent + 1, minify))).collect::<Vec<_>>().join("\n"),
            Stmt::Program(stmts)            => stmts.iter().map(|stmt| format!("{};", stmt.generate_sqf(indent, minify))).collect::<Vec<_>>().join("\n"),
            Stmt::Return(expr)              => format!("{}{} breakOut \"__func__\"", indent_str, expr.generate_sqf(indent, minify)),
            Stmt::Break                     => format!("{}break", indent_str),
            Stmt::Continue                  => format!("{}continue", indent_str),
            Stmt::If { condition, if_block, else_block } => {
                format!(
                    "{}if({})then{{\n{}\n{}{}}}",
                    indent_str,
                    condition.generate_sqf(indent, minify),
                    if_block.generate_sqf(indent, minify),
                    else_block.as_ref().map(|b| format!("{}}}else{{\n{}\n", indent_str, b.generate_sqf(indent, minify))).unwrap_or_else(|| "".to_string()),
                    indent_str
                )
            },
            Stmt::For { init, condition, step, block } => {
                format!(
                    "{}for [{{{}}},{{{}}},{{{}}}] do {{\n{}\n{}}}",
                    indent_str,
                    init.generate_sqf(indent, minify),
                    condition.generate_sqf(indent, minify),
                    step.generate_sqf(indent, minify),
                    block.generate_sqf(indent, minify),
                    indent_str
                )
            },
            Stmt::While { condition, block } => {
                format!(
                    "{}while{{{}}}do{{\n{}\n{}}}",
                    indent_str,
                    condition.generate_sqf(indent, minify),
                    block.generate_sqf(indent, minify),
                    indent_str
                )
            },
            _ => {
                println!("Warning: Unhandled statement ({:?}) type in generate_sqf", self);
                "".to_string()
            }
        };

        if minify {
            res.replace("\n", "")
        } else {
            res
        }
    }
}