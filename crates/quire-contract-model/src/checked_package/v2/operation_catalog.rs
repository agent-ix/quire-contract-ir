//! The closed `quire.checked-operation-catalog/v1` every V2 `application`
//! term's `operation` member is validated against (`validate_operations`).
//!
//! This module reads the catalog from its home and holds no copy of it. The
//! bytes it used to embed were copied from a private repository into this
//! public one; both copies were deleted, and neither is coming back. A copy is
//! a copy wherever it is spelled, and `schemas/` was not a different rule from
//! `tests/fixtures/`.
//!
//! Between that deletion and this module's current form the build did not
//! compile, deliberately (agent-ix/quire-contract-ir#169). The alternatives
//! were to keep the copy, or to drop catalog-driven validation and let this
//! reader start admitting `operation` members it used to refuse — a silent
//! weakening of the contract, decided by an agent, in a crate other
//! repositories depend on. A build that stops and says why was the honest one
//! of the three.
//!
//! What unblocked it was the third option asked for on #166: a published source
//! to depend on rather than copy. `quire-verification-contracts` is now the
//! catalog's home; this crate depends on it and owns only the reader. Restoring
//! the deleted bytes here, under any name or path, is still not one of the
//! options.

use super::{
    ApplicationOperator, CheckedArtifactRef, LawRole, OperationConstraintKind, OperationMemberKind,
    OperationModeKind,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The catalog, read from its home rather than copied here.
///
/// `quire-verification-contracts` owns `quire.checked-operation-catalog/v1` and
/// publishes the bytes and a digest over them; this crate owns the reader that
/// decides what they admit. That split is the point: one definition of the closed
/// vocabulary, one implementation of the rules it drives. A copy of these bytes in
/// this repository, under any name or path, is a defect — it is what left this
/// module unable to compile at all.
const CATALOG_BYTES: &str =
    quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1;

/// The vocabulary identity this reader's rules are written against.
const CATALOG_VERSION: &str =
    quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1_VERSION;

/// One `operation-catalog.json` `operations[]` entry.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperationCatalogEntry {
    /// Catalogued `OperationIdentity`.
    pub(super) identity: Box<str>,
    /// Required application `operator` class.
    pub(super) operator: ApplicationOperator,
    /// Fixed leading operand families or group names, in order.
    pub(super) operands: Vec<Box<str>>,
    /// The family or group name every operand past `operands` must fit, or
    /// `None` when no further operand is admitted.
    pub(super) rest: Option<Box<str>>,
    /// The result form; `inner:<n>` over a `reference` operand is checked
    /// (`check_inner_result`), the other forms are not.
    pub(super) result: Box<str>,
    /// Required law roles, in order.
    pub(super) laws: Vec<LawRole>,
    /// Required mode kind, or `None` when the operation carries no mode.
    pub(super) mode: Option<OperationModeKind>,
    /// Required member kind, or `None` when the operation carries no member.
    pub(super) member: Option<OperationMemberKind>,
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
    pub(super) kind: OperationConstraintKind,
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
    version: Box<str>,
    #[allow(dead_code)]
    families: Vec<Box<str>>,
    groups: BTreeMap<Box<str>, Vec<Box<str>>>,
    law_roles: BTreeMap<LawRole, Vec<CheckedArtifactRef>>,
    #[allow(dead_code)]
    profile_law_roles: Vec<Box<str>>,
    #[allow(dead_code)]
    modes: BTreeMap<Box<str>, Vec<Box<str>>>,
    type_pinned_modes: BTreeMap<OperationModeKind, Vec<Box<str>>>,
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
    law_roles: BTreeMap<LawRole, Vec<CheckedArtifactRef>>,
    type_pinned_modes: BTreeMap<OperationModeKind, Vec<Box<str>>>,
}

impl OperationCatalog {
    /// The catalogued entry for an `operation.identity`, if it names one.
    pub(super) fn entry(&self, identity: &str) -> Option<&OperationCatalogEntry> {
        self.operations.get(identity)
    }

    /// Every catalogued definition of a value law role (e.g.
    /// `integer_division`), if `role` names one.
    pub(super) fn law_role_definitions(&self, role: LawRole) -> Option<&[CheckedArtifactRef]> {
        self.law_roles.get(&role).map(Vec::as_slice)
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
    pub(super) fn is_type_pinned_mode(&self, kind: OperationModeKind) -> bool {
        self.type_pinned_modes.contains_key(&kind)
    }
}

static CATALOG: OnceLock<OperationCatalog> = OnceLock::new();

/// The parsed, cached production catalog. Panics on first use if the
/// embedded catalog bytes are not valid — a build-time defect, never a
/// caller input, so this asserts rather than threading a refusal for it.
pub(super) fn operation_catalog() -> &'static OperationCatalog {
    CATALOG.get_or_init(|| parse_catalog(CATALOG_BYTES))
}

