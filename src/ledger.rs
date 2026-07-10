use crate::account::AccountBook;
use crate::amount::Amount;
use crate::asset::{AssetDefinition, AssetRegistry};
use crate::audit::{AuditSnapshot, LedgerAuditor};
use crate::errors::{NocturneError, Result};
use crate::events::{EventKind, EventLog, LedgerEvent};
use crate::ids::{AccountId, AssetId, CommitmentId, PositionId, ReceiptId, UserId, WindowId};
use crate::nullifier::{NullifierRecord, NullifierSet};
use crate::policy::{AdmissionPolicy, WithdrawalPolicy};
use crate::position::PositionBook;
use crate::privacy::{derive_nullifier, Commitment, CommitmentOpening, Note};
use crate::proof::CommitmentVerifier;
use crate::receipt::{DepositReceipt, Receipt, SpendReceipt, WithdrawalReceipt};
use crate::reconcile::{Reconciler, ReconciliationReport};
use crate::vault::VaultBook;
use crate::window::WindowBook;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerConfig {
    pub protocol_domain: String,
    pub operator: String,
    pub admission: AdmissionPolicy,
    pub withdrawal: WithdrawalPolicy,
}

impl Default for LedgerConfig {
    fn default() -> Self {
        Self {
            protocol_domain: "nocturne-dtl-main".to_string(),
            operator: "operator.local".to_string(),
            admission: AdmissionPolicy::default(),
            withdrawal: WithdrawalPolicy::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerSnapshot {
    pub clock: u64,
    pub accounts: usize,
    pub assets: usize,
    pub windows: usize,
    pub positions: usize,
    pub commitments: usize,
    pub nullifiers: usize,
    pub events: usize,
    pub audit: AuditSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NocturneLedger {
    config: LedgerConfig,
    assets: AssetRegistry,
    accounts: AccountBook,
    positions: PositionBook,
    commitments: BTreeMap<CommitmentId, Commitment>,
    nullifiers: NullifierSet,
    vaults: VaultBook,
    windows: WindowBook,
    events: EventLog,
    receipts: Vec<Receipt>,
    clock: u64,
}

impl NocturneLedger {
    pub fn new(config: LedgerConfig) -> Result<Self> {
        let assets = AssetRegistry::with_default_assets()?;
        let mut vaults = VaultBook::new();
        for asset in assets.all() {
            vaults.ensure(asset.id.clone());
        }
        Ok(Self {
            config,
            assets,
            accounts: AccountBook::new(),
            positions: PositionBook::new(),
            commitments: BTreeMap::new(),
            nullifiers: NullifierSet::new(),
            vaults,
            windows: WindowBook::new(),
            events: EventLog::new(),
            receipts: Vec::new(),
            clock: 0,
        })
    }

    pub fn default_ledger() -> Result<Self> {
        let mut ledger = Self::new(LedgerConfig::default())?;
        ledger.open_window(WindowId::new(1), "genesis")?;
        Ok(ledger)
    }

    pub fn config(&self) -> &LedgerConfig {
        &self.config
    }

    pub fn assets(&self) -> &AssetRegistry {
        &self.assets
    }

    pub fn accounts(&self) -> &AccountBook {
        &self.accounts
    }

    pub fn positions(&self) -> &PositionBook {
        &self.positions
    }

    pub fn commitments(&self) -> &BTreeMap<CommitmentId, Commitment> {
        &self.commitments
    }

    pub fn nullifiers(&self) -> &NullifierSet {
        &self.nullifiers
    }

    pub fn vaults(&self) -> &VaultBook {
        &self.vaults
    }

    pub fn events(&self) -> &EventLog {
        &self.events
    }

    pub fn receipts(&self) -> &[Receipt] {
        &self.receipts
    }

    fn tick(&mut self) -> u64 {
        self.clock = self.clock.saturating_add(1);
        self.clock
    }

    fn receipt_id(&self, prefix: &str) -> ReceiptId {
        ReceiptId::new(format!("{}_{}", prefix, self.clock.saturating_add(1)))
    }

    fn push_event(&mut self, kind: EventKind, memo: impl Into<String>) -> LedgerEvent {
        let seq = self.tick();
        LedgerEvent::new(seq, kind, memo)
    }

    pub fn register_asset(&mut self, symbol: impl Into<String>, decimals: u8) -> Result<AssetId> {
        let definition = AssetDefinition::new(symbol, decimals)?;
        let id = definition.id.clone();
        self.assets.register(definition)?;
        self.vaults.ensure(id.clone());
        let event = self
            .push_event(EventKind::AssetRegistered, "asset registered")
            .with_asset_amount(id.clone(), Amount::ZERO);
        self.events.push(event);
        Ok(id)
    }

    pub fn open_window(&mut self, id: WindowId, anchor: impl Into<String>) -> Result<()> {
        self.windows.open(id, anchor);
        let event = self
            .push_event(EventKind::WindowOpened, "window opened")
            .with_window(id);
        self.events.push(event);
        Ok(())
    }

    pub fn deposit(
        &mut self,
        owner: UserId,
        account: AccountId,
        asset: AssetId,
        amount: Amount,
        position: PositionId,
        window: WindowId,
        blinding: impl Into<String>,
    ) -> Result<(Note, DepositReceipt)> {
        self.assets.get(&asset)?;
        self.windows.ensure_open(window);
        self.windows.ensure_accepting(window)?;
        self.config.admission.validate_amount(amount)?;

        let nonce = self.tick();
        let opening = CommitmentOpening::new(
            owner.clone(),
            account.clone(),
            asset.clone(),
            amount,
            position.clone(),
            window,
            blinding,
            nonce,
        );
        let (note, commitment) = Note::new(opening, nonce);
        if self.commitments.contains_key(&commitment.id) {
            return Err(NocturneError::CommitmentMismatch(commitment.id));
        }

        self.vaults.deposit(asset.clone(), amount)?;
        self.accounts.ensure(account.clone(), owner.clone());
        self.windows.get_mut(window)?.record_commitment(amount)?;
        self.positions
            .open_or_validate(position.clone(), owner, asset.clone(), amount, window)?
            .record_commitment(commitment.id.clone());
        self.commitments
            .insert(commitment.id.clone(), commitment.clone());

        let receipt = DepositReceipt {
            id: self.receipt_id("dep"),
            account: account.clone(),
            asset: asset.clone(),
            amount,
            position: position.clone(),
            commitment: commitment.id.clone(),
            window,
        };
        self.receipts.push(Receipt::Deposit(receipt.clone()));

        let event = self
            .push_event(EventKind::CommitmentCreated, "commitment accepted")
            .with_window(window)
            .with_account(account)
            .with_asset_amount(asset, amount)
            .with_commitment(commitment.id)
            .with_position(position)
            .with_receipt(receipt.id.clone());
        self.events.push(event);
        Ok((note, receipt))
    }

    pub fn roll_note(
        &mut self,
        note: &Note,
        next_window: WindowId,
        blinding: impl Into<String>,
    ) -> Result<Note> {
        let existing = self
            .commitments
            .get(&note.commitment)
            .ok_or_else(|| NocturneError::UnknownCommitment(note.commitment.clone()))?;
        if !existing.status.is_spendable() {
            return Err(NocturneError::CommitmentConsumed(note.commitment.clone()));
        }
        CommitmentVerifier::verify_note(note, existing)?;
        self.windows.ensure_open(next_window);
        self.windows.ensure_accepting(next_window)?;

        let nonce = self.tick();
        let opening = note.opening.rotated(next_window, blinding, nonce);
        let (rolled_note, commitment) = Note::new(opening, nonce);
        self.windows
            .get_mut(next_window)?
            .record_commitment(commitment.amount)?;
        self.positions
            .get_mut(&commitment.position)?
            .record_commitment(commitment.id.clone());
        self.commitments
            .insert(commitment.id.clone(), commitment.clone());

        let event = self
            .push_event(EventKind::CommitmentRotated, "commitment rotated")
            .with_window(next_window)
            .with_account(commitment.account.clone())
            .with_asset_amount(commitment.asset.clone(), commitment.amount)
            .with_commitment(commitment.id.clone())
            .with_position(commitment.position.clone());
        self.events.push(event);
        Ok(rolled_note)
    }

    pub fn spend(
        &mut self,
        note: &Note,
        recipient: AccountId,
        memo: impl Into<String>,
    ) -> Result<SpendReceipt> {
        let memo = memo.into();
        let (commitment_id, owner, asset, amount, position, window) = {
            let commitment = self
                .commitments
                .get(&note.commitment)
                .ok_or_else(|| NocturneError::UnknownCommitment(note.commitment.clone()))?;
            if !commitment.status.is_spendable() {
                return Err(NocturneError::CommitmentConsumed(commitment.id.clone()));
            }
            CommitmentVerifier::verify_note(note, commitment)?;
            (
                commitment.id.clone(),
                commitment.owner.clone(),
                commitment.asset.clone(),
                commitment.amount,
                commitment.position.clone(),
                commitment.window,
            )
        };

        let nullifier = derive_nullifier(note, &self.config.protocol_domain);
        self.nullifiers.consume(NullifierRecord {
            id: nullifier.clone(),
            commitment: commitment_id.clone(),
            window,
            consumed_at: self.clock.saturating_add(1),
            memo,
        })?;

        if let Some(commitment) = self.commitments.get_mut(&commitment_id) {
            commitment.mark_spent();
        }
        self.positions.get_mut(&position)?.record_consumption();
        self.windows.get_mut(window)?.record_spend();
        self.accounts
            .credit(recipient.clone(), owner, asset.clone(), amount)?;

        let receipt = SpendReceipt {
            id: self.receipt_id("spend"),
            account: recipient.clone(),
            asset: asset.clone(),
            amount,
            position: position.clone(),
            commitment: commitment_id.clone(),
            nullifier: nullifier.clone(),
            window,
        };
        self.receipts.push(Receipt::Spend(receipt.clone()));

        let event = self
            .push_event(EventKind::CommitmentSpent, "commitment spent")
            .with_window(window)
            .with_account(recipient)
            .with_asset_amount(asset, amount)
            .with_commitment(commitment_id)
            .with_nullifier(nullifier)
            .with_position(position)
            .with_receipt(receipt.id.clone());
        self.events.push(event);
        Ok(receipt)
    }

    pub fn withdraw(
        &mut self,
        account: AccountId,
        asset: AssetId,
        amount: Amount,
        destination: impl Into<String>,
        window: WindowId,
    ) -> Result<WithdrawalReceipt> {
        self.assets.get(&asset)?;
        self.config.withdrawal.validate(amount)?;
        self.accounts.debit(&account, &asset, amount)?;
        self.vaults.withdraw(&asset, amount)?;
        self.accounts
            .record_withdrawal(&account, asset.clone(), amount)?;
        self.windows.ensure_open(window);
        self.windows.get_mut(window)?.record_withdrawal(amount)?;

        let receipt = WithdrawalReceipt {
            id: self.receipt_id("wd"),
            account: account.clone(),
            asset: asset.clone(),
            amount,
            destination: destination.into(),
            window,
        };
        self.receipts.push(Receipt::Withdrawal(receipt.clone()));

        let event = self
            .push_event(EventKind::WithdrawalExecuted, "withdrawal executed")
            .with_window(window)
            .with_account(account)
            .with_asset_amount(asset, amount)
            .with_receipt(receipt.id.clone());
        self.events.push(event);
        Ok(receipt)
    }

    pub fn reconcile(&mut self) -> ReconciliationReport {
        let sequence = self.clock.saturating_add(1);
        let report = Reconciler::build(self, sequence);
        let event = self.push_event(EventKind::ReconciliationBuilt, "reconciliation built");
        self.events.push(event);
        report
    }

    pub fn snapshot(&self) -> LedgerSnapshot {
        LedgerSnapshot {
            clock: self.clock,
            accounts: self.accounts.all().count(),
            assets: self.assets.len(),
            windows: self.windows.all().count(),
            positions: self.positions.len(),
            commitments: self.commitments.len(),
            nullifiers: self.nullifiers.len(),
            events: self.events.len(),
            audit: LedgerAuditor::snapshot(self),
        }
    }
}
