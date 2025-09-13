use crate::errors::{AppErrors, AppResult};
use crate::models::domain_state::DisputeState;
use crate::models::tx_command::ResolveCommand;
use crate::services::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

impl TxCommandTrait for ResolveCommand {
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_resolve_command(app_state, self)
    }
}

fn process_resolve_command(app_state: &mut AppState, cmd: &ResolveCommand) -> AppResult<()> {
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
    acc.available = acc
        .available
        .checked_add(rec.amount)
        .ok_or(AppErrors::Overflow)?;
    rec.state = DisputeState::Normal;
    Ok(())
}
