use clap::Parser;
use std::{collections::HashMap, fs};

use drd::{read_eval, serialise::serialise, serve::serve};

const EVAL: &[&str] = &["file", "eval"];
const SERVE: &[&str] = &["directory", "port", "timeout"];

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

    /// The directory to store database files
    #[arg(conflicts_with_all = EVAL, short, long, value_name = "PATH", default_value = "db")]
    directory: String,

    /// Start the database on a port
    #[arg(conflicts_with_all = EVAL, short, long, value_name = "PORT", default_value = "2345")]
    port: u16,

    /// Timeout for connections in milliseconds. 0 for no timeout
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
        let dir = cli.directory;
        let port = cli.port;
        let timeout = cli.timeout;
        println!("Starting server");
        println!("Directory: {dir}");
        println!("Timeout: {timeout}");
        println!("http://localhost:{port}");
        if let Err(e) = serve(dir, port, timeout) {
            eprintln!("Error starting server: {}", e)
        }
    }
}

fn read_eval_print(text: &str) {
    match read_eval(text, &HashMap::new()) {
        Ok((program, _)) => println!("{}", serialise(program)),
        Err(e) => eprintln!("Error evaluating program: {}", e),
    }
}
