use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::pratt_parser::PrattParser;
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "dgen.pest"]
pub struct CLikeParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        PrattParser::new()
            .op(Op::infix(add, Left) | Op::infix(sub, Left))
            .op(Op::infix(mul, Left) | Op::infix(div, Left))
            .op(Op::infix(pow, Right))
            .op(Op::prefix(neg) | Op::prefix(inc) | Op::prefix(dec))
            .op(Op::postfix(inc) | Op::postfix(dec))
    };
}

#[derive(Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Inc,
    Dec,
    Neg,
}

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Bool(bool),
    String(String),
    Identifier(String),
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

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    VarDecl {
        typename: String,
        name: String,
        value: Expr
    },
    Assign {
        name: String,
        value: Expr
    },
    Block(Vec<Stmt>)
}

fn parse_expr(pairs: Pairs<Rule>, pratt: &PrattParser<Rule>) -> Expr {
    pratt
        .map_primary(|primary| match primary.as_rule() {
            Rule::number        => Expr::Number(primary.as_str().parse().unwrap()),
            Rule::boolean       => Expr::Bool(primary.as_str().parse().unwrap()),
            Rule::string        => Expr::String(primary.as_str().to_string()),
            Rule::identifier    => Expr::Identifier(primary.as_str().to_string()),
            Rule::expr          => parse_expr(primary.into_inner(), pratt), // from "(" ~ expr ~ ")"
            _                   => unreachable!(),
        })

        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::neg           => Expr::UnaryOp { op: Operator::Neg, expr: Box::new(rhs), is_postfix: false },
            Rule::inc           => Expr::UnaryOp { op: Operator::Inc, expr: Box::new(rhs), is_postfix: false },
            Rule::dec           => Expr::UnaryOp { op: Operator::Dec, expr: Box::new(rhs), is_postfix: false },
            _                   => unreachable!(),
        })

        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::inc           => Expr::UnaryOp { op: Operator::Inc, expr: Box::new(lhs), is_postfix: true },
            Rule::dec           => Expr::UnaryOp { op: Operator::Dec, expr: Box::new(lhs), is_postfix: true },
            _                   => unreachable!(),
        })

        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::add           => Expr::BinaryOp { op: Operator::Add, left: Box::new(lhs), right: Box::new(rhs) },
            Rule::sub           => Expr::BinaryOp { op: Operator::Sub, left: Box::new(lhs), right: Box::new(rhs) },
            Rule::mul           => Expr::BinaryOp { op: Operator::Mul, left: Box::new(lhs), right: Box::new(rhs) },
            Rule::div           => Expr::BinaryOp { op: Operator::Div, left: Box::new(lhs), right: Box::new(rhs) },
            Rule::pow           => Expr::BinaryOp { op: Operator::Pow, left: Box::new(lhs), right: Box::new(rhs) },
            _                   => unreachable!(),
        })

        .parse(pairs)
}

fn parse_stmt(stmt: Pair<Rule>) -> Stmt {
    match stmt.as_rule() {
        Rule::var_decl => {
            let mut inner = stmt.into_inner();
            let typename = inner.next().unwrap().as_str().to_string();
            let name = inner.next().unwrap().as_str().to_string();
            let expr = parse_expr(inner.next().unwrap().into_inner(), &PRATT_PARSER);

            Stmt::VarDecl {
                typename: typename,
                name: name,
                value: expr
            }
        }

        Rule::assignment => {
            let mut inner = stmt.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let expr = parse_expr(inner.next().unwrap().into_inner(), &PRATT_PARSER);

            Stmt::Assign {
                name: name,
                value: expr
            }
        }

        _ => {
            println!("Unhandled rule: {:?}", stmt.as_rule());
            unreachable!()
        }
    }
}

pub fn parse(src: String) -> Stmt {
    match CLikeParser::parse(Rule::program, &src) {
        Ok(pairs) => {
            let mut block: Vec<Stmt> = Vec::new();

            for pair in pairs {
                if pair.as_rule() != Rule::EOI {
                    block.push(parse_stmt(pair));
                }
            }

            Stmt::Block(block)
        }

        Err(e) => {
            eprintln!("Parse failed: {:?}", e);
            unreachable!()
        }
    }
}