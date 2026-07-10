use crate::amount::Amount;
use crate::errors::{NocturneError, Result};
use crate::ids::{AccountId, AssetId, UserId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
    pub owner: UserId,
    pub balances: BTreeMap<AssetId, Amount>,
    pub withdrawals: BTreeMap<AssetId, Amount>,
}

impl Account {
    pub fn new(id: AccountId, owner: UserId) -> Self {
        Self {
            id,
            owner,
            balances: BTreeMap::new(),
            withdrawals: BTreeMap::new(),
        }
    }

    pub fn balance(&self, asset: &AssetId) -> Amount {
        self.balances.get(asset).copied().unwrap_or(Amount::ZERO)
    }

    pub fn credit(&mut self, asset: AssetId, amount: Amount) -> Result<()> {
        let current = self.balance(&asset);
        self.balances.insert(asset, current.checked_add(amount)?);
        Ok(())
    }

    pub fn debit(&mut self, asset: &AssetId, amount: Amount) -> Result<()> {
        let current = self.balance(asset);
        if current < amount {
            return Err(NocturneError::BalanceTooLow {
                account: self.id.clone(),
                asset: asset.clone(),
                requested: amount.units(),
                available: current.units(),
            });
        }
        self.balances
            .insert(asset.clone(), current.checked_sub(amount)?);
        Ok(())
    }

    pub fn record_withdrawal(&mut self, asset: AssetId, amount: Amount) -> Result<()> {
        let current = self
            .withdrawals
            .get(&asset)
            .copied()
            .unwrap_or(Amount::ZERO);
        self.withdrawals.insert(asset, current.checked_add(amount)?);
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AccountBook {
    accounts: BTreeMap<AccountId, Account>,
}

impl AccountBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure(&mut self, id: AccountId, owner: UserId) -> &mut Account {
        self.accounts
            .entry(id.clone())
            .or_insert_with(|| Account::new(id, owner))
    }

    pub fn get(&self, id: &AccountId) -> Result<&Account> {
        self.accounts
            .get(id)
            .ok_or_else(|| NocturneError::UnknownAccount(id.clone()))
    }

    pub fn get_mut(&mut self, id: &AccountId) -> Result<&mut Account> {
        self.accounts
            .get_mut(id)
            .ok_or_else(|| NocturneError::UnknownAccount(id.clone()))
    }

    pub fn credit(
        &mut self,
        account: AccountId,
        owner: UserId,
        asset: AssetId,
        amount: Amount,
    ) -> Result<()> {
        self.ensure(account, owner).credit(asset, amount)
    }

    pub fn debit(&mut self, account: &AccountId, asset: &AssetId, amount: Amount) -> Result<()> {
        self.get_mut(account)?.debit(asset, amount)
    }

    pub fn record_withdrawal(
        &mut self,
        account: &AccountId,
        asset: AssetId,
        amount: Amount,
    ) -> Result<()> {
        self.get_mut(account)?.record_withdrawal(asset, amount)
    }

    pub fn total_balance(&self, asset: &AssetId) -> Amount {
        self.accounts
            .values()
            .fold(Amount::ZERO, |acc, account| acc + account.balance(asset))
    }

    pub fn all(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }
}
