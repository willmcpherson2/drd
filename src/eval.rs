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
                return Err("expected table".to_string());
            };
            let exps = select(&select_vars, &table_vars, exps);
            Ok(Table(select_vars, exps))
        }
        Where(table, cond) => {
            let Table(table_vars, exps) = eval_exp(*table, env.clone())? else {
                return Err("expected table".to_string());
            };
            let exps = filter(&table_vars, exps, *cond, env)?;
            Ok(Table(table_vars, exps))
        }
        Union(l, r) => {
            let Table(vars, mut exps) = eval_exp(*l, env.clone())? else {
                return Err("expected table".to_string());
            };
            let Table(r_vars, mut r_exps) = eval_exp(*r, env)? else {
                return Err("expected table".to_string());
            };
            if vars != r_vars {
                return Err("expected tables with matching columns in union".to_string());
            }
            exps.append(&mut r_exps);
            Ok(Table(vars, exps))
        }
        Difference(l, r) => {
            let Table(l_vars, l_exps) = eval_exp(*l, env.clone())? else {
                return Err("expected table".to_string());
            };
            let Table(r_vars, r_exps) = eval_exp(*r, env)? else {
                return Err("expected table".to_string());
            };
            if l_vars != r_vars {
                return Err("expected tables with matching columns in difference".to_string());
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
                return Err("expected table".to_string());
            };
            let Table(r_vars, r_exps) = eval_exp(*r, env)? else {
                return Err("expected table".to_string());
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
                return Err("expected boolean in where clause".to_string());
            };

            if keep {
                for exp in exps {
                    acc.push(exp.clone());
                }
            }

            Ok(acc)
        })
}