/// Parses and indexes catalog bytes of this reader's vocabulary identity.
/// Panics on invalid bytes: the production catalog is a build-time input,
/// and a test passes a catalog it read itself.
pub(super) fn parse_catalog(bytes: &str) -> OperationCatalog {
    let wire: OperationCatalogWire =
        serde_json::from_str(bytes).expect("checked-operation-catalog-v1.json is valid");
    // The dependency `rev` is provenance and pins nothing about content: a rev
    // bump that landed a different closed vocabulary would leave this reader
    // validating every package against it, silently. The identity is what the
    // reader's rules are written for, so it is checked here rather than assumed.
    assert_eq!(
        wire.version.as_ref(),
        CATALOG_VERSION,
        "the catalog read from its home declares a different vocabulary identity \
         than this reader implements"
    );
    let operations = wire
        .operations
        .into_iter()
        .map(|entry| (entry.identity.clone(), entry))
        .collect();
    OperationCatalog {
        operations,
        groups: wire.groups,
        law_roles: wire.law_roles,
        type_pinned_modes: wire.type_pinned_modes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    /// The catalog this crate compiled against is the document its home
    /// published, checked by content rather than by the dependency `rev`.
    ///
    /// The `rev` is provenance: it says where the bytes came from, and it dies
    /// if that history is rewritten — which is exactly what happened to the
    /// copy this dependency replaced. The digest is over the bytes themselves,
    /// so a rev bump that landed a different vocabulary fails here instead of
    /// silently changing what every CheckedPackage V2 is validated against.
    #[test]
    fn the_catalog_read_from_its_home_is_the_document_that_home_published() {
        let measured = format!("{:x}", Sha256::digest(CATALOG_BYTES.as_bytes()));
        assert_eq!(
            measured,
            quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1_SHA256,
            "the operation catalog's bytes do not match the digest its home publishes for them"
        );
    }

    /// Every catalog vocabulary the reader decodes into an enum is exactly
    /// the enum's member set, and `LawRole::selection_role` is `Some` for
    /// exactly the catalog's profile law roles.
    ///
    /// Tracing: TC-048
    /// ACs: FR-038-AC-34
    #[test]
    fn tc_048_the_catalog_vocabularies_are_the_decoded_enums() {
        use std::collections::BTreeSet;
        let wire: OperationCatalogWire =
            serde_json::from_str(CATALOG_BYTES).expect("the catalog is valid");
        let words = |list: &[Box<str>]| -> BTreeSet<String> {
            list.iter().map(|word| word.to_string()).collect()
        };
        let members = |all: Vec<&'static str>| -> BTreeSet<String> {
            all.into_iter().map(str::to_owned).collect()
        };
        assert_eq!(
            words(&wire.member_kinds),
            members(
                OperationMemberKind::ALL
                    .iter()
                    .map(|k| k.as_wire())
                    .collect()
            )
        );
        assert_eq!(
            words(&wire.constraint_kinds),
            members(
                OperationConstraintKind::ALL
                    .iter()
                    .map(|k| k.as_wire())
                    .collect()
            )
        );
        assert_eq!(
            wire.modes
                .keys()
                .map(|k| k.to_string())
                .collect::<BTreeSet<_>>(),
            members(OperationModeKind::ALL.iter().map(|k| k.as_wire()).collect())
        );
        let value_roles: BTreeSet<String> = wire
            .law_roles
            .keys()
            .map(|role| role.as_wire().to_owned())
            .collect();
        let profile_roles = words(&wire.profile_law_roles);
        assert_eq!(
            value_roles
                .union(&profile_roles)
                .cloned()
                .collect::<BTreeSet<_>>(),
            members(LawRole::ALL.iter().map(|k| k.as_wire()).collect())
        );
        for role in LawRole::ALL {
            assert_eq!(
                role.selection_role().is_some(),
                profile_roles.contains(role.as_wire()),
                "{role:?}: a selection role exactly for a catalog profile role"
            );
        }
    }

    /// The reader refuses a catalog of another vocabulary identity rather than
    /// applying v1 rules to it. Paired with the digest check above: that one
    /// catches changed bytes, this one catches a deliberate version change.
    #[test]
    fn the_catalog_declares_the_vocabulary_this_reader_implements() {
        let wire: OperationCatalogWire =
            serde_json::from_str(CATALOG_BYTES).expect("the catalog is valid");
        assert_eq!(wire.version.as_ref(), "quire.checked-operation-catalog/v1");
        assert_eq!(CATALOG_VERSION, "quire.checked-operation-catalog/v1");
    }
}
