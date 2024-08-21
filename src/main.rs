use clap::Parser;
use std::fs;

use drd::{eval::eval, parse::parse, serialise::serialise};

/// The Drd programming language
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to process
    #[arg(conflicts_with_all = ["eval", "port"])]
    file: Option<String>,

    /// Evaluate a string instead of a file
    #[arg(conflicts_with_all = ["file", "port"], short, long, value_name = "STRING")]
    eval: Option<String>,

    /// Start the database on a port
    #[arg(conflicts_with_all = ["file", "eval"], short, long, value_name = "PORT", default_value = "2345")]
    port: Option<u16>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(filename) = cli.file {
        match fs::read_to_string(filename) {
            Ok(text) => read_eval_print(&text),
            Err(e) => eprintln!("Error reading file: {}", e),
        }
    } else if let Some(text) = cli.eval {
        read_eval_print(&text);
    } else if let Some(port) = cli.port {
        println!("Starting on port: {port}");
    }
}

fn read_eval_print(text: &str) {
    let program = match parse(text) {
        Ok(program) => program,
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

    println!("{}", serialise(program));
}
