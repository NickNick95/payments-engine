use crate::cli::Cli;
use crate::errors::AppResult;
use clap::Parser;
use log::info;

mod cli;
mod consts;
mod errors;
mod models;
mod services;
mod state;

fn main() -> AppResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Application started");

    let args = Cli::parse();

    let path = args.input;
    info!("Input file path: {}", path);

    Ok(())
}
