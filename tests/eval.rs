use drd::{eval::eval, exp::Exp::*, parse::parse};

macro_rules! run {
    ($input:expr, $output:expr) => {{
        let (exp, _) = eval(parse($input).unwrap()).unwrap();
        assert_eq!(exp, $output);
    }};
}

#[test]
fn test_select() {
    run!(
        "name <- name, id : 'Alice', 1, 'Bob', 2",
        Table(
            vec!["name".to_string()],
            vec![Str("Alice".to_string()), Str("Bob".to_string())]
        )
    );

    run!(
        "id <- name, id : 'Alice', 1, 'Bob', 2",
        Table(vec!["id".to_string()], vec![Int(1), Int(2)])
    );

    run!(
        "foo <- name, id : 'Alice', 1, 'Bob', 2",
        Table(vec!["foo".to_string()], vec![])
    );
}

#[test]
fn test_where() {
    run!(
        "name, id : 'Alice', 1, 'Bob', 2 ? name == 'Alice'",
        Table(
            vec!["name".to_string(), "id".to_string()],
            vec![Str("Alice".to_string()), Int(1)]
        )
    );

    run!(
        "name, id : 'Alice', 1, 'Bob', 2 ? id == 2",
        Table(
            vec!["name".to_string(), "id".to_string()],
            vec![Str("Bob".to_string()), Int(2)]
        )
    );

    run!(
        "name, id : 'Alice', 1, 'Bob', 2 ? id == 1 || id == 2",
        Table(
            vec!["name".to_string(), "id".to_string()],
            vec![
                Str("Alice".to_string()),
                Int(1),
                Str("Bob".to_string()),
                Int(2)
            ]
        )
    );

    run!(
        "name, id : 'Alice', 1, 'Bob', 2 ? name == 'Foo'",
        Table(vec!["name".to_string(), "id".to_string()], vec![])
    );
}

#[test]
fn test_union() {
    run!(
        "name, id : 'Alice', 1 + name, id : 'Bob', 2",
        Table(
            vec!["name".to_string(), "id".to_string()],
            vec![
                Str("Alice".to_string()),
                Int(1),
                Str("Bob".to_string()),
                Int(2)
            ]
        )
    );

    run!(
        "table = name, id : 'Alice', 1; table + table",
        Table(
            vec!["name".to_string(), "id".to_string()],
            vec![
                Str("Alice".to_string()),
                Int(1),
                Str("Alice".to_string()),
                Int(1),
            ]
        )
    );
}

#[test]
fn test_product() {
    run!(
        r#"
Colors =
  color, hex :
  'Red', '#FF0000',
  'Green', '#00FF00',
  'Blue', '#0000FF';

Sizes = size : 'Small', 'Medium', 'Large';

Colors * Sizes
"#,
        Table(
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
        )
    );
}

#[test]
fn test_difference() {
    run!(
        r#"
Left =
  a, b :
  1, 2,
  3, 4;

Right =
  a, b :
  1, 2;

Left - Right
"#,
        Table(vec!["a".to_string(), "b".to_string()], vec![Int(3), Int(4)])
    );

    run!(
        r#"
Left =
  a, b :
  1, 2,
  3, 4;

Right =
  a, b :
  1, 'something else';

Left - Right
"#,
        Table(
            vec!["a".to_string(), "b".to_string()],
            vec![Int(1), Int(2), Int(3), Int(4)],
        )
    );
}
