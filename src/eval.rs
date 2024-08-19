use std::collections::HashMap;

use crate::exp::{Exp, Exp::*};

type Env = HashMap<String, Exp>;

pub fn eval(exp: Exp) -> Result<Exp, String> {
    eval_exp(exp, HashMap::new())
}

fn eval_exp(exp: Exp, mut env: Env) -> Result<Exp, String> {
    match exp {
        Let(var, exp, body) => {
            let exp = eval_exp(*exp, env.clone())?;
            env.insert(var, exp);
            let body = eval_exp(*body, env)?;
            Ok(body)
        }
        Select(select_vars, table) => {
            let Table(table_vars, exps) = eval_exp(*table, env)? else {
                return Err(format!("expected table"));
            };
            let exps = select(&select_vars, &table_vars, exps);
            Ok(Table(select_vars, exps))
        }
        Where(table, cond) => {
            let Table(table_vars, exps) = eval_exp(*table, env.clone())? else {
                return Err(format!("expected table"));
            };
            let exps = filter(&table_vars, exps, *cond, env)?;
            Ok(Table(table_vars, exps))
        }
        Union(l, r) => {
            let Table(vars, mut exps) = eval_exp(*l, env.clone())? else {
                return Err(format!("expected table"));
            };
            let Table(r_vars, mut r_exps) = eval_exp(*r, env)? else {
                return Err(format!("expected table"));
            };
            if vars != r_vars {
                return Err(format!("expected tables with matching columns in union"));
            }
            exps.append(&mut r_exps);
            Ok(Table(vars, exps))
        }
        Difference(l, r) => {
            let Table(l_vars, l_exps) = eval_exp(*l, env.clone())? else {
                return Err(format!("expected table"));
            };
            let Table(r_vars, r_exps) = eval_exp(*r, env)? else {
                return Err(format!("expected table"));
            };
            if l_vars != r_vars {
                return Err(format!(
                    "expected tables with matching columns in difference"
                ));
            }
            let vars = l_vars;
            let exps = l_exps
                .chunks(vars.len())
                .filter(|&l_exp| r_exps.chunks(vars.len()).all(|r_exp| l_exp != r_exp))
                .flat_map(|chunk| chunk.to_vec())
                .collect();
            Ok(Table(vars, exps))
        }
        Product(l, r) => {
            let Table(l_vars, l_exps) = eval_exp(*l, env.clone())? else {
                return Err(format!("expected table"));
            };
            let Table(r_vars, r_exps) = eval_exp(*r, env)? else {
                return Err(format!("expected table"));
            };
            let exps = l_exps
                .chunks(l_vars.len())
                .flat_map(|l_row| {
                    r_exps
                        .chunks(r_vars.len())
                        .flat_map(move |r_row| [l_row, r_row].concat())
                })
                .collect::<Vec<_>>();
            let vars = [l_vars, r_vars].concat();
            Ok(Table(vars, exps))
        }
        Table(l, r) => {
            let exps = r
                .into_iter()
                .map(|exp| eval_exp(exp, env.clone()))
                .collect::<Result<Vec<Exp>, String>>()?;
            Ok(Table(l, exps))
        }
        Or(l, r) => {
            let l = eval_exp(*l, env.clone())?;
            if let Bool(true) = l {
                return Ok(Bool(true));
            }
            let r = eval_exp(*r, env)?;
            if let Bool(true) = r {
                return Ok(Bool(true));
            }
            Ok(Bool(false))
        }
        Equals(l, r) => {
            let l = eval_exp(*l, env.clone())?;
            let r = eval_exp(*r, env)?;
            Ok(Bool(l == r))
        }
        And(l, r) => {
            let l = eval_exp(*l, env.clone())?;
            if let Bool(false) = l {
                return Ok(Bool(false));
            }
            let r = eval_exp(*r, env)?;
            if let Bool(false) = r {
                return Ok(Bool(false));
            }
            Ok(Bool(true))
        }
        Not(exp) => {
            let exp = eval_exp(*exp, env)?;
            match exp {
                Bool(bool) => Ok(Bool(!bool)),
                _ => Err(format!("Expected boolean, found {:?}", exp)),
            }
        }
        Var(var) => match env.get(&var) {
            Some(exp) => Ok(exp.clone()),
            None => Err(format!("Variable `{}` not defined", var)),
        },
        exp => Ok(exp),
    }
}

fn select(keep: &[String], target: &[String], items: Vec<Exp>) -> Vec<Exp> {
    let target_indices = target
        .iter()
        .enumerate()
        .map(|(i, s)| (s, i))
        .collect::<HashMap<_, _>>();

    let indices_to_keep = keep
        .iter()
        .filter_map(|k| target_indices.get(k))
        .cloned()
        .collect::<Vec<_>>();

    items
        .chunks(target.len())
        .flat_map(|row| indices_to_keep.iter().filter_map(|&i| row.get(i).cloned()))
        .collect()
}

fn filter(
    vars: &[String],
    exps: Vec<Exp>,
    cond: Exp,
    env: HashMap<String, Exp>,
) -> Result<Vec<Exp>, String> {
    exps.chunks(vars.len())
        .try_fold(vec![], |mut acc: Vec<Exp>, exps: &[Exp]| {
            let mut env = env.clone();
            for (var, exp) in vars.iter().zip(exps) {
                env.insert(var.clone(), exp.clone());
            }

            let Bool(keep) = eval_exp(cond.clone(), env)? else {
                return Err(format!("expected boolean in where clause"));
            };

            if keep {
                for exp in exps {
                    acc.push(exp.clone());
                }
            }

            Ok(acc)
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::*;

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
}
