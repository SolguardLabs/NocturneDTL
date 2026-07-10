use crate::amount::Amount;
use crate::ids::{AccountId, AssetId, BatchId, WindowId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawalRequest {
    pub account: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub destination: String,
    pub requested_window: WindowId,
}

impl WithdrawalRequest {
    pub fn new(
        account: AccountId,
        asset: AssetId,
        amount: Amount,
        destination: impl Into<String>,
        requested_window: WindowId,
    ) -> Self {
        Self {
            account,
            asset,
            amount,
            destination: destination.into(),
            requested_window,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawalBatch {
    pub id: BatchId,
    pub window: WindowId,
    pub requests: Vec<WithdrawalRequest>,
}

impl WithdrawalBatch {
    pub fn new(id: BatchId, window: WindowId) -> Self {
        Self {
            id,
            window,
            requests: Vec::new(),
        }
    }

    pub fn push(&mut self, request: WithdrawalRequest) {
        self.requests.push(request);
    }

    pub fn total_for(&self, asset: &AssetId) -> Amount {
        self.requests
            .iter()
            .filter(|request| &request.asset == asset)
            .fold(Amount::ZERO, |acc, request| acc + request.amount)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WithdrawalPlanner {
    pub batches: Vec<WithdrawalBatch>,
}

impl WithdrawalPlanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append_batch(&mut self, batch: WithdrawalBatch) {
        self.batches.push(batch);
    }

    pub fn pending_count(&self) -> usize {
        self.batches.iter().map(|batch| batch.requests.len()).sum()
    }
}
