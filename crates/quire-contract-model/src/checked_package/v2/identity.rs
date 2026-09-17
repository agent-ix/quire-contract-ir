//! Closed nominal node identity preimages (enum declaration, enum member,
//! dimension, declared unit) and their semantic and cross-field rules.
//!
//! The node key of a nominal form is SHA-256 of the RFC 8785 canonical bytes
//! of its preimage under `quire.checked-semantic-node/v1`. Every violation is
//! `invalid_semantic_graph`.

use super::natural::coprime;
use super::CheckedPackageLockV2;
use super::{CheckedNodeTag, CheckedSemanticNodeV2, WorkMeter};
use crate::checked_package::common::{digest_json, ValidationFailure};
use crate::checked_package::shared::{CheckedNodeId, CheckedPackageRefusalCode};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

/// The closed nominal identity preimage carried by a V2 node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "version", deny_unknown_fields)]
pub enum NominalIdentityPreimage {
    /// `quire.enum-declaration-node/v1`.
    #[serde(rename = "quire.enum-declaration-node/v1")]
    EnumDeclaration(EnumDeclarationPreimage),
    /// `quire.enum-member-node/v1`.
    #[serde(rename = "quire.enum-member-node/v1")]
    EnumMember(EnumMemberPreimage),
    /// `quire.dimension-node/v1`.
    #[serde(rename = "quire.dimension-node/v1")]
    Dimension(DimensionPreimage),
    /// `quire.unit-node/v1`.
    #[serde(rename = "quire.unit-node/v1")]
    Unit(UnitPreimage),
}

/// Owner subject of a nominal declaration; joins an exact lock selection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NominalOwner {
    /// A locked raw source, joined by authority and identity.
    Source {
        /// Source authority.
        authority: Box<str>,
        /// Source identity.
        identity: Box<str>,
    },
    /// A selected definition, joined by authority and identity.
    Definition {
        /// Definition authority.
        authority: Box<str>,
        /// Definition identity.
        identity: Box<str>,
    },
    /// A declaration in a selected domain package, joined by the package
    /// identity; `node` is the IR node identity inside that package.
    Model {
        /// Domain package identity.
        identity: Box<str>,
        /// IR node identity within the domain package.
        node: Box<str>,
    },
}

/// Enum declaration identity preimage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumDeclarationPreimage {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Whether member order is semantic.
    pub ordered: bool,
    /// Member identifiers.
    pub members: Vec<Box<str>>,
}

/// Enum member identity preimage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumMemberPreimage {
    /// Declaring enum node key.
    pub declaration_node_id: CheckedNodeId,
    /// Member identifier.
    pub case: Box<str>,
}

/// Dimension identity preimage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionPreimage {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Base-dimension terms; empty for a base dimension.
    pub terms: Vec<DimensionTerm>,
}

/// One base-dimension power in a derived dimension.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionTerm {
    /// Base dimension node key.
    pub dimension_node_id: CheckedNodeId,
    /// Canonical nonzero integer exponent.
    pub exponent: Box<str>,
}

/// Declared unit identity preimage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitPreimage {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Dimension node key.
    pub dimension_node_id: CheckedNodeId,
    /// Target unit key; `None` (wire `null`, required member) for the root.
    #[serde(deserialize_with = "required_nullable")]
    pub target_unit_node_id: Option<CheckedNodeId>,
    /// Exact reduced scale to the target.
    pub scale: CheckedRational,
    /// Exact reduced offset to the target.
    pub offset: CheckedRational,
}

/// A canonical decimal rational.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedRational {
    /// Canonical integer string.
    pub numerator: Box<str>,
    /// Canonical positive integer string.
    pub denominator: Box<str>,
}

fn required_nullable<'de, D>(deserializer: D) -> Result<Option<CheckedNodeId>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<CheckedNodeId>::deserialize(deserializer)
}

/// Which nominal preimage a `(node_tag, semantic_form)` pair requires.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NominalKind {
    EnumDeclaration,
    EnumMember,
    Dimension,
    Unit,
}

impl NominalKind {
    fn required_by(tag: CheckedNodeTag, form: &str) -> Option<Self> {
        match (tag, form) {
            (CheckedNodeTag::ScalarType, "enum") => Some(Self::EnumDeclaration),
            (CheckedNodeTag::Value, "enum_value") => Some(Self::EnumMember),
            (CheckedNodeTag::ScalarType, "dimension") => Some(Self::Dimension),
            (CheckedNodeTag::ScalarType, "unit") => Some(Self::Unit),
            _ => None,
        }
    }

    fn of(preimage: &NominalIdentityPreimage) -> Self {
        match preimage {
            NominalIdentityPreimage::EnumDeclaration(_) => Self::EnumDeclaration,
            NominalIdentityPreimage::EnumMember(_) => Self::EnumMember,
            NominalIdentityPreimage::Dimension(_) => Self::Dimension,
            NominalIdentityPreimage::Unit(_) => Self::Unit,
        }
    }
}

