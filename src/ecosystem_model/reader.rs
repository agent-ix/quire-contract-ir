//! Strict model reading by independent re-export.

use crate::bridge::{canonical, BridgeDigest};

use super::decision::{ModelCauseCode as Code, ModelDecision};
use super::document::{export, validate_wire_identity, ModelDocument, ModelWire};
use super::manifest::{bridge_limits, map_canonical, CheckedManifestSet, EcosystemLimits};

/// Constructor-private model validated against its complete source manifest.
///
/// The public type exposes no authority or acceptance transition:
///
/// ```compile_fail
/// use quire_contract_ir::ecosystem_model::ValidatedEcosystemModel;
///
/// fn authorize(model: &ValidatedEcosystemModel) {
///     model.accept();
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ValidatedEcosystemModel {
    document: ModelDocument,
}

impl ValidatedEcosystemModel {
    /// Returns the exact strict-read model bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.document.bytes()
    }

    /// Returns the model content identity.
    #[must_use]
    pub const fn identity(&self) -> BridgeDigest {
        self.document.identity()
    }

    /// Returns SHA-256 of the complete model bytes.
    #[must_use]
    pub const fn byte_digest(&self) -> BridgeDigest {
        self.document.byte_digest()
    }

    /// Returns the immutable source manifest identity.
    #[must_use]
    pub const fn manifest_identity(&self) -> BridgeDigest {
        self.document.manifest_identity()
    }
}

/// Strict-reads a model and requires byte equality with independent re-export.
pub fn read(
    bytes: &[u8],
    manifest: &CheckedManifestSet,
    limits: EcosystemLimits,
) -> Result<ValidatedEcosystemModel, ModelDecision> {
    let limits = limits.effective();
    let wire: ModelWire = canonical::decode(bytes, bridge_limits(limits, false))
        .map_err(|error| map_canonical(error, "model"))?;
    validate_wire_identity(&wire, limits)?;
    let expected = export(manifest, limits)?;
    if expected.bytes() != bytes {
        return Err(ModelDecision::new(
            Code::GraphMismatch,
            "model",
            "model differs from complete manifest re-export",
        ));
    }
    Ok(ValidatedEcosystemModel { document: expected })
}
