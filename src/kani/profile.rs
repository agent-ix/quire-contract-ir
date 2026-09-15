//! Versioned profile and complete support/refusal/inconclusive matrix.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{KaniOutcome, KaniOutcomeKind, PROFILE};

/// One construct's selected bounded-Kani disposition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDisposition {
    /// A named shared or semantic-family module owns exact lowering.
    Supported { module: String },
    /// The selected profile refuses the construct at the source boundary.
    Refused { code: String },
    /// A bounded cause prevents a qualified result.
    Inconclusive { code: String },
}

/// One unique construct entry in a capability matrix.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CapabilityEntry {
    /// Stable native construct identifier.
    pub construct: String,
    /// Exactly one disposition for the construct.
    pub disposition: CapabilityDisposition,
}

/// Immutable selection of the Kani executable and profile ABI.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProfileSelection {
    /// Profile family identifier.
    pub profile: String,
    /// Immutable profile revision.
    pub revision: String,
    /// Kani executable SHA-256 identity.
    pub executable_digest: String,
    /// Digest of selected Kani options.
    pub options_digest: String,
    /// Shared ABI revision.
    pub abi_revision: String,
}

/// Complete selected profile and its exact construct matrix.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KaniProfile {
    /// Exact profile/tool/ABI selection.
    pub selection: ProfileSelection,
    /// Complete unique disposition matrix.
    pub capabilities: Vec<CapabilityEntry>,
}

/// Profile-selection or matrix refusal.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ProfileError {
    /// The profile family is not the selected bounded Kani profile.
    #[error("unsupported bounded-Kani profile `{0}`")]
    UnsupportedProfile(String),
    /// Required immutable selection data is absent.
    #[error("missing profile selection field `{0}`")]
    MissingSelectionField(&'static str),
    /// A construct entry is empty or duplicated.
    #[error("invalid capability matrix: {0}")]
    InvalidMatrix(String),
    /// An encountered construct lacks an entry.
    #[error("capability matrix omits construct `{0}`")]
    MissingConstruct(String),
}

impl KaniProfile {
    /// Creates a profile only when its immutable selection and matrix are coherent.
    pub fn new(
        selection: ProfileSelection,
        capabilities: Vec<CapabilityEntry>,
    ) -> Result<Self, ProfileError> {
        if selection.profile != PROFILE {
            return Err(ProfileError::UnsupportedProfile(selection.profile));
        }
        for (name, value) in [
            ("revision", selection.revision.as_str()),
            ("executable_digest", selection.executable_digest.as_str()),
            ("options_digest", selection.options_digest.as_str()),
            ("abi_revision", selection.abi_revision.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(ProfileError::MissingSelectionField(name));
            }
        }
        let mut seen = BTreeSet::new();
        for entry in &capabilities {
            if entry.construct.trim().is_empty() || !seen.insert(entry.construct.as_str()) {
                return Err(ProfileError::InvalidMatrix(entry.construct.clone()));
            }
        }
        Ok(Self {
            selection,
            capabilities,
        })
    }

    /// Returns a complete pre-lowering disposition for each encountered construct.
    pub fn classify(
        &self,
        constructs: &[String],
        source_id: &str,
    ) -> Result<Vec<CapabilityEntry>, KaniOutcome> {
        let mut encountered = BTreeSet::new();
        let mut selected = Vec::with_capacity(constructs.len());
        for construct in constructs {
            if construct.trim().is_empty() || !encountered.insert(construct.as_str()) {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_capability_request_invalid",
                    source_id,
                    self.selection.revision.clone(),
                ));
            }
            let Some(entry) = self
                .capabilities
                .iter()
                .find(|entry| entry.construct == *construct)
            else {
                return Err(KaniOutcome::non_success(
                    KaniOutcomeKind::Refused,
                    "kani_capability_missing",
                    source_id,
                    construct.clone(),
                ));
            };
            selected.push(entry.clone());
        }
        Ok(selected)
    }
}
