use sdb::{client, read_eval, serialise, server, Cli, Env};

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
                Some(url) => match client(&text, &url) {
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("Error running client: {}", e),
                },
                None => match read_eval(&text, &Env::new()) {
                    Ok((result, _)) => println!("{}", serialise(result)),
                    Err(e) => eprintln!("Error evaluating program: {}", e),
                },
            }
        }
        Cli::Start(conf) => {
            println!("Starting server");
            println!("Directory: {}", conf.directory);
            println!("http://localhost:{}", conf.port);
            server(conf).unwrap_or_else(|e| eprintln!("Error starting server: {}", e));
        }
    }
}
