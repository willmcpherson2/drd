use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{alpha1, alphanumeric1, digit1, multispace0},
    combinator::{map, map_res, opt, recognize, value},
    multi::many0,
    sequence::{pair, tuple},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
enum Exp {
    Let(Let),
    Select(Select),
    Where(Where),
    Union(Union),
    Difference(Difference),
    Product(Product),
    Or(Or),
    Equals(Equals),
    And(And),
    Not(Not),
    Table(Table),
    Row(Row),
    Bool(Bool),
    Int(Int),
    Str(Str),
    Var(Var),
}

#[derive(Debug, PartialEq, Clone)]
struct Let(Var, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Select(Vec<Var>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Where(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Union(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Difference(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Product(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Or(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Equals(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct And(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Table(Vec<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Row(Vec<(Var, Exp)>);

#[derive(Debug, PartialEq, Clone)]
struct Not(Box<Exp>);

#[derive(Debug, PartialEq, Copy, Clone)]
struct Bool(bool);

#[derive(Debug, PartialEq, Copy, Clone)]
struct Int(i64);

#[derive(Debug, PartialEq, Clone)]
struct Str(String);

#[derive(Debug, PartialEq, Clone)]
struct Var(String);

fn main() {
    println!(
        "{:?}",
        parse_program(
            "Staff = [{name 'Alice'} {name 'Bob'}]; bob = name <- Staff ? name == 'Bob';"
        )
    );
}

fn parse_program(input: &str) -> IResult<&str, Vec<Exp>> {
    map(many0(tuple((parse_exp, tag(";")))), |exps| {
        exps.into_iter().map(|(exp, _)| exp).collect()
    })(input)
}

fn parse_exp(input: &str) -> IResult<&str, Exp> {
    parse_let(input)
}

fn parse_let(input: &str) -> IResult<&str, Exp> {
    alt((
        map(
            tuple((parse_var, ws, tag("="), ws, parse_select)),
            |(var, _, _, _, exp)| Exp::Let(Let(var, Box::new(exp))),
        ),
        parse_select,
    ))(input)
}

fn parse_select(input: &str) -> IResult<&str, Exp> {
    fn parse_select_vars(input: &str) -> IResult<&str, Vec<Var>> {
        map(many0(tuple((parse_var, ws))), |vars| {
            vars.into_iter().map(|(var, _)| var).collect()
        })(input)
    }

    alt((
        map(
            tuple((parse_select_vars, ws, tag("<-"), ws, parse_select)),
            |(vars, _, _, _, exp)| Exp::Select(Select(vars, Box::new(exp))),
        ),
        parse_where,
    ))(input)
}

fn parse_where(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Where(Where(l, r)),
        parse_union,
        "?",
        parse_where,
    )
}

fn parse_union(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Union(Union(l, r)),
        parse_difference,
        "+",
        parse_union,
    )
}

fn parse_difference(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Difference(Difference(l, r)),
        parse_product,
        "-",
        parse_difference,
    )
}

fn parse_product(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Product(Product(l, r)),
        parse_equals,
        "*",
        parse_product,
    )
}

fn parse_equals(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Equals(Equals(l, r)),
        parse_or,
        "==",
        parse_equals,
    )
}

fn parse_or(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(input, |l, r| Exp::Or(Or(l, r)), parse_and, "|", parse_or)
}

fn parse_and(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(input, |l, r| Exp::And(And(l, r)), parse_not, "&", parse_and)
}

fn parse_binary_op<'a>(
    input: &'a str,
    constructor: fn(Box<Exp>, Box<Exp>) -> Exp,
    parse_left: fn(&str) -> IResult<&str, Exp>,
    op: &'static str,
    parse_right: fn(&str) -> IResult<&str, Exp>,
) -> IResult<&'a str, Exp> {
    alt((
        map(
            tuple((parse_left, ws, tag(op), ws, parse_right)),
            |(l, _, _, _, r)| constructor(Box::new(l), Box::new(r)),
        ),
        parse_left,
    ))(input)
}

fn parse_not(input: &str) -> IResult<&str, Exp> {
    parse_unary_op(input, |exp| Exp::Not(Not(exp)), parse_atom, "!", parse_not)
}

fn parse_unary_op<'a>(
    input: &'a str,
    constructor: fn(Box<Exp>) -> Exp,
    parse_left: fn(&str) -> IResult<&str, Exp>,
    op: &'static str,
    parse_right: fn(&str) -> IResult<&str, Exp>,
) -> IResult<&'a str, Exp> {
    alt((
        map(tuple((tag(op), ws, parse_right)), |(_, _, r)| {
            constructor(Box::new(r))
        }),
        parse_left,
    ))(input)
}

fn parse_atom(input: &str) -> IResult<&str, Exp> {
    alt((
        parse_parens,
        map(parse_table, Exp::Table),
        map(parse_row, Exp::Row),
        map(parse_bool, Exp::Bool),
        map(parse_int, Exp::Int),
        map(parse_str, Exp::Str),
        map(parse_var, Exp::Var),
    ))(input)
}

fn parse_parens(input: &str) -> IResult<&str, Exp> {
    map(
        tuple((tag("("), ws, parse_exp, ws, tag(")"))),
        |(_, _, exp, _, _)| exp,
    )(input)
}

fn parse_table(input: &str) -> IResult<&str, Table> {
    map(
        tuple((tag("["), ws, many0(pair(parse_exp, ws)), tag("]"))),
        |(_, _, exps, _)| Table(exps.into_iter().map(|(exp, _)| exp).collect()),
    )(input)
}

fn parse_row(input: &str) -> IResult<&str, Row> {
    fn parse_row_item(input: &str) -> IResult<&str, (Var, Exp)> {
        map(tuple((parse_var, ws, parse_exp)), |(var, _, exp)| {
            (var, exp)
        })(input)
    }

    map(
        tuple((tag("{"), ws, many0(pair(parse_row_item, ws)), tag("}"))),
        |(_, _, items, _)| Row(items.into_iter().map(|(item, _)| item).collect()),
    )(input)
}

fn parse_bool(input: &str) -> IResult<&str, Bool> {
    alt((
        value(Bool(true), tag("true")),
        value(Bool(false), tag("false")),
    ))(input)
}

fn parse_int(input: &str) -> IResult<&str, Int> {
    fn to_int(s: &str) -> Result<Int, std::num::ParseIntError> {
        s.parse().map(Int)
    }

    map_res(recognize(pair(opt(tag("-")), digit1)), to_int)(input)
}

fn parse_str(input: &str) -> IResult<&str, Str> {
    map(
        tuple((tag("'"), recognize(many0(is_not("'"))), tag("'"))),
        |(_, s, _): (_, &str, _)| Str(s.to_string()),
    )(input)
}

fn parse_var(input: &str) -> IResult<&str, Var> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| Var(s.to_string()),
    )(input)
}

