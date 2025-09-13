use crate::models::amount::Amount;
use crate::models::identifiers::{ClientId, TxId};

#[derive(Debug, Clone)]
pub enum TxCommand {
    Deposit(DepositCommand),
    Withdrawal(WithdrawalCommand),
    Dispute(DisputeCommand),
    Resolve(ResolveCommand),
    Chargeback(ChargebackCommand),
}

#[derive(Debug, Clone)]
pub struct DepositCommand {
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Amount,
}

#[derive(Debug, Clone)]
pub struct WithdrawalCommand {
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Amount,
}

#[derive(Debug, Clone)]
pub struct DisputeCommand {
    pub client: ClientId,
    pub tx: TxId,
}

#[derive(Debug, Clone)]
pub struct ResolveCommand {
    pub client: ClientId,
    pub tx: TxId,
}

#[derive(Debug, Clone)]
pub struct ChargebackCommand {
    pub client: ClientId,
    pub tx: TxId,
}
