use crate::amount::Amount;
use crate::errors::{NocturneError, Result};
use crate::ids::AssetId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vault {
    pub asset: AssetId,
    pub reserves: Amount,
    pub total_deposits: Amount,
    pub total_withdrawals: Amount,
    pub fees_accrued: Amount,
}

impl Vault {
    pub fn new(asset: AssetId) -> Self {
        Self {
            asset,
            reserves: Amount::ZERO,
            total_deposits: Amount::ZERO,
            total_withdrawals: Amount::ZERO,
            fees_accrued: Amount::ZERO,
        }
    }

    pub fn deposit(&mut self, amount: Amount) -> Result<()> {
        self.reserves = self.reserves.checked_add(amount)?;
        self.total_deposits = self.total_deposits.checked_add(amount)?;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: Amount) -> Result<()> {
        if self.reserves < amount {
            return Err(NocturneError::VaultReserveTooLow {
                asset: self.asset.clone(),
                requested: amount.units(),
                available: self.reserves.units(),
            });
        }
        self.reserves = self.reserves.checked_sub(amount)?;
        self.total_withdrawals = self.total_withdrawals.checked_add(amount)?;
        Ok(())
    }

    pub fn accrue_fee(&mut self, amount: Amount) -> Result<()> {
        self.fees_accrued = self.fees_accrued.checked_add(amount)?;
        Ok(())
    }

    pub fn conservation_holds(&self) -> bool {
        self.total_deposits == self.reserves + self.total_withdrawals
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VaultBook {
    vaults: BTreeMap<AssetId, Vault>,
}

impl VaultBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure(&mut self, asset: AssetId) -> &mut Vault {
        self.vaults
            .entry(asset.clone())
            .or_insert_with(|| Vault::new(asset))
    }

    pub fn get(&self, asset: &AssetId) -> Result<&Vault> {
        self.vaults
            .get(asset)
            .ok_or_else(|| NocturneError::UnknownAsset(asset.clone()))
    }

    pub fn get_mut(&mut self, asset: &AssetId) -> Result<&mut Vault> {
        self.vaults
            .get_mut(asset)
            .ok_or_else(|| NocturneError::UnknownAsset(asset.clone()))
    }

    pub fn deposit(&mut self, asset: AssetId, amount: Amount) -> Result<()> {
        self.ensure(asset).deposit(amount)
    }

    pub fn withdraw(&mut self, asset: &AssetId, amount: Amount) -> Result<()> {
        self.get_mut(asset)?.withdraw(amount)
    }

    pub fn all(&self) -> impl Iterator<Item = &Vault> {
        self.vaults.values()
    }
}
