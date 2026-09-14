//! Domain-separated bridge identities.

use core::fmt;
use core::str::FromStr;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest as _, Sha256};

/// Exact lowercase SHA-256 identity used by bridge-owned records.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BridgeDigest([u8; 32]);

/// A digest string was not exactly 64 lowercase hexadecimal characters.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("digest must be 64 lowercase hexadecimal characters")]
pub struct BridgeDigestParseError;

impl BridgeDigest {
    /// Parses exactly 64 lowercase hexadecimal characters.
    pub fn parse(value: &str) -> Result<Self, BridgeDigestParseError> {
        if value.len() != 64
            || value
                .bytes()
                .any(|byte| !matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(BridgeDigestParseError);
        }
        let mut bytes = [0_u8; 32];
        let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
        if !remainder.is_empty() {
            return Err(BridgeDigestParseError);
        }
        for (index, pair) in pairs.iter().enumerate() {
            bytes[index] = (hex(pair[0]) << 4) | hex(pair[1]);
        }
        Ok(Self(bytes))
    }

    /// Hashes raw bytes without adding a semantic domain.
    #[must_use]
    pub fn raw(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    /// Hashes `quire-contract-ir`, NUL, `profile`, NUL, and `bytes`.
    #[must_use]
    pub fn domain(profile: &str, bytes: &[u8]) -> Self {
        let mut hash = Sha256::new();
        hash.update(b"quire-contract-ir");
        hash.update([0]);
        hash.update(profile.as_bytes());
        hash.update([0]);
        hash.update(bytes);
        Self(hash.finalize().into())
    }

    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl FromStr for BridgeDigest {
    type Err = BridgeDigestParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl fmt::Display for BridgeDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in self.0 {
            formatter.write_str(
                core::str::from_utf8(&[HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 0x0f)]])
                    .map_err(|_| fmt::Error)?,
            )?;
        }
        Ok(())
    }
}

impl Serialize for BridgeDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for BridgeDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(&String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

const fn hex(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}