impl NominalIdentityPreimage {
    /// Lowercase SHA-256 of the canonical preimage bytes.
    pub fn digest(&self) -> Option<String> {
        serde_json::to_value(self)
            .ok()
            .and_then(|value| digest_json(&value).ok())
    }
}

const PATH: &str = "semantic_graph.nodes.nominal_identity_preimage";

fn invalid() -> ValidationFailure {
    ValidationFailure::Refused(CheckedPackageRefusalCode::InvalidSemanticGraph, PATH)
}

fn require(condition: bool) -> Result<(), ValidationFailure> {
    if condition {
        Ok(())
    } else {
        Err(invalid())
    }
}

/// Validates every node's nominal binding, key re-derivation, owner join,
/// semantic rules and cross-field joins. `tags` holds each node's parsed tag.
pub(super) fn validate_nominal_nodes(
    nodes: &[CheckedSemanticNodeV2],
    tags: &[CheckedNodeTag],
    index: &BTreeMap<&CheckedNodeId, usize>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = NominalGraph { nodes, index };
    let mut roots = BTreeSet::new();
    for (node, tag) in nodes.iter().zip(tags) {
        let required = NominalKind::required_by(*tag, &node.semantic_form);
        let preimage = match (required, &node.nominal_identity_preimage) {
            (None, None) => continue,
            (Some(kind), Some(preimage)) if NominalKind::of(preimage) == kind => preimage,
            _ => return Err(invalid()),
        };
        meter.charge(1)?;
        require(preimage.digest().as_deref() == Some(node.node_id.digest.as_ref()))?;
        match preimage {
            NominalIdentityPreimage::EnumDeclaration(declaration) => {
                validate_enum_declaration(declaration, lock, meter)?;
            }
            NominalIdentityPreimage::EnumMember(member) => {
                validate_enum_member(node, member, &graph)?;
            }
            NominalIdentityPreimage::Dimension(dimension) => {
                validate_dimension(node, dimension, &graph, lock, meter)?;
            }
            NominalIdentityPreimage::Unit(unit) => {
                validate_unit(node, unit, &graph, lock, meter)?;
                if unit.target_unit_node_id.is_none() {
                    // A dimension has exactly one targetless root unit.
                    require(roots.insert(unit.dimension_node_id.clone()))?;
                }
            }
        }
    }
    Ok(())
}

struct NominalGraph<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
}

impl NominalGraph<'_> {
    fn preimage(&self, id: &CheckedNodeId) -> Option<&NominalIdentityPreimage> {
        self.index
            .get(id)
            .and_then(|position| self.nodes.get(*position))
            .and_then(|node| node.nominal_identity_preimage.as_ref())
    }
}

fn validate_owner(
    owner: &NominalOwner,
    lock: &CheckedPackageLockV2,
) -> Result<(), ValidationFailure> {
    let joined = match owner {
        NominalOwner::Source {
            authority,
            identity,
        } => lock
            .sources
            .iter()
            .any(|source| source.authority == *authority && source.identity == *identity),
        NominalOwner::Definition {
            authority,
            identity,
        } => lock.definition_selections.iter().any(|definition| {
            definition.authority == *authority && definition.identity == *identity
        }),
        // The lock selects whole domain packages; the node is not a lock
        // member, so the join is by package identity and the node is only
        // required to be present.
        NominalOwner::Model { identity, node } => {
            !node.is_empty()
                && lock
                    .model_selections
                    .iter()
                    .any(|model| model.identity == *identity)
        }
    };
    require(joined)
}

fn is_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn validate_qualified_name(name: &[Box<str>]) -> Result<(), ValidationFailure> {
    require(!name.is_empty() && name.iter().all(|segment| is_identifier(segment)))
}

/// `^(0|-?[1-9][0-9]*)$`; returns the unsigned magnitude.
fn integer_magnitude(value: &str) -> Option<&str> {
    if value == "0" {
        return Some(value);
    }
    let magnitude = value.strip_prefix('-').unwrap_or(value);
    positive_integer(magnitude).then_some(magnitude)
}

/// `^[1-9][0-9]*$`.
fn positive_integer(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|first| (b'1'..=b'9').contains(&first))
        && bytes.all(|byte| byte.is_ascii_digit())
}

fn validate_rational(
    rational: &CheckedRational,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let numerator = integer_magnitude(&rational.numerator).ok_or_else(invalid)?;
    require(positive_integer(&rational.denominator))?;
    require(coprime(numerator, &rational.denominator, meter)?)
}

fn validate_enum_declaration(
    declaration: &EnumDeclarationPreimage,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    validate_owner(&declaration.owner, lock)?;
    validate_qualified_name(&declaration.qualified_declaration)?;
    require(!declaration.members.is_empty())?;
    let mut seen = BTreeSet::new();
    for member in &declaration.members {
        meter.charge(1)?;
        require(is_identifier(member) && seen.insert(member.as_ref()))?;
    }
    if !declaration.ordered {
        // Identifiers are escape-free ASCII, so JCS key order is byte order.
        require(
            declaration
                .members
                .windows(2)
                .all(|pair| matches!(pair, [left, right] if left < right)),
        )?;
    }
    Ok(())
}

