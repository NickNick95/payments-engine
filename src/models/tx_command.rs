use crate::models::amount::Amount;
use crate::models::identifiers::{ClientId, TxId};

/// Represents a deposit command.
/// Contains details about the client, transaction ID, and the amount to be deposited.
#[derive(Debug, Clone)]
pub struct DepositCommand {
    /// The ID of the client making the deposit.
    pub client: ClientId,
    /// The unique identifier for the deposit transaction.
    pub tx: TxId,
    /// The amount to be deposited.
    pub amount: Amount,
}

/// Represents a withdrawal command.
/// Contains details about the client, transaction ID, and the amount to be withdrawn.
#[derive(Debug, Clone)]
pub struct WithdrawalCommand {
    /// The ID of the client making the withdrawal.
    pub client: ClientId,
    /// The unique identifier for the withdrawal transaction.
    pub tx: TxId,
    /// The amount to be withdrawn.
    pub amount: Amount,
}

/// Represents a dispute command.
/// Contains details about the client and the transaction being disputed.
#[derive(Debug, Clone)]
pub struct DisputeCommand {
    /// The ID of the client initiating the dispute.
    pub client: ClientId,
    /// The unique identifier for the disputed transaction.
    pub tx: TxId,
}

/// Represents a resolve command.
/// Contains details about the client and the transaction being resolved.
#[derive(Debug, Clone)]
pub struct ResolveCommand {
    /// The ID of the client resolving the dispute.
    pub client: ClientId,
    /// The unique identifier for the resolved transaction.
    pub tx: TxId,
}

/// Represents a chargeback command.
/// Contains details about the client and the transaction being charged back.
#[derive(Debug, Clone)]
pub struct ChargebackCommand {
    /// The ID of the client initiating the chargeback.
    pub client: ClientId,
    /// The unique identifier for the charged-back transaction.
    pub tx: TxId,
}
