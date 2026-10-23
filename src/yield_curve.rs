use crate::amount::{Amount, BasisPoints};
use crate::errors::{NocturneError, Result};
use crate::ids::WindowId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const INDEX_SCALE: u128 = 1_000_000_000_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct YieldCurve {
    pub window_rate_bps: BasisPoints,
    pub scheduled_rates_bps: BTreeMap<u64, BasisPoints>,
    pub maximum_epoch: u64,
}

impl Default for YieldCurve {
    fn default() -> Self {
        Self {
            window_rate_bps: BasisPoints::ZERO,
            scheduled_rates_bps: BTreeMap::new(),
            maximum_epoch: 256,
        }
    }
}

impl YieldCurve {
    pub fn index_at(&self, window: WindowId) -> Result<u128> {
        let epoch = window.epoch();
        if epoch == 0 || epoch > self.maximum_epoch {
            return Err(NocturneError::InvalidWindowTransition);
        }
        let mut index = INDEX_SCALE;
        for current in 2..=epoch {
            let rate = self
                .scheduled_rates_bps
                .get(&current)
                .copied()
                .unwrap_or(self.window_rate_bps);
            let factor = 10_000_u128 + u128::from(rate.value());
            index = index
                .checked_mul(factor)
                .ok_or(NocturneError::AmountOverflow)?
                / 10_000;
        }
        Ok(index)
    }

    pub fn value_at(&self, amount: Amount, from: WindowId, to: WindowId) -> Result<Amount> {
        let from_index = self.index_at(from)?;
        let to_index = self.index_at(to)?;
        if to.epoch() <= from.epoch() || to_index < from_index {
            return Err(NocturneError::InvalidWindowTransition);
        }
        let numerator = amount
            .units()
            .checked_mul(to_index)
            .ok_or(NocturneError::AmountOverflow)?;
        Ok(Amount::from_units(numerator / from_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_compounds_by_window() {
        let curve = YieldCurve {
            window_rate_bps: BasisPoints::new(1_000).unwrap(),
            ..YieldCurve::default()
        };
        assert_eq!(curve.index_at(WindowId::new(1)).unwrap(), INDEX_SCALE);
        assert_eq!(curve.index_at(WindowId::new(2)).unwrap(), 1_100_000_000_000);
        assert_eq!(curve.index_at(WindowId::new(3)).unwrap(), 1_210_000_000_000);
        assert_eq!(
            curve
                .value_at(
                    Amount::from_units(1_000),
                    WindowId::new(1),
                    WindowId::new(3)
                )
                .unwrap()
                .units(),
            1_210
        );
    }

    #[test]
    fn schedule_applies_each_window_rate_to_the_prior_index() {
        let mut curve = YieldCurve::default();
        curve
            .scheduled_rates_bps
            .insert(2, BasisPoints::new(800).unwrap());
        curve
            .scheduled_rates_bps
            .insert(3, BasisPoints::new(1_200).unwrap());
        assert_eq!(curve.index_at(WindowId::new(3)).unwrap(), 1_209_600_000_000);
    }
}
