use sdb::{parse, Exp::*};

#[test]
fn test_bool() {
    assert_eq!(parse("true"), Ok(Bool(true)));
    assert_eq!(parse("false"), Ok(Bool(false)));
}

#[test]
fn test_int() {
    assert_eq!(parse("123"), Ok(Int(123)));
    assert_eq!(parse("-42"), Ok(Int(-42)));
}

#[test]
fn test_str() {
    assert_eq!(parse("''"), Ok(Str("".to_string())));
    assert_eq!(parse("'hello'"), Ok(Str("hello".to_string())));
    assert_eq!(parse("'hello world'"), Ok(Str("hello world".to_string())));
}

#[test]
fn test_var() {
    assert_eq!(parse("x"), Ok(Var("x".to_string())));
    assert_eq!(parse("_x_1"), Ok(Var("_x_1".to_string())));
}

#[test]
fn test_comment() {
    assert_eq!(parse("1 -- hello"), Ok(Int(1)));
    assert_eq!(parse("1 --\n"), Ok(Int(1)));
    assert_eq!(parse("1 --"), Ok(Int(1)));
    assert_eq!(parse("not /* hello */ true"), Ok(Not(Box::new(Bool(true)))));
    assert_eq!(parse("/**/1/**/"), Ok(Int(1)));
}

#[test]
fn test_program() {
    let program = Let(
        "Staff".to_string(),
        Box::new(Table(
            vec!["name".to_string(), "id".to_string(), "employed".to_string()],
            vec![
                Str("Alice".to_string()),
                Int(1),
                Bool(true),
                Str("Bob".to_string()),
                Int(2),
                Bool(false),
            ],
        )),
        Box::new(Let(
            "alice_or_bob_employed".to_string(),
            Box::new(Let(
                "alice".to_string(),
                Box::new(Select(
                    vec!["employed".to_string()],
                    Box::new(Where(
                        Box::new(Var("Staff".to_string())),
                        Box::new(Equals(
                            Box::new(Var("name".to_string())),
                            Box::new(Str("Alice".to_string())),
                        )),
                    )),
                )),
                Box::new(Let(
                    "bob".to_string(),
                    Box::new(Select(
                        vec!["employed".to_string()],
                        Box::new(Where(
                            Box::new(Var("Staff".to_string())),
                            Box::new(Equals(
                                Box::new(Var("name".to_string())),
                                Box::new(Str("Bob".to_string())),
                            )),
                        )),
                    )),
                    Box::new(Or(
                        Box::new(Var("alice".to_string())),
                        Box::new(Var("bob".to_string())),
                    )),
                )),
            )),
            Box::new(Var("alice_or_bob_employed".to_string())),
        )),
    );

    assert_eq!(
        parse(
            r#"
/* welcome to
my program */

Staff =
  name, id, employed:
  'Alice', 1, true,
  'Bob', 2, false;

alice_or_bob_employed = (
  alice = employed <- Staff ? name == 'Alice';
  bob = employed <- Staff ? name == 'Bob';
  alice || bob
);

alice_or_bob_employed
"#
        ),
        Ok(program.clone()),
    );

    assert_eq!(
        parse(
            r#"
Staff=name,id,employed:'Alice',1,true,'Bob',2,false;
alice_or_bob_employed=(alice=employed<-Staff?name=='Alice';bob=employed<-Staff?name=='Bob';alice||bob);
alice_or_bob_employed
"#
        ),
        Ok(program),
    );
}
