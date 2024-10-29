#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Inc,
    Dec,
    Neg,
    Not,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
}

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
    Stmt(Box<Stmt>),
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Dummy,
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
    FuncCall {
        name: String,
        args: Box<Stmt>,
    },
    TypeList(Vec<Type>),
    ParamList(Vec<Stmt>),
    ExprList(Vec<Expr>),
    Block(Vec<Stmt>),
    Return(Expr)
}