use crate::amount::Amount;
use crate::codec;
use crate::errors::{NocturneError, Result};
use crate::ids::{AccountId, AssetId, PositionId, UserId, WindowId};
use crate::ledger::{LedgerConfig, LedgerSnapshot, NocturneLedger};
use crate::privacy::{Note, NotePublicView};
use crate::receipt::Receipt;
use crate::reconcile::ReconciliationReport;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioInput {
    #[serde(default)]
    pub config: Option<LedgerConfig>,
    pub operations: Vec<ScenarioOperation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ScenarioOperation {
    RegisterAsset {
        symbol: String,
        decimals: u8,
    },
    OpenWindow {
        window: u64,
        anchor: String,
    },
    Deposit {
        note: String,
        owner: String,
        account: String,
        asset: String,
        amount: u64,
        position: String,
        window: u64,
        blinding: String,
    },
    Roll {
        source: String,
        note: String,
        window: u64,
        blinding: String,
    },
    Spend {
        note: String,
        recipient: String,
        #[serde(default)]
        memo: String,
    },
    Withdraw {
        account: String,
        asset: String,
        amount: u64,
        destination: String,
        #[serde(default = "default_window")]
        window: u64,
    },
    Reconcile,
    Snapshot,
}

fn default_window() -> u64 {
    1
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioOutput {
    pub notes: BTreeMap<String, NotePublicView>,
    pub receipts: Vec<Receipt>,
    pub reconciliations: Vec<ReconciliationReport>,
    pub snapshot: LedgerSnapshot,
}

#[derive(Default)]
struct ScenarioNotes {
    private: BTreeMap<String, Note>,
    public: BTreeMap<String, NotePublicView>,
}

impl ScenarioNotes {
    fn insert(&mut self, label: String, note: Note) {
        self.public.insert(label.clone(), note.public_view());
        self.private.insert(label, note);
    }

    fn get(&self, label: &str) -> Result<&Note> {
        self.private
            .get(label)
            .ok_or_else(|| NocturneError::ScenarioNoteMissing(label.to_string()))
    }
}

pub fn run_scenario(input: ScenarioInput) -> Result<ScenarioOutput> {
    let config = input.config.unwrap_or_default();
    let mut ledger = NocturneLedger::new(config)?;
    ledger.open_window(WindowId::new(1), "genesis")?;

    let mut notes = ScenarioNotes::default();
    let mut reconciliations = Vec::new();

    for operation in input.operations {
        match operation {
            ScenarioOperation::RegisterAsset { symbol, decimals } => {
                ledger.register_asset(symbol, decimals)?;
            }
            ScenarioOperation::OpenWindow { window, anchor } => {
                ledger.open_window(WindowId::new(window), anchor)?;
            }
            ScenarioOperation::Deposit {
                note,
                owner,
                account,
                asset,
                amount,
                position,
                window,
                blinding,
            } => {
                let (created, _) = ledger.deposit(
                    UserId::new(owner),
                    AccountId::new(account),
                    AssetId::new(asset),
                    Amount::new(u128::from(amount))?,
                    PositionId::new(position),
                    WindowId::new(window),
                    blinding,
                )?;
                notes.insert(note, created);
            }
            ScenarioOperation::Roll {
                source,
                note,
                window,
                blinding,
            } => {
                let created =
                    ledger.roll_note(notes.get(&source)?, WindowId::new(window), blinding)?;
                notes.insert(note, created);
            }
            ScenarioOperation::Spend {
                note,
                recipient,
                memo,
            } => {
                ledger.spend(notes.get(&note)?, AccountId::new(recipient), memo)?;
            }
            ScenarioOperation::Withdraw {
                account,
                asset,
                amount,
                destination,
                window,
            } => {
                ledger.withdraw(
                    AccountId::new(account),
                    AssetId::new(asset),
                    Amount::new(u128::from(amount))?,
                    destination,
                    WindowId::new(window),
                )?;
            }
            ScenarioOperation::Reconcile => {
                reconciliations.push(ledger.reconcile());
            }
            ScenarioOperation::Snapshot => {}
        }
    }

    Ok(ScenarioOutput {
        notes: notes.public,
        receipts: ledger.receipts().to_vec(),
        reconciliations,
        snapshot: ledger.snapshot(),
    })
}

pub fn run_scenario_json(input: &str) -> Result<String> {
    let scenario: ScenarioInput = codec::from_json(input)?;
    let output = run_scenario(scenario)?;
    codec::to_json(&output)
}

pub fn demo_scenario() -> ScenarioInput {
    ScenarioInput {
        config: None,
        operations: vec![
            ScenarioOperation::OpenWindow {
                window: 2,
                anchor: "night-cycle".to_string(),
            },
            ScenarioOperation::Deposit {
                note: "alice_0".to_string(),
                owner: "alice".to_string(),
                account: "alice-main".to_string(),
                asset: "DTL".to_string(),
                amount: 100,
                position: "alice-pos-1".to_string(),
                window: 1,
                blinding: "b0".to_string(),
            },
            ScenarioOperation::Roll {
                source: "alice_0".to_string(),
                note: "alice_1".to_string(),
                window: 2,
                blinding: "b1".to_string(),
            },
            ScenarioOperation::Spend {
                note: "alice_1".to_string(),
                recipient: "alice-main".to_string(),
                memo: "demo spend".to_string(),
            },
            ScenarioOperation::Withdraw {
                account: "alice-main".to_string(),
                asset: "DTL".to_string(),
                amount: 100,
                destination: "dtl:alice".to_string(),
                window: 2,
            },
            ScenarioOperation::Reconcile,
        ],
    }
}
