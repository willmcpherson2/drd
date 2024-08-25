use crate::{Exp, Exp::*};

use std::{cmp::max, collections::HashMap};

pub type Env = HashMap<String, Exp>;

pub fn eval(exp: &Exp, env: &Env) -> Result<(Exp, Env), String> {
    match exp {
        Let(var, exp, body) => {
            let (exp, _) = eval(exp, env)?;
            let mut env = env.clone();
            env.insert(var.clone(), exp);
            eval(body, &env)
        }
        Select(select_vars, table) => {
            let (Table(table_vars, exps), _) = eval(table, env)? else {
                return Err("expected table".to_string());
            };
            let var_indices = table_vars
                .iter()
                .enumerate()
                .map(|(i, s)| (s, i))
                .collect::<HashMap<_, _>>();
            let keep_indices = select_vars
                .iter()
                .filter_map(|k| var_indices.get(k))
                .cloned()
                .collect::<Vec<_>>();
            let exps = exps
                .chunks(max(table_vars.len(), 1))
                .flat_map(|row| keep_indices.iter().filter_map(|&i| row.get(i).cloned()))
                .collect();
            Ok((Table(select_vars.clone(), exps), env.clone()))
        }
        Where(table, cond) => {
            let (Table(vars, exps), _) = eval(table, env)? else {
                return Err("expected table".to_string());
            };
            let exps = exps
                .chunks(max(vars.len(), 1))
                .try_fold(vec![], |mut acc, exps| {
                    let env = vars
                        .iter()
                        .zip(exps)
                        .map(|(var, exp)| (var.clone(), exp.clone()))
                        .collect();

                    match eval(cond, &env)? {
                        (Bool(true), _) => {
                            acc.extend_from_slice(exps);
                            Ok(acc)
                        }
                        (Bool(false), _) => Ok(acc),
                        _ => Err("expected boolean in where clause".to_string()),
                    }
                })?;
            Ok((Table(vars, exps), env.clone()))
        }
        Union(l, r) => {
            let (Table(vars, mut exps), _) = eval(l, env)? else {
                return Err("expected table".to_string());
            };
            let (Table(r_vars, mut r_exps), _) = eval(r, env)? else {
                return Err("expected table".to_string());
            };
            if vars != r_vars {
                return Err("expected tables with matching columns in union".to_string());
            }
            exps.append(&mut r_exps);
            Ok((Table(vars, exps), env.clone()))
        }
        Difference(l, r) => {
            let (Table(l_vars, l_exps), _) = eval(l, env)? else {
                return Err("expected table".to_string());
            };
            let (Table(r_vars, r_exps), _) = eval(r, env)? else {
                return Err("expected table".to_string());
            };
            if l_vars != r_vars {
                return Err("expected tables with matching columns in difference".to_string());
            }
            let vars = l_vars;
            let exps = l_exps
                .chunks(max(vars.len(), 1))
                .filter(|&l_exp| {
                    r_exps
                        .chunks(max(vars.len(), 1))
                        .all(|r_exp| l_exp != r_exp)
                })
                .flat_map(|chunk| chunk.to_vec())
                .collect();
            Ok((Table(vars, exps), env.clone()))
        }
        Product(l, r) => {
            let (Table(l_vars, l_exps), _) = eval(l, env)? else {
                return Err("expected table".to_string());
            };
            let (Table(r_vars, r_exps), _) = eval(r, env)? else {
                return Err("expected table".to_string());
            };
            let exps = l_exps
                .chunks(max(l_vars.len(), 1))
                .flat_map(|l_row| {
                    r_exps
                        .chunks(max(r_vars.len(), 1))
                        .flat_map(move |r_row| [l_row, r_row].concat())
                })
                .collect::<Vec<_>>();
            let vars = [l_vars, r_vars].concat();
            Ok((Table(vars, exps), env.clone()))
        }
        Table(l, r) => {
            let exps = r
                .iter()
                .map(|exp| eval(exp, env).map(|(exp, _)| exp))
                .collect::<Result<Vec<Exp>, String>>()?;
            Ok((Table(l.clone(), exps), env.clone()))
        }
        Or(l, r) => {
            if let (Bool(true), _) = eval(l, env)? {
                return Ok((Bool(true), env.clone()));
            }
            if let (Bool(true), _) = eval(r, env)? {
                return Ok((Bool(true), env.clone()));
            }
            Ok((Bool(false), env.clone()))
        }
        Equals(l, r) => {
            let (l, _) = eval(l, env)?;
            let (r, _) = eval(r, env)?;
            Ok((Bool(l == r), env.clone()))
        }
        And(l, r) => {
            if let (Bool(false), _) = eval(l, env)? {
                return Ok((Bool(false), env.clone()));
            }
            if let (Bool(false), _) = eval(r, env)? {
                return Ok((Bool(false), env.clone()));
            }
            Ok((Bool(true), env.clone()))
        }
        Not(exp) => {
            let exp = eval(exp, env)?;
            match exp {
                (Bool(bool), _) => Ok((Bool(!bool), env.clone())),
                _ => Err(format!("Expected boolean, found {:?}", exp)),
            }
        }
        Var(var) => match env.get(var) {
            Some(exp) => Ok((exp.clone(), env.clone())),
            None => Err(format!("Variable `{}` not defined", var)),
        },
        exp => Ok((exp.clone(), env.clone())),
    }
}
