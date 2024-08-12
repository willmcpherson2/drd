use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{alpha1, alphanumeric1, digit1, multispace1},
    combinator::{fail, map, map_res, opt, recognize, value},
    multi::many0,
    sequence::{pair, tuple},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
struct Program(Vec<Exp>);

#[derive(Debug, PartialEq, Clone)]
enum Exp {
    Let(Let),
    Select(Select),
    Where(Where),
    Union(Union),
    Difference(Difference),
    Product(Product),
    Table(Table),
    Row(Row),
    Cell(Cell),
    Or(Or),
    Equals(Equals),
    And(And),
    Not(Not),
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
struct Table(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Row(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Cell(Var, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Or(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct Equals(Box<Exp>, Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
struct And(Box<Exp>, Box<Exp>);

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
        "{:#?}",
        parse_program(
            r#"
/* welcome to
my database */

Staff =
  name: 'Alice'; -- first row
  name: 'Bob'    -- second row

bob = name /* add more columns here */ <- Staff ? name == 'Bob'
"#
        )
    );
}

fn parse_program(input: &str) -> IResult<&str, Program> {
    map(tuple((junk, many0(pair(parse_exp, junk)))), |(_, exps)| {
        Program(exps.into_iter().map(|(exp, _)| exp).collect())
    })(input)
}

fn parse_exp(input: &str) -> IResult<&str, Exp> {
    parse_let(input)
}

fn parse_let(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Let(Let(l, Box::new(r))),
        parse_var,
        "=",
        parse_let,
        parse_select,
    )
}

fn parse_select(input: &str) -> IResult<&str, Exp> {
    fn parse_select_vars(input: &str) -> IResult<&str, Vec<Var>> {
        alt((
            |input| {
                parse_binary_op(
                    input,
                    |var, vars| [&[var], &vars[..]].concat(),
                    parse_var,
                    ",",
                    parse_select_vars,
                    |s| fail(s),
                )
            },
            map(parse_var, |var| vec![var]),
        ))(input)
    }

    parse_binary_op(
        input,
        |l, r| Exp::Select(Select(l, Box::new(r))),
        parse_select_vars,
        "<-",
        parse_select,
        parse_where,
    )
}

fn parse_where(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Where(Where(Box::new(l), Box::new(r))),
        parse_union,
        "?",
        parse_where,
        parse_union,
    )
}

fn parse_union(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Union(Union(Box::new(l), Box::new(r))),
        parse_difference,
        "+",
        parse_union,
        parse_difference,
    )
}

fn parse_difference(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Difference(Difference(Box::new(l), Box::new(r))),
        parse_product,
        "-",
        parse_difference,
        parse_product,
    )
}

fn parse_product(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Product(Product(Box::new(l), Box::new(r))),
        parse_table,
        "*",
        parse_product,
        parse_table,
    )
}

fn parse_table(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Table(Table(Box::new(l), Box::new(r))),
        parse_row,
        ";",
        parse_table,
        parse_row,
    )
}

fn parse_row(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Row(Row(Box::new(l), Box::new(r))),
        parse_cell,
        ",",
        parse_row,
        parse_cell,
    )
}

fn parse_cell(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Cell(Cell(l, Box::new(r))),
        parse_var,
        ":",
        parse_cell,
        parse_equals,
    )
}

fn parse_equals(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Equals(Equals(Box::new(l), Box::new(r))),
        parse_or,
        "==",
        parse_equals,
        parse_or,
    )
}

fn parse_or(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::Or(Or(Box::new(l), Box::new(r))),
        parse_and,
        "|",
        parse_or,
        parse_and,
    )
}

fn parse_and(input: &str) -> IResult<&str, Exp> {
    parse_binary_op(
        input,
        |l, r| Exp::And(And(Box::new(l), Box::new(r))),
        parse_not,
        "&",
        parse_and,
        parse_not,
    )
}

fn parse_not(input: &str) -> IResult<&str, Exp> {
    parse_unary_op(
        input,
        |exp| Exp::Not(Not(Box::new(exp))),
        "!",
        parse_not,
        parse_atom,
    )
}

fn parse_atom(input: &str) -> IResult<&str, Exp> {
    alt((
        parse_parens,
        map(parse_bool, Exp::Bool),
        map(parse_int, Exp::Int),
        map(parse_str, Exp::Str),
        map(parse_var, Exp::Var),
    ))(input)
}

fn parse_parens(input: &str) -> IResult<&str, Exp> {
    map(
        tuple((tag("("), junk, parse_exp, junk, tag(")"))),
        |(_, _, exp, _, _)| exp,
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
        tuple((tag("'"), many0(is_not("'")), tag("'"))),
        |(_, s, _)| Str(s.concat()),
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

fn parse_binary_op<'a, L, R, T>(
    input: &'a str,
    constructor: fn(L, R) -> T,
    parse_left: fn(&str) -> IResult<&str, L>,
    op: &'static str,
    parse_right: fn(&str) -> IResult<&str, R>,
    parse_next: fn(&str) -> IResult<&str, T>,
) -> IResult<&'a str, T> {
    alt((
        map(
            tuple((parse_left, junk, tag(op), junk, parse_right)),
            |(l, _, _, _, r)| constructor(l, r),
        ),
        parse_next,
    ))(input)
}

