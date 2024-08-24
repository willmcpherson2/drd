use drd::{read_eval, serialise, serve, Cli, Env};

use clap::Parser;
use std::fs;

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
        println!("Starting server");
        println!("Directory: {}", cli.directory);
        println!("http://localhost:{}", cli.port);
        serve(cli).unwrap_or_else(|e| eprintln!("Error starting server: {}", e));
    }
}

fn read_eval_print(text: &str) {
    match read_eval(text, &Env::new()) {
        Ok((program, _)) => println!("{}", serialise(program)),
        Err(e) => eprintln!("Error evaluating program: {}", e),
    }
}
