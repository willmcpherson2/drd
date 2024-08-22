use eval::Env;
use exp::Exp;

pub mod eval;
pub mod exp;
pub mod parse;
pub mod serialise;
pub mod serve;

pub fn read_eval(text: &str, env: &Env) -> Result<(Exp, Env), String> {
    eval::eval(parse::parse(text)?, env)
}