fn parse_unary_op<'a, R, T>(
    input: &'a str,
    constructor: fn(R) -> T,
    op: &'static str,
    parse_right: fn(&str) -> IResult<&str, R>,
    parse_next: fn(&str) -> IResult<&str, T>,
) -> IResult<&'a str, T> {
    alt((
        map(tuple((tag(op), junk, parse_right)), |(_, _, r)| {
            constructor(r)
        }),
        parse_next,
    ))(input)
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
    value((), pair(tag("--"), is_not("\n")))(input)
}

fn multi_line_comment(input: &str) -> IResult<&str, ()> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::{error::Error, Err};

    #[test]
    fn test_parse_program() {
        let program = Program(vec![
            Exp::Let(Let(
                Var("Staff".to_string()),
                Box::new(Exp::Table(Table(
                    Box::new(Exp::Row(Row(
                        Box::new(Exp::Cell(Cell(
                            Var("name".to_string()),
                            Box::new(Exp::Str(Str("Alice".to_string()))),
                        ))),
                        Box::new(Exp::Cell(Cell(
                            Var("id".to_string()),
                            Box::new(Exp::Int(Int(1))),
                        ))),
                    ))),
                    Box::new(Exp::Row(Row(
                        Box::new(Exp::Cell(Cell(
                            Var("name".to_string()),
                            Box::new(Exp::Str(Str("Bob".to_string()))),
                        ))),
                        Box::new(Exp::Cell(Cell(
                            Var("id".to_string()),
                            Box::new(Exp::Int(Int(2))),
                        ))),
                    ))),
                ))),
            )),
            Exp::Let(Let(
                Var("bob".to_string()),
                Box::new(Exp::Select(Select(
                    vec![Var("name".to_string())],
                    Box::new(Exp::Where(Where(
                        Box::new(Exp::Var(Var("Staff".to_string()))),
                        Box::new(Exp::Equals(Equals(
                            Box::new(Exp::Var(Var("name".to_string()))),
                            Box::new(Exp::Str(Str("Bob".to_string()))),
                        ))),
                    ))),
                ))),
            )),
        ]);

        assert_eq!(
            parse_program(
                r#"
/* welcome to
my database */

Staff =
  name: 'Alice', id: 1; -- first row
  name: 'Bob', id: 2    -- second row

bob = name /* columns... */ <- Staff ? name == 'Bob'
"#
            ),
            Ok(("", program.clone()))
        );

        assert_eq!(
            parse_program("Staff=name:'Alice',id:1;name:'Bob',id:2bob=name<-Staff?name=='Bob'"),
            Ok(("", program.clone()))
        );

        assert_eq!(
            parse_program(
                r#"
a = b
c, d <- e
"#
            ),
            Ok((
                "",
                Program(vec![
                    Exp::Let(Let(
                        Var("a".to_string()),
                        Box::new(Exp::Var(Var("b".to_string())))
                    )),
                    Exp::Select(Select(
                        vec![Var("c".to_string()), Var("d".to_string())],
                        Box::new(Exp::Var(Var("e".to_string())))
                    ))
                ])
            ))
        );
    }

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
            parse_select("x, y <- true"),
            Ok((
                "",
                Exp::Select(Select(
                    vec![Var("x".to_string()), Var("y".to_string())],
                    Box::new(Exp::Bool(Bool(true)))
                ))
            ))
        );
        assert_eq!(
            parse_select("x, y, z <- true"),
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
    fn test_parse_table() {
        assert_eq!(
            parse_table("name: 'Alice', id: 1; name: 'Bob', id: 2"),
            Ok((
                "",
                Exp::Table(Table(
                    Box::new(Exp::Row(Row(
                        Box::new(Exp::Cell(Cell(
                            Var("name".to_string()),
                            Box::new(Exp::Str(Str("Alice".to_string())))
                        ))),
                        Box::new(Exp::Cell(Cell(
                            Var("id".to_string()),
                            Box::new(Exp::Int(Int(1)))
                        )))
                    ))),
                    Box::new(Exp::Row(Row(
                        Box::new(Exp::Cell(Cell(
                            Var("name".to_string()),
                            Box::new(Exp::Str(Str("Bob".to_string())))
                        ))),
                        Box::new(Exp::Cell(Cell(
                            Var("id".to_string()),
                            Box::new(Exp::Int(Int(2)))
                        )))
                    )))
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
        assert_eq!(parse_str("''"), Ok(("", Str("".to_string()))));
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
    fn test_junk() {
        assert_eq!(junk(" "), Ok(("", ())));
        assert_eq!(junk("\n"), Ok(("", ())));
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
