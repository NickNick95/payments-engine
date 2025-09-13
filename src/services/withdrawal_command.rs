use crate::errors::{AppErrors, AppResult};
use crate::models::domain_state::{DisputeState, TxKind, TxRecord};
use crate::models::tx_command::WithdrawalCommand;
use crate::services::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

impl TxCommandTrait for WithdrawalCommand {
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_withdrawal_command(app_state, self)
    }
}

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
