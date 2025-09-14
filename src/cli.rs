use clap::Parser;

/// Represents the command-line interface (CLI) for the application.
/// Parses input arguments provided by the user.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The path to the input CSV file containing transactions.
    pub input: String,
}
