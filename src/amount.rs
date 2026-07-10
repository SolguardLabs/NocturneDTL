use crate::errors::{NocturneError, Result};
use crate::ids::AssetId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount {
    units: u128,
}

impl Amount {
    pub const ZERO: Self = Self { units: 0 };

    pub fn new(units: u128) -> Result<Self> {
        if units == 0 {
            return Err(NocturneError::InvalidAmount);
        }
        Ok(Self { units })
    }

    pub fn from_units(units: u128) -> Self {
        Self { units }
    }

    pub fn units(self) -> u128 {
        self.units
    }

    pub fn is_zero(self) -> bool {
        self.units == 0
    }

    pub fn checked_add(self, other: Self) -> Result<Self> {
        self.units
            .checked_add(other.units)
            .map(Self::from_units)
            .ok_or(NocturneError::AmountOverflow)
    }

    pub fn checked_sub(self, other: Self) -> Result<Self> {
        self.units
            .checked_sub(other.units)
            .map(Self::from_units)
            .ok_or(NocturneError::InvalidAmount)
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        Self::from_units(self.units.saturating_sub(other.units))
    }

    pub fn min(self, other: Self) -> Self {
        if self <= other {
            self
        } else {
            other
        }
    }

    pub fn max(self, other: Self) -> Self {
        if self >= other {
            self
        } else {
            other
        }
    }

    pub fn mul_bps(self, bps: BasisPoints) -> Result<Self> {
        let scaled = self
            .units
            .checked_mul(u128::from(bps.value()))
            .ok_or(NocturneError::AmountOverflow)?;
        Ok(Self::from_units(scaled / 10_000))
    }

    pub fn ratio_floor(self, numerator: u128, denominator: u128) -> Result<Self> {
        if denominator == 0 {
            return Err(NocturneError::InvalidAmount);
        }
        let scaled = self
            .units
            .checked_mul(numerator)
            .ok_or(NocturneError::AmountOverflow)?;
        Ok(Self::from_units(scaled / denominator))
    }
}

impl Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.units.min(u128::from(u64::MAX)) as u64)
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let units = u64::deserialize(deserializer)?;
        Ok(Self::from_units(u128::from(units)))
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.units)
    }
}

impl Add for Amount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_units(self.units.saturating_add(rhs.units))
    }
}

impl Sub for Amount {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_units(self.units.saturating_sub(rhs.units))
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedAmount {
    units: i128,
}

impl SignedAmount {
    pub fn from_units(units: i128) -> Self {
        Self { units }
    }

    pub fn units(self) -> i128 {
        self.units
    }

    pub fn sign(self) -> Ordering {
        self.units.cmp(&0)
    }

    pub fn abs_amount(self) -> Amount {
        Amount::from_units(self.units.unsigned_abs())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BasisPoints(u16);

impl BasisPoints {
    pub const ZERO: Self = Self(0);

    pub fn new(value: u16) -> Result<Self> {
        if value > 10_000 {
            return Err(NocturneError::InvalidAmount);
        }
        Ok(Self(value))
    }

    pub fn value(self) -> u16 {
        self.0
    }
}

impl Default for BasisPoints {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetAmount {
    pub asset: AssetId,
    pub amount: Amount,
}

impl AssetAmount {
    pub fn new(asset: AssetId, amount: Amount) -> Self {
        Self { asset, amount }
    }

    pub fn zero(asset: AssetId) -> Self {
        Self {
            asset,
            amount: Amount::ZERO,
        }
    }

    pub fn units(&self) -> u128 {
        self.amount.units()
    }
}
