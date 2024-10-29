use std::usize;

use crate::sqf_ast::*;

struct CodeGenerator {
    indent_level: usize,
}

impl CodeGenerator {
    fn new() -> Self {
        CodeGenerator { indent_level: 0 }
    }

    fn indent(&self, off: usize) -> String {
        "    ".repeat(self.indent_level - 1 + off)
    }

    fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrease_indent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }
}

trait SQFCodeGen {
    fn generate_expr(&mut self, expr: &Expr) -> String;
    fn generate_stmt(&mut self, stmt: &Stmt) -> String;
    fn generate_operator(&mut self, op: &Operator) -> String;
}

impl SQFCodeGen for CodeGenerator {
    fn generate_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Number(n) => n.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::String(s) => format!("\"{}\"", s),
            Expr::Identifier(id) => id.clone(),
            Expr::UnaryOp { op, expr, is_postfix } => {
                let op_str = self.generate_operator(op);
                let expr_str = self.generate_expr(expr);
                if *is_postfix {
                    format!("{}{}", expr_str, op_str)
                } else {
                    format!("{}{}", op_str, expr_str)
                }
            }
            Expr::BinaryOp { op, left, right } => {
                let left_str = self.generate_expr(left);
                let right_str = self.generate_expr(right);
                let op_str = self.generate_operator(op);
                format!("({}) {} ({})", left_str, op_str, right_str)
            }
            Expr::Stmt(stmt) => self.generate_stmt(stmt),
        }
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Dummy => "".to_string(),
            Stmt::Expr(expr) => format!("{}{}", self.indent(0), self.generate_expr(expr)),
            Stmt::VarDecl { name, value } => match value {
                Some(val) => format!("{}private {} = {};", self.indent(0), name, self.generate_expr(val)),
                None => format!("{}private {};", self.indent(0), name),
            },
            Stmt::Assign { name, value } => format!("{}{} = {};", self.indent(0), name, self.generate_expr(value)),
            Stmt::FuncDef { name, params, body } => {
                let params_str = self.generate_stmt(params);
                let body_str = self.generate_stmt(body);
                format!(
                    "{}{} = {{\n{}\n{}\n{}}};",
                    self.indent(0),
                    name,
                    params_str,
                    body_str,
                    self.indent(0)
                )
            }
            Stmt::FuncCall { name, args } => format!("[{}] call {}", self.generate_stmt(args), name),
            Stmt::ParamList(params) => format!("{}params [\"{}\"];", self.indent(1), params.join("\", \"")),
            Stmt::ExprList(exprs) => exprs.iter().map(|e| self.generate_expr(e)).collect::<Vec<_>>().join(", "),
            Stmt::Block(stmts) => {
                self.increase_indent();
                let stmts_str = stmts.iter().map(|stmt| self.generate_stmt(stmt)).collect::<Vec<_>>().join("\n");
                let res = format!("{}{};", self.indent(0), stmts_str);
                self.decrease_indent();
                res
            }
            Stmt::Return(expr) => self.generate_expr(expr),
            _ => "".to_string()
        }
    }

    fn generate_operator(&mut self, op: &Operator) -> String {
        match op {
            Operator::Add => "+".to_string(),
            Operator::Sub => "-".to_string(),
            Operator::Mul => "*".to_string(),
            Operator::Div => "/".to_string(),
            Operator::Mod => "%".to_string(),
            Operator::Neg => "-".to_string(),
            Operator::Not => "!".to_string(),
            Operator::And => "&&".to_string(),
            Operator::Or  => "||".to_string(),
            Operator::Eq  => "==".to_string(),
            Operator::Neq => "!=".to_string(),
            Operator::Lt  => "<".to_string(),
            Operator::Gt  => ">".to_string(),
            Operator::Lte => "<=".to_string(),
            Operator::Gte => ">=".to_string(),
        }
    }
}

pub fn generate_sqf_code(ast: &Stmt) -> String {
    let mut generator = CodeGenerator::new();
    generator.generate_stmt(ast)
}