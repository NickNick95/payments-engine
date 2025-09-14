use crate::models::domain_state::Account;
use crate::models::identifiers::{ClientId, TxId};
use serde::{Deserialize, Serialize};

/// Represents the kind of transaction in a CSV file.
/// The variants correspond to different transaction types.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CsvTxType {
    /// A deposit transaction.
    Deposit,
    /// A withdrawal transaction.
    Withdrawal,
    /// A dispute transaction.
    Dispute,
    /// A resolve transaction.
    Resolve,
    /// A chargeback transaction.
    Chargeback,
}

/// Represents a row in the input CSV file.
/// Contains transaction details such as type, client ID, transaction ID, and an optional amount.
#[derive(Debug, Deserialize)]
pub struct InputRow {
    /// The type of the transaction (e.g., deposit, withdrawal).
    #[serde(rename = "type")]
    pub t: CsvTxType,
    /// The ID of the client associated with the transaction.
    pub client: ClientId,
    /// The ID of the transaction.
    pub tx: TxId,
    /// The amount involved in the transaction, if applicable.
    pub amount: Option<String>,
}

/// Represents a row in the output CSV file.
/// Contains account details such as available balance, held balance, total balance, and lock status.
#[derive(Debug, Serialize)]
pub struct OutputRow {
    /// The ID of the client associated with the account.
    pub client: ClientId,
    /// The available balance in the account as a string.
    pub available: String,
    /// The held balance in the account as a string.
    pub held: String,
    /// The total balance in the account as a string.
    pub total: String,
    /// Indicates whether the account is locked.
    pub locked: bool,
}

impl From<(&ClientId, &Account)> for OutputRow {
    /// Converts a tuple of `ClientId` and `Account` into an `OutputRow`.
    ///
    /// # Arguments
    ///
    /// * `(client, acc)` - A tuple containing the client ID and the account details.
    ///
    /// # Returns
    ///
    /// * An `OutputRow` containing the serialized account details.
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
