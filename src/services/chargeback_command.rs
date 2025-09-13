use crate::errors::{AppErrors, AppResult};
use crate::models::domain_state::DisputeState;
use crate::models::tx_command::ChargebackCommand;
use crate::services::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

impl TxCommandTrait for ChargebackCommand {
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_chargeback_command(app_state, self)
    }
}

fn process_chargeback_command(app_state: &mut AppState, cmd: &ChargebackCommand) -> AppResult<()> {
    let client = cmd.client;
    let tx = cmd.tx;

    let mut state = app_state.clone();
    let Some(rec) = state.engine.txs.get_mut(&tx) else {
        return Ok(());
    };
    if rec.client != client {
        return Ok(());
    }
    if rec.state != DisputeState::Disputed {
        return Ok(());
    }

    let acc = app_state.engine.acct_mut(client);
    acc.held = acc
        .held
        .checked_sub(rec.amount)
        .ok_or(AppErrors::Overflow)?;
    acc.locked = true;
    rec.state = DisputeState::ChargedBack;
    Ok(())
}