fn multi_line_comment(input: &str) -> IResult<&str, ()> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(input)
}

fn line_comment(input: &str) -> IResult<&str, ()> {
    value((), pair(tag("--"), is_not("\n")))(input)
}

fn ws(input: &str) -> IResult<&str, ()> {
    value((), multispace0)(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::{error::Error, Err};

    #[test]
    fn test_parse_exp() {
        assert_eq!(
            parse_exp("true | false & !true"),
            Ok((
                "",
                Exp::Or(Or(
                    Box::new(Exp::Bool(Bool(true))),
                    Box::new(Exp::And(And(
                        Box::new(Exp::Bool(Bool(false))),
                        Box::new(Exp::Not(Not(Box::new(Exp::Bool(Bool(true))))))
                    )))
                ))
            ))
        );
        assert_eq!(
            parse_exp(
                "alice = id name <- [{id 1 name 'Alice'} {id 2 name 'Bob'}] ? name == 'Alice'"
            ),
            Ok((
                "",
                Exp::Let(Let(
                    Var("alice".to_string()),
                    Box::new(Exp::Select(Select(
                        vec![Var("id".to_string()), Var("name".to_string())],
                        Box::new(Exp::Where(Where(
                            Box::new(Exp::Table(Table(vec![
                                Exp::Row(Row(vec![
                                    (Var("id".to_string()), Exp::Int(Int(1))),
                                    (Var("name".to_string()), Exp::Str(Str("Alice".to_string())))
                                ])),
                                Exp::Row(Row(vec![
                                    (Var("id".to_string()), Exp::Int(Int(2))),
                                    (Var("name".to_string()), Exp::Str(Str("Bob".to_string())))
                                ]))
                            ]))),
                            Box::new(Exp::Equals(Equals(
                                Box::new(Exp::Var(Var("name".to_string()))),
                                Box::new(Exp::Str(Str("Alice".to_string())))
                            )))
                        )))
                    )))
                ))
            ))
        );
    }

    #[test]
    fn test_parse_let() {
        assert_eq!(
            parse_let("x = true"),
            Ok((
                "",
                Exp::Let(Let(Var("x".to_string()), Box::new(Exp::Bool(Bool(true)))))
            ))
        );
        assert_eq!(
            parse_let("x = true | false"),
            Ok((
                "",
                Exp::Let(Let(
                    Var("x".to_string()),
                    Box::new(Exp::Or(Or(
                        Box::new(Exp::Bool(Bool(true))),
                        Box::new(Exp::Bool(Bool(false)))
                    )))
                ))
            ))
        );
    }

    #[test]
    fn test_parse_select() {
        assert_eq!(
            parse_select("x <- true"),
            Ok((
                "",
                Exp::Select(Select(
                    vec![Var("x".to_string())],
                    Box::new(Exp::Bool(Bool(true)))
                ))
            ))
        );
        assert_eq!(
            parse_select("x y <- true"),
            Ok((
                "",
                Exp::Select(Select(
                    vec![Var("x".to_string()), Var("y".to_string())],
                    Box::new(Exp::Bool(Bool(true)))
                ))
            ))
        );
        assert_eq!(
            parse_select("x y z <- true"),
            Ok((
                "",
                Exp::Select(Select(
                    vec![
                        Var("x".to_string()),
                        Var("y".to_string()),
                        Var("z".to_string())
                    ],
                    Box::new(Exp::Bool(Bool(true)))
                ))
            ))
        );
    }

    #[test]
    fn test_parse_or() {
        assert_eq!(
            parse_or("true | false"),
            Ok((
                "",
                Exp::Or(Or(
                    Box::new(Exp::Bool(Bool(true))),
                    Box::new(Exp::Bool(Bool(false)))
                ))
            ))
        );
        assert_eq!(
            parse_or("true & false | !true"),
            Ok((
                "",
                Exp::Or(Or(
                    Box::new(Exp::And(And(
                        Box::new(Exp::Bool(Bool(true))),
                        Box::new(Exp::Bool(Bool(false)))
                    ))),
                    Box::new(Exp::Not(Not(Box::new(Exp::Bool(Bool(true))))))
                ))
            ))
        );
    }

    #[test]
    fn test_parse_and() {
        assert_eq!(
            parse_and("true & false"),
            Ok((
                "",
                Exp::And(And(
                    Box::new(Exp::Bool(Bool(true))),
                    Box::new(Exp::Bool(Bool(false)))
                ))
            ))
        );
        assert_eq!(
            parse_and("true & false & !true"),
            Ok((
                "",
                Exp::And(And(
                    Box::new(Exp::Bool(Bool(true))),
                    Box::new(Exp::And(And(
                        Box::new(Exp::Bool(Bool(false))),
                        Box::new(Exp::Not(Not(Box::new(Exp::Bool(Bool(true))))))
                    )))
                ))
            ))
        );
    }

    #[test]
    fn test_parse_not() {
        assert_eq!(
            parse_not("! true"),
            Ok(("", Exp::Not(Not(Box::new(Exp::Bool(Bool(true)))))))
        );
        assert_eq!(
            parse_not("!x"),
            Ok(("", Exp::Not(Not(Box::new(Exp::Var(Var("x".to_string())))))))
        );
        assert_eq!(
            parse_not("!!x"),
            Ok((
                "",
                Exp::Not(Not(Box::new(Exp::Not(Not(Box::new(Exp::Var(Var(
                    "x".to_string()
                ))))))))
            ))
        );
    }

    #[test]
    fn test_parse_atom() {
        assert_eq!(parse_atom("true"), Ok(("", Exp::Bool(Bool(true)))));
        assert_eq!(parse_atom("123"), Ok(("", Exp::Int(Int(123)))));
        assert_eq!(
            parse_atom("'hello'"),
            Ok(("", Exp::Str(Str("hello".to_string()))))
        );
        assert_eq!(parse_atom("x"), Ok(("", Exp::Var(Var("x".to_string())))));
    }

    #[test]
    fn test_parse_table() {
        assert_eq!(parse_table("[]"), Ok(("", Table(vec![]))));
        assert_eq!(
            parse_table("[true]"),
            Ok(("", Table(vec![Exp::Bool(Bool(true))])))
        );
        assert_eq!(
            parse_table("[ ( true | true & false ) true ]"),
            Ok((
                "",
                Table(vec![
                    Exp::Or(Or(
                        Box::new(Exp::Bool(Bool(true))),
                        Box::new(Exp::And(And(
                            Box::new(Exp::Bool(Bool(true))),
                            Box::new(Exp::Bool(Bool(false)))
                        )))
                    )),
                    Exp::Bool(Bool(true))
                ])
            ))
        );
    }

    #[test]
    fn test_parse_row() {
        assert_eq!(parse_row("{}"), Ok(("", Row(vec![]))));
        assert_eq!(
            parse_row("{ x true }"),
            Ok(("", Row(vec![(Var("x".to_string()), Exp::Bool(Bool(true)))])))
        );
        assert_eq!(
            parse_row("{ x true y false }"),
            Ok((
                "",
                Row(vec![
                    (Var("x".to_string()), Exp::Bool(Bool(true))),
                    (Var("y".to_string()), Exp::Bool(Bool(false)))
                ])
            ))
        );
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("true"), Ok(("", Bool(true))));
        assert_eq!(parse_bool("false"), Ok(("", Bool(false))));
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int("123"), Ok(("", Int(123))));
        assert_eq!(parse_int("-42hello"), Ok(("hello", Int(-42))));
    }

    #[test]
    fn test_parse_str() {
        assert_eq!(parse_str("'hello'"), Ok(("", Str("hello".to_string()))));
        assert_eq!(
            parse_str("'hello'world"),
            Ok(("world", Str("hello".to_string())))
        );
    }

    #[test]
    fn test_parse_var() {
        assert_eq!(parse_var("x"), Ok(("", Var("x".to_string()))));
        assert_eq!(parse_var("_x_1"), Ok(("", Var("_x_1".to_string()))));
    }

    #[test]
    fn test_ws() {
        assert_eq!(ws(" "), Ok(("", ())));
        assert_eq!(ws("\n"), Ok(("", ())));
    }

    #[test]
    fn test_comment() {
        assert_eq!(line_comment("-- hello"), Ok(("", ())));
        assert_eq!(line_comment("-- hello\n"), Ok(("\n", ())));
        assert_eq!(line_comment("-- hello\nworld"), Ok(("\nworld", ())));
    }

    #[test]
    fn test_multi_line_comment() {
        assert_eq!(multi_line_comment("/* hello */"), Ok(("", ())));
        assert_eq!(multi_line_comment("/* hello */world"), Ok(("world", ())));
        assert_eq!(
            multi_line_comment("/* hello"),
            Err(Err::Error(Error {
                input: " hello",
                code: nom::error::ErrorKind::TakeUntil
            }))
        );
    }
}
