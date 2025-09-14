use crate::models::amount::Amount;
use crate::models::identifiers::ClientId;

/// Represents the type of a transaction.
/// A transaction can either be a deposit or a withdrawal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxKind {
    /// A deposit transaction.
    Deposit,
    /// A withdrawal transaction.
    Withdrawal,
}

/// Represents the state of a dispute for a transaction.
/// A transaction can be in one of three states: normal, disputed, or charged back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisputeState {
    /// The transaction is in a normal state (no dispute).
    Normal,
    /// The transaction is currently disputed.
    Disputed,
    /// The transaction has been charged back.
    ChargedBack,
}

/// Represents a record of a transaction.
/// Contains details about the client, transaction type, amount, and dispute state.
#[derive(Debug, Clone)]
pub struct TxRecord {
    /// The ID of the client associated with the transaction.
    pub client: ClientId,
    /// The type of the transaction (deposit or withdrawal).
    pub kind: TxKind,
    /// The amount involved in the transaction.
    pub amount: Amount,
    /// The current dispute state of the transaction.
    pub state: DisputeState,
}

/// Represents a client's account.
/// Contains details about the available balance, held balance, and lock status.
#[derive(Default, Debug, Clone)]
pub struct Account {
    /// The available balance in the account.
    pub available: Amount,
    /// The held balance in the account (e.g., due to disputes).
    pub held: Amount,
    /// Indicates whether the account is locked.
    pub locked: bool,
}

impl Account {
    /// Calculates the total balance of the account.
    /// The total balance is the sum of the available and held balances.
    ///
    /// # Returns
    ///
    /// * `Amount` - The total balance of the account.
    #[inline]
    pub fn total(&self) -> Amount {
        Amount(self.available.0 + self.held.0)
    }
}
