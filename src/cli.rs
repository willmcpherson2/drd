use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    /// Run an expression
    Run(Client),
    /// Start the database server
    Start(Server),
}

#[derive(Parser, Debug, Clone)]
pub struct Client {
    /// Expression or file containing expression
    pub target: String,

    /// Interpret target as expression rather than file
    #[arg(short, long)]
    pub expression: bool,

    /// Send expression to a running server
    #[arg(short, long, value_name = "URL")]
    pub server: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct Server {
    /// The directory to store database files
    #[arg(short, long, value_name = "PATH", default_value = "db")]
    pub directory: String,

    /// Start the database on a port
    #[arg(short, long, value_name = "PORT", default_value = "2345")]
    pub port: u16,

    /// Log connections to stdout
    #[arg(short, long, global = true)]
    pub verbose: bool,
}
