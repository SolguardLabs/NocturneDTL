use crate::reconcile::ReconciliationReport;

pub fn reconciliation_summary(report: &ReconciliationReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("sequence={}", report.sequence));
    lines.push(format!("balanced={}", report.balanced));
    lines.push(format!("nullifiers={}", report.nullifiers));
    for asset in &report.assets {
        lines.push(format!(
            "asset={} deposits={} withdrawals={} reserves={} balances={}",
            asset.asset, asset.deposits, asset.withdrawals, asset.reserves, asset.account_balances
        ));
    }
    lines.join("\n")
}
