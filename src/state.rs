use crate::models::domain_state::{Account, TxRecord};
use crate::models::identifiers::{ClientId, TxId};
use std::collections::HashMap;
/// Represents the application state, which contains the engine responsible
/// for managing accounts and transactions.
#[derive(Clone, Default)]
pub struct AppState {
    /// The engine that handles accounts and transaction records.
    pub engine: Engine,
}

/// Represents the core engine of the application, responsible for managing
/// client accounts and transaction records.
#[derive(Default, Clone)]
pub struct Engine {
    /// A mapping of client IDs to their respective accounts.
    accounts: HashMap<ClientId, Account>,

    /// A mapping of transaction IDs to their respective transaction records.
    pub txs: HashMap<TxId, TxRecord>,
}

impl Engine {
    /// Returns a mutable reference to the account for the given client,
    /// creating a new empty account if it does not exist.
    pub fn acct_mut(&mut self, c: ClientId) -> &mut Account {
        self.accounts.entry(c).or_default()
    }

    /// Returns an iterator over all client accounts.
    pub fn accounts_iter(&self) -> impl Iterator<Item = (&ClientId, &Account)> {
        self.accounts.iter()
    }

    /// Returns a mutable reference to the account for the given client,
    /// or `None` if the account does not exist.
    pub fn acct_mut_if_exists(&mut self, client: &ClientId) -> Option<&mut Account> {
        self.accounts.get_mut(client)
    }

    /// Returns an immutable reference to the account for the given client,
    /// or `None` if the account does not exist.
    pub fn acct(&self, client: ClientId) -> Option<&Account> {
        self.accounts.get(&client)
    }
}
