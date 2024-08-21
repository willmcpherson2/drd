use clap::Parser;
use std::fs;

use drd::{eval::eval, parse::parse, serialise::serialise, serve::serve};

const EVAL: &[&str] = &["file", "eval"];
const SERVE: &[&str] = &["port", "timeout"];

/// The Drd programming language
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to process
    #[arg(conflicts_with = "eval", conflicts_with_all = SERVE)]
    file: Option<String>,

    /// Evaluate a string instead of a file
    #[arg(conflicts_with = "file", conflicts_with_all = SERVE, short, long, value_name = "STRING")]
    eval: Option<String>,

    /// Start the database on a port
    #[arg(conflicts_with_all = EVAL, short, long, value_name = "PORT", default_value = "2345")]
    port: u16,

    /// Timeout for connections in milliseconds
    #[arg(conflicts_with_all = EVAL, short, long, value_name = "TIMEOUT", default_value = "5000")]
    timeout: u64,
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
    } else {
        let port = cli.port;
        let timeout = cli.timeout;
        println!("Starting server");
        println!("http://localhost:{port}");
        if let Err(e) = serve(port, timeout) {
            eprintln!("Error starting server: {}", e)
        }
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
        Ok((program, _)) => program,
        Err(e) => {
            eprintln!("Error evaluating program: {}", e);
            return;
        }
    };

    println!("{}", serialise(program));
}
