use crate::errors::{NocturneError, Result};
use crate::privacy::{derive_commitment, Commitment, Note};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitmentProof {
    pub commitment_digest: String,
    pub opening_digest: String,
    pub verifier_label: String,
}

impl CommitmentProof {
    pub fn for_note(note: &Note) -> Self {
        let recomputed = derive_commitment(&note.opening, 0);
        Self {
            commitment_digest: note.digest.as_hex().to_string(),
            opening_digest: recomputed.digest.as_hex().to_string(),
            verifier_label: "nocturne-proof-sim".to_string(),
        }
    }

    pub fn verify(&self) -> bool {
        self.commitment_digest == self.opening_digest
    }
}

#[derive(Clone, Debug, Default)]
pub struct CommitmentVerifier;

impl CommitmentVerifier {
    pub fn verify_note(note: &Note, stored: &Commitment) -> Result<CommitmentProof> {
        let recomputed = derive_commitment(&note.opening, stored.created_at);
        if recomputed.id != stored.id || recomputed.digest != stored.digest {
            return Err(NocturneError::CommitmentMismatch(stored.id.clone()));
        }
        Ok(CommitmentProof {
            commitment_digest: stored.digest.as_hex().to_string(),
            opening_digest: recomputed.digest.as_hex().to_string(),
            verifier_label: "nocturne-proof-sim".to_string(),
        })
    }
}
