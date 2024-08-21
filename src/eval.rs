use std::collections::HashMap;

use crate::exp::{Exp, Exp::*};

type Env = HashMap<String, Exp>;

pub fn eval(exp: Exp) -> Result<(Exp, Env), String> {
    eval_exp(exp, HashMap::new())
}

fn eval_exp(exp: Exp, mut env: Env) -> Result<(Exp, Env), String> {
    match exp {
        Let(var, exp, body) => {
            let (exp, _) = eval_exp(*exp, env.clone())?;
            env.insert(var, exp);
            eval_exp(*body, env)
        }
        Select(select_vars, table) => {
            let (Table(table_vars, exps), _) = eval_exp(*table, env.clone())? else {
                return Err("expected table".to_string());
            };
            let exps = select(&select_vars, &table_vars, exps);
            Ok((Table(select_vars, exps), env))
        }
        Where(table, cond) => {
            let (Table(table_vars, exps), _) = eval_exp(*table, env.clone())? else {
                return Err("expected table".to_string());
            };
            let exps = filter(&table_vars, exps, *cond, env.clone())?;
            Ok((Table(table_vars, exps), env))
        }
        Union(l, r) => {
            let (Table(vars, mut exps), _) = eval_exp(*l, env.clone())? else {
                return Err("expected table".to_string());
            };
            let (Table(r_vars, mut r_exps), _) = eval_exp(*r, env.clone())? else {
                return Err("expected table".to_string());
            };
            if vars != r_vars {
                return Err("expected tables with matching columns in union".to_string());
            }
            exps.append(&mut r_exps);
            Ok((Table(vars, exps), env))
        }
        Difference(l, r) => {
            let (Table(l_vars, l_exps), _) = eval_exp(*l, env.clone())? else {
                return Err("expected table".to_string());
            };
            let (Table(r_vars, r_exps), _) = eval_exp(*r, env.clone())? else {
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
            Ok((Table(vars, exps), env))
        }
        Product(l, r) => {
            let (Table(l_vars, l_exps), _) = eval_exp(*l, env.clone())? else {
                return Err("expected table".to_string());
            };
            let (Table(r_vars, r_exps), _) = eval_exp(*r, env.clone())? else {
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
            Ok((Table(vars, exps), env))
        }
        Table(l, r) => {
            let exps = r
                .into_iter()
                .map(|exp| eval_exp(exp, env.clone()).map(|(exp, _)| exp))
                .collect::<Result<Vec<Exp>, String>>()?;
            Ok((Table(l, exps), env))
        }
        Or(l, r) => {
            if let (Bool(true), _) = eval_exp(*l, env.clone())? {
                return Ok((Bool(true), env));
            }
            if let (Bool(true), _) = eval_exp(*r, env.clone())? {
                return Ok((Bool(true), env));
            }
            Ok((Bool(false), env))
        }
        Equals(l, r) => {
            let (l, _) = eval_exp(*l, env.clone())?;
            let (r, _) = eval_exp(*r, env.clone())?;
            Ok((Bool(l == r), env))
        }
        And(l, r) => {
            if let (Bool(false), _) = eval_exp(*l, env.clone())? {
                return Ok((Bool(false), env));
            }
            if let (Bool(false), _) = eval_exp(*r, env.clone())? {
                return Ok((Bool(false), env));
            }
            Ok((Bool(true), env))
        }
        Not(exp) => {
            let exp = eval_exp(*exp, env.clone())?;
            match exp {
                (Bool(bool), _) => Ok((Bool(!bool), env)),
                _ => Err(format!("Expected boolean, found {:?}", exp)),
            }
        }
        Var(var) => match env.get(&var) {
            Some(exp) => Ok((exp.clone(), env)),
            None => Err(format!("Variable `{}` not defined", var)),
        },
        exp => Ok((exp, env)),
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

            let (Bool(keep), _) = eval_exp(cond.clone(), env)? else {
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
