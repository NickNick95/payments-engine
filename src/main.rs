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

fn main() -> AppResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Application started");

    let args = Cli::parse();

    let mut app_state = AppState::default();

    run_from_csv_path(&args.input, &mut app_state)?;
    emit_accounts_to_stdout(&app_state)?;

    Ok(())
}

pub fn emit_accounts_to_stdout(app_state: &AppState) -> AppResult<()> {
    // adapt to your actual writer/output model
    let out = io::stdout();
    let handle = out.lock();
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(handle);

    for (client, acc) in app_state.engine.accounts_iter() {
        let row = OutputRow::from((client, acc));
        wtr.serialize(row)
            .map_err(|e| AppErrors::Io(format!("write csv: {e}")))?;
    }
    wtr.flush()
        .map_err(|e| AppErrors::Io(format!("flush csv: {e}")))?;
    Ok(())
}
