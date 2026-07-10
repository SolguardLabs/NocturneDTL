use crate::ids::{AccountId, AssetId, CommitmentId, NullifierId, PositionId, WindowId};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, NocturneError>;

#[derive(Debug, Error)]
pub enum NocturneError {
    #[error("asset not registered: {0}")]
    UnknownAsset(AssetId),
    #[error("account not found: {0}")]
    UnknownAccount(AccountId),
    #[error("commitment not found: {0}")]
    UnknownCommitment(CommitmentId),
    #[error("position not found: {0}")]
    UnknownPosition(PositionId),
    #[error("window not found: {0}")]
    UnknownWindow(WindowId),
    #[error("invalid amount")]
    InvalidAmount,
    #[error("amount overflow")]
    AmountOverflow,
    #[error(
        "balance too low for {account} on {asset}: requested {requested}, available {available}"
    )]
    BalanceTooLow {
        account: AccountId,
        asset: AssetId,
        requested: u128,
        available: u128,
    },
    #[error("vault reserve too low for {asset}: requested {requested}, available {available}")]
    VaultReserveTooLow {
        asset: AssetId,
        requested: u128,
        available: u128,
    },
    #[error("nullifier already spent: {0}")]
    NullifierAlreadySpent(NullifierId),
    #[error("commitment already consumed: {0}")]
    CommitmentConsumed(CommitmentId),
    #[error("commitment opening mismatch: {0}")]
    CommitmentMismatch(CommitmentId),
    #[error("window is not accepting commitments: {0}")]
    WindowClosed(WindowId),
    #[error("position metadata mismatch: {0}")]
    PositionMismatch(PositionId),
    #[error("scenario note label not found: {0}")]
    ScenarioNoteMissing(String),
    #[error("scenario error: {0}")]
    Scenario(String),
    #[error("json error: {0}")]
    Json(String),
}

impl From<serde_json::Error> for NocturneError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}
