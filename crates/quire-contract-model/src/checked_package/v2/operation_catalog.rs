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
    #[allow(dead_code)]
    version: Box<str>,
    #[allow(dead_code)]
    families: Vec<Box<str>>,
    groups: BTreeMap<Box<str>, Vec<Box<str>>>,
    /// Each entry is exactly `{authority, identity}` (QSpec `DefinitionRef`).
    law_roles: BTreeMap<LawRole, Vec<CheckedArtifactRef>>,
    #[allow(dead_code)]
    profile_law_roles: Vec<Box<str>>,
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
    modes: BTreeMap<Box<str>, Vec<Box<str>>>,
    type_pinned_modes: BTreeMap<OperationModeKind, Vec<Box<str>>>,
}

impl OperationCatalog {
    /// Whether `value` is one of the catalog's closed values for mode `kind`.
    pub(super) fn mode_value_admitted(&self, kind: OperationModeKind, value: &str) -> bool {
        self.modes
            .get(kind.as_wire())
            .is_some_and(|values| values.iter().any(|candidate| &**candidate == value))
    }

    /// Every catalogued entry, so a test reads a set of entries from the
    /// catalog this build depends on rather than listing identities.
    #[cfg(test)]
    pub(super) fn entries(&self) -> impl Iterator<Item = &OperationCatalogEntry> {
        self.operations.values()
    }

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

/// The parsed, cached production catalog, read once through [`read_catalog`].
/// Panics on first use if the build's own catalog bytes are unreadable: that
/// is a build-time defect, never a caller input, so this one read asserts
/// rather than threading a package refusal for it (FR-038-AC-58).
pub(super) fn operation_catalog() -> &'static OperationCatalog {
    CATALOG.get_or_init(|| {
        read_catalog(CATALOG_BYTES).unwrap_or_else(|error| {
            panic!("the checked-operation catalog this build depends on is unreadable: {error}")
        })
    })
}

/// A catalog read failure: the member the bytes failed at and why. It names
/// the unreadable entry, for example `law_roles.integer_division[0]`.
#[derive(Debug)]
pub(super) struct CatalogReadError {
    path: String,
    message: String,
}

impl std::fmt::Display for CatalogReadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "at `{}`: {}", self.path, self.message)
    }
}

impl CatalogReadError {
    /// The path of the entry the read failed at.
    #[cfg(test)]
    pub(super) fn path(&self) -> &str {
        &self.path
    }
}

/// Reads and indexes catalog bytes. Bytes the reader cannot read, such as a
/// `law_roles` entry that is not exactly `{authority, identity}`, return an
/// error naming the entry; nothing here panics on supplied bytes.
pub(super) fn read_catalog(bytes: &str) -> Result<OperationCatalog, CatalogReadError> {
    let value: serde_json::Value =
        serde_json::from_str(bytes).map_err(|error| CatalogReadError {
            path: ".".to_owned(),
            message: error.to_string(),
        })?;
    let wire: OperationCatalogWire = serde_path_to_error::deserialize(&value).map_err(|error| {
        // A closed vocabulary's decode error does not echo the word (the
        // package reader classifies by message), so the catalog read names
        // the offending word from the bytes it was given.
        let mut at = Some(&value);
        for segment in error.path() {
            at = at.and_then(|current| match segment {
                serde_path_to_error::Segment::Seq { index } => current.get(*index),
                serde_path_to_error::Segment::Map { key } => current.get(key.as_str()),
                serde_path_to_error::Segment::Enum { .. }
                | serde_path_to_error::Segment::Unknown => None,
            });
        }
        let word = at
            .and_then(serde_json::Value::as_str)
            .map(|word| format!(" (word `{word}`)"))
            .unwrap_or_default();
        CatalogReadError {
            path: error.path().to_string(),
            message: format!("{}{word}", error.inner()),
        }
    })?;
    Ok(index_catalog(wire))
}

