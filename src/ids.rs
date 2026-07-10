use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

macro_rules! string_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                let mut raw = value.into();
                if raw.trim().is_empty() {
                    raw = format!("{}default", $prefix);
                }
                Self(raw)
            }

            pub fn tagged(value: impl AsRef<str>) -> Self {
                let raw = value.as_ref();
                if raw.starts_with($prefix) {
                    Self::new(raw)
                } else {
                    Self::new(format!("{}{}", $prefix, raw))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }
    };
}

string_id!(AccountId, "acct_");
string_id!(UserId, "usr_");
string_id!(AssetId, "asset_");
string_id!(PositionId, "pos_");
string_id!(CommitmentId, "cm_");
string_id!(NullifierId, "nf_");
string_id!(ReceiptId, "rcpt_");
string_id!(BatchId, "batch_");

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WindowId(u64);

impl WindowId {
    pub fn new(epoch: u64) -> Self {
        Self(epoch)
    }

    pub fn epoch(self) -> u64 {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl Display for WindowId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "w{}", self.0)
    }
}

impl From<u64> for WindowId {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}
