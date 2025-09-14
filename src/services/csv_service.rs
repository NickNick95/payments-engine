use crate::errors::{AppErrors, AppResult};
use crate::models::amount::Amount;
use crate::models::csv_models::transaction::{CsvTxType, InputRow};
use crate::models::tx_command::{
    ChargebackCommand, DepositCommand, DisputeCommand, ResolveCommand, WithdrawalCommand,
};
use crate::services::commands::traits::tx_command_trait::TxCommandTrait;
use crate::state::AppState;
use csv::ReaderBuilder;
use log::error;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

/// Processes transactions from a CSV file and updates the application state.
///
/// # Arguments
/// * `path` - The file path to the CSV file containing transaction data.
/// * `app_state` - A mutable reference to the application state.
///
/// # Returns
/// * `AppResult<()>` - Returns `Ok(())` if successful, or an `AppErrors` variant if an error occurs.
pub fn run_from_csv_path(path: &str, app_state: &mut AppState) -> AppResult<()> {
    let file = File::open(path).map_err(|e| AppErrors::Io(format!("open {path}: {e}")))?;
    let mut rdr = ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(BufReader::new(file));

    for rec in rdr.deserialize::<InputRow>() {
        match rec {
            Ok(row) => match row_to_command(row) {
                Ok(cmd) => {
                    if let Err(e) = cmd.execute(app_state) {
                        error!("ignored command due to error: {e}");
                    }
                }
                Err(e) => {
                    error!("skip row: {e}");
                }
            },
            Err(e) => {
                error!("skip malformed CSV row: {e}");
            }
        }
    }
    Ok(())
}

/// Converts a CSV row into a transaction command.
///
/// # Arguments
/// * `row` - A single row from the CSV file, parsed into an `InputRow` struct.
///
/// # Returns
/// * `AppResult<Box<dyn TxCommandTrait>>` - Returns a boxed transaction command if successful,
///   or an `AppErrors` variant if an error occurs.
fn row_to_command(row: InputRow) -> AppResult<Box<dyn TxCommandTrait>> {
    match row.t {
        CsvTxType::Deposit => {
            let s = row
                .amount
                .ok_or(AppErrors::InvalidInput("deposit missing amount"))?;
            let amount = Amount::from_str(&s).map_err(|_| AppErrors::InvalidInput("bad amount"))?;
            Ok(Box::new(DepositCommand {
                client: row.client,
                tx: row.tx,
                amount,
            }))
        }
        CsvTxType::Withdrawal => {
            let s = row
                .amount
                .ok_or(AppErrors::InvalidInput("withdrawal missing amount"))?;
            let amount = Amount::from_str(&s).map_err(|_| AppErrors::InvalidInput("bad amount"))?;
            Ok(Box::new(WithdrawalCommand {
                client: row.client,
                tx: row.tx,
                amount,
            }))
        }
        CsvTxType::Dispute => Ok(Box::new(DisputeCommand {
            client: row.client,
            tx: row.tx,
        })),
        CsvTxType::Resolve => Ok(Box::new(ResolveCommand {
            client: row.client,
            tx: row.tx,
        })),
        CsvTxType::Chargeback => Ok(Box::new(ChargebackCommand {
            client: row.client,
            tx: row.tx,
        })),
    }
}
