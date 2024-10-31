use std::io;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::pratt_parser::PrattParser;
use pest::Parser;

use thiserror::Error;

use crate::generic::*;
use crate::dgen_ast::*;
use crate::boxable::Boxable;

#[derive(pest_derive::Parser)]
#[grammar = "dgen.pest"]
pub struct CLikeParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        PrattParser::new()
            .op(Op::infix(or, Left))
            .op(Op::infix(and, Left))
            .op(Op::infix(eq, Left) | Op::infix(neq, Left))
            .op(Op::infix(lt, Left) | Op::infix(lte, Left) | Op::infix(gt, Left) | Op::infix(gte, Left))
            .op(Op::infix(add, Left) | Op::infix(sub, Left))
            .op(Op::infix(mul, Left) | Op::infix(div, Left) | Op::infix(mmod, Left))
            .op(Op::prefix(inc) | Op::prefix(dec) | Op::prefix(neg) | Op::prefix(not))
            .op(Op::postfix(inc) | Op::postfix(dec))
    };
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to parse: {0}")]
    ParseFailure(String),

    #[error("Can't open file: {0}")]
    FileOpenErr(String),

    #[error("Unknown error occurred")]
    Unknown,
}

impl From<ParserError> for io::Error {
    fn from(err: ParserError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, format!("Parser error: {}", err))
    }
}

fn parse_type(rule: Pair<Rule>) -> Type {
    match rule.as_rule() {
        Rule::tnum          => Type::Number,
        Rule::tstr          => Type::String,
        Rule::tbool         => Type::Boolean,
        Rule::tvoid         => Type::Void,
        Rule::tobj          => Type::Object,
        _ => unreachable!(),
    }
}

fn parse_func_call(mut pairs: Pairs<Rule>) -> Expr {
    let name = pairs.next().unwrap().as_str().to_string();
    let args = parse_stmt(pairs.next().unwrap()).into_box();

    Expr::FuncCall { name, args }
}

fn parse_expr(pairs: Pairs<Rule>, pratt: &PrattParser<Rule>) -> Expr {
    use Expr::*;
    use Operator::*;

    pratt
        .map_primary(|primary| match primary.as_rule() {
            Rule::btrue         => Bool(true),
            Rule::bfalse        => Bool(false),
            Rule::number        => Number(primary.as_str().parse().unwrap()),
            Rule::string        => String(primary.as_str().to_string().replace("\"", "")),
            Rule::identifier    => Identifier(primary.as_str().to_string()),
            Rule::expr          => parse_expr(primary.into_inner(), pratt), // from "(" ~ expr ~ ")"
            Rule::func_call     => parse_func_call(primary.into_inner()),
            _                   => unreachable!(),
        })

        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::neg           => UnaryOp { op: Neg, expr: rhs.into_box(), is_postfix: false },
            Rule::inc           => UnaryOp { op: Inc, expr: rhs.into_box(), is_postfix: false },
            Rule::dec           => UnaryOp { op: Dec, expr: rhs.into_box(), is_postfix: false },
            Rule::not           => UnaryOp { op: Not, expr: rhs.into_box(), is_postfix: false },
            _                   => unreachable!(),
        })

        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::inc           => UnaryOp { op: Inc, expr: lhs.into_box(), is_postfix: true },
            Rule::dec           => UnaryOp { op: Dec, expr: lhs.into_box(), is_postfix: true },
            _                   => unreachable!(),
        })

        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::add           => BinaryOp { op: Add, left: lhs.into_box(), right: rhs.into_box() },
            Rule::sub           => BinaryOp { op: Sub, left: lhs.into_box(), right: rhs.into_box() },
            Rule::mul           => BinaryOp { op: Mul, left: lhs.into_box(), right: rhs.into_box() },
            Rule::div           => BinaryOp { op: Div, left: lhs.into_box(), right: rhs.into_box() },
            Rule::mmod          => BinaryOp { op: Mod, left: lhs.into_box(), right: rhs.into_box() },
            Rule::and           => BinaryOp { op: And, left: lhs.into_box(), right: rhs.into_box() },
            Rule::or            => BinaryOp { op: Or,  left: lhs.into_box(), right: rhs.into_box() },
            Rule::eq            => BinaryOp { op: Eq,  left: lhs.into_box(), right: rhs.into_box() },
            Rule::neq           => BinaryOp { op: Neq, left: lhs.into_box(), right: rhs.into_box() },
            Rule::lt            => BinaryOp { op: Lt,  left: lhs.into_box(), right: rhs.into_box() },
            Rule::gt            => BinaryOp { op: Gt,  left: lhs.into_box(), right: rhs.into_box() },
            Rule::lte           => BinaryOp { op: Lte, left: lhs.into_box(), right: rhs.into_box() },
            Rule::gte           => BinaryOp { op: Gte, left: lhs.into_box(), right: rhs.into_box() },
            _                   => unreachable!(),
        })

        .parse(pairs)
}

