use crate::generic::*;
use crate::dgen_ast::*;
use crate::boxable::Boxable;

impl Expr {
    fn eval(self) -> Expr {
        use Expr::*;
        use Operator::*;

        match self {
            UnaryOp { op, expr, is_postfix } => {
                let e_expr = expr.eval();

                match (op.clone(), e_expr.clone()) {
                    (Neg, Number(n)) => Number(-n),
                    (Not, Bool(b))   => Bool(!b),
                    _ => UnaryOp { op, expr: e_expr.into_box(), is_postfix },
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
                        left: e_left.into_box(),
                        right: e_right.into_box(),
                    },
                }
            },
            FuncCall { name, args }             => FuncCall {
                name,
                args: optimize(*args).into_box()
            },
            _ => self,
        }
    }
}

pub fn optimize(root: Stmt) -> Stmt {
    use Stmt::*;

    match root {
        Block(stmts)                      => Block(stmts
                                                    .into_iter()
                                                    .map(|stmt| optimize(stmt))
                                                    .collect()),
        Program(stmts)                      => Program(stmts
                                                    .into_iter()
                                                    .map(|stmt| optimize(stmt))
                                                    .collect()),
        Expr(expr)                        => Expr(expr.eval()),
        Assign { name, value }            => Assign {
            name,
            value: value.eval()
        },
        VarDecl { typename, name, value } => {
            let res = match value.clone() {
                Option::Some(v) => Some(v.eval()),
                Option::None    => None
            };

            VarDecl { typename, name, value: res }
        },
        ExprList(exprs)                   => ExprList(exprs
                                                        .into_iter()
                                                        .map(|expr| expr.eval())
                                                        .collect()),
        FuncDef { return_type, name, params, body } => FuncDef {
            return_type,
            name,
            params: params,
            body: optimize(*body).into_box()
        },
        Return(expr)                        => Return(expr.eval()),
        If { condition, if_block, else_block } => If {
            condition: condition.eval(),
            if_block: optimize(*if_block).into_box(),
            else_block: else_block.map(|b| optimize(*b).into_box())
        },
        For { init, condition, step, block } => For {
            init: optimize(*init).into_box(),
            condition: condition.eval(),
            step: optimize(*step).into_box(),
            block: optimize(*block).into_box()
        },
        While { condition, block }          => While {
            condition: condition.eval(),
            block: optimize(*block).into_box()
        },
        Break                               => Break,
        Continue                            => Continue,
        FuncDecl { .. }                     => root,
        _ => {
            println!("Optimization is not supported for node {:#?}", root);
            root
        }
    }
}
