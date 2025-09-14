use crate::errors::{AppErrors, AppResult};
use crate::models::amount::Amount;
use crate::models::domain_state::DisputeState;
use crate::models::tx_command::ChargebackCommand;
use crate::services::commands::traits::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

/// Implements the `TxCommandTrait` for the `ChargebackCommand` struct.
/// This enables execution of chargeback commands within the application state.
impl TxCommandTrait for ChargebackCommand {
    /// Executes the chargeback command by processing it and updating the application state.
    ///
    /// # Arguments
    /// * `app_state` - A mutable reference to the application state.
    ///
    /// # Returns
    /// * `AppResult<()>` - Returns `Ok(())` if the command is successfully executed,
    ///   or an `AppErrors` variant if an error occurs.
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_chargeback_command(app_state, self)
    }
}

/// Processes a chargeback command and updates the application state.
///
/// A chargeback finalizes a dispute: the disputed amount is removed from `held`,
/// the transaction state is set to `ChargedBack`, and the account is locked.
///
/// # Arguments
/// * `app_state` - A mutable reference to the application state.
/// * `cmd` - A reference to the `ChargebackCommand` containing client and transaction details.
///
/// # Returns
/// * `AppResult<()>` - Returns `Ok(())` if the chargeback is processed successfully,
///   or an `AppErrors` variant if an error occurs.
fn process_chargeback_command(app_state: &mut AppState, cmd: &ChargebackCommand) -> AppResult<()> {
    let client = cmd.client;
    let tx = cmd.tx;

    let (amount, ok) = if let Some(rec) = app_state.engine.txs.get(&tx) {
        if rec.client != client || rec.state != DisputeState::Disputed {
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
        acc.held = acc.held.checked_sub(amount).ok_or(AppErrors::Overflow)?;
        acc.locked = true;
    }

    if let Some(rec) = app_state.engine.txs.get_mut(&tx) {
        rec.state = DisputeState::ChargedBack;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::domain_state::{TxKind, TxRecord};
    use crate::models::identifiers::{ClientId, TxId};

    fn disputed_deposit_record(client: ClientId, amount: Amount) -> TxRecord {
        TxRecord {
            client,
            kind: TxKind::Deposit,
            amount,
            state: DisputeState::Disputed,
        }
    }

    #[test]
    fn chargeback_happy_path_locks_and_reduces_held() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 2;
        let tx: TxId = 200;
        let amt = Amount(20_000); // 2.0000

        state.engine.txs.insert(tx, disputed_deposit_record(c, amt));
        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount::zero();
            acc.held = amt;
            acc.locked = false;
        }

        // act
        let res = process_chargeback_command(&mut state, &ChargebackCommand { client: c, tx });

        // assert
        assert!(res.is_ok());
        let acc = state.engine.acct(c).expect("account exists");
        assert_eq!(acc.available, Amount::zero());
        assert_eq!(acc.held, Amount::zero());
        assert!(acc.locked, "account should be locked after chargeback");

        let rec = state.engine.txs.get(&tx).expect("tx exists");
        assert_eq!(rec.state, DisputeState::ChargedBack);
    }

    #[test]
    fn chargeback_ignored_if_tx_missing() {
        let mut state = AppState::default();
        let c: ClientId = 3;
        let tx: TxId = 300;

        // act
        let res = process_chargeback_command(&mut state, &ChargebackCommand { client: c, tx });

        // assert
        assert!(res.is_ok());
        assert!(!state.engine.txs.contains_key(&tx));
        assert!(state.engine.acct(c).is_none());
    }

    #[test]
    fn chargeback_ignored_if_wrong_client_or_not_disputed() {
        let mut state = AppState::default();
        let c: ClientId = 4;
        let other: ClientId = 5;
        let tx: TxId = 400;
        let amt = Amount(10_000);

        // Not disputed yet
        state.engine.txs.insert(
            tx,
            TxRecord {
                client: other,
                kind: TxKind::Deposit,
                amount: amt,
                state: DisputeState::Normal,
            },
        );
        {
            let acc = state.engine.acct_mut(other);
            acc.held = Amount::zero();
            acc.available = amt;
        }

        // act (client mismatch and not disputed)
        let res = process_chargeback_command(&mut state, &ChargebackCommand { client: c, tx });

        // assert
        assert!(res.is_ok()); // ignored
        let rec = state.engine.txs.get(&tx).unwrap();
        assert_eq!(rec.state, DisputeState::Normal);
        let acc_other = state.engine.acct(other).unwrap();
        assert_eq!(acc_other.available, amt);
        assert_eq!(acc_other.held, Amount::zero());
        assert!(!acc_other.locked);
    }
}
