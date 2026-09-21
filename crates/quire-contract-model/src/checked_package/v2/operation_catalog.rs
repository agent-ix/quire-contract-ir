//! The closed `quire.checked-operation-catalog/v1` every V2 `application`
//! term's `operation` member is validated against (`validate_operations`).
//!
//! This module has no catalog to read. The bytes it embedded were copied from
//! a private repository into this public one; both copies are now deleted, and
//! neither is coming back. A copy is a copy wherever it is spelled, and
//! `schemas/` is not a different rule from `tests/fixtures/`.
//!
//! The build therefore does not compile, deliberately. The alternatives were
//! to keep the copy, or to drop catalog-driven validation and let this reader
//! start admitting `operation` members it used to refuse — a silent weakening
//! of the contract, decided by an agent, in a crate other repositories depend
//! on. A build that stops and says why is the honest one of the three.
//!
//! What unblocks it, on agent-ix/quire-contract-ir#166: a published source for
//! `quire.checked-operation-catalog/v1` this crate can depend on rather than
//! copy, or a catalog authored here in its own right. Restoring the deleted
//! bytes is not one of the options.

use super::CheckedArtifactRef;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

compile_error!(
    "quire.checked-operation-catalog/v1 has no source in this repository. The bytes this \
     module embedded were copied from a private repository and were deleted; see \
     agent-ix/quire-contract-ir#166. Resolve it with a published source or a catalog \
     authored here — not by restoring the deleted file."
);

// Empty only so the rest of this module still type-checks and the build stops
// on the `compile_error!` above rather than on a cascade of consequential
// errors that bury it. Nothing reads it: the crate does not compile.
const CATALOG_BYTES: &str = "";

/// One `operation-catalog.json` `operations[]` entry.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperationCatalogEntry {
    /// Catalogued `OperationIdentity`.
    pub(super) identity: Box<str>,
    /// Required application `operator` class.
    pub(super) operator: Box<str>,
    /// Fixed leading operand families or group names, in order.
    pub(super) operands: Vec<Box<str>>,
    /// The family or group name every operand past `operands` must fit, or
    /// `None` when no further operand is admitted.
    pub(super) rest: Option<Box<str>>,
    /// The result form (unused by this reader's current checks, kept for
    /// fidelity with the catalog's own closed shape).
    #[allow(dead_code)]
    pub(super) result: Box<str>,
    /// Required law roles, in order.
    pub(super) laws: Vec<Box<str>>,
    /// Required mode kind, or `None` when the operation carries no mode.
    pub(super) mode: Option<Box<str>>,
    /// Required member kind, or `None` when the operation carries no member.
    pub(super) member: Option<Box<str>>,
    /// Cross-operand constraints.
    pub(super) constraints: Vec<OperationConstraint>,
    /// The leaf source this operation compares, or `None` when it carries no
    /// leaves.
    pub(super) leaves: Option<Box<str>>,
}

/// One `operations[].constraints[]` entry.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperationConstraint {
    pub(super) kind: Box<str>,
    pub(super) operands: Vec<u64>,
    #[allow(dead_code)]
    pub(super) family: Option<Box<str>>,
}

/// The full top-level `operation-catalog.json` shape. `deny_unknown_fields`
/// requires every published key to be declared here even though
/// [`OperationCatalog`] (built from this once, in [`operation_catalog`])
/// keeps only the tables this reader's checks actually consult; the
/// `#[allow(dead_code)]` fields below are parsed for that fidelity and then
/// dropped, never read again.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationCatalogWire {
    #[allow(dead_code)]
    version: Box<str>,
    #[allow(dead_code)]
    families: Vec<Box<str>>,
    groups: BTreeMap<Box<str>, Vec<Box<str>>>,
    law_roles: BTreeMap<Box<str>, Vec<CheckedArtifactRef>>,
    profile_law_roles: Vec<Box<str>>,
    #[allow(dead_code)]
    modes: BTreeMap<Box<str>, Vec<Box<str>>>,
    type_pinned_modes: BTreeMap<Box<str>, Vec<Box<str>>>,
    #[allow(dead_code)]
    member_kinds: Vec<Box<str>>,
    #[allow(dead_code)]
    constraint_kinds: Vec<Box<str>>,
    #[allow(dead_code)]
    result_forms: Vec<Box<str>>,
    #[allow(dead_code)]
    leaf_sources: Vec<Box<str>>,
    operations: Vec<OperationCatalogEntry>,
}

/// The parsed, indexed `quire.checked-operation-catalog/v1`.
pub(super) struct OperationCatalog {
    operations: BTreeMap<Box<str>, OperationCatalogEntry>,
    groups: BTreeMap<Box<str>, Vec<Box<str>>>,
    law_roles: BTreeMap<Box<str>, Vec<CheckedArtifactRef>>,
    profile_law_roles: Vec<Box<str>>,
    type_pinned_modes: BTreeMap<Box<str>, Vec<Box<str>>>,
}

impl OperationCatalog {
    /// The catalogued entry for an `operation.identity`, if it names one.
    pub(super) fn entry(&self, identity: &str) -> Option<&OperationCatalogEntry> {
        self.operations.get(identity)
    }

    /// Every catalogued definition of a value law role (e.g.
    /// `integer_division`), if `role` names one.
    pub(super) fn law_role_definitions(&self, role: &str) -> Option<&[CheckedArtifactRef]> {
        self.law_roles.get(role).map(Vec::as_slice)
    }

    /// Whether `role` is a clause/profile law role (`temporal_profile`,
    /// `protocol_profile`), joined against the lock's `profile_selections`
    /// rather than its `definition_selections`.
    pub(super) fn is_profile_role(&self, role: &str) -> bool {
        self.profile_law_roles
            .iter()
            .any(|candidate| **candidate == *role)
    }

    /// Whether `actual` is `expected` itself or a member of the group
    /// `expected` names.
    pub(super) fn family_fits(&self, actual: &str, expected: &str) -> bool {
        actual == expected
            || self
                .groups
                .get(expected)
                .is_some_and(|members| members.iter().any(|member| **member == *actual))
    }

    /// Whether a mode `kind` (`rounding`, `text_profile`) is one whose value
    /// an operand or leaf's own type authoritatively pins.
    pub(super) fn is_type_pinned_mode(&self, kind: &str) -> bool {
        self.type_pinned_modes.contains_key(kind)
    }
}

static CATALOG: OnceLock<OperationCatalog> = OnceLock::new();

/// The parsed, cached production catalog. Panics on first use if the
/// embedded catalog bytes are not valid — a build-time defect, never a
/// caller input, so this asserts rather than threading a refusal for it.
pub(super) fn operation_catalog() -> &'static OperationCatalog {
    CATALOG.get_or_init(|| {
        let wire: OperationCatalogWire = serde_json::from_str(CATALOG_BYTES)
            .expect("embedded checked-operation-catalog-v1.json is valid");
        let operations = wire
            .operations
            .into_iter()
            .map(|entry| (entry.identity.clone(), entry))
            .collect();
        OperationCatalog {
            operations,
            groups: wire.groups,
            law_roles: wire.law_roles,
            profile_law_roles: wire.profile_law_roles,
            type_pinned_modes: wire.type_pinned_modes,
        }
    })
}
