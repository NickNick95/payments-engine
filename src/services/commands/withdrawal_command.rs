use crate::errors::{AppErrors, AppResult};
use crate::models::domain_state::{DisputeState, TxKind, TxRecord};
use crate::models::tx_command::WithdrawalCommand;
use crate::services::commands::traits::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

/// Implements the `TxCommandTrait` for the `WithdrawalCommand` struct.
/// This allows the execution of withdrawal commands within the application state.
impl TxCommandTrait for WithdrawalCommand {
    /// Executes the withdrawal command by processing it and updating the application state.
    ///
    /// # Arguments
    /// * `app_state` - A mutable reference to the application state.
    ///
    /// # Returns
    /// * `AppResult<()>` - Returns `Ok(())` if the command is successfully executed,
    ///   or an `AppErrors` variant if an error occurs.
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_withdrawal_command(app_state, self)
    }
}

/// Processes a withdrawal command and updates the application state.
///
/// # Arguments
/// * `app_state` - A mutable reference to the application state.
/// * `cmd` - A reference to the `WithdrawalCommand` to be processed.
///
/// # Returns
/// * `AppResult<()>` - Returns `Ok(())` if the command is successfully processed,
///   or an `AppErrors` variant if an error occurs.
fn process_withdrawal_command(app_state: &mut AppState, cmd: &WithdrawalCommand) -> AppResult<()> {
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
    if acc.available.0 < amount.0 {
        return Ok(());
    }

    acc.available = acc
        .available
        .checked_sub(amount)
        .ok_or(AppErrors::Overflow)?;

    app_state.engine.txs.insert(
        tx,
        TxRecord {
            client,
            kind: TxKind::Withdrawal,
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

    fn wc(client: ClientId, tx: TxId, raw_amount: i64) -> WithdrawalCommand {
        WithdrawalCommand {
            client,
            tx,
            amount: Amount(raw_amount),
        }
    }

    #[test]
    fn withdrawal_happy_path_decrements_available_and_records_tx() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 1;
        let tx: TxId = 10;

        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount(20_000);
        }

        // act
        let res = process_withdrawal_command(&mut state, &wc(c, tx, 12_500));

        // assert
        assert!(res.is_ok());
        let acc = state.engine.acct(c).expect("account exists");
        assert_eq!(acc.available, Amount(7_500), "available should be 0.7500");
        assert_eq!(acc.held, Amount(0));
        assert!(!acc.locked);

        let rec = state.engine.txs.get(&tx).expect("tx recorded");
        assert_eq!(rec.client, c);
        assert_eq!(rec.amount, Amount(12_500));
        assert_eq!(rec.kind, TxKind::Withdrawal);
        assert_eq!(rec.state, DisputeState::Normal);
    }

    #[test]
    fn withdrawal_ignored_if_insufficient_funds() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 2;
        let tx: TxId = 20;

        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount(10_000);
        }

        // act
        let res = process_withdrawal_command(&mut state, &wc(c, tx, 10_001));

        // assert
        assert!(
            res.is_ok(),
            "policy: insufficient funds is ignored, not an error"
        );
        let acc = state.engine.acct(c).unwrap();
        assert_eq!(acc.available, Amount(10_000), "balance unchanged");
        assert!(!state.engine.txs.contains_key(&tx), "no tx recorded");
    }

    #[test]
    fn withdrawal_ignored_if_account_locked() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 3;
        let tx: TxId = 30;

        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount(50_000);
            acc.locked = true;
        }

        // act
        let res = process_withdrawal_command(&mut state, &wc(c, tx, 10_000));

        // assert
        assert!(res.is_ok());
        let acc = state.engine.acct(c).unwrap();
        assert_eq!(acc.available, Amount(50_000), "no changes when locked");
        assert!(!state.engine.txs.contains_key(&tx), "no tx recorded");
    }

    #[test]
    fn withdrawal_ignored_if_duplicate_tx_id() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 4;
        let tx: TxId = 40;

        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount(30_000);
        }

        // act
        process_withdrawal_command(&mut state, &wc(c, tx, 10_000)).unwrap();

        process_withdrawal_command(&mut state, &wc(c, tx, 5_000)).unwrap();

        // assert: only first one applied
        let acc = state.engine.acct(c).unwrap();
        assert_eq!(acc.available, Amount(20_000), "should subtract only once");
        let rec = state.engine.txs.get(&tx).unwrap();
        assert_eq!(rec.amount, Amount(10_000), "original amount retained");
    }
}
