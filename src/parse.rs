use crate::{Exp, Exp::*};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until, take_while},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace1},
    combinator::{all_consuming, map, map_res, opt, recognize, value},
    error::Error,
    multi::many0,
    sequence::{delimited, pair, preceded, tuple},
    Finish, IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Bexp {
    Binary(Box<Bexp>, Op, Box<Bexp>),
    Parens(Box<Bexp>),
    Bool(bool),
    Int(i64),
    Nil,
    Str(String),
    Var(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Op {
    In,
    Let,
    Select,
    Where,
    Union,
    Difference,
    Product,
    Table,
    Item,
    Or,
    Equals,
    And,
    App,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Op {
    pub fn assoc(&self) -> Side {
        match *self {
            Op::In => Side::Right,
            Op::Let => Side::Left,
            Op::Select => Side::Left,
            Op::Where => Side::Left,
            Op::Union => Side::Left,
            Op::Difference => Side::Left,
            Op::Product => Side::Left,
            Op::Table => Side::Left,
            Op::Item => Side::Right,
            Op::Or => Side::Left,
            Op::And => Side::Left,
            Op::Equals => Side::Left,
            Op::App => Side::Left,
        }
    }

    pub fn prec(&self) -> u32 {
        match *self {
            Op::In => 1,
            Op::Let => 2,
            Op::Select => 3,
            Op::Where => 4,
            Op::Union => 5,
            Op::Difference => 6,
            Op::Product => 7,
            Op::Table => 8,
            Op::Item => 9,
            Op::Or => 10,
            Op::And => 11,
            Op::Equals => 12,
            Op::App => 13,
        }
    }
}

pub fn parse(input: &str) -> Result<Exp, String> {
    match all_consuming(parse_bexp)(input).finish() {
        Ok((_, bexp)) => parse_exp(bexp),
        Result::Err(Error { input, code }) => Err(format!("{:?}, input: {:?}", code, input)),
    }
}

fn parse_exp(bexp: Bexp) -> Result<Exp, String> {
    match bexp {
        Bexp::Binary(l, op, r) => match op {
            Op::In => match *l {
                Bexp::Binary(var, Op::Let, exp) => match parse_exp(*var)? {
                    Var(var) => Ok(Let(
                        var,
                        Box::new(parse_exp(*exp)?),
                        Box::new(parse_exp(*r)?),
                    )),
                    exp => Err(format!("expected var, got {:?}", exp)),
                },
                bexp => Err(format!("expected let, got {:?}", bexp)),
            },
            Op::Let => Err("let not allowed here".to_string()),
            Op::Select => Ok(Select(parse_var_list(*l)?, Box::new(parse_exp(*r)?))),
            Op::Where => Ok(Where(Box::new(parse_exp(*l)?), Box::new(parse_exp(*r)?))),
            Op::Union => Ok(Union(Box::new(parse_exp(*l)?), Box::new(parse_exp(*r)?))),
            Op::Difference => Ok(Difference(
                Box::new(parse_exp(*l)?),
                Box::new(parse_exp(*r)?),
            )),
            Op::Product => Ok(Product(Box::new(parse_exp(*l)?), Box::new(parse_exp(*r)?))),
            Op::Table => Ok(Table(parse_var_list(*l)?, parse_exp_list(*r)?)),
            Op::Item => Err("item not allowed here".to_string()),
            Op::Or => Ok(Or(Box::new(parse_exp(*l)?), Box::new(parse_exp(*r)?))),
            Op::Equals => Ok(Equals(Box::new(parse_exp(*l)?), Box::new(parse_exp(*r)?))),
            Op::And => Ok(And(Box::new(parse_exp(*l)?), Box::new(parse_exp(*r)?))),
            Op::App => match parse_exp(*l)? {
                Var(var) => match var.as_str() {
                    "not" => Ok(Not(Box::new(parse_exp(*r)?))),
                    s => Err(format!("unknown function: {}", s)),
                },
                exp => Err(format!("cannot apply {:?}", exp)),
            },
        },
        Bexp::Parens(bexp) => parse_exp(*bexp),
        Bexp::Bool(bool) => Ok(Bool(bool)),
        Bexp::Int(int) => Ok(Int(int)),
        Bexp::Nil => Ok(Table(vec![], vec![])),
        Bexp::Str(str) => Ok(Str(str)),
        Bexp::Var(var) => Ok(Exp::Var(var)),
    }
}

fn parse_var_list(bexp: Bexp) -> Result<Vec<String>, String> {
    match bexp {
        Bexp::Nil => Ok(vec![]),
        Bexp::Var(var) => Ok(vec![var]),
        Bexp::Binary(var, Op::Item, vars) => match *var {
            Bexp::Var(var) => {
                let mut result = vec![var];
                result.append(&mut parse_var_list(*vars)?);
                Ok(result)
            }
            _ => Err("expected variable".to_string()),
        },
        _ => Err("expected variables".to_string()),
    }
}

fn parse_exp_list(bexp: Bexp) -> Result<Vec<Exp>, String> {
    match bexp {
        Bexp::Nil => Ok(vec![]),
        Bexp::Binary(exp, Op::Item, exps) => {
            let exp = parse_exp(*exp)?;
            let mut result = vec![exp];
            result.append(&mut parse_exp_list(*exps)?);
            Ok(result)
        }
        exp => Ok(vec![parse_exp(exp)?]),
    }
}

fn parse_bexp(input: &str) -> IResult<&str, Bexp> {
    let (input, _) = junk(input)?;
    let (input, first) = parse_atom(input)?;
    let (input, rest) = many0(pair(preceded(junk, parse_op), preceded(junk, parse_atom)))(input)?;
    let (input, _) = junk(input)?;

    let exp = re_associate(left_associate(first, rest));

    Ok((input, exp))
}

fn parse_atom(input: &str) -> IResult<&str, Bexp> {
    alt((
        parse_parens,
        parse_bool,
        parse_int,
        parse_nil,
        parse_str,
        parse_var,
    ))(input)
}

fn parse_parens(input: &str) -> IResult<&str, Bexp> {
    map(delimited(char('('), parse_bexp, char(')')), |exp| {
        Bexp::Parens(Box::new(exp))
    })(input)
}

fn parse_bool(input: &str) -> IResult<&str, Bexp> {
    alt((
        value(Bexp::Bool(true), tag("true")),
        value(Bexp::Bool(false), tag("false")),
    ))(input)
}

fn parse_int(input: &str) -> IResult<&str, Bexp> {
    map_res(recognize(pair(opt(tag("-")), digit1)), |s: &str| {
        s.parse().map(Bexp::Int)
    })(input)
}

fn parse_nil(input: &str) -> IResult<&str, Bexp> {
    value(Bexp::Nil, tag("nil"))(input)
}

fn parse_str(input: &str) -> IResult<&str, Bexp> {
    map(delimited(tag("'"), many0(is_not("'")), tag("'")), |s| {
        Bexp::Str(s.concat())
    })(input)
}

fn parse_var(input: &str) -> IResult<&str, Bexp> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| Bexp::Var(s.to_string()),
    )(input)
}

fn parse_op(input: &str) -> IResult<&str, Op> {
    alt((
        value(Op::In, tag(";")),
        value(Op::Equals, tag("==")),
        value(Op::Let, tag("=")),
        value(Op::Select, tag("<-")),
        value(Op::Where, tag("?")),
        value(Op::Union, tag("+")),
        value(Op::Difference, tag("-")),
        value(Op::Product, tag("*")),
        value(Op::Table, tag(":")),
        value(Op::Item, tag(",")),
        value(Op::Or, tag("||")),
        value(Op::And, tag("&&")),
        value(Op::App, tag("")),
    ))(input)
}

fn left_associate(first: Bexp, rest: Vec<(Op, Bexp)>) -> Bexp {
    rest.into_iter().fold(first, |acc, (op, exp)| {
        Bexp::Binary(Box::new(acc), op, Box::new(exp))
    })
}

fn re_associate(exp: Bexp) -> Bexp {
    // (a l b) r c

    let Bexp::Binary(left, r, c) = exp else {
        return exp;
    };
    let c = re_associate(*c);
    let left = re_associate(*left);
    let Bexp::Binary(a, l, b) = left else {
        return Bexp::Binary(Box::new(left), r, Box::new(c));
    };

    if r.prec() > l.prec() || r.prec() == l.prec() && r.assoc() == Side::Right {
        // a l (b r c)
        let left = a;
        let right = Bexp::Binary(b, r, Box::new(c));
        re_associate(Bexp::Binary(left, l, Box::new(right)))
    } else {
        // (a l b) r c
        let left = Bexp::Binary(a, l, b);
        let right = c;
        Bexp::Binary(Box::new(left), r, Box::new(right))
    }
}

fn junk(input: &str) -> IResult<&str, ()> {
    value(
        (),
        many0(alt((whitespace, line_comment, multi_line_comment))),
    )(input)
}

fn whitespace(input: &str) -> IResult<&str, ()> {
    value((), multispace1)(input)
}

fn line_comment(input: &str) -> IResult<&str, ()> {
    value((), pair(tag("--"), take_while(|c| c != '\n')))(input)
}

fn multi_line_comment(input: &str) -> IResult<&str, ()> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(input)
}
