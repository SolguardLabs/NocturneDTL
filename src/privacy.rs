use crate::amount::Amount;
use crate::hashing::{domain_hash, tagged_id, HashDigest, HashTranscript};
use crate::ids::{AccountId, AssetId, CommitmentId, NullifierId, PositionId, UserId, WindowId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitmentOpening {
    pub owner: UserId,
    pub account: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub position: PositionId,
    pub window: WindowId,
    pub blinding: String,
    pub nonce: u64,
}

impl CommitmentOpening {
    pub fn new(
        owner: UserId,
        account: AccountId,
        asset: AssetId,
        amount: Amount,
        position: PositionId,
        window: WindowId,
        blinding: impl Into<String>,
        nonce: u64,
    ) -> Self {
        Self {
            owner,
            account,
            asset,
            amount,
            position,
            window,
            blinding: blinding.into(),
            nonce,
        }
    }

    pub fn rotated(&self, window: WindowId, blinding: impl Into<String>, nonce: u64) -> Self {
        Self {
            owner: self.owner.clone(),
            account: self.account.clone(),
            asset: self.asset.clone(),
            amount: self.amount,
            position: self.position.clone(),
            window,
            blinding: blinding.into(),
            nonce,
        }
    }

    pub fn transcript(&self) -> HashTranscript {
        HashTranscript::new("nocturne.commitment.v1")
            .append(self.owner.as_str())
            .append(self.account.as_str())
            .append(self.asset.as_str())
            .append(self.amount.units())
            .append(self.position.as_str())
            .append(self.window.epoch())
            .append(&self.blinding)
            .append(self.nonce)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommitmentStatus {
    Pending,
    Active,
    Spent,
    Archived,
}

impl CommitmentStatus {
    pub fn is_spendable(&self) -> bool {
        matches!(self, Self::Pending | Self::Active)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commitment {
    pub id: CommitmentId,
    pub owner: UserId,
    pub account: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub position: PositionId,
    pub window: WindowId,
    pub digest: HashDigest,
    pub status: CommitmentStatus,
    pub created_at: u64,
}

impl Commitment {
    pub fn from_opening(opening: &CommitmentOpening, created_at: u64) -> Self {
        let digest = opening.transcript().digest();
        Self {
            id: CommitmentId::new(tagged_id("cm_", &digest)),
            owner: opening.owner.clone(),
            account: opening.account.clone(),
            asset: opening.asset.clone(),
            amount: opening.amount,
            position: opening.position.clone(),
            window: opening.window,
            digest,
            status: CommitmentStatus::Active,
            created_at,
        }
    }

    pub fn mark_spent(&mut self) {
        self.status = CommitmentStatus::Spent;
    }

    pub fn archive(&mut self) {
        self.status = CommitmentStatus::Archived;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub commitment: CommitmentId,
    pub opening: CommitmentOpening,
    pub digest: HashDigest,
}

impl Note {
    pub fn new(opening: CommitmentOpening, created_at: u64) -> (Self, Commitment) {
        let commitment = Commitment::from_opening(&opening, created_at);
        let note = Self {
            commitment: commitment.id.clone(),
            opening,
            digest: commitment.digest.clone(),
        };
        (note, commitment)
    }

    pub fn public_view(&self) -> NotePublicView {
        NotePublicView {
            commitment: self.commitment.clone(),
            digest: self.digest.as_hex().to_string(),
            owner: self.opening.owner.clone(),
            account: self.opening.account.clone(),
            asset: self.opening.asset.clone(),
            amount: self.opening.amount.units().min(u128::from(u64::MAX)) as u64,
            position: self.opening.position.clone(),
            window: self.opening.window,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotePublicView {
    pub commitment: CommitmentId,
    pub digest: String,
    pub owner: UserId,
    pub account: AccountId,
    pub asset: AssetId,
    pub amount: u64,
    pub position: PositionId,
    pub window: WindowId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpendWitness {
    pub note: Note,
    pub memo: String,
}

impl SpendWitness {
    pub fn new(note: Note, memo: impl Into<String>) -> Self {
        Self {
            note,
            memo: memo.into(),
        }
    }
}

pub fn derive_commitment(opening: &CommitmentOpening, created_at: u64) -> Commitment {
    Commitment::from_opening(opening, created_at)
}

pub fn derive_nullifier(note: &Note, protocol_domain: &str) -> NullifierId {
    let digest = domain_hash(
        "nocturne.nullifier.v1",
        &[
            protocol_domain.to_string(),
            note.commitment.as_str().to_string(),
            note.digest.as_hex().to_string(),
            note.opening.window.epoch().to_string(),
        ],
    );
    NullifierId::new(tagged_id("nf_", &digest))
}

pub fn verify_note_digest(note: &Note) -> bool {
    let commitment = derive_commitment(&note.opening, 0);
    commitment.id == note.commitment && commitment.digest == note.digest
}
