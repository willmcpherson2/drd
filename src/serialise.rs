use crate::{
    Bexp,
    Exp::{self, *},
    Op, Side,
};

pub fn serialise(exp: Exp) -> String {
    serialise_bexp(serialise_exp(exp))
}

fn serialise_exp(exp: Exp) -> Bexp {
    match exp {
        Let(var, exp, body) => Bexp::Binary(
            Box::new(Bexp::Binary(
                Box::new(Bexp::Var(var)),
                Op::Let,
                Box::new(with_parens(*exp, Op::Let, Side::Right)),
            )),
            Op::In,
            Box::new(with_parens(*body, Op::In, Side::Right)),
        ),
        Select(l, r) => Bexp::Binary(
            Box::new(serialise_var_list(l)),
            Op::Select,
            Box::new(with_parens(*r, Op::Select, Side::Right)),
        ),
        Where(l, r) => Bexp::Binary(
            Box::new(with_parens(*l, Op::Where, Side::Left)),
            Op::Where,
            Box::new(with_parens(*r, Op::Where, Side::Right)),
        ),
        Union(l, r) => Bexp::Binary(
            Box::new(with_parens(*l, Op::Union, Side::Left)),
            Op::Union,
            Box::new(with_parens(*r, Op::Union, Side::Right)),
        ),
        Difference(l, r) => Bexp::Binary(
            Box::new(with_parens(*l, Op::Difference, Side::Left)),
            Op::Difference,
            Box::new(with_parens(*r, Op::Difference, Side::Right)),
        ),
        Product(l, r) => Bexp::Binary(
            Box::new(with_parens(*l, Op::Product, Side::Left)),
            Op::Product,
            Box::new(with_parens(*r, Op::Product, Side::Right)),
        ),
        Table(vars, exps) => {
            if vars.is_empty() && exps.is_empty() {
                Bexp::Nil
            } else {
                Bexp::Binary(
                    Box::new(serialise_var_list(vars)),
                    Op::Table,
                    Box::new(serialise_exp_list(exps)),
                )
            }
        }
        Or(l, r) => Bexp::Binary(
            Box::new(with_parens(*l, Op::Or, Side::Left)),
            Op::Or,
            Box::new(with_parens(*r, Op::Or, Side::Right)),
        ),
        Equals(l, r) => Bexp::Binary(
            Box::new(with_parens(*l, Op::Equals, Side::Left)),
            Op::Equals,
            Box::new(with_parens(*r, Op::Equals, Side::Right)),
        ),
        And(l, r) => Bexp::Binary(
            Box::new(with_parens(*l, Op::And, Side::Left)),
            Op::And,
            Box::new(with_parens(*r, Op::And, Side::Right)),
        ),
        Not(exp) => Bexp::Binary(
            Box::new(Bexp::Var("not".to_string())),
            Op::App,
            Box::new(with_parens(*exp, Op::App, Side::Left)),
        ),
        Bool(bool) => Bexp::Bool(bool),
        Int(int) => Bexp::Int(int),
        Str(str) => Bexp::Str(str),
        Var(var) => Bexp::Var(var),
    }
}

fn serialise_var_list(mut vars: Vec<String>) -> Bexp {
    if vars.is_empty() {
        Bexp::Nil
    } else {
        let first = Bexp::Var(vars.remove(0));
        vars.into_iter().fold(first, |acc, var| {
            Bexp::Binary(Box::new(acc), Op::Item, Box::new(Bexp::Var(var)))
        })
    }
}

fn serialise_exp_list(mut exps: Vec<Exp>) -> Bexp {
    if exps.is_empty() {
        Bexp::Nil
    } else {
        let first = serialise_exp(exps.remove(0));
        exps.into_iter().fold(first, |acc, exp| {
            Bexp::Binary(Box::new(acc), Op::Item, Box::new(serialise_exp(exp)))
        })
    }
}

fn with_parens(exp: Exp, parent: Op, side: Side) -> Bexp {
    let bexp = serialise_exp(exp);
    match bexp {
        Bexp::Binary(_, op, _) => {
            if op.prec() < parent.prec() || op.prec() == parent.prec() && op.assoc() != side {
                Bexp::Parens(Box::new(bexp))
            } else {
                bexp
            }
        }
        _ => bexp,
    }
}

fn serialise_bexp(exp: Bexp) -> String {
    match exp {
        Bexp::Binary(l, op, r) => format!(
            "{}{}{}",
            serialise_bexp(*l),
            serialise_op(op),
            serialise_bexp(*r)
        ),
        Bexp::Parens(bexp) => format!("({})", serialise_bexp(*bexp),),
        Bexp::Bool(bool) => bool.to_string(),
        Bexp::Int(int) => int.to_string(),
        Bexp::Nil => "nil".to_string(),
        Bexp::Str(str) => format!("'{}'", str),
        Bexp::Var(var) => var,
    }
}

fn serialise_op(op: Op) -> &'static str {
    match op {
        Op::In => "; ",
        Op::Let => " = ",
        Op::Select => " <- ",
        Op::Where => " ? ",
        Op::Union => " + ",
        Op::Difference => " - ",
        Op::Product => " * ",
        Op::Table => " : ",
        Op::Item => ", ",
        Op::Or => " || ",
        Op::Equals => " == ",
        Op::And => " && ",
        Op::App => " ",
    }
}
