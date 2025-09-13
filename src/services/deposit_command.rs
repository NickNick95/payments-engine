use crate::errors::{AppErrors, AppResult};
use crate::models::domain_state::{DisputeState, TxKind, TxRecord};
use crate::models::tx_command::DepositCommand;
use crate::services::tx_command_trait::TxCommandTrait;
use crate::state::AppState;

impl TxCommandTrait for DepositCommand {
    fn execute(&self, app_state: &mut AppState) -> AppResult<()> {
        process_deposit_command(app_state, self)
    }
}

fn process_deposit_command(app_state: &mut AppState, cmd: &DepositCommand) -> AppResult<()> {
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
    acc.available = acc
        .available
        .checked_add(amount)
        .ok_or(AppErrors::Overflow)?;
    app_state.engine.txs.insert(
        tx,
        TxRecord {
            client,
            kind: TxKind::Deposit,
            amount,
            state: DisputeState::Normal,
        },
    );
    Ok(())
}
