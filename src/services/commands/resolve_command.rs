use crate::errors::{AppErrors, AppResult};
use crate::models::amount::Amount;
use crate::models::domain_state::DisputeState;
use crate::models::tx_command::ResolveCommand;
use crate::services::commands::traits::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

impl TxCommandTrait for ResolveCommand {
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_resolve_command(app_state, self)
    }
}

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
