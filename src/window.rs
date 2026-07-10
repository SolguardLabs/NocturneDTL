use crate::amount::Amount;
use crate::errors::{NocturneError, Result};
use crate::ids::WindowId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowPolicy {
    pub max_commitments: usize,
    pub rollover_grace: u64,
    pub minimum_age: u64,
    pub anchor: String,
}

impl Default for WindowPolicy {
    fn default() -> Self {
        Self {
            max_commitments: 10_000,
            rollover_grace: 2,
            minimum_age: 0,
            anchor: "genesis".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowMetrics {
    pub commitments_created: usize,
    pub commitments_spent: usize,
    pub total_value: Amount,
    pub withdrawn_value: Amount,
}

impl Default for WindowMetrics {
    fn default() -> Self {
        Self {
            commitments_created: 0,
            commitments_spent: 0,
            total_value: Amount::ZERO,
            withdrawn_value: Amount::ZERO,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum WindowStatus {
    Accepting,
    Settling,
    Closed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivacyWindow {
    pub id: WindowId,
    pub status: WindowStatus,
    pub policy: WindowPolicy,
    pub metrics: WindowMetrics,
}

impl PrivacyWindow {
    pub fn new(id: WindowId, anchor: impl Into<String>) -> Self {
        Self {
            id,
            status: WindowStatus::Accepting,
            policy: WindowPolicy {
                anchor: anchor.into(),
                ..WindowPolicy::default()
            },
            metrics: WindowMetrics::default(),
        }
    }

    pub fn ensure_accepting(&self) -> Result<()> {
        if self.status == WindowStatus::Accepting {
            Ok(())
        } else {
            Err(NocturneError::WindowClosed(self.id))
        }
    }

    pub fn record_commitment(&mut self, amount: Amount) -> Result<()> {
        self.ensure_accepting()?;
        self.metrics.commitments_created += 1;
        self.metrics.total_value = self.metrics.total_value.checked_add(amount)?;
        Ok(())
    }

    pub fn record_spend(&mut self) {
        self.metrics.commitments_spent += 1;
    }

    pub fn record_withdrawal(&mut self, amount: Amount) -> Result<()> {
        self.metrics.withdrawn_value = self.metrics.withdrawn_value.checked_add(amount)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WindowBook {
    windows: BTreeMap<WindowId, PrivacyWindow>,
}

impl WindowBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, id: WindowId, anchor: impl Into<String>) -> &PrivacyWindow {
        self.windows
            .entry(id)
            .or_insert_with(|| PrivacyWindow::new(id, anchor))
    }

    pub fn ensure_open(&mut self, id: WindowId) {
        self.windows
            .entry(id)
            .or_insert_with(|| PrivacyWindow::new(id, format!("anchor-{}", id.epoch())));
    }

    pub fn get(&self, id: WindowId) -> Result<&PrivacyWindow> {
        self.windows
            .get(&id)
            .ok_or(NocturneError::UnknownWindow(id))
    }

    pub fn get_mut(&mut self, id: WindowId) -> Result<&mut PrivacyWindow> {
        self.windows
            .get_mut(&id)
            .ok_or(NocturneError::UnknownWindow(id))
    }

    pub fn ensure_accepting(&self, id: WindowId) -> Result<()> {
        self.get(id)?.ensure_accepting()
    }

    pub fn settle(&mut self, id: WindowId) -> Result<()> {
        self.get_mut(id)?.status = WindowStatus::Settling;
        Ok(())
    }

    pub fn close(&mut self, id: WindowId) -> Result<()> {
        self.get_mut(id)?.status = WindowStatus::Closed;
        Ok(())
    }

    pub fn all(&self) -> impl Iterator<Item = &PrivacyWindow> {
        self.windows.values()
    }
}
