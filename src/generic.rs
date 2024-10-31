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

impl Operator {
    pub fn to_string(&self) -> String {
        match self {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::Mod => "%",
            Operator::Inc => "++",
            Operator::Dec => "--",
            Operator::Neg => "-",
            Operator::Not => "!",
            Operator::And => "&&",
            Operator::Or  => "||",
            Operator::Eq  => "==",
            Operator::Neq => "!=",
            Operator::Lt  => "<",
            Operator::Gt  => ">",
            Operator::Lte => "<=",
            Operator::Gte => ">=",
        }.to_string()
    }
}