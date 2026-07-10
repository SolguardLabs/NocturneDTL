use crate::errors::{NocturneError, Result};
use crate::ids::{CommitmentId, NullifierId, WindowId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NullifierRecord {
    pub id: NullifierId,
    pub commitment: CommitmentId,
    pub window: WindowId,
    pub consumed_at: u64,
    pub memo: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NullifierSet {
    spent: BTreeMap<NullifierId, NullifierRecord>,
}

impl NullifierSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn consume(&mut self, record: NullifierRecord) -> Result<()> {
        if self.spent.contains_key(&record.id) {
            return Err(NocturneError::NullifierAlreadySpent(record.id));
        }
        self.spent.insert(record.id.clone(), record);
        Ok(())
    }

    pub fn contains(&self, id: &NullifierId) -> bool {
        self.spent.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.spent.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spent.is_empty()
    }

    pub fn records(&self) -> impl Iterator<Item = &NullifierRecord> {
        self.spent.values()
    }
}
