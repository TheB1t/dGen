use crate::parser::*;

trait Evaluate {
    fn eval(self) -> Expr;
}

impl Evaluate for Expr {
    fn eval(self) -> Expr {
        use Expr::*;
        use Operator::*;

        match self {
            UnaryOp { op, expr, is_postfix } => {
                let e_expr = expr.eval();

                match (op.clone(), e_expr.clone()) {
                    (Neg, Number(n)) => Number(-n),
                    (Not, Bool(b))   => Bool(!b),
                    _ => UnaryOp { op, expr: Box::new(e_expr), is_postfix },
                }
            },

            BinaryOp { op, left, right } => {
                let e_left = left.eval();
                let e_right = right.eval();

                match (op.clone(), e_left.clone(), e_right.clone()) {
                    (Add, Number(l), Number(r)) => Number(l + r),
                    (Sub, Number(l), Number(r)) => Number(l - r),
                    (Mul, Number(l), Number(r)) => Number(l * r),
                    (Div, Number(l), Number(r)) => Number(l / r),
                    (Mod, Number(l), Number(r)) => Number(l % r),
                    (Eq,  Number(l), Number(r)) => Bool(l == r),
                    (Neq, Number(l), Number(r)) => Bool(l != r),
                    (Gt,  Number(l), Number(r)) => Bool(l > r),
                    (Lt,  Number(l), Number(r)) => Bool(l < r),
                    (Lte, Number(l), Number(r)) => Bool(l <= r),
                    (Gte, Number(l), Number(r)) => Bool(l >= r),
                    (And, Bool(l), Bool(r))     => Bool(l && r),
                    (Or,  Bool(l), Bool(r))     => Bool(l || r),
                    (Eq,  Bool(l), Bool(r))     => Bool(l == r),
                    (Neq, Bool(l), Bool(r))     => Bool(l != r),
                    (Add, String(l), String(r)) => String(format!("{}{}", l, r)),
                    (Eq,  String(l), String(r)) => Bool(l == r),
                    (Neq, String(l), String(r)) => Bool(l != r),

                    _ => BinaryOp {
                        op,
                        left: Box::new(e_left),
                        right: Box::new(e_right),
                    },
                }
            },

            _ => self,
        }
    }
}

pub fn optimize(root: Stmt) -> Stmt {
    match root {
        Stmt::Block(stmts)                      => Stmt::Block(stmts
                                                                .into_iter()
                                                                .map(|stmt| optimize(stmt))
                                                                .collect()),
        Stmt::Expr(expr)                        => Stmt::Expr(expr.eval()),
        Stmt::Assign { name, value }            => Stmt::Assign {
            name,
            value: value.eval()
        },
        Stmt::VarDecl { typename, name, value } => Stmt::VarDecl {
            typename,
            name,
            value: value.eval()
        }
    }
}
