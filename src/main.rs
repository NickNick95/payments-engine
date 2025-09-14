use crate::cli::Cli;
use crate::errors::{AppErrors, AppResult};
use crate::models::csv_models::transaction::OutputRow;
use crate::services::csv_service::run_from_csv_path;
use crate::state::AppState;
use clap::Parser;
use csv::WriterBuilder;
use log::info;
use std::io;

mod cli;
mod consts;
mod errors;
mod models;
mod services;
mod state;

/// Application entry point.
///
/// Responsibilities:
/// - Initialize the logger (`env_logger` with default level `info`).
/// - Parse CLI arguments using `clap`.
/// - Run the main application logic via [`run_app`].
///
/// Logs "Application started" and "Application ended" at INFO level.
///
/// # Returns
/// * `AppResult<()>` - Returns `Ok(())` if the application runs successfully,
///   or an `AppErrors` variant if an error occurs.
fn main() -> AppResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Application started");

    let args = Cli::parse();
    run_app(&args)?;

    info!("Application ended");
    Ok(())
}

/// Run the core application logic.
///
/// Responsibilities:
/// - Create a fresh [`AppState`] which holds the engine (accounts + transactions).
/// - Process transactions from the input CSV file (via [`run_from_csv_path`]).
/// - Emit the final account states to stdout (via [`emit_accounts_to_stdout`]).
///
/// Logs when processing starts and ends.
///
/// # Arguments
/// * `args` - A reference to the parsed CLI arguments.
///
/// # Returns
/// * `AppResult<()>` - Returns `Ok(())` if the application logic runs successfully,
///   or an `AppErrors` variant if an error occurs.
pub fn run_app(args: &Cli) -> AppResult<()> {
    info!("Starting to process input file: {}", args.input);

    let mut app_state = AppState::default();
    run_from_csv_path(&args.input, &mut app_state)?;

    info!("Finished processing input file: {}", args.input);
    info!("Emitting results to stdout...");
    emit_accounts_to_stdout(&app_state)?;

    info!("Results successfully emitted");
    Ok(())
}

/// Emit final account states to stdout in CSV format.
///
/// Responsibilities:
/// - Create a CSV writer bound to `stdout`.
/// - Iterate over all accounts in the engine.
/// - Serialize each account as an [`OutputRow`] with `available`, `held`, `total`
///   reported to 4 decimal places, and `locked` as a boolean.
/// - Flush the writer at the end.
///
/// Logs the number of accounts written.
///
/// # Arguments
/// * `app_state` - A reference to the application state containing the engine.
///
/// # Returns
/// * `AppResult<()>` - Returns `Ok(())` if the accounts are successfully emitted,
///   or an `AppErrors` variant if an error occurs.
pub fn emit_accounts_to_stdout(app_state: &AppState) -> AppResult<()> {
    let out = io::stdout();
    let handle = out.lock();
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(handle);

    let mut count = 0;
    for (client, acc) in app_state.engine.accounts_iter() {
        let row = OutputRow::from((client, acc));
        wtr.serialize(row)
            .map_err(|e| AppErrors::Io(format!("write csv: {e}")))?;
        count += 1;
    }
    wtr.flush()
        .map_err(|e| AppErrors::Io(format!("flush csv: {e}")))?;

    info!("Emitted {} account(s) to stdout", count);
    Ok(())
}