fn parse_stmt(stmt: Pair<Rule>) -> Stmt {
    let mut inner = stmt.clone().into_inner();

    match stmt.as_rule() {
        Rule::var_decl => {
            let typename = parse_type(inner.next().unwrap());
            let name = inner.next().unwrap().as_str().to_string();

            let value = if inner.peek() == None {
                None
            } else {
                Some(parse_expr(inner.next().unwrap().into_inner(), &PRATT_PARSER))
            };

            Stmt::VarDecl { typename, name, value }
        }

        Rule::assignment => {
            let name = inner.next().unwrap().as_str().to_string();
            let value = parse_expr(inner.next().unwrap().into_inner(), &PRATT_PARSER);

            Stmt::Assign { name, value }
        }

        Rule::expr => {
            Stmt::Expr(parse_expr(stmt.into_inner(), &PRATT_PARSER))
        }

        Rule::func_def => {
            let return_type = parse_type(inner.next().unwrap());
            let name = inner.next().unwrap().as_str().to_string();
            let params = parse_stmt(inner.next().unwrap()).into_box();
            let body = parse_stmt(inner.next().unwrap()).into_box();

            Stmt::FuncDef { return_type, name, params, body }
        }

        Rule::func_decl => {
            let return_type = parse_type(inner.next().unwrap());
            let name = inner.next().unwrap().as_str().to_string();
            let params = parse_stmt(inner.next().unwrap()).into_box();

            Stmt::FuncDecl { return_type, name, params }
        }

        Rule::param_list => {
            let mut params = Vec::new();

            for param in inner {
                let mut param_inner = param.into_inner();
                let typename = parse_type(param_inner.next().unwrap());
                let name = param_inner.next().unwrap().as_str().to_string();

                params.push(Stmt::VarDecl { typename, name, value: None });
            };

            Stmt::ParamList(params)
        }

        Rule::compound_stmt => Stmt::Block(inner
                                                .into_iter()
                                                .map(|param| parse_stmt(param))
                                                .collect()),
        Rule::type_list     => Stmt::TypeList(inner
                                                .into_iter()
                                                .map(|param| parse_type(param))
                                                .collect()),
        Rule::expr_list     => Stmt::ExprList(inner
                                                .into_iter()
                                                .map(|param| parse_expr(param.into_inner(), &PRATT_PARSER))
                                                .collect()),
        Rule::return_stmt   => Stmt::Return(parse_expr(inner.next().unwrap().into_inner(), &PRATT_PARSER)),
        _ => {
            println!("Unhandled rule: {:?}", stmt.as_rule());
            unreachable!()
        }
    }
}

pub fn parse(src: String) -> Result<Stmt, io::Error> {
    match CLikeParser::parse(Rule::program, &src) {
        Ok(pairs) => Ok(Stmt::Program(pairs.into_iter().filter_map(|pair| {
            if pair.as_rule() != Rule::EOI {
                Some(parse_stmt(pair))
            } else {
                None
            }
        }).collect())),
        Err(e) => Err(ParserError::ParseFailure(e.to_string()).into())
    }
}

pub fn parse_file(path: String) -> Result<Stmt, io::Error> {
    match std::fs::read_to_string(path) {
        Ok(src) => parse(src),
        Err(e) => Err(ParserError::FileOpenErr(e.to_string()).into()),
    }
}