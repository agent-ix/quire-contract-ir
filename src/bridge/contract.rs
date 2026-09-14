//! Immutable owner-contract selections used by bridge decisions.

use serde::{Deserialize, Serialize};

use super::BridgeDigest;

/// Exact immutable owner contract selected before content admission.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractSelection {
    contract: String,
    package_version: String,
    repository: String,
    revision: String,
    schema_digest: BridgeDigest,
}

impl ContractSelection {
    /// Constructs an untrusted selection for explicit admission by a bridge.
    pub fn new(
        contract: impl Into<String>,
        package_version: impl Into<String>,
        repository: impl Into<String>,
        revision: impl Into<String>,
        schema_digest: BridgeDigest,
    ) -> Self {
        Self {
            contract: contract.into(),
            package_version: package_version.into(),
            repository: repository.into(),
            revision: revision.into(),
            schema_digest,
        }
    }

    /// Returns the exact owner contract label.
    #[must_use]
    pub fn contract(&self) -> &str {
        &self.contract
    }

    /// Returns the selected Cargo package version.
    #[must_use]
    pub fn package_version(&self) -> &str {
        &self.package_version
    }

    /// Returns the immutable repository identity.
    #[must_use]
    pub fn repository(&self) -> &str {
        &self.repository
    }

    /// Returns the exact 40-character commit OID.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Returns the exact schema-byte digest.
    #[must_use]
    pub const fn schema_digest(&self) -> BridgeDigest {
        self.schema_digest
    }

    /// Checks the closed immutable selection shape without interpreting content.
    #[must_use]
    pub fn structurally_valid(&self) -> bool {
        valid_text(&self.contract)
            && valid_text(&self.package_version)
            && valid_text(&self.repository)
            && self.revision.len() == 40
            && self
                .revision
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    }
}

fn valid_text(value: &str) -> bool {
    !value.is_empty() && value.len() <= 1_024
}
