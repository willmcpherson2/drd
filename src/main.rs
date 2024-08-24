use drd::{read_eval, serialise, serve, Cli, Env};

use clap::Parser;
use std::fs;

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Run { file } => match fs::read_to_string(file) {
            Ok(text) => read_eval_print(&text),
            Err(e) => eprintln!("Error reading file: {}", e),
        },
        Cli::Eval { text } => read_eval_print(&text),
        Cli::Start(conf) => {
            println!("Starting server");
            println!("Directory: {}", conf.directory);
            println!("http://localhost:{}", conf.port);
            serve(conf).unwrap_or_else(|e| eprintln!("Error starting server: {}", e));
        }
    }
}

fn read_eval_print(text: &str) {
    match read_eval(text, &Env::new()) {
        Ok((program, _)) => println!("{}", serialise(program)),
        Err(e) => eprintln!("Error evaluating program: {}", e),
    }
}
