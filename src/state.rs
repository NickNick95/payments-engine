use crate::models::domain_state::{Account, TxRecord};
use crate::models::identifiers::{ClientId, TxId};
use std::collections::HashMap;
#[derive(Clone)]
pub struct AppState {
    pub engine: Engine,
}

#[derive(Default, Clone)]
pub struct Engine {
    pub accounts: HashMap<ClientId, Account>,
    pub txs: HashMap<TxId, TxRecord>,
}

impl Engine {
    pub fn acct_mut(&mut self, c: ClientId) -> &mut Account {
        self.accounts.entry(c).or_default()
    }
}
