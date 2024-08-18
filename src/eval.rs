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
        Union(_, _) => todo!(),
        Difference(_, _) => todo!(),
        Product(_, _) => todo!(),
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
            eval(parse("name, id : 'Alice', 1, 'Bob', 2 ? (id == 1) | (id == 2)").unwrap()),
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
}
