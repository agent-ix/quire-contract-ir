//! Owner-contract selections used by bridge decisions.

use serde::{Deserialize, Serialize};

/// Exact owner contract selected before content admission.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractSelection {
    contract: String,
    package_version: String,
    repository: String,
}

impl ContractSelection {
    /// Constructs an untrusted selection for explicit admission by a bridge.
    pub fn new(
        contract: impl Into<String>,
        package_version: impl Into<String>,
        repository: impl Into<String>,
    ) -> Self {
        Self {
            contract: contract.into(),
            package_version: package_version.into(),
            repository: repository.into(),
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

    /// Returns the owning repository identity.
    #[must_use]
    pub fn repository(&self) -> &str {
        &self.repository
    }

    /// Checks the closed selection shape without interpreting content.
    #[must_use]
    pub fn structurally_valid(&self) -> bool {
        valid_text(&self.contract)
            && valid_text(&self.package_version)
            && valid_text(&self.repository)
    }
}

fn valid_text(value: &str) -> bool {
    !value.is_empty() && value.len() <= 1_024
}
