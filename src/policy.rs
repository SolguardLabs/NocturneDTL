use crate::amount::{Amount, BasisPoints};
use crate::errors::{NocturneError, Result};
use crate::ids::AssetId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdmissionPolicy {
    pub minimum_note_amount: Amount,
    pub max_rollover_gap: u64,
    pub max_notes_per_position: usize,
    pub require_registered_asset: bool,
}

impl Default for AdmissionPolicy {
    fn default() -> Self {
        Self {
            minimum_note_amount: Amount::from_units(1),
            max_rollover_gap: 8,
            max_notes_per_position: 32,
            require_registered_asset: true,
        }
    }
}

impl AdmissionPolicy {
    pub fn validate_amount(&self, amount: Amount) -> Result<()> {
        if amount < self.minimum_note_amount {
            return Err(NocturneError::InvalidAmount);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawalPolicy {
    pub minimum_withdrawal: Amount,
    pub daily_limit: Amount,
    pub fee_bps: BasisPoints,
    pub allow_partial_batches: bool,
}

impl Default for WithdrawalPolicy {
    fn default() -> Self {
        Self {
            minimum_withdrawal: Amount::from_units(1),
            daily_limit: Amount::from_units(10_000_000_000),
            fee_bps: BasisPoints::ZERO,
            allow_partial_batches: true,
        }
    }
}

impl WithdrawalPolicy {
    pub fn validate(&self, amount: Amount) -> Result<()> {
        if amount < self.minimum_withdrawal || amount > self.daily_limit {
            return Err(NocturneError::InvalidAmount);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AssetRiskPolicy {
    limits: BTreeMap<AssetId, Amount>,
}

impl AssetRiskPolicy {
    pub fn set_limit(&mut self, asset: AssetId, limit: Amount) {
        self.limits.insert(asset, limit);
    }

    pub fn limit_for(&self, asset: &AssetId) -> Option<Amount> {
        self.limits.get(asset).copied()
    }

    pub fn check(&self, asset: &AssetId, amount: Amount) -> Result<()> {
        if let Some(limit) = self.limit_for(asset) {
            if amount > limit {
                return Err(NocturneError::InvalidAmount);
            }
        }
        Ok(())
    }
}
