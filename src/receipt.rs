use crate::amount::Amount;
use crate::ids::{AccountId, AssetId, CommitmentId, NullifierId, PositionId, ReceiptId, WindowId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepositReceipt {
    pub id: ReceiptId,
    pub account: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub position: PositionId,
    pub commitment: CommitmentId,
    pub window: WindowId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpendReceipt {
    pub id: ReceiptId,
    pub account: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub position: PositionId,
    pub commitment: CommitmentId,
    pub nullifier: NullifierId,
    pub window: WindowId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawalReceipt {
    pub id: ReceiptId,
    pub account: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub destination: String,
    pub window: WindowId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Receipt {
    Deposit(DepositReceipt),
    Spend(SpendReceipt),
    Withdrawal(WithdrawalReceipt),
}

impl Receipt {
    pub fn id(&self) -> &ReceiptId {
        match self {
            Self::Deposit(receipt) => &receipt.id,
            Self::Spend(receipt) => &receipt.id,
            Self::Withdrawal(receipt) => &receipt.id,
        }
    }
}
