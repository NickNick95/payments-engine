use crate::errors::{AppErrors, AppResult};
use crate::models::domain_state::{DisputeState, TxKind, TxRecord};
use crate::models::tx_command::DepositCommand;
use crate::services::commands::traits::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

/// Implements the `TxCommandTrait` for the `DepositCommand` struct.
/// This allows the execution of deposit commands within the application state.
impl TxCommandTrait for DepositCommand {
    /// Executes the deposit command by processing it and updating the application state.
    ///
    /// # Arguments
    /// * `app_state` - A mutable reference to the application state.
    ///
    /// # Returns
    /// * `AppResult<()>` - Returns `Ok(())` if the command is successfully executed,
    ///   or an `AppErrors` variant if an error occurs.
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_deposit_command(app_state, self)
    }
}

/// Processes a deposit command and updates the application state.
///
/// A deposit increases the `available` funds of the client account and
/// records the transaction as a deposit in the transaction log.
///
/// # Arguments
/// * `app_state` - A mutable reference to the application state.
/// * `cmd` - A reference to the `DepositCommand` containing client, transaction, and amount details.
///
/// # Returns
/// * `AppResult<()>` - Returns `Ok(())` if the deposit is processed successfully,
///   or an `AppErrors` variant if an error occurs.
fn process_deposit_command(app_state: &mut AppState, cmd: &DepositCommand) -> AppResult<()> {
    let client = cmd.client;
    let tx = cmd.tx;

    if app_state.engine.txs.contains_key(&tx) {
        return Ok(());
    }

    let acc = app_state.engine.acct_mut(client);
    if acc.locked {
        return Ok(());
    }

    let amount = cmd.amount;
    acc.available = acc
        .available
        .checked_add(amount)
        .ok_or(AppErrors::Overflow)?;
    app_state.engine.txs.insert(
        tx,
        TxRecord {
            client,
            kind: TxKind::Deposit,
            amount,
            state: DisputeState::Normal,
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::amount::Amount;
    use crate::models::identifiers::{ClientId, TxId};

    fn cmd(client: ClientId, tx: TxId, amount: i64) -> DepositCommand {
        DepositCommand {
            client,
            tx,
            amount: Amount(amount),
        }
    }

    #[test]
    fn deposit_happy_path_increases_available_and_records_tx() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 1;
        let tx: TxId = 10;
        let amount = Amount(12_345);

        // act
        let res = process_deposit_command(
            &mut state,
            &DepositCommand {
                client: c,
                tx,
                amount,
            },
        );

        // assert
        assert!(res.is_ok());
        let acc = state
            .engine
            .acct_mut_if_exists(&c)
            .expect("account created");
        assert_eq!(acc.available, amount);
        assert_eq!(acc.held, Amount::zero());
        assert!(!acc.locked);

        let rec = state.engine.txs.get(&tx).expect("tx recorded");
        assert_eq!(rec.client, c);
        assert_eq!(rec.amount, amount);
        assert_eq!(rec.kind, TxKind::Deposit);
        assert_eq!(rec.state, DisputeState::Normal);
    }

    #[test]
    fn deposit_ignored_if_duplicate_tx_id() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 1;
        let tx: TxId = 42;

        let first = Amount(10_000);
        process_deposit_command(&mut state, &cmd(c, tx, first.0)).unwrap();

        // act
        let second = Amount(5_000);
        process_deposit_command(&mut state, &cmd(c, tx, second.0)).unwrap();

        // assert
        let acc = state.engine.acct_mut_if_exists(&c).unwrap();
        assert_eq!(acc.available, first);

        let rec = state.engine.txs.get(&tx).unwrap();
        assert_eq!(rec.amount, first);
    }

    #[test]
    fn deposit_ignored_if_account_locked() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 7;

        {
            let acc = state.engine.acct_mut(c);
            acc.locked = true;
        }

        let tx: TxId = 2;
        let amount = Amount(20_000); // 2.0000

        // act
        process_deposit_command(
            &mut state,
            &DepositCommand {
                client: c,
                tx,
                amount,
            },
        )
        .unwrap();

        // assert
        let acc = state.engine.acct_mut_if_exists(&c).unwrap();
        assert_eq!(acc.available, Amount::zero());
        assert_eq!(acc.held, Amount::zero());
        assert!(acc.locked);

        assert!(!state.engine.txs.contains_key(&tx));
    }

    #[test]
    fn deposit_fails_with_overflow() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 9;
        let tx: TxId = 100;

        // Set available near i64::MAX and try to add a positive amount to trigger checked_add overflow
        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount(i64::MAX - 1);
        }
        let amount = Amount(10);

        // act
        let res = process_deposit_command(
            &mut state,
            &DepositCommand {
                client: c,
                tx,
                amount,
            },
        );

        // assert
        assert!(matches!(res, Err(AppErrors::Overflow)));
        assert!(!state.engine.txs.contains_key(&tx));
    }
}
