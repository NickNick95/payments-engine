use crate::models::domain_state::Account;
use crate::models::identifiers::{ClientId, TxId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CsvTxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// As read from input CSV
#[derive(Debug, Deserialize)]
pub struct InputRow {
    #[serde(rename = "type")]
    pub t: CsvTxType,
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Option<String>,
}

/// As written to output CSV
#[derive(Debug, Serialize)]
pub struct OutputRow {
    pub client: ClientId,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}

impl From<(&ClientId, &Account)> for OutputRow {
    fn from((client, acc): (&ClientId, &Account)) -> Self {
        Self {
            client: *client,
            available: acc.available.to_string(),
            held: acc.held.to_string(),
            total: acc.total().to_string(),
            locked: acc.locked,
        }
    }
}
