use clap::Parser;
use std::fs;

use crate::{eval::eval, parse::parse};

mod eval;
mod exp;
mod parse;

/// The Drd programming language
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to process
    #[arg(conflicts_with = "eval")]
    file: Option<String>,

    /// Evaluate a string instead of a file
    #[arg(short, long, value_name = "STRING")]
    eval: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let text = if let Some(text) = cli.eval {
        text
    } else if let Some(filename) = cli.file {
        match fs::read_to_string(filename) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                return;
            }
        }
    } else {
        eprintln!("Either a file or --eval must be provided");
        return;
    };

    let program = match parse(&text) {
        Ok((_, program)) => program,
        Err(e) => {
            eprintln!("Error parsing program: {}", e);
            return;
        }
    };

    let program = match eval(program) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Error evaluating program: {}", e);
            return;
        }
    };

    println!("{:#?}", program);
}
