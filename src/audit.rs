use crate::amount::Amount;
use crate::ids::{AssetId, PositionId};
use crate::ledger::NocturneLedger;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionExposure {
    pub position: PositionId,
    pub asset: AssetId,
    pub principal: u64,
    pub commitments_issued: usize,
    pub commitments_consumed: usize,
    pub active_value: u64,
    pub consumed_value: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditSnapshot {
    pub exposures: Vec<PositionExposure>,
    pub total_active_by_asset: BTreeMap<AssetId, u64>,
    pub total_consumed_by_asset: BTreeMap<AssetId, u64>,
}

pub struct LedgerAuditor;

impl LedgerAuditor {
    pub fn snapshot(ledger: &NocturneLedger) -> AuditSnapshot {
        let mut active: BTreeMap<PositionId, Amount> = BTreeMap::new();
        let mut consumed: BTreeMap<PositionId, Amount> = BTreeMap::new();
        let mut total_active_by_asset: BTreeMap<AssetId, Amount> = BTreeMap::new();
        let mut total_consumed_by_asset: BTreeMap<AssetId, Amount> = BTreeMap::new();

        for commitment in ledger.commitments().values() {
            if commitment.status.is_spendable() {
                let current = active
                    .get(&commitment.position)
                    .copied()
                    .unwrap_or(Amount::ZERO);
                active.insert(commitment.position.clone(), current + commitment.amount);
                let aggregate = total_active_by_asset
                    .get(&commitment.asset)
                    .copied()
                    .unwrap_or(Amount::ZERO);
                total_active_by_asset
                    .insert(commitment.asset.clone(), aggregate + commitment.amount);
            } else {
                let current = consumed
                    .get(&commitment.position)
                    .copied()
                    .unwrap_or(Amount::ZERO);
                consumed.insert(commitment.position.clone(), current + commitment.amount);
                let aggregate = total_consumed_by_asset
                    .get(&commitment.asset)
                    .copied()
                    .unwrap_or(Amount::ZERO);
                total_consumed_by_asset
                    .insert(commitment.asset.clone(), aggregate + commitment.amount);
            }
        }

        let exposures = ledger
            .positions()
            .all()
            .map(|position| PositionExposure {
                position: position.id.clone(),
                asset: position.asset.clone(),
                principal: to_u64(position.principal),
                commitments_issued: position.notes_issued,
                commitments_consumed: position.notes_consumed,
                active_value: active
                    .get(&position.id)
                    .copied()
                    .unwrap_or(Amount::ZERO)
                    .units()
                    .min(u128::from(u64::MAX)) as u64,
                consumed_value: consumed
                    .get(&position.id)
                    .copied()
                    .unwrap_or(Amount::ZERO)
                    .units()
                    .min(u128::from(u64::MAX)) as u64,
            })
            .collect();

        AuditSnapshot {
            exposures,
            total_active_by_asset: total_active_by_asset
                .into_iter()
                .map(|(asset, amount)| (asset, to_u64(amount)))
                .collect(),
            total_consumed_by_asset: total_consumed_by_asset
                .into_iter()
                .map(|(asset, amount)| (asset, to_u64(amount)))
                .collect(),
        }
    }
}

fn to_u64(amount: Amount) -> u64 {
    amount.units().min(u128::from(u64::MAX)) as u64
}
