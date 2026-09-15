//! Content identities and retained generator provenance.

use sha2::{Digest, Sha256};
use thiserror::Error;

/// A stable SHA-256 identity for one generated bounded-Kani artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactIdentity(String);

impl ArtifactIdentity {
    /// Computes a domain-separated identity over the exact retained bytes.
    #[must_use]
    pub fn of(domain: &str, bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(domain.as_bytes());
        hasher.update([0]);
        hasher.update(bytes);
        Self(format!("sha256:{:x}", hasher.finalize()))
    }

    /// Canonical textual identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Input provenance every generator must retain before emitting an artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratorProvenance {
    /// Checked native clause identity.
    pub clause_id: String,
    /// Source model/snapshot/population input identity.
    pub input_id: String,
    /// Selected Kani profile revision.
    pub profile_revision: String,
    /// Exact selected Kani executable digest.
    pub executable_digest: String,
    /// Exact selected options digest.
    pub options_digest: String,
    /// Ordered declared assumptions after finite-input validation.
    pub assumptions: Vec<String>,
    /// Ordered proof dependency identities.
    pub proof_dependencies: Vec<String>,
}

/// A provenance record cannot be created with a missing selection field.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ProvenanceError {
    /// One exact identity or digest field is absent.
    #[error("missing generator provenance field `{0}`")]
    MissingField(&'static str),
    /// An assumption or dependency has no identity.
    #[error("empty generator provenance entry in `{0}`")]
    EmptyEntry(&'static str),
}

impl GeneratorProvenance {
    /// Validates the complete provenance required for a generated artifact.
    pub fn validate(&self) -> Result<(), ProvenanceError> {
        for (name, value) in [
            ("clause_id", self.clause_id.as_str()),
            ("input_id", self.input_id.as_str()),
            ("profile_revision", self.profile_revision.as_str()),
            ("executable_digest", self.executable_digest.as_str()),
            ("options_digest", self.options_digest.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(ProvenanceError::MissingField(name));
            }
        }
        if self.assumptions.iter().any(|entry| entry.trim().is_empty()) {
            return Err(ProvenanceError::EmptyEntry("assumptions"));
        }
        if self
            .proof_dependencies
            .iter()
            .any(|entry| entry.trim().is_empty())
        {
            return Err(ProvenanceError::EmptyEntry("proof_dependencies"));
        }
        Ok(())
    }

    /// Computes the identity of a generator artifact bound to this provenance.
    pub fn identify(
        &self,
        artifact_kind: &str,
        bytes: &[u8],
    ) -> Result<ArtifactIdentity, ProvenanceError> {
        self.validate()?;
        let mut preimage = Vec::new();
        for field in [
            self.clause_id.as_str(),
            self.input_id.as_str(),
            self.profile_revision.as_str(),
            self.executable_digest.as_str(),
            self.options_digest.as_str(),
        ] {
            preimage.extend_from_slice(field.as_bytes());
            preimage.push(0);
        }
        for entry in self.assumptions.iter().chain(&self.proof_dependencies) {
            preimage.extend_from_slice(entry.as_bytes());
            preimage.push(0);
        }
        preimage.extend_from_slice(bytes);
        Ok(ArtifactIdentity::of(artifact_kind, &preimage))
    }
}