fn validate_enum_member(
    node: &CheckedSemanticNodeV2,
    member: &EnumMemberPreimage,
    graph: &NominalGraph<'_>,
) -> Result<(), ValidationFailure> {
    require(is_identifier(&member.case))?;
    let Some(NominalIdentityPreimage::EnumDeclaration(declaration)) =
        graph.preimage(&member.declaration_node_id)
    else {
        return Err(invalid());
    };
    require(declaration.members.contains(&member.case))?;
    require(node.semantic_type == member.declaration_node_id)?;
    require(node.dependencies.as_slice() == std::slice::from_ref(&member.declaration_node_id))?;
    require(
        node.body
            == json!({"term": "literal", "value_kind": "enum", "value": member.case.as_ref()}),
    )
}

fn validate_dimension(
    node: &CheckedSemanticNodeV2,
    dimension: &DimensionPreimage,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    validate_owner(&dimension.owner, lock)?;
    validate_qualified_name(&dimension.qualified_declaration)?;
    require(node.semantic_type == node.node_id)?;
    let mut keys = Vec::with_capacity(dimension.terms.len());
    let mut bases = BTreeSet::new();
    for term in &dimension.terms {
        meter.charge(1)?;
        let magnitude = integer_magnitude(&term.exponent).ok_or_else(invalid)?;
        require(magnitude != "0")?;
        require(bases.insert(&term.dimension_node_id))?;
        let Some(NominalIdentityPreimage::Dimension(base)) =
            graph.preimage(&term.dimension_node_id)
        else {
            return Err(invalid());
        };
        require(base.terms.is_empty())?;
        keys.push(serde_json::to_vec(term).map_err(|_| invalid())?);
    }
    require(
        keys.windows(2)
            .all(|pair| matches!(pair, [left, right] if left < right)),
    )?;
    let mut expected = dimension
        .terms
        .iter()
        .map(|term| &term.dimension_node_id)
        .collect::<Vec<_>>();
    let mut actual = node.dependencies.iter().collect::<Vec<_>>();
    expected.sort();
    actual.sort();
    require(expected == actual)
}

fn validate_unit(
    node: &CheckedSemanticNodeV2,
    unit: &UnitPreimage,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    validate_owner(&unit.owner, lock)?;
    validate_qualified_name(&unit.qualified_declaration)?;
    require(matches!(
        graph.preimage(&unit.dimension_node_id),
        Some(NominalIdentityPreimage::Dimension(_))
    ))?;
    validate_rational(&unit.scale, meter)?;
    validate_rational(&unit.offset, meter)?;
    require(node.semantic_type == unit.dimension_node_id)?;
    let mut expected = vec![&unit.dimension_node_id];
    match &unit.target_unit_node_id {
        None => {
            require(is_rational(&unit.scale, "1", "1") && is_rational(&unit.offset, "0", "1"))?;
        }
        Some(target) => {
            require(unit.scale.numerator.as_ref() != "0")?;
            expected.push(target);
            validate_unit_path(node, unit, graph, meter)?;
        }
    }
    let mut actual = node.dependencies.iter().collect::<Vec<_>>();
    expected.sort();
    actual.sort();
    require(expected == actual)
}

fn is_rational(rational: &CheckedRational, numerator: &str, denominator: &str) -> bool {
    rational.numerator.as_ref() == numerator && rational.denominator.as_ref() == denominator
}

/// Follows targets to a root; every hop is a same-dimension unit, acyclic.
fn validate_unit_path(
    node: &CheckedSemanticNodeV2,
    unit: &UnitPreimage,
    graph: &NominalGraph<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut visited = BTreeSet::from([&node.node_id]);
    let mut next = unit.target_unit_node_id.as_ref();
    while let Some(target) = next {
        meter.charge(1)?;
        require(visited.insert(target))?;
        let Some(NominalIdentityPreimage::Unit(target_unit)) = graph.preimage(target) else {
            return Err(invalid());
        };
        require(target_unit.dimension_node_id == unit.dimension_node_id)?;
        next = target_unit.target_unit_node_id.as_ref();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tracing: TC-048, FR-038-AC-5
    #[test]
    fn tc_048_integer_and_identifier_grammars_are_exact() {
        assert_eq!(integer_magnitude("0"), Some("0"));
        assert_eq!(integer_magnitude("-12"), Some("12"));
        assert_eq!(integer_magnitude("-0"), None);
        assert_eq!(integer_magnitude("012"), None);
        assert_eq!(integer_magnitude(""), None);
        assert!(!positive_integer("0"));
        assert!(is_identifier("_a1"));
        assert!(!is_identifier("1a"));
        assert!(!is_identifier(""));
    }
}
