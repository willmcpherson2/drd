use crate::{eval::eval_program, parse::parse_program};

use std::{env, fs};

mod ast;
mod eval;
mod parse;

fn main() {
    let Some(filename) = env::args().nth(1) else {
        eprintln!("Usage: drd <filename>");
        return;
    };

    let input = match fs::read_to_string(filename) {
        Ok(input) => input,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    let program = match parse_program(&input) {
        Ok((_, program)) => program,
        Err(e) => {
            eprintln!("Error parsing program: {}", e);
            return;
        }
    };

    let program = match eval_program(program) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Error evaluating program: {}", e);
            return;
        }
    };

    println!("{:#?}", program);
}
