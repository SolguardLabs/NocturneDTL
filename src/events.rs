use crate::amount::Amount;
use crate::ids::{AccountId, AssetId, CommitmentId, NullifierId, PositionId, ReceiptId, WindowId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventKind {
    AssetRegistered,
    WindowOpened,
    CommitmentCreated,
    CommitmentRotated,
    CommitmentSpent,
    WithdrawalExecuted,
    ReconciliationBuilt,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerEvent {
    pub seq: u64,
    pub kind: EventKind,
    pub window: Option<WindowId>,
    pub account: Option<AccountId>,
    pub asset: Option<AssetId>,
    pub amount: Option<Amount>,
    pub commitment: Option<CommitmentId>,
    pub nullifier: Option<NullifierId>,
    pub position: Option<PositionId>,
    pub receipt: Option<ReceiptId>,
    pub memo: String,
}

impl LedgerEvent {
    pub fn new(seq: u64, kind: EventKind, memo: impl Into<String>) -> Self {
        Self {
            seq,
            kind,
            window: None,
            account: None,
            asset: None,
            amount: None,
            commitment: None,
            nullifier: None,
            position: None,
            receipt: None,
            memo: memo.into(),
        }
    }

    pub fn with_window(mut self, window: WindowId) -> Self {
        self.window = Some(window);
        self
    }

    pub fn with_account(mut self, account: AccountId) -> Self {
        self.account = Some(account);
        self
    }

    pub fn with_asset_amount(mut self, asset: AssetId, amount: Amount) -> Self {
        self.asset = Some(asset);
        self.amount = Some(amount);
        self
    }

    pub fn with_commitment(mut self, commitment: CommitmentId) -> Self {
        self.commitment = Some(commitment);
        self
    }

    pub fn with_nullifier(mut self, nullifier: NullifierId) -> Self {
        self.nullifier = Some(nullifier);
        self
    }

    pub fn with_position(mut self, position: PositionId) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_receipt(mut self, receipt: ReceiptId) -> Self {
        self.receipt = Some(receipt);
        self
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventLog {
    events: Vec<LedgerEvent>,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: LedgerEvent) {
        self.events.push(event);
    }

    pub fn all(&self) -> &[LedgerEvent] {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
