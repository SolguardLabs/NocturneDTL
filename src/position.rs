use crate::amount::Amount;
use crate::errors::{NocturneError, Result};
use crate::ids::{AssetId, CommitmentId, PositionId, UserId, WindowId};
use serde::{Deserialize, Serialize};
use std::collections::{btree_map::Entry, BTreeMap};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PositionLifecycle {
    Open,
    Settling,
    Closed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomicPosition {
    pub id: PositionId,
    pub owner: UserId,
    pub asset: AssetId,
    pub principal: Amount,
    pub opened_window: WindowId,
    pub lifecycle: PositionLifecycle,
    pub commitments: Vec<CommitmentId>,
    pub notes_issued: usize,
    pub notes_consumed: usize,
}

impl EconomicPosition {
    pub fn new(
        id: PositionId,
        owner: UserId,
        asset: AssetId,
        principal: Amount,
        opened_window: WindowId,
    ) -> Self {
        Self {
            id,
            owner,
            asset,
            principal,
            opened_window,
            lifecycle: PositionLifecycle::Open,
            commitments: Vec::new(),
            notes_issued: 0,
            notes_consumed: 0,
        }
    }

    pub fn ensure_matches(&self, owner: &UserId, asset: &AssetId, principal: Amount) -> Result<()> {
        if &self.owner == owner && &self.asset == asset && self.principal == principal {
            Ok(())
        } else {
            Err(NocturneError::PositionMismatch(self.id.clone()))
        }
    }

    pub fn record_commitment(&mut self, commitment: CommitmentId) {
        self.commitments.push(commitment);
        self.notes_issued += 1;
    }

    pub fn record_consumption(&mut self) {
        self.notes_consumed += 1;
        self.lifecycle = PositionLifecycle::Settling;
    }

    pub fn close(&mut self) {
        self.lifecycle = PositionLifecycle::Closed;
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PositionBook {
    positions: BTreeMap<PositionId, EconomicPosition>,
}

impl PositionBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_or_validate(
        &mut self,
        id: PositionId,
        owner: UserId,
        asset: AssetId,
        principal: Amount,
        window: WindowId,
    ) -> Result<&mut EconomicPosition> {
        match self.positions.entry(id.clone()) {
            Entry::Occupied(entry) => {
                entry.get().ensure_matches(&owner, &asset, principal)?;
                Ok(entry.into_mut())
            }
            Entry::Vacant(entry) => {
                Ok(entry.insert(EconomicPosition::new(id, owner, asset, principal, window)))
            }
        }
    }

    pub fn get(&self, id: &PositionId) -> Result<&EconomicPosition> {
        self.positions
            .get(id)
            .ok_or_else(|| NocturneError::UnknownPosition(id.clone()))
    }

    pub fn get_mut(&mut self, id: &PositionId) -> Result<&mut EconomicPosition> {
        self.positions
            .get_mut(id)
            .ok_or_else(|| NocturneError::UnknownPosition(id.clone()))
    }

    pub fn all(&self) -> impl Iterator<Item = &EconomicPosition> {
        self.positions.values()
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}
