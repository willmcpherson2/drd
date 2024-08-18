use std::collections::HashMap;

use crate::exp::*;

type Env = HashMap<String, Exp>;

pub fn eval(exp: Exp) -> Result<Exp, String> {
    eval_exp(exp, HashMap::new())
}

fn eval_exp(exp: Exp, mut env: Env) -> Result<Exp, String> {
    match exp {
        Exp::Let(var, exp, body) => {
            let exp = eval_exp(*exp, env.clone())?;
            env.insert(var, exp);
            let body = eval_exp(*body, env)?;
            Ok(body)
        }
        Exp::Select(select_vars, table) => {
            let Exp::Table(table_vars, exps) = eval_exp(*table, env)? else {
                return Err(format!("expected table"));
            };
            let exps = select(&select_vars, &table_vars, exps);
            Ok(Exp::Table(select_vars, exps))
        }
        Exp::Where(_, _) => todo!(),
        Exp::Union(_, _) => todo!(),
        Exp::Difference(_, _) => todo!(),
        Exp::Product(_, _) => todo!(),
        Exp::Table(l, r) => {
            let exps = r
                .into_iter()
                .map(|exp| eval_exp(exp, env.clone()))
                .collect::<Result<Vec<Exp>, String>>()?;
            Ok(Exp::Table(l, exps))
        }
        Exp::Or(l, r) => {
            let l = eval_exp(*l, env.clone())?;
            if let Exp::Bool(true) = l {
                return Ok(Exp::Bool(true));
            }
            let r = eval_exp(*r, env)?;
            if let Exp::Bool(true) = r {
                return Ok(Exp::Bool(true));
            }
            Ok(Exp::Bool(false))
        }
        Exp::Equals(l, r) => {
            let l = eval_exp(*l, env.clone())?;
            let r = eval_exp(*r, env)?;
            Ok(Exp::Bool(l == r))
        }
        Exp::And(l, r) => {
            let l = eval_exp(*l, env.clone())?;
            if let Exp::Bool(false) = l {
                return Ok(Exp::Bool(false));
            }
            let r = eval_exp(*r, env)?;
            if let Exp::Bool(false) = r {
                return Ok(Exp::Bool(false));
            }
            Ok(Exp::Bool(true))
        }
        Exp::Not(exp) => {
            let exp = eval_exp(*exp, env)?;
            match exp {
                Exp::Bool(bool) => Ok(Exp::Bool(!bool)),
                _ => Err(format!("Expected boolean, found {:?}", exp)),
            }
        }
        Exp::Var(var) => match env.get(&var) {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::exp::Exp::*;
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
}
