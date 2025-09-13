use crate::errors::{AppErrors, AppResult};
use crate::models::domain_state::{DisputeState, TxKind};
use crate::models::tx_command::DisputeCommand;
use crate::services::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

impl TxCommandTrait for DisputeCommand {
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_deposit_command(app_state, self)
    }
}

fn process_deposit_command(app_state: &mut AppState, cmd: &DisputeCommand) -> AppResult<()> {
    let client = cmd.client;
    let tx = cmd.tx;

    let mut state = app_state.clone();
    let Some(rec) = state.engine.txs.get_mut(&tx) else {
        return Ok(());
    };
    if rec.client != client {
        return Ok(());
    }

    if rec.kind != TxKind::Deposit {
        return Ok(());
    }
    if rec.state != DisputeState::Normal {
        return Ok(());
    }

    let acc = app_state.engine.acct_mut(client);
    if acc.available.0 < rec.amount.0 {
        return Ok(());
    }
    acc.available = acc
        .available
        .checked_sub(rec.amount)
        .ok_or(AppErrors::Overflow)?;
    acc.held = acc
        .held
        .checked_add(rec.amount)
        .ok_or(AppErrors::Overflow)?;
    rec.state = DisputeState::Disputed;
    Ok(())
}
