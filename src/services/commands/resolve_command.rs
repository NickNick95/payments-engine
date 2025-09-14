use crate::errors::{AppErrors, AppResult};
use crate::models::amount::Amount;
use crate::models::domain_state::DisputeState;
use crate::models::tx_command::ResolveCommand;
use crate::services::commands::traits::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

/// Implements the `TxCommandTrait` for the `ResolveCommand` struct.
/// This allows the execution of resolve commands within the application state.
impl TxCommandTrait for ResolveCommand {
    /// Executes the resolve command by processing it and updating the application state.
    ///
    /// # Arguments
    /// * `app_state` - A mutable reference to the application state.
    ///
    /// # Returns
    /// * `AppResult<()>` - Returns `Ok(())` if the command is successfully executed,
    ///   or an `AppErrors` variant if an error occurs.
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_resolve_command(app_state, self)
    }
}

/// Processes a resolve command and updates the application state.
///
/// # Arguments
/// * `app_state` - A mutable reference to the application state.
/// * `cmd` - A reference to the `ResolveCommand` to be processed.
///
/// # Returns
/// * `AppResult<()>` - Returns `Ok(())` if the command is successfully processed,
///   or an `AppErrors` variant if an error occurs.
fn process_resolve_command(app_state: &mut AppState, cmd: &ResolveCommand) -> AppResult<()> {
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

        if acc.held.0 < amount.0 {
            return Err(AppErrors::Overflow);
        }

        acc.held = acc.held.checked_sub(amount).ok_or(AppErrors::Overflow)?;
        acc.available = acc
            .available
            .checked_add(amount)
            .ok_or(AppErrors::Overflow)?;
    }

    if let Some(rec) = app_state.engine.txs.get_mut(&tx) {
        rec.state = DisputeState::Normal;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::domain_state::{TxKind, TxRecord};
    use crate::models::identifiers::{ClientId, TxId};

    fn disputed_deposit(client: ClientId, amount: Amount) -> TxRecord {
        TxRecord {
            client,
            kind: TxKind::Deposit,
            amount,
            state: DisputeState::Disputed,
        }
    }

    fn normal_deposit(client: ClientId, amount: Amount) -> TxRecord {
        TxRecord {
            client,
            kind: TxKind::Deposit,
            amount,
            state: DisputeState::Normal,
        }
    }

    #[test]
    fn resolve_happy_path_moves_held_to_available_and_marks_normal() {
        // arrange
        let mut state = AppState::default();
        let c: ClientId = 1;
        let tx: TxId = 100;
        let amt = Amount(12_345);

        state.engine.txs.insert(tx, disputed_deposit(c, amt));
        {
            let acc = state.engine.acct_mut(c);
            acc.available = Amount(50_000);
            acc.held = amt;
        }

        // act
        let res = process_resolve_command(&mut state, &ResolveCommand { client: c, tx });

        // assert
        assert!(res.is_ok());
        let acc = state.engine.acct(c).expect("account exists");
        assert_eq!(acc.held, Amount(0));
        assert_eq!(acc.available, Amount(62_345));
        let rec = state.engine.txs.get(&tx).expect("tx exists");
        assert_eq!(rec.state, DisputeState::Normal);
    }

    #[test]
    fn resolve_ignored_if_tx_missing() {
        let mut state = AppState::default();
        let c: ClientId = 2;
        let tx: TxId = 200;

        let res = process_resolve_command(&mut state, &ResolveCommand { client: c, tx });
        assert!(res.is_ok());
        assert!(!state.engine.txs.contains_key(&tx));
        assert!(state.engine.acct(c).is_none());
    }

    #[test]
    fn resolve_ignored_if_wrong_client() {
        let mut state = AppState::default();
        let owner: ClientId = 3;
        let caller: ClientId = 33;
        let tx: TxId = 300;
        let amt = Amount(10_000);

        state.engine.txs.insert(tx, disputed_deposit(owner, amt));
        {
            let acc = state.engine.acct_mut(owner);
            acc.held = amt;
        }

        let res = process_resolve_command(&mut state, &ResolveCommand { client: caller, tx });
        assert!(res.is_ok());

        let acc = state.engine.acct(owner).unwrap();
        assert_eq!(acc.held, amt);
        assert_eq!(acc.available, Amount(0));
        assert_eq!(
            state.engine.txs.get(&tx).unwrap().state,
            DisputeState::Disputed
        );
    }

    #[test]
    fn resolve_ignored_if_not_in_disputed_state() {
        let mut state = AppState::default();
        let c: ClientId = 4;
        let tx: TxId = 400;
        let amt = Amount(7_500);

        state.engine.txs.insert(tx, normal_deposit(c, amt));
        {
            let acc = state.engine.acct_mut(c);
            acc.held = amt;
        }

        let res = process_resolve_command(&mut state, &ResolveCommand { client: c, tx });
        assert!(res.is_ok());

        let acc = state.engine.acct(c).unwrap();
        assert_eq!(acc.held, amt);
        assert_eq!(acc.available, Amount(0));
        assert_eq!(
            state.engine.txs.get(&tx).unwrap().state,
            DisputeState::Normal
        );
    }

    #[test]
    fn resolve_errors_if_held_underflow() {
        let mut state = AppState::default();
        let c: ClientId = 5;
        let tx: TxId = 500;
        let amt = Amount(10_000);

        state.engine.txs.insert(tx, disputed_deposit(c, amt));
        {
            let acc = state.engine.acct_mut(c);
            acc.held = Amount(1_000);
        }

        let res = process_resolve_command(&mut state, &ResolveCommand { client: c, tx });
        assert!(matches!(res, Err(AppErrors::Overflow)));
    }

    #[test]
    fn resolve_errors_if_available_overflow() {
        let mut state = AppState::default();
        let c: ClientId = 6;
        let tx: TxId = 600;
        let amt = Amount(10);

        state.engine.txs.insert(tx, disputed_deposit(c, amt));
        {
            let acc = state.engine.acct_mut(c);
            acc.held = amt;
            acc.available = Amount(i64::MAX - 5);
        }

        let res = process_resolve_command(&mut state, &ResolveCommand { client: c, tx });
        assert!(matches!(res, Err(AppErrors::Overflow)));
    }
}