fn index_catalog(wire: OperationCatalogWire) -> OperationCatalog {
    let operations = wire
        .operations
        .into_iter()
        .map(|entry| (entry.identity.clone(), entry))
        .collect();
    OperationCatalog {
        operations,
        groups: wire.groups,
        law_roles: wire.law_roles,
        modes: wire.modes,
        type_pinned_modes: wire.type_pinned_modes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    /// FR-038-AC-65: each of the six new words converts wire string to enum
    /// member and back; the operator classes the catalog's entries name are
    /// exactly the enum's members (read from the catalog this build depends
    /// on, no copy); and the production catalog reads.
    ///
    /// Tracing: TC-048, FR-038-AC-65
    #[trace("TC-048", "FR-038-AC-65")]
    #[test]
    fn tc_048_the_new_catalog_words_round_trip_and_the_catalog_reads() {
        use std::collections::BTreeSet;
        for (wire, member) in [
            ("case", ApplicationOperator::Case),
            ("temporal_formula", ApplicationOperator::TemporalFormula),
            ("temporal_fairness", ApplicationOperator::TemporalFairness),
        ] {
            assert_eq!(ApplicationOperator::from_wire(wire), Some(member));
            assert_eq!(member.as_wire(), wire);
        }
        for (wire, member) in [
            ("temporal_interval", OperationMemberKind::TemporalInterval),
            ("fairness", OperationMemberKind::Fairness),
        ] {
            assert_eq!(OperationMemberKind::from_wire(wire), Some(member));
            assert_eq!(member.as_wire(), wire);
        }
        assert_eq!(
            OperationConstraintKind::from_wire("union_arms"),
            Some(OperationConstraintKind::UnionArms)
        );
        assert_eq!(OperationConstraintKind::UnionArms.as_wire(), "union_arms");

        let supplied: serde_json::Value =
            serde_json::from_str(CATALOG_BYTES).expect("the production catalog is JSON");
        let operators: BTreeSet<&str> = supplied["operations"]
            .as_array()
            .expect("operations")
            .iter()
            .filter_map(|entry| entry["operator"].as_str())
            .collect();
        let enum_operators: BTreeSet<&str> = ApplicationOperator::ALL
            .iter()
            .map(|o| o.as_wire())
            .collect();
        assert_eq!(operators, enum_operators);
        assert!(read_catalog(CATALOG_BYTES).is_ok());
    }

    /// FR-038-AC-65: catalog bytes naming an operator class, a member kind or
    /// a constraint kind outside its vocabulary return the typed error naming
    /// the word, with no panic.
    ///
    /// Tracing: TC-048, FR-038-AC-65
    #[trace("TC-048", "FR-038-AC-65")]
    #[test]
    fn tc_048_a_catalog_word_outside_its_vocabulary_is_a_typed_error_naming_it() {
        let supplied: serde_json::Value =
            serde_json::from_str(CATALOG_BYTES).expect("the production catalog is JSON");
        let position = |identity: &str| {
            supplied["operations"]
                .as_array()
                .expect("operations")
                .iter()
                .position(|entry| entry["identity"] == identity)
                .expect("catalogued identity")
        };
        let not = position("quire.op.boolean.not");
        let eventually = position("quire.op.temporal.eventually");
        let case = position("quire.op.control.case");
        for (index, member, word) in [
            (not, "operator", "not_an_operator"),
            (eventually, "member", "not_a_member_kind"),
            (case, "constraints", "not_a_constraint_kind"),
        ] {
            let mut edited = supplied.clone();
            let entry = &mut edited["operations"][index];
            match member {
                "member" => entry["member"] = serde_json::json!(word),
                "constraints" => entry["constraints"][0]["kind"] = serde_json::json!(word),
                other => entry[other] = serde_json::json!(word),
            }
            let error = match read_catalog(&edited.to_string()) {
                Ok(_) => panic!("`{word}` must not read"),
                Err(error) => error,
            };
            assert!(
                error.path().starts_with(&format!("operations[{index}]")),
                "{error}"
            );
            assert!(error.to_string().contains(word), "{error}");
        }
    }

    /// FR-038-AC-58: the catalog read over supplied bytes whose `law_roles`
    /// entries are exactly `{authority, identity}` returns the catalog, and
    /// over bytes in which an entry carries `revision`, `digest_domain` or
    /// `digest` returns an error naming the entry, without a panic. The
    /// supplied bytes are the production catalog's own, with one entry edited.
    ///
    /// Tracing: TC-048, FR-038-AC-58
    #[trace("TC-048", "FR-038-AC-58")]
    #[test]
    fn tc_048_the_catalog_read_names_a_law_role_entry_that_is_not_authority_and_identity() {
        let supplied: serde_json::Value =
            serde_json::from_str(CATALOG_BYTES).expect("the production catalog is JSON");
        assert!(read_catalog(&supplied.to_string()).is_ok());
        let entries = supplied["law_roles"]["integer_division"]
            .as_array()
            .expect("integer_division is catalogued");
        assert!(entries.iter().all(|entry| entry
            .as_object()
            .is_some_and(|members| members.len() == 2
                && members.contains_key("authority")
                && members.contains_key("identity"))));
        for (member, value) in [
            (
                "revision",
                serde_json::json!({"namespace": "n", "value": "v"}),
            ),
            (
                "digest_domain",
                serde_json::json!("quire.definition.bytes/v1"),
            ),
            ("digest", serde_json::json!("0".repeat(64))),
        ] {
            let mut edited = supplied.clone();
            edited["law_roles"]["integer_division"][1][member] = value;
            let error = match read_catalog(&edited.to_string()) {
                Ok(_) => panic!("an entry carrying `{member}` must not read"),
                Err(error) => error,
            };
            assert!(
                error.path().starts_with("law_roles.integer_division[1]"),
                "{member}: {error}"
            );
        }
    }

    /// Every catalog vocabulary the reader decodes into an enum is exactly
    /// the enum's member set, and `LawRole::selection_role` is `Some` for
    /// exactly the catalog's profile law roles.
    ///
    /// Tracing: TC-048
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
}
