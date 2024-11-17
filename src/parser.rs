use std::io;

use std::fmt::Debug;
use std::hash::Hash;

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

fn pair_to_string<'a, Rule>(pair: Pair<'a, Rule>) -> String
where
    Rule: Debug + Clone + Copy + Hash + Ord,
{
    pair.as_str().to_string()
}

fn pair_to_type_array<'a, Rule>(pair: Pair<'a, Rule>) -> Vec<Type>
where
    Rule: Debug + Clone + Copy + Hash + Ord,
    Type: From<Pair<'a, Rule>>
{
    pair.into_inner().into_iter().map(|pair| pair.into()).collect()
}

fn pair_to_args_array<'a, Rule>(pair: Pair<'a, Rule>) -> Vec<(Type, String)>
where
    Rule: Debug + Clone + Copy + Hash + Ord,
    Type: From<Pair<'a, Rule>>
{
    pair.into_inner().into_iter().map(|pair| {
        let mut inner = pair.into_inner();
        let ty = inner.next().unwrap().into();
        let name = inner.next().unwrap().as_str().to_string();
        (ty, name)
    }).collect()
}

impl From<Pair<'_, Rule>> for Type {
    fn from(pair: Pair<'_, Rule>) -> Self {
        let inner = pair.clone().into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::tnum          => Type::Number,
            Rule::tstr          => Type::String,
            Rule::tbool         => Type::Boolean,
            Rule::tvoid         => Type::Void,
            Rule::tobj          => Type::Object,
            Rule::tarr          => Type::Array(Into::<Type>::into(inner).wrap()),
            _ => {
                println!("Bad type: {:?}", pair);
                unreachable!()
            }
        }
    }
}

impl From<Pair<'_, Rule>> for Expr {
    fn from(pair: Pair<'_, Rule>) -> Self {
        parse_expr(pair.into_inner(), &PRATT_PARSER)
    }
}

