use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    /// Run a Drd file
    Run {
        /// Input file to process
        file: String,
    },
    /// Evaluate a Drd expression
    Eval {
        /// String to evaluate
        text: String,
    },
    /// Start the Drd database server
    Start(Config),
}

#[derive(Parser, Debug, Clone)]
pub struct Config {
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
