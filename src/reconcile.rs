use crate::amount::Amount;
use crate::ids::AssetId;
use crate::ledger::NocturneLedger;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetReconciliation {
    pub asset: AssetId,
    pub deposits: u64,
    pub withdrawals: u64,
    pub reserves: u64,
    pub account_balances: u64,
    pub active_commitments: u64,
    pub spent_commitments: u64,
    pub conservation_holds: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub sequence: u64,
    pub balanced: bool,
    pub nullifiers: usize,
    pub commitments: usize,
    pub positions: usize,
    pub assets: Vec<AssetReconciliation>,
}

pub struct Reconciler;

impl Reconciler {
    pub fn build(ledger: &NocturneLedger, sequence: u64) -> ReconciliationReport {
        let mut active_by_asset: BTreeMap<AssetId, Amount> = BTreeMap::new();
        let mut spent_by_asset: BTreeMap<AssetId, Amount> = BTreeMap::new();

        for commitment in ledger.commitments().values() {
            let target = if commitment.status.is_spendable() {
                &mut active_by_asset
            } else {
                &mut spent_by_asset
            };
            let current = target
                .get(&commitment.asset)
                .copied()
                .unwrap_or(Amount::ZERO);
            target.insert(commitment.asset.clone(), current + commitment.amount);
        }

        let mut assets = Vec::new();
        for vault in ledger.vaults().all() {
            let account_balances = ledger.accounts().total_balance(&vault.asset);
            let active_commitments = active_by_asset
                .get(&vault.asset)
                .copied()
                .unwrap_or(Amount::ZERO);
            let spent_commitments = spent_by_asset
                .get(&vault.asset)
                .copied()
                .unwrap_or(Amount::ZERO);
            assets.push(AssetReconciliation {
                asset: vault.asset.clone(),
                deposits: to_u64(vault.total_deposits),
                withdrawals: to_u64(vault.total_withdrawals),
                reserves: to_u64(vault.reserves),
                account_balances: to_u64(account_balances),
                active_commitments: to_u64(active_commitments),
                spent_commitments: to_u64(spent_commitments),
                conservation_holds: vault.conservation_holds(),
            });
        }

        let balanced = assets.iter().all(|asset| asset.conservation_holds);

        ReconciliationReport {
            sequence,
            balanced,
            nullifiers: ledger.nullifiers().len(),
            commitments: ledger.commitments().len(),
            positions: ledger.positions().len(),
            assets,
        }
    }
}

fn to_u64(amount: Amount) -> u64 {
    amount.units().min(u128::from(u64::MAX)) as u64
}
