use crate::amount::{Amount, BasisPoints};
use crate::errors::Result;
use crate::ids::AssetId;
use crate::policy::WithdrawalPolicy;
use crate::vault::Vault;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiquidityProfile {
    pub asset: AssetId,
    pub reserve: Amount,
    pub total_deposits: Amount,
    pub total_withdrawals: Amount,
    pub haircut_bps: BasisPoints,
    pub stressed_reserve: Amount,
    pub stress_loss: Amount,
    pub request_limit: Amount,
    pub observed_capacity: Amount,
    pub nominal_conservation: bool,
}

impl LiquidityProfile {
    pub fn evaluate(
        vault: &Vault,
        policy: &WithdrawalPolicy,
        haircut_bps: BasisPoints,
    ) -> Result<Self> {
        let retained_bps = BasisPoints::new(10_000 - haircut_bps.value())?;
        let stressed_reserve = vault.reserves.mul_bps(retained_bps)?;
        let stress_loss = vault.reserves.checked_sub(stressed_reserve)?;
        Ok(Self {
            asset: vault.asset.clone(),
            reserve: vault.reserves,
            total_deposits: vault.total_deposits,
            total_withdrawals: vault.total_withdrawals,
            haircut_bps,
            stressed_reserve,
            stress_loss,
            request_limit: policy.daily_limit,
            observed_capacity: stressed_reserve.min(policy.daily_limit),
            nominal_conservation: vault.conservation_holds(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_tracks_reserve_after_settlement() {
        let mut vault = Vault::new(AssetId::new("DTL"));
        vault.deposit(Amount::from_units(350)).unwrap();
        vault.withdraw(Amount::from_units(200)).unwrap();
        let policy = WithdrawalPolicy::default();
        let profile =
            LiquidityProfile::evaluate(&vault, &policy, BasisPoints::new(2_000).unwrap()).unwrap();
        assert_eq!(profile.reserve.units(), 150);
        assert_eq!(profile.stressed_reserve.units(), 120);
        assert_eq!(profile.stress_loss.units(), 30);
        assert_eq!(profile.observed_capacity.units(), 120);
        assert!(profile.nominal_conservation);
    }
}
