use crate::errors::{AppErrors, AppResult};
use crate::models::amount::Amount;
use crate::models::domain_state::{DisputeState, TxKind};
use crate::models::tx_command::DisputeCommand;
use crate::services::commands::traits::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

/// Implements the `TxCommandTrait` for the `DisputeCommand` struct.
/// This allows the execution of dispute commands within the application state.
impl TxCommandTrait for DisputeCommand {
    /// Executes the dispute command by processing it and updating the application state.
    ///
    /// # Arguments
    /// * `app_state` - A mutable reference to the application state.
    ///
    /// # Returns
    /// * `AppResult<()>` - Returns `Ok(())` if the command is successfully executed,
    ///   or an `AppErrors` variant if an error occurs.
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_dispute_command(app_state, self)
    }
}

/// Processes a dispute command and updates the application state.
///
/// A dispute moves funds from `available` to `held` for a given deposit transaction,
/// and marks the transaction state as `Disputed`.
///
/// # Arguments
/// * `app_state` - A mutable reference to the application state.
/// * `cmd` - A reference to the `DisputeCommand` containing client and transaction details.
///
/// # Returns
/// * `AppResult<()>` - Returns `Ok(())` if the dispute is processed successfully,
///   or an `AppErrors` variant if an error occurs.
fn process_dispute_command(app_state: &mut AppState, cmd: &DisputeCommand) -> AppResult<()> {
    let client = cmd.client;
    let tx = cmd.tx;

    let (amount, ok) = if let Some(rec) = app_state.engine.txs.get(&tx) {
        if rec.client != client || rec.kind != TxKind::Deposit || rec.state != DisputeState::Normal
        {
            (Amount::zero(), false)
        } else {
            (rec.amount, true)
        }
    } else {
        (Amount::zero(), false)
    };
    if !ok {
        return Ok(());
    }

    {
        let acc = app_state.engine.acct_mut(client);
        if acc.available.0 < amount.0 {
            return Ok(());
        }
        acc.available = acc
            .available
            .checked_sub(amount)
            .ok_or(AppErrors::Overflow)?;
        acc.held = acc.held.checked_add(amount).ok_or(AppErrors::Overflow)?;
    }

    if let Some(rec) = app_state.engine.txs.get_mut(&tx) {
        rec.state = DisputeState::Disputed;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::domain_state::TxRecord;
    use crate::models::identifiers::{ClientId, TxId};

    fn normal_deposit(client: ClientId, amount: Amount) -> TxRecord {
        TxRecord {
            client,
            kind: TxKind::Deposit,
            amount,
            state: DisputeState::Normal,
        }
    }

    fn normal_withdrawal(client: ClientId, amount: Amount) -> TxRecord {
        TxRecord {
            client,
            kind: TxKind::Withdrawal,
            amount,
            state: DisputeState::Normal,
        }
    }

    #[test]
    fn dispute_happy_path_moves_available_to_held_and_marks_disputed() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 1;
        let tx: TxId = 100;
        let amt = Amount(12_345); // 1.2345

        state.engine.txs.insert(tx, normal_deposit(c, amt));
        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount(50_000); // 5.0000
            acc.held = Amount(0);
        }

        // act
        let res = process_dispute_command(&mut state, &DisputeCommand { client: c, tx });

        // assert
        assert!(res.is_ok());
        let acc = state.engine.acct(c).expect("account exists");
        assert_eq!(acc.available, Amount(50_000 - 12_345));
        assert_eq!(acc.held, amt);
        let rec = state.engine.txs.get(&tx).expect("tx exists");
        assert_eq!(rec.state, DisputeState::Disputed);
    }

    #[test]
    fn dispute_ignored_if_tx_missing() {
        let mut state = AppState::default();
        let c: ClientId = 2;
        let tx: TxId = 200;

        let res = process_dispute_command(&mut state, &DisputeCommand { client: c, tx });
        assert!(res.is_ok());
        assert!(!state.engine.txs.contains_key(&tx));
        assert!(state.engine.acct(c).is_none());
    }

    #[test]
    fn dispute_ignored_if_wrong_client() {
        let mut state = AppState::default();
        let owner: ClientId = 3;
        let caller: ClientId = 33;
        let tx: TxId = 300;
        let amt = Amount(10_000);

        state.engine.txs.insert(tx, normal_deposit(owner, amt));
        {
            let acc = state.engine.acct_mut(owner);
            acc.available = Amount(10_000);
        }

        let res = process_dispute_command(&mut state, &DisputeCommand { client: caller, tx });
        assert!(res.is_ok());

        let acc = state.engine.acct(owner).unwrap();
        assert_eq!(acc.available, Amount(10_000));
        assert_eq!(acc.held, Amount(0));
        assert_eq!(
            state.engine.txs.get(&tx).unwrap().state,
            DisputeState::Normal
        );
    }

    #[test]
    fn dispute_ignored_if_tx_is_withdrawal() {
        let mut state = AppState::default();
        let c: ClientId = 4;
        let tx: TxId = 400;
        let amt = Amount(7_500);

        state.engine.txs.insert(tx, normal_withdrawal(c, amt));
        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount(10_000);
        }

        let res = process_dispute_command(&mut state, &DisputeCommand { client: c, tx });
        assert!(res.is_ok());

        let acc = state.engine.acct(c).unwrap();
        assert_eq!(acc.available, Amount(10_000));
        assert_eq!(acc.held, Amount(0));
        assert_eq!(
            state.engine.txs.get(&tx).unwrap().state,
            DisputeState::Normal
        );
    }

    #[test]
    fn dispute_ignored_if_already_disputed() {
        let mut state = AppState::default();
        let c: ClientId = 5;
        let tx: TxId = 500;
        let amt = Amount(4_000);

        state.engine.txs.insert(
            tx,
            TxRecord {
                client: c,
                kind: TxKind::Deposit,
                amount: amt,
                state: DisputeState::Disputed,
            },
        );
        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount(10_000);
            acc.held = amt;
        }

        let res = process_dispute_command(&mut state, &DisputeCommand { client: c, tx });
        assert!(res.is_ok());

        let acc = state.engine.acct(c).unwrap();
        assert_eq!(acc.available, Amount(10_000));
        assert_eq!(acc.held, amt);
        assert_eq!(
            state.engine.txs.get(&tx).unwrap().state,
            DisputeState::Disputed
        );
    }

    #[test]
    fn dispute_ignored_if_insufficient_available() {
        let mut state = AppState::default();
        let c: ClientId = 6;
        let tx: TxId = 600;
        let amt = Amount(5_000);

        state.engine.txs.insert(tx, normal_deposit(c, amt));
        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount(4_999);
            acc.held = Amount(0);
        }

        let res = process_dispute_command(&mut state, &DisputeCommand { client: c, tx });
        assert!(res.is_ok());

        let acc = state.engine.acct(c).unwrap();
        assert_eq!(acc.available, Amount(4_999), "no change");
        assert_eq!(acc.held, Amount(0), "no change");
        assert_eq!(
            state.engine.txs.get(&tx).unwrap().state,
            DisputeState::Normal
        );
    }

    #[test]
    fn dispute_errors_on_held_overflow() {
        let mut state = AppState::default();
        let c: ClientId = 7;
        let tx: TxId = 700;
        let amt = Amount(10);

        state.engine.txs.insert(tx, normal_deposit(c, amt));
        {
            let acc = state.engine.acct_mut(c);
            acc.available = amt;
            acc.held = Amount(i64::MAX - 5);
        }

        let res = process_dispute_command(&mut state, &DisputeCommand { client: c, tx });
        assert!(matches!(res, Err(AppErrors::Overflow)));
    }
}
