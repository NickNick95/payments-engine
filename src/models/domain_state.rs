use crate::models::amount::Amount;
use crate::models::identifiers::ClientId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxKind {
    Deposit,
    Withdrawal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisputeState {
    Normal,
    Disputed,
    ChargedBack,
}

#[derive(Debug, Clone)]
pub struct TxRecord {
    pub client: ClientId,
    pub kind: TxKind,
    pub amount: Amount,
    pub state: DisputeState,
}

#[derive(Default, Debug, Clone)]
pub struct Account {
    pub available: Amount,
    pub held: Amount,
    pub locked: bool,
}
impl Account {
    #[inline]
    pub fn total(&self) -> Amount {
        Amount(self.available.0 + self.held.0)
    }
}
