use std::collections::HashMap;

use crate::exp::*;

type Env = HashMap<String, Exp>;

pub fn eval_exp(exp: Exp, mut env: Env) -> Result<Exp, String> {
    match exp {
        Exp::Let(Let(Var(var), exp, body)) => {
            let exp = eval_exp(*exp, env.clone())?;
            env.insert(var, exp);
            let body = eval_exp(*body, env)?;
            Ok(body)
        }
        Exp::Select(_) => todo!(),
        Exp::Where(_) => todo!(),
        Exp::Union(_) => todo!(),
        Exp::Difference(_) => todo!(),
        Exp::Product(_) => todo!(),
        Exp::Table(Table(l, r)) => {
            let l = eval_exp(*l, env.clone())?;
            let r = eval_exp(*r, env)?;
            Ok(Exp::Table(Table(Box::new(l), Box::new(r))))
        }
        Exp::Row(Row(l, r)) => {
            let l = eval_exp(*l, env.clone())?;
            let r = eval_exp(*r, env)?;
            Ok(Exp::Row(Row(Box::new(l), Box::new(r))))
        }
        Exp::Cell(Cell(var, exp)) => {
            let exp = eval_exp(*exp, env)?;
            Ok(Exp::Cell(Cell(var, Box::new(exp))))
        }
        Exp::Or(Or(l, r)) => {
            let l = eval_exp(*l, env.clone())?;
            if let Exp::Bool(Bool(true)) = l {
                return Ok(Exp::Bool(Bool(true)));
            }
            let r = eval_exp(*r, env)?;
            if let Exp::Bool(Bool(true)) = r {
                return Ok(Exp::Bool(Bool(true)));
            }
            Ok(Exp::Bool(Bool(false)))
        }
        Exp::Equals(Equals(l, r)) => {
            let l = eval_exp(*l, env.clone())?;
            let r = eval_exp(*r, env)?;
            Ok(Exp::Bool(Bool(l == r)))
        }
        Exp::And(And(l, r)) => {
            let l = eval_exp(*l, env.clone())?;
            if let Exp::Bool(Bool(false)) = l {
                return Ok(Exp::Bool(Bool(false)));
            }
            let r = eval_exp(*r, env)?;
            if let Exp::Bool(Bool(false)) = r {
                return Ok(Exp::Bool(Bool(false)));
            }
            Ok(Exp::Bool(Bool(true)))
        }
        Exp::Not(Not(exp)) => {
            let exp = eval_exp(*exp, env)?;
            match exp {
                Exp::Bool(Bool(bool)) => Ok(Exp::Bool(Bool(!bool))),
                _ => Err(format!("Expected boolean, found {:?}", exp)),
            }
        }
        Exp::Var(Var(var)) => match env.get(&var) {
            Some(exp) => Ok(exp.clone()),
            None => Err(format!("Variable `{}` not defined", var)),
        },
        exp => Ok(exp),
    }
}
