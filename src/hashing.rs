use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HashDigest(String);

impl HashDigest {
    pub fn new(hex_value: impl Into<String>) -> Self {
        Self(hex_value.into())
    }

    pub fn as_hex(&self) -> &str {
        &self.0
    }

    pub fn short(&self) -> String {
        self.0.chars().take(16).collect()
    }
}

impl Display for HashDigest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub fn domain_hash(domain: &str, fields: &[String]) -> HashDigest {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    for field in fields {
        hasher.update(field.as_bytes());
        hasher.update(&[0xff]);
    }
    HashDigest::new(hasher.finalize().to_hex().to_string())
}

pub fn tagged_id(prefix: &str, digest: &HashDigest) -> String {
    format!(
        "{}{}",
        prefix,
        digest.as_hex().chars().take(32).collect::<String>()
    )
}

#[derive(Clone, Debug, Default)]
pub struct HashTranscript {
    domain: String,
    fields: Vec<String>,
}

impl HashTranscript {
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            fields: Vec::new(),
        }
    }

    pub fn append(mut self, value: impl ToString) -> Self {
        self.fields.push(value.to_string());
        self
    }

    pub fn digest(&self) -> HashDigest {
        domain_hash(&self.domain, &self.fields)
    }

    pub fn tagged(&self, prefix: &str) -> String {
        tagged_id(prefix, &self.digest())
    }
}
