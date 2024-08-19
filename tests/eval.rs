use drd::{eval::eval, exp::Exp::*, parse::parse};

#[test]
fn test_select() {
    assert_eq!(
        eval(parse("name <- name, id : 'Alice', 1, 'Bob', 2").unwrap()),
        Ok(Table(
            vec!["name".to_string()],
            vec![Str("Alice".to_string()), Str("Bob".to_string())]
        )),
    );

    assert_eq!(
        eval(parse("id <- name, id : 'Alice', 1, 'Bob', 2").unwrap()),
        Ok(Table(vec!["id".to_string()], vec![Int(1), Int(2)])),
    );

    assert_eq!(
        eval(parse("foo <- name, id : 'Alice', 1, 'Bob', 2").unwrap()),
        Ok(Table(vec!["foo".to_string()], vec![])),
    );
}

#[test]
fn test_where() {
    assert_eq!(
        eval(parse("name, id : 'Alice', 1, 'Bob', 2 ? name == 'Alice'").unwrap()),
        Ok(Table(
            vec!["name".to_string(), "id".to_string()],
            vec![Str("Alice".to_string()), Int(1)]
        )),
    );

    assert_eq!(
        eval(parse("name, id : 'Alice', 1, 'Bob', 2 ? id == 2").unwrap()),
        Ok(Table(
            vec!["name".to_string(), "id".to_string()],
            vec![Str("Bob".to_string()), Int(2)]
        )),
    );

    assert_eq!(
        eval(parse("name, id : 'Alice', 1, 'Bob', 2 ? id == 1 || id == 2").unwrap()),
        Ok(Table(
            vec!["name".to_string(), "id".to_string()],
            vec![
                Str("Alice".to_string()),
                Int(1),
                Str("Bob".to_string()),
                Int(2)
            ]
        )),
    );

    assert_eq!(
        eval(parse("name, id : 'Alice', 1, 'Bob', 2 ? name == 'Foo'").unwrap()),
        Ok(Table(vec!["name".to_string(), "id".to_string()], vec![])),
    );
}

#[test]
fn test_union() {
    assert_eq!(
        eval(parse("name, id : 'Alice', 1 + name, id : 'Bob', 2").unwrap()),
        Ok(Table(
            vec!["name".to_string(), "id".to_string()],
            vec![
                Str("Alice".to_string()),
                Int(1),
                Str("Bob".to_string()),
                Int(2)
            ]
        )),
    );

    assert_eq!(
        eval(parse("table = name, id : 'Alice', 1; table + table").unwrap()),
        Ok(Table(
            vec!["name".to_string(), "id".to_string()],
            vec![
                Str("Alice".to_string()),
                Int(1),
                Str("Alice".to_string()),
                Int(1),
            ]
        )),
    );
}

#[test]
fn test_product() {
    assert_eq!(
        eval(
            parse(
                r#"
Colors =
  color, hex :
  'Red', '#FF0000',
  'Green', '#00FF00',
  'Blue', '#0000FF';

Sizes = size : 'Small', 'Medium', 'Large';

Colors * Sizes
"#
            )
            .unwrap()
        ),
        Ok(Table(
            vec!["color".to_string(), "hex".to_string(), "size".to_string()],
            vec![
                Str("Red".to_string()),
                Str("#FF0000".to_string()),
                Str("Small".to_string()),
                Str("Red".to_string()),
                Str("#FF0000".to_string()),
                Str("Medium".to_string()),
                Str("Red".to_string()),
                Str("#FF0000".to_string()),
                Str("Large".to_string()),
                Str("Green".to_string()),
                Str("#00FF00".to_string()),
                Str("Small".to_string()),
                Str("Green".to_string()),
                Str("#00FF00".to_string()),
                Str("Medium".to_string()),
                Str("Green".to_string()),
                Str("#00FF00".to_string()),
                Str("Large".to_string()),
                Str("Blue".to_string()),
                Str("#0000FF".to_string()),
                Str("Small".to_string()),
                Str("Blue".to_string()),
                Str("#0000FF".to_string()),
                Str("Medium".to_string()),
                Str("Blue".to_string()),
                Str("#0000FF".to_string()),
                Str("Large".to_string()),
            ],
        )),
    );
}

#[test]
fn test_difference() {
    assert_eq!(
        eval(
            parse(
                r#"
Left =
  a, b :
  1, 2,
  3, 4;

Right =
  a, b :
  1, 2;

Left - Right
"#
            )
            .unwrap()
        ),
        Ok(Table(
            vec!["a".to_string(), "b".to_string()],
            vec![Int(3), Int(4)],
        ))
    );

    assert_eq!(
        eval(
            parse(
                r#"
Left =
  a, b :
  1, 2,
  3, 4;

Right =
  a, b :
  1, 'something else';

Left - Right
"#
            )
            .unwrap()
        ),
        Ok(Table(
            vec!["a".to_string(), "b".to_string()],
            vec![Int(1), Int(2), Int(3), Int(4)],
        ))
    );
}
