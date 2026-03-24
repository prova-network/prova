//! Core types for the Prova chain layer.

use std::fmt;

/// A 32-byte hash (SHA-256).
pub type Hash = [u8; 32];

/// Epoch number (block height).
pub type Epoch = u64;

/// Address — simplified as a 20-byte identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Address(pub [u8; 20]);

impl serde::Serialize for Address {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let hex: String = self.0.iter().map(|b| format!("{b:02x}")).collect();
        serializer.serialize_str(&format!("0x{hex}"))
    }
}

impl<'de> serde::Deserialize<'de> for Address {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);
        if s.len() != 40 {
            return Err(serde::de::Error::custom("address must be 20 bytes hex"));
        }
        let mut bytes = [0u8; 20];
        for i in 0..20 {
            bytes[i] =
                u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(serde::de::Error::custom)?;
        }
        Ok(Address(bytes))
    }
}

use serde::Deserialize;

impl Address {
    pub fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// Create a test address from a single byte (for testing).
    pub fn test(id: u8) -> Self {
        let mut bytes = [0u8; 20];
        bytes[0] = id;
        Self(bytes)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for b in &self.0[..4] {
            write!(f, "{b:02x}")?;
        }
        write!(f, "…")
    }
}

/// Model identifier — hash of the model manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelId(pub Hash);

/// Unique inference commit identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommitId(pub u64);

impl fmt::Display for CommitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "commit-{}", self.0)
    }
}

/// Architecture group for determinism guarantees.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchGroup {
    /// e.g., "nvidia-sm89-int8" or "nvidia-sm90-int8"
    pub identifier: String,
}

impl ArchGroup {
    pub fn new(id: &str) -> Self {
        Self {
            identifier: id.to_string(),
        }
    }
}

/// Stake amount in smallest denomination.
pub type StakeAmount = u128;

/// Duration in epochs.
pub type EpochDuration = u64;
