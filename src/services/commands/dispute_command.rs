use crate::errors::{AppErrors, AppResult};
use crate::models::amount::Amount;
use crate::models::domain_state::{DisputeState, TxKind};
use crate::models::tx_command::DisputeCommand;
use crate::services::commands::traits::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

impl TxCommandTrait for DisputeCommand {
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_dispute_command(app_state, self)
    }
}

fn process_dispute_command(app_state: &mut AppState, cmd: &DisputeCommand) -> AppResult<()> {
    let client = cmd.client;
    let tx = cmd.tx;
    
    let (amount, ok) = if let Some(rec) = app_state.engine.txs.get(&tx) {
        if rec.client != client {
            (Amount::zero(), false)
        } else if rec.kind != TxKind::Deposit {
            (Amount::zero(), false)
        } else if rec.state != DisputeState::Normal {
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