impl From<Pair<'_, Rule>> for Stmt {
    fn from(pair: Pair<'_, Rule>) -> Self {
        let mut inner = pair.clone().into_inner();

        match pair.as_rule() {
            Rule::var_decl => {
                let tname   = inner.expect(Rule::r#type);
                let name    = pair_to_string(inner.expect(Rule::identifier));

                let value = if inner.peek() == None {
                    None
                } else {
                    Some(inner.expect(Rule::expr))
                };

                Stmt::VarDecl(tname, name, value)
            }

            Rule::assignment => {
                let name    = pair_to_string(inner.expect(Rule::identifier));
                let value   = inner.expect(Rule::expr);

                Stmt::Assign(name, value)
            }

            Rule::expr => {
                Stmt::Expr(pair.into())
            }

            Rule::func_def => {
                let rtype   = inner.expect(Rule::r#type);
                let name    = pair_to_string(inner.expect(Rule::identifier));
                let params  = pair_to_args_array(inner.expect(Rule::param_list));
                let body : Stmt    = inner.expect(Rule::compound_stmt);

                Stmt::FuncDef(rtype, name, params, body.wrap())
            }

            Rule::func_decl => {
                let rtype   = inner.expect(Rule::r#type);
                let name    = pair_to_string(inner.expect(Rule::identifier));
                let params  = pair_to_type_array(inner.expect(Rule::type_list));

                Stmt::FuncDecl(rtype, name, params)
            }

            // Rule::param_list => {
            //     let mut params = Vec::new();

            //     for param in inner {
            //         let mut param_inner = param.into_inner();
            //         let typename = parse_type(param_inner.to_next());
            //         let name = param_inner.to_next().as_str().to_string();

            //         params.push(Stmt::VarDecl(typename, name, None));
            //     };

            //     Stmt::ParamList(params)
            // }

            Rule::compound_stmt => Stmt::Block(inner
                                                    .into_iter()
                                                    .map(|param| param.into())
                                                    .collect()),

            // Rule::type_list     => Stmt::TypeList(inner
            //                                         .into_iter()
            //                                         .map(|param| parse_type(param))
            //                                         .collect()),

            // Rule::expr_list     => Stmt::ExprList(inner
            //                                         .into_iter()
            //                                         .map(|param| parse_expr(param.into_inner(), &PRATT_PARSER))
            //                                         .collect()),

            Rule::return_stmt   => Stmt::Return(inner.expect(Rule::expr)),
            Rule::if_stmt       => {
                let cond        = inner.expect(Rule::expr);
                let if_block    : Stmt = inner.expect(Rule::compound_stmt);
                let else_block  : Option<Box<Stmt>> = if inner.peek() == None {
                    None
                } else {
                    Some(<Pairs<'_, Rule> as Expectable<Rule, Stmt>>::expect(&mut inner, Rule::compound_stmt).wrap())
                };

                Stmt::If(cond, if_block.wrap(), else_block)
            }
            Rule::for_stmt      => {
                let init    : Stmt = inner.expect(Rule::var_decl);
                let cond    = inner.expect(Rule::expr);
                let step    : Stmt = inner.expect(Rule::assignment);
                let block   : Stmt = inner.expect(Rule::compound_stmt);

                Stmt::For(init.wrap(), cond, step.wrap(), block.wrap())
            }
            Rule::while_stmt    => {
                let cond    = inner.expect(Rule::expr);
                let block   : Stmt = inner.expect(Rule::compound_stmt);

                Stmt::While(cond, block.wrap())
            }
            Rule::break_stmt    => Stmt::Break,
            Rule::continue_stmt => Stmt::Continue,
            _ => {
                println!("Unhandled rule: {:?}", pair.as_rule());
                unreachable!()
            }
        }
    }
}

trait Expectable<I, O> {
    fn expect(&mut self, val: I) -> O;
}

impl<'a, I, O> Expectable<I, O> for Pair<'a, I>
where
    I: Eq + Debug + Clone + Copy + Hash + Ord,
    Pair<'a, I>: Into<O>,
{
    fn expect(&mut self, val: I) -> O {
        if self.as_rule() == val {
            <Pair<'a, I> as Into<O>>::into(self.clone())
        } else {
            panic!("Expected {:?}, got {:?}", val, self.as_rule());
        }
    }
}

impl<'a, I, O> Expectable<I, O> for Pairs<'a, I>
where
    I: Eq + Debug + Clone + Copy + Hash + Ord,
    Pair<'a, I>: Into<O>,
{
    fn expect(&mut self, val: I) -> O {
        let peek = self.peek().unwrap();

        if peek.as_rule() == val {
            <Pair<'a, I> as Into<O>>::into(self.next().unwrap())
        } else {
            panic!("Expected {:?}, got {:?}", val, peek.as_rule());
        }
    }
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to parse input")]
    ParseFailure,

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

fn parse_func_call(mut pairs: Pairs<Rule>) -> Expr {
    let name : Pair<'_, _> = pairs.expect(Rule::identifier);

    let mut args = Vec::new();

    for mut pair in pairs {
        args.push(pair.expect(Rule::expr));
    }

    Expr::FuncCall(name.as_str().to_string(), args)
}

fn parse_array_init(pairs: Pairs<Rule>) -> Expr {
    let mut exprs = Vec::new();

    for mut pair in pairs {
        exprs.push(pair.expect(Rule::expr));
    }

    Expr::Array(exprs)
}

fn parse_array_access(mut pairs: Pairs<Rule>) -> Expr {
    let array       : Pair<'_, _> = pairs.expect(Rule::identifier);
    let index       : Expr = pairs.expect(Rule::expr);

    Expr::ArrayAccess(array.as_str().to_string(), index.wrap())
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
            Rule::array_access  => parse_array_access(primary.into_inner()),
            Rule::array_init    => parse_array_init(primary.into_inner()),
            _                   => unreachable!(),
        })

        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::neg           => UnaryOp(Neg, rhs.wrap(), false),
            Rule::inc           => UnaryOp(Inc, rhs.wrap(), false),
            Rule::dec           => UnaryOp(Dec, rhs.wrap(), false),
            Rule::not           => UnaryOp(Not, rhs.wrap(), false),
            _                   => unreachable!(),
        })

        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::inc           => UnaryOp(Inc, lhs.wrap(), true),
            Rule::dec           => UnaryOp(Dec, lhs.wrap(), true),
            _                   => unreachable!(),
        })

        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::add           => BinaryOp(Add, lhs.wrap(), rhs.wrap()),
            Rule::sub           => BinaryOp(Sub, lhs.wrap(), rhs.wrap()),
            Rule::mul           => BinaryOp(Mul, lhs.wrap(), rhs.wrap()),
            Rule::div           => BinaryOp(Div, lhs.wrap(), rhs.wrap()),
            Rule::mmod          => BinaryOp(Mod, lhs.wrap(), rhs.wrap()),
            Rule::and           => BinaryOp(And, lhs.wrap(), rhs.wrap()),
            Rule::or            => BinaryOp(Or,  lhs.wrap(), rhs.wrap()),
            Rule::eq            => BinaryOp(Eq,  lhs.wrap(), rhs.wrap()),
            Rule::neq           => BinaryOp(Neq, lhs.wrap(), rhs.wrap()),
            Rule::lt            => BinaryOp(Lt,  lhs.wrap(), rhs.wrap()),
            Rule::gt            => BinaryOp(Gt,  lhs.wrap(), rhs.wrap()),
            Rule::lte           => BinaryOp(Lte, lhs.wrap(), rhs.wrap()),
            Rule::gte           => BinaryOp(Gte, lhs.wrap(), rhs.wrap()),
            _                   => unreachable!(),
        })

        .parse(pairs)
}

pub fn parse(src: String) -> Result<Stmt, io::Error> {
    match CLikeParser::parse(Rule::program, &src) {
        Ok(pairs) => Ok(Stmt::Program(pairs.into_iter().filter_map(|pair| {
            if pair.as_rule() != Rule::EOI {
                Some(pair.into())
            } else {
                None
            }
        }).collect())),
        Err(e) => {
            println!("{}", e);
            Err(ParserError::ParseFailure.into())
        }
    }
}

pub fn parse_file(path: String) -> Result<Stmt, io::Error> {
    match std::fs::read_to_string(path) {
        Ok(src) => parse(src),
        Err(e) => Err(ParserError::FileOpenErr(e.to_string()).into()),
    }
}