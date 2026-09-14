//! Predicate identity derived directly from constructor-private QSL authority.

use core::fmt;

use quire_spec_language::protocol_artifact::checked_predicate::ValidatedCheckedPredicate;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::bridge::{
    canonical, BridgeDigest, BridgeDigestParseError, BridgeError, BridgeLimits, ContractSelection,
};

use super::PREDICATE_REF_PROFILE;

/// Content-addressed checked predicate identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PredicateRef(BridgeDigest);

impl PredicateRef {
    /// Parses an exact lowercase SHA-256 identity.
    pub fn parse(value: &str) -> Result<Self, BridgeDigestParseError> {
        BridgeDigest::parse(value).map(Self)
    }

    /// Returns exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Returns the underlying bridge digest.
    #[must_use]
    pub const fn digest(self) -> BridgeDigest {
        self.0
    }
}

impl fmt::Display for PredicateRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl core::str::FromStr for PredicateRef {
    type Err = BridgeDigestParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

/// Read-only definition facts extracted from a QSL checked-predicate view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateDefinition {
    predicate_ref: PredicateRef,
    owner_document_identity: String,
    owner_document_digest: BridgeDigest,
    parent_kind: String,
    declaration: u32,
    package_identity: String,
    source_identity: String,
    source_revision: String,
    clause_identity: String,
    model_identities: Vec<String>,
    expression_identities: Vec<String>,
    profile_identities: Vec<String>,
    canonical_tuple: Vec<u8>,
}

impl PredicateDefinition {
    pub(crate) fn from_checked(
        checked: &ValidatedCheckedPredicate,
        native_contract: &ContractSelection,
        limits: BridgeLimits,
    ) -> Result<Self, BridgeError> {
        let document = checked.document();
        let owner_digest_text = document.digest().to_string();
        let owner_document_digest = BridgeDigest::parse(
            owner_digest_text
                .strip_prefix("sha256:")
                .unwrap_or(owner_digest_text.as_str()),
        )
        .map_err(|_| {
            BridgeError::new(
                crate::bridge::BridgeErrorCode::InvalidNativePredicateProjection,
                "QSL checked-predicate digest is not lowercase SHA-256",
                "predicate.owner_document_digest",
            )
        })?;
        let (source, span) = checked.source();
        let (requirement, clause, execution) = checked.clause();
        let parent_kind = checked.parent_kind().unwrap_or("unknown").to_owned();
        let bindings: Vec<Value> = checked
            .bindings()
            .map(|(kind, identity)| json!({"identity": identity, "kind": kind}))
            .collect();
        let model_identities = checked
            .bindings()
            .filter(|(kind, _)| *kind == "model")
            .map(|(_, identity)| identity.to_owned())
            .collect::<Vec<_>>();
        let expression_identities = checked
            .bindings()
            .filter(|(kind, _)| *kind == "expression")
            .map(|(_, identity)| identity.to_owned())
            .collect::<Vec<_>>();
        let profile_identities = checked
            .profile_identities()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let tuple = json!({
            "bindings": bindings,
            "bridge_profile": super::PROFILE,
            "clause": {
                "execution": execution,
                "identity": clause,
                "requirement": requirement,
            },
            "expression": {
                "leaf": checked.leaf(),
                "parameters": checked.parameters().unwrap_or(&[]),
                "profiles": &profile_identities,
                "type_indices": checked.type_indices(),
            },
            "native_contract": native_contract,
            "owner_document": {
                "digest": owner_document_digest,
                "identity": document.identity(),
            },
            "package": {
                "artifact": checked.package_artifact(),
                "digest": checked.package_digest().to_string(),
            },
            "source": {
                "record": source,
                "span": span,
            },
            "subject": {
                "declaration": checked.declaration(),
                "parent_kind": &parent_kind,
            },
        });
        let canonical_tuple = canonical::encode(&tuple, limits)?;
        let predicate_ref = PredicateRef(BridgeDigest::domain(
            PREDICATE_REF_PROFILE,
            &canonical_tuple,
        ));
        Ok(Self {
            predicate_ref,
            owner_document_identity: document.identity().to_owned(),
            owner_document_digest,
            parent_kind,
            declaration: checked.declaration(),
            package_identity: checked.package_artifact().identity.clone(),
            source_identity: source.native.identity.clone(),
            source_revision: source.native.revision.clone(),
            clause_identity: clause.to_owned(),
            model_identities,
            expression_identities,
            profile_identities,
            canonical_tuple,
        })
    }

    /// Returns the canonical predicate identity.
    #[must_use]
    pub const fn predicate_ref(&self) -> PredicateRef {
        self.predicate_ref
    }

    /// Returns the QSL checked-predicate document identity.
    #[must_use]
    pub fn owner_document_identity(&self) -> &str {
        &self.owner_document_identity
    }

    /// Returns the raw digest of exact QSL checked-predicate bytes.
    #[must_use]
    pub const fn owner_document_digest(&self) -> BridgeDigest {
        self.owner_document_digest
    }

    /// Returns the owner declaration family containing the leaf.
    #[must_use]
    pub fn parent_kind(&self) -> &str {
        &self.parent_kind
    }

    /// Returns the QSL declaration-table identity.
    #[must_use]
    pub const fn declaration(&self) -> u32 {
        self.declaration
    }

    /// Returns the selected compiled-package artifact identity.
    #[must_use]
    pub fn package_identity(&self) -> &str {
        &self.package_identity
    }

    /// Returns the editable native source identity.
    #[must_use]
    pub fn source_identity(&self) -> &str {
        &self.source_identity
    }

    /// Returns the editable native source revision.
    #[must_use]
    pub fn source_revision(&self) -> &str {
        &self.source_revision
    }

    /// Returns the authored clause identity.
    #[must_use]
    pub fn clause_identity(&self) -> &str {
        &self.clause_identity
    }

    /// Returns sorted exact model identities from the owner view.
    #[must_use]
    pub fn model_identities(&self) -> &[String] {
        &self.model_identities
    }

    /// Returns exact expression identities from the owner view.
    #[must_use]
    pub fn expression_identities(&self) -> &[String] {
        &self.expression_identities
    }

    /// Returns the exact evaluation and definedness profile identities.
    #[must_use]
    pub fn profile_identities(&self) -> &[String] {
        &self.profile_identities
    }

    /// Returns canonical bytes whose domain-separated hash is [`PredicateRef`].
    #[must_use]
    pub fn canonical_tuple(&self) -> &[u8] {
        &self.canonical_tuple
    }
}
