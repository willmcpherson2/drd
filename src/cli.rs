use clap::Parser;

const EVAL: &[&str] = &["file", "eval"];
const SERVE: &[&str] = &["directory", "port", "timeout"];

/// The Drd programming language
#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Input file to process
    #[arg(conflicts_with = "eval", conflicts_with_all = SERVE)]
    pub file: Option<String>,

    /// Evaluate a string instead of a file
    #[arg(conflicts_with = "file", conflicts_with_all = SERVE, short, long, value_name = "STRING")]
    pub eval: Option<String>,

    /// The directory to store database files
    #[arg(conflicts_with_all = EVAL, short, long, value_name = "PATH", default_value = "db")]
    pub directory: String,

    /// Start the database on a port
    #[arg(conflicts_with_all = EVAL, short, long, value_name = "PORT", default_value = "2345")]
    pub port: u16,

    /// Timeout for connections in milliseconds. 0 for no timeout
    #[arg(conflicts_with_all = EVAL, short, long, value_name = "TIMEOUT", default_value = "5000")]
    pub timeout: u64,

    /// Log connections to stdout
    #[arg(short, long)]
    pub verbose: bool,
}
