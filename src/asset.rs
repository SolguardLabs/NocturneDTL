use crate::amount::BasisPoints;
use crate::errors::{NocturneError, Result};
use crate::ids::AssetId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetDefinition {
    pub id: AssetId,
    pub symbol: String,
    pub decimals: u8,
    pub settlement_fee_bps: BasisPoints,
    pub withdrawal_fee_bps: BasisPoints,
    pub minimum_deposit: u128,
    pub enabled: bool,
}

impl AssetDefinition {
    pub fn new(symbol: impl Into<String>, decimals: u8) -> Result<Self> {
        let symbol = symbol.into().to_uppercase();
        Ok(Self {
            id: AssetId::new(symbol.clone()),
            symbol,
            decimals,
            settlement_fee_bps: BasisPoints::new(0)?,
            withdrawal_fee_bps: BasisPoints::new(0)?,
            minimum_deposit: 1,
            enabled: true,
        })
    }

    pub fn with_fees(mut self, settlement_fee_bps: u16, withdrawal_fee_bps: u16) -> Result<Self> {
        self.settlement_fee_bps = BasisPoints::new(settlement_fee_bps)?;
        self.withdrawal_fee_bps = BasisPoints::new(withdrawal_fee_bps)?;
        Ok(self)
    }

    pub fn with_minimum(mut self, minimum_deposit: u128) -> Self {
        self.minimum_deposit = minimum_deposit;
        self
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AssetRegistry {
    assets: BTreeMap<AssetId, AssetDefinition>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_default_assets() -> Result<Self> {
        let mut registry = Self::new();
        registry.register(
            AssetDefinition::new("DTL", 6)?
                .with_fees(3, 2)?
                .with_minimum(10),
        )?;
        registry.register(
            AssetDefinition::new("USDC", 6)?
                .with_fees(2, 1)?
                .with_minimum(10),
        )?;
        Ok(registry)
    }

    pub fn register(&mut self, definition: AssetDefinition) -> Result<()> {
        self.assets.insert(definition.id.clone(), definition);
        Ok(())
    }

    pub fn get(&self, asset: &AssetId) -> Result<&AssetDefinition> {
        self.assets
            .get(asset)
            .filter(|definition| definition.enabled)
            .ok_or_else(|| NocturneError::UnknownAsset(asset.clone()))
    }

    pub fn contains(&self, asset: &AssetId) -> bool {
        self.assets.contains_key(asset)
    }

    pub fn all(&self) -> impl Iterator<Item = &AssetDefinition> {
        self.assets.values()
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}
