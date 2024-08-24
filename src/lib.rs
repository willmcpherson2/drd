mod cli;
mod client;
mod eval;
mod exp;
mod parse;
mod serialise;
mod serve;

pub use cli::{Cli, Client, Server};
pub use client::client;
pub use eval::{eval, Env};
pub use exp::Exp;
pub use parse::{parse, Bexp, Op, Side};
pub use serialise::serialise;
pub use serve::serve;

pub fn read_eval(text: &str, env: &Env) -> Result<(Exp, Env), String> {
    eval(&parse(text)?, env)
}
