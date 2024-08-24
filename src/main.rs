use drd::{client, read_eval, serialise, serve, Cli, Env};

use clap::Parser;
use std::fs;

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Run(conf) => {
            let text = if conf.expression {
                conf.target
            } else {
                match fs::read_to_string(conf.target) {
                    Ok(text) => text,
                    Err(e) => return eprintln!("Error reading file: {}", e),
                }
            };

            match conf.server {
                Some(url) => {
                    client(&text, &url).unwrap_or_else(|e| eprintln!("Error running client: {}", e))
                }
                None => match read_eval(&text, &Env::new()) {
                    Ok((program, _)) => println!("{}", serialise(program)),
                    Err(e) => eprintln!("Error evaluating program: {}", e),
                },
            }
        }
        Cli::Start(conf) => {
            println!("Starting server");
            println!("Directory: {}", conf.directory);
            println!("http://localhost:{}", conf.port);
            serve(conf).unwrap_or_else(|e| eprintln!("Error starting server: {}", e));
        }
    }
}
