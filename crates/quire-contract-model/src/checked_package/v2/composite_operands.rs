// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038: metered, package-authored domains of structural equality operands.

use super::operation_catalog::operation_catalog;
use super::structural::{
    aggregate_members, binding, checked_node_identity, is_integer_bound, is_non_negative_integer,
    reference_target,
};
use super::{
    BoundedDomainForm, CheckedCollectionKind, CheckedNodeId, CheckedNodeKind, CheckedOccurrence,
    CheckedPackageV2, CheckedScalarOperandChild, CheckedSemanticNodeV2, CompositeTypeForm,
    NominalIdentityPreimage, ScalarTypeForm, ValueForm,
};
#[cfg(test)]
use crate::checked_package::common::NODE_DOMAIN;
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;

/// The authentic operands of one selected application occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedCompositeOperands {
    /// The selected application node.
    pub application: CheckedNodeId,
    /// The selected occurrence belonging to that node.
    pub occurrence: CheckedOccurrence,
    /// One entry per argument, without deduplication.
    pub operands: Vec<CheckedCompositeOperand>,
    /// The exact logical work consumed by this projection.
    pub consumed_work: u64,
}

/// One authentic positional operand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedCompositeOperand {
    /// Zero-based argument position.
    pub ordinal: u64,
    /// The original graph child identity.
    pub child: CheckedScalarOperandChild,
    /// The operand's admitted semantic type.
    pub semantic_type: CheckedNodeId,
    /// Closed literal or authored parameter domain.
    pub domain: CheckedCompositeOperandDomain,
}

/// The authored domain of an operand, without harness draw values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedCompositeOperandDomain {
    /// A finite closed graph value; no free introduced positions.
    Literal,
    /// A parameter's retained type structure and introduced positions.
    Parameter {
        /// Type nodes in depth-first child order.
        shape: Vec<CheckedCompositeShapeEntry>,
        /// Unique introduced positions, rooted at the parameter.
        positions: Vec<CheckedCompositeDomainPosition>,
    },
}

/// One type node at one authentic child path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedCompositeShapeEntry {
    /// Original source type node.
    pub type_node: CheckedNodeId,
    /// Closed source kind.
    pub kind: CheckedNodeKind,
    /// Child-index path from the parameter root.
    pub path: Vec<u32>,
    /// Original child edges in source order.
    pub edges: Vec<CheckedCompositeChildEdge>,
}

/// One declared child edge, including path-neutral forwarding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedCompositeChildEdge {
    /// Original target type identity.
    pub target: CheckedNodeId,
    /// Field/tuple/inner position; absent only for forwarding.
    pub ordinal: Option<u32>,
    /// Record-field name, absent on all other edges.
    pub name: Option<Box<str>>,
    /// Whether the record field uses the optional-presence wrapper.
    pub optional_presence: bool,
}

/// The key and authored descriptor of one introduced position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedCompositeDomainPosition {
    /// Parameter-rooted child path.
    pub key: CheckedCompositeDomainKey,
    /// Original descriptor source type.
    pub type_node: CheckedNodeId,
    /// Authored constraints, never harness substitutions.
    pub authored: CheckedAuthoredCompositeDomain,
}

/// A position introduced by a package-authored operand type.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CheckedCompositeDomainKey {
    /// The original parameter node and its child-index path.
    Node {
        /// Original parameter identity.
        node: CheckedNodeId,
        /// Authentic child-index path.
        path: Vec<u32>,
    },
}

/// An admitted canonical decimal endpoint, without fixed-width narrowing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedCanonicalIntegerBound(Box<str>);

impl CheckedCanonicalIntegerBound {
    /// Returns the exact admitted canonical decimal spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The closed authored descriptor vocabulary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedAuthoredCompositeDomain {
    /// Exact authored inclusive integer endpoints.
    IntegerRange {
        /// Inclusive lower endpoint.
        lower: CheckedCanonicalIntegerBound,
        /// Inclusive upper endpoint.
        upper: CheckedCanonicalIntegerBound,
    },
    /// Integer with no authored finite range.
    UnboundedInteger,
    /// Exact authored collection cardinality endpoints.
    Collection {
        /// Original collection form.
        kind: CheckedCollectionKind,
        /// Inclusive minimum cardinality.
        minimum: CheckedCanonicalIntegerBound,
        /// Inclusive maximum cardinality.
        maximum: CheckedCanonicalIntegerBound,
    },
    /// Collection with no authored finite cardinality maximum.
    UnboundedCollection {
        /// Original collection form.
        kind: CheckedCollectionKind,
    },
    /// Original nominal enum order and identifiers.
    Enum {
        /// Whether preimage order is semantically ordered.
        ordered: bool,
        /// Admitted preimage order, byte-ascending when unordered.
        members: Vec<Box<str>>,
    },
    /// Recursion at a record/tuple's first-entry path.
    UnboundedDepth {
        /// Actual recursive declaration node.
        declaration: CheckedNodeId,
        /// All distinct actual reentry paths, lexicographically sorted.
        reentry_paths: Vec<Vec<u32>>,
    },
    /// An authored leaf whose downstream coverage is owned by its consumer.
    Whole {
        /// Original source kind, including bounded wrappers.
        kind: CheckedNodeKind,
    },
}

/// The exact unsupported operand condition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedUnsupportedCompositeOperand {
    /// Defensive integer-inline case excluded by catalog admission.
    InlineInteger,
    /// Any other top-level inline literal, including none.
    InlineNonInteger,
    /// An application subterm rather than a parameter or closed graph value.
    ApplicationSubterm,
    /// A semantic type outside the structural-kind family.
    NonStructuralType,
    /// A graph subtree containing a free parameter, application or value cycle.
    NonliteralGraphValue,
}

/// Why the authored projection cannot return a complete result.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CheckedCompositeOperandError {
    /// Requested application identity is absent.
    #[error("unknown checked node")]
    UnknownNode { node: CheckedNodeId },
    /// Requested node has no application term.
    #[error("checked node is not an application")]
    NotApplication { node: CheckedNodeId },
    /// Supplied occurrence is absent from that application.
    #[error("application occurrence is absent")]
    MissingOccurrence {
        application: CheckedNodeId,
        occurrence: CheckedOccurrence,
    },
    /// Catalog identity is missing or unknown.
    #[error("unknown application operation")]
    UnknownOperator { application: CheckedNodeId },
    /// Catalog operation is not structural.eq.
    #[error("ineligible application operation")]
    IneligibleOperator { application: CheckedNodeId },
    /// Valid typed target names an absent graph value.
    #[error("referenced operand child is absent")]
    MissingChild { ordinal: u64, child: CheckedNodeId },
    /// Target is absent or cannot decode as a checked identity.
    #[error("operand reference target is malformed")]
    MalformedChild { ordinal: u64 },
    /// The exact unsupported operand and its available type locus.
    #[error("unsupported composite operand")]
    UnsupportedOperand {
        ordinal: u64,
        type_node: Option<CheckedNodeId>,
        reason: CheckedUnsupportedCompositeOperand,
    },
    /// Missing/malformed type structure or unanchored cycle.
    #[error("malformed authored domain")]
    MalformedDomain {
        ordinal: u64,
        type_node: Option<CheckedNodeId>,
    },
    /// Union payload projection is outside this accessor.
    #[error("unsupported authored domain")]
    UnsupportedDomain {
        ordinal: u64,
        type_node: CheckedNodeId,
    },
    /// Actual positional width conversion failed.
    #[error("composite position cannot fit its wire width")]
    PositionOutOfRange {
        ordinal: Option<u64>,
        type_node: Option<CheckedNodeId>,
    },
    /// The first unpayable attempted charge, including overflow.
    #[error("composite accessor work limit exhausted")]
    WorkLimit { limit: u64, consumed: u64 },
}

type Error = CheckedCompositeOperandError;
type Result<T> = std::result::Result<T, Error>;

struct Meter {
    limit: u64,
    consumed: u64,
}

impl Meter {
    fn charge(&mut self, amount: u64) -> Result<()> {
        let next = self.consumed.checked_add(amount);
        self.consumed = next.unwrap_or(u64::MAX);
        if next.is_none() || self.consumed > self.limit {
            Err(Error::WorkLimit {
                limit: self.limit,
                consumed: self.consumed,
            })
        } else {
            Ok(())
        }
    }

    fn length(&mut self, length: usize) -> Result<()> {
        let amount = u64::try_from(length).map_err(|_| Error::WorkLimit {
            limit: self.limit,
            consumed: u64::MAX,
        })?;
        self.charge(amount)
    }

    fn path(&mut self, path: &[u32]) -> Result<Vec<u32>> {
        self.length(path.len())?;
        Ok(path.to_vec())
    }

    fn text(&mut self, text: &str) -> Result<Box<str>> {
        self.length(text.len())?;
        Ok(text.into())
    }
}

struct Graph<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    index: &'a BTreeMap<CheckedNodeId, usize>,
}

impl<'a> Graph<'a> {
    fn node(
        &self,
        id: &CheckedNodeId,
        meter: &mut Meter,
    ) -> Result<Option<&'a CheckedSemanticNodeV2>> {
        meter.charge(1)?;
        Ok(self
            .index
            .get(id)
            .and_then(|position| self.nodes.get(*position)))
    }
    fn kind(&self, node: &CheckedSemanticNodeV2) -> Option<CheckedNodeKind> {
        self.index
            .get(&node.node_id)
            .and_then(|position| self.kinds.get(*position))
            .copied()
    }
}

fn target(term: &Value) -> Option<CheckedNodeId> {
    reference_target(term)
}

fn type_error(ordinal: u64, node: Option<CheckedNodeId>) -> Error {
    Error::MalformedDomain {
        ordinal,
        type_node: node,
    }
}

fn unsupported(
    ordinal: u64,
    node: Option<CheckedNodeId>,
    reason: CheckedUnsupportedCompositeOperand,
) -> Error {
    Error::UnsupportedOperand {
        ordinal,
        type_node: node,
        reason,
    }
}

fn collection(kind: CheckedNodeKind) -> Option<CheckedCollectionKind> {
    match kind {
        CheckedNodeKind::CompositeType(CompositeTypeForm::Sequence) => {
            Some(CheckedCollectionKind::Sequence)
        }
        CheckedNodeKind::CompositeType(CompositeTypeForm::Set) => Some(CheckedCollectionKind::Set),
        CheckedNodeKind::CompositeType(CompositeTypeForm::Bag) => Some(CheckedCollectionKind::Bag),
        CheckedNodeKind::CompositeType(CompositeTypeForm::OrderedSet) => {
            Some(CheckedCollectionKind::OrderedSet)
        }
        _ => None,
    }
}

impl CheckedPackageV2 {
    /// Returns authored domains of `structural.eq` operands in argument order.
    ///
    /// The supplied occurrence must belong to the application. The finite work
    /// limit bounds logical graph visits, retained paths and copied text. No
    /// partial result or harness-bound substitution is returned on refusal.
    pub fn composite_application_operands(
        &self,
        application: &CheckedNodeId,
        occurrence: &CheckedOccurrence,
        work_limit: u64,
    ) -> Result<CheckedCompositeOperands> {
        let graph = Graph {
            nodes: &self.wire.semantic_graph.nodes,
            kinds: &self.kinds,
            index: &self.node_index,
        };
        let mut meter = Meter {
            limit: work_limit,
            consumed: 0,
        };
        let node = graph
            .node(application, &mut meter)?
            .ok_or_else(|| Error::UnknownNode {
                node: application.clone(),
            })?;
        if node.body.get("term").and_then(Value::as_str) != Some("application") {
            return Err(Error::NotApplication {
                node: application.clone(),
            });
        }
        if !node.occurrences.contains(occurrence) {
            return Err(Error::MissingOccurrence {
                application: application.clone(),
                occurrence: occurrence.clone(),
            });
        }
        let identity = node
            .body
            .pointer("/operation/identity")
            .and_then(Value::as_str)
            .filter(|id| operation_catalog().entry(id).is_some())
            .ok_or_else(|| Error::UnknownOperator {
                application: application.clone(),
            })?;
        if identity != "quire.op.structural.eq" {
            return Err(Error::IneligibleOperator {
                application: application.clone(),
            });
        }
        let arguments = node
            .body
            .get("arguments")
            .and_then(Value::as_array)
            .ok_or_else(|| Error::NotApplication {
                node: application.clone(),
            })?;
        let mut operands = Vec::new();
        for (index, argument) in arguments.iter().enumerate() {
            let ordinal = argument_ordinal(index)?;
            meter.charge(1)?;
            let term = argument.get("term").and_then(Value::as_str);
            if term == Some("literal") {
                let reason =
                    if argument.get("value_kind").and_then(Value::as_str) == Some("integer") {
                        CheckedUnsupportedCompositeOperand::InlineInteger
                    } else {
                        CheckedUnsupportedCompositeOperand::InlineNonInteger
                    };
                return Err(unsupported(ordinal, typed_member(argument, "type"), reason));
            }
            if term == Some("application") {
                return Err(unsupported(
                    ordinal,
                    typed_member(argument, "result_type"),
                    CheckedUnsupportedCompositeOperand::ApplicationSubterm,
                ));
            }
            let id = target(argument).ok_or(Error::MalformedChild { ordinal })?;
            let child = graph
                .node(&id, &mut meter)?
                .ok_or_else(|| Error::MissingChild {
                    ordinal,
                    child: id.clone(),
                })?;
            if child.body.get("term").and_then(Value::as_str) == Some("application") {
                return Err(unsupported(
                    ordinal,
                    typed_member(&child.body, "result_type"),
                    CheckedUnsupportedCompositeOperand::ApplicationSubterm,
                ));
            }
            let family =
                super::operations::resolve_family_with(
                    &child.semantic_type,
                    |visit| match visit {
                        super::operations::FamilyVisit::Node(id) => {
                            graph.node(id, &mut meter).map(|node| {
                                node.and_then(|node| graph.kind(node).map(|kind| (node, kind)))
                            })
                        }
                        super::operations::FamilyVisit::Forward => {
                            meter.charge(1)?;
                            Ok(None)
                        }
                    },
                )?
                .ok_or_else(|| type_error(ordinal, Some(child.semantic_type.clone())))?;
            if !operation_catalog().family_fits(family, "structural_kind") {
                return Err(unsupported(
                    ordinal,
                    Some(child.semantic_type.clone()),
                    CheckedUnsupportedCompositeOperand::NonStructuralType,
                ));
            }
            let domain = if graph.kind(child) == Some(CheckedNodeKind::Value(ValueForm::Parameter))
            {
                let (shape, positions) =
                    project_type(&graph, &id, &child.semantic_type, ordinal, &mut meter)?;
                CheckedCompositeOperandDomain::Parameter { shape, positions }
            } else {
                closed_literal(&graph, child, ordinal, &mut meter)?;
                CheckedCompositeOperandDomain::Literal
            };
            operands.push(CheckedCompositeOperand {
                ordinal,
                child: CheckedScalarOperandChild::GraphChild(id),
                semantic_type: child.semantic_type.clone(),
                domain,
            });
        }
        Ok(CheckedCompositeOperands {
            application: application.clone(),
            occurrence: occurrence.clone(),
            operands,
            consumed_work: meter.consumed,
        })
    }
}

fn typed_member(value: &Value, member: &str) -> Option<CheckedNodeId> {
    checked_node_identity(value.get(member)?)
}

fn argument_ordinal(index: usize) -> Result<u64> {
    u64::try_from(index).map_err(|_| Error::PositionOutOfRange {
        ordinal: None,
        type_node: None,
    })
}

fn child_ordinal(index: usize, ordinal: u64, node: &CheckedNodeId) -> Result<u32> {
    u32::try_from(index).map_err(|_| Error::PositionOutOfRange {
        ordinal: Some(ordinal),
        type_node: Some(node.clone()),
    })
}

fn forwarding_target(node: &CheckedSemanticNodeV2, kind: CheckedNodeKind) -> Option<CheckedNodeId> {
    match kind {
        CheckedNodeKind::CompositeType(CompositeTypeForm::Alias) => {
            let [inner] = aggregate_members(&node.body)? else {
                return None;
            };
            target(inner)
        }
        CheckedNodeKind::BoundedDomain(form) if form != BoundedDomainForm::ModelPopulation => {
            Some(node.semantic_type.clone())
        }
        _ => None,
    }
}

fn descriptor_kind(
    graph: &Graph<'_>,
    id: &CheckedNodeId,
    ordinal: u64,
    meter: &mut Meter,
) -> Result<CheckedNodeKind> {
    let mut current = id.clone();
    let mut seen = Vec::new();
    loop {
        let node = graph
            .node(&current, meter)?
            .ok_or_else(|| type_error(ordinal, Some(current.clone())))?;
        let kind = graph
            .kind(node)
            .ok_or_else(|| type_error(ordinal, Some(current.clone())))?;
        if seen.contains(&current) {
            return Err(type_error(ordinal, Some(current)));
        }
        if let Some(next) = forwarding_target(node, kind) {
            meter.charge(1)?;
            seen.push(current);
            current = next;
        } else if matches!(
            kind,
            CheckedNodeKind::CompositeType(CompositeTypeForm::Alias)
        ) {
            return Err(type_error(ordinal, Some(current)));
        } else {
            return Ok(kind);
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Suppression {
    None,
    Integer,
    Collection,
    Whole,
}

struct TypeFrame<'a> {
    node: &'a CheckedSemanticNodeV2,
    kind: CheckedNodeKind,
    path: Vec<u32>,
    suppression: Suppression,
    next: usize,
    shape: usize,
}

fn anchor(kind: CheckedNodeKind) -> bool {
    matches!(
        kind,
        CheckedNodeKind::CompositeType(CompositeTypeForm::Record | CompositeTypeForm::Tuple)
    )
}

fn position(
    positions: &mut Vec<CheckedCompositeDomainPosition>,
    root: &CheckedNodeId,
    source: &CheckedNodeId,
    path: &[u32],
    authored: CheckedAuthoredCompositeDomain,
    meter: &mut Meter,
) -> Result<()> {
    let path = meter.path(path)?;
    positions.push(CheckedCompositeDomainPosition {
        key: CheckedCompositeDomainKey::Node {
            node: root.clone(),
            path,
        },
        type_node: source.clone(),
        authored,
    });
    Ok(())
}

fn depth_position(
    positions: &mut Vec<CheckedCompositeDomainPosition>,
    root: &CheckedNodeId,
    declaration: &CheckedNodeId,
    entered: &[u32],
    reentry: &[u32],
    ordinal: u64,
    meter: &mut Meter,
) -> Result<()> {
    meter.charge(1)?;
    if let Some(found) = positions.iter_mut().find(|entry| {
        matches!(&entry.key, CheckedCompositeDomainKey::Node { node, path } if node == root && path == entered)
    }) {
        let CheckedAuthoredCompositeDomain::UnboundedDepth { reentry_paths, .. } = &mut found.authored else {
            return Err(Error::MalformedDomain { ordinal, type_node: Some(declaration.clone()) });
        };
        if !reentry_paths.iter().any(|path| path == reentry) {
            reentry_paths.push(meter.path(reentry)?);
            reentry_paths.sort();
        }
    } else {
        let entered = meter.path(entered)?;
        let reentry = meter.path(reentry)?;
        positions.push(CheckedCompositeDomainPosition {
            key: CheckedCompositeDomainKey::Node { node: root.clone(), path: entered },
            type_node: declaration.clone(), authored: CheckedAuthoredCompositeDomain::UnboundedDepth {
                declaration: declaration.clone(), reentry_paths: vec![reentry],
            },
        });
    }
    Ok(())
}

fn bound(
    node: &CheckedSemanticNodeV2,
    name: &str,
    ordinal: u64,
    natural: bool,
    meter: &mut Meter,
) -> Result<CheckedCanonicalIntegerBound> {
    let malformed = || type_error(ordinal, Some(node.node_id.clone()));
    let term = aggregate_members(&node.body)
        .and_then(|members| members.iter().find_map(|m| binding(m, name)))
        .ok_or_else(malformed)?;
    let text = term
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(malformed)?;
    meter.length(text.len())?;
    let valid = if natural {
        is_non_negative_integer(text)
    } else {
        is_integer_bound(text)
    };
    if term.get("term").and_then(Value::as_str) != Some("literal")
        || term.get("value_kind").and_then(Value::as_str) != Some("integer")
        || !valid
    {
        return Err(malformed());
    }
    Ok(CheckedCanonicalIntegerBound(text.into()))
}

fn descriptor(
    graph: &Graph<'_>,
    frame: &TypeFrame<'_>,
    ordinal: u64,
    meter: &mut Meter,
) -> Result<(Option<CheckedAuthoredCompositeDomain>, Suppression)> {
    use CheckedAuthoredCompositeDomain as D;
    use CheckedNodeKind as K;
    let node = frame.node;
    let inherited = frame.suppression;
    let malformed = || type_error(ordinal, Some(node.node_id.clone()));
    let suppress = |required| {
        if inherited == Suppression::None {
            Ok(false)
        } else if inherited == required {
            Ok(true)
        } else {
            Err(malformed())
        }
    };
    if matches!(
        frame.kind,
        K::ScalarType(_)
            | K::BoundedDomain(_)
            | K::CompositeType(
                CompositeTypeForm::Reference
                    | CompositeTypeForm::Sequence
                    | CompositeTypeForm::Set
                    | CompositeTypeForm::Bag
                    | CompositeTypeForm::OrderedSet
            )
    ) && frame.kind != K::ScalarType(ScalarTypeForm::Boolean)
        && inherited == Suppression::None
    {
        meter.charge(1)?;
    }
    let value = match frame.kind {
        K::ScalarType(ScalarTypeForm::Boolean) => (None, Suppression::None),
        K::ScalarType(ScalarTypeForm::Integer) => (
            (!suppress(Suppression::Integer)?).then_some(D::UnboundedInteger),
            inherited,
        ),
        K::ScalarType(ScalarTypeForm::Enum) => {
            if inherited != Suppression::None {
                return Err(malformed());
            }
            let Some(NominalIdentityPreimage::EnumDeclaration(preimage)) =
                &node.nominal_identity_preimage
            else {
                return Err(malformed());
            };
            let members = preimage
                .members
                .iter()
                .map(|member| meter.text(member))
                .collect::<Result<Vec<_>>>()?;
            (
                Some(D::Enum {
                    ordered: preimage.ordered,
                    members,
                }),
                Suppression::None,
            )
        }
        K::ScalarType(
            ScalarTypeForm::Rational
            | ScalarTypeForm::Decimal
            | ScalarTypeForm::Float32
            | ScalarTypeForm::Float64
            | ScalarTypeForm::Text
            | ScalarTypeForm::Dimension
            | ScalarTypeForm::Unit
            | ScalarTypeForm::CompoundUnit,
        ) => (
            (!suppress(Suppression::Whole)?).then_some(D::Whole { kind: frame.kind }),
            inherited,
        ),
        K::BoundedDomain(BoundedDomainForm::IntegerRange) => {
            let source = descriptor_kind(graph, &node.semantic_type, ordinal, meter)?;
            if source != K::ScalarType(ScalarTypeForm::Integer) {
                return Err(malformed());
            }
            if suppress(Suppression::Integer)? {
                (None, inherited)
            } else {
                (
                    Some(D::IntegerRange {
                        lower: bound(node, "min", ordinal, false, meter)?,
                        upper: bound(node, "max", ordinal, false, meter)?,
                    }),
                    Suppression::Integer,
                )
            }
        }
        K::BoundedDomain(BoundedDomainForm::CollectionBounds) => {
            let source = descriptor_kind(graph, &node.semantic_type, ordinal, meter)?;
            let kind = collection(source).ok_or_else(malformed)?;
            if suppress(Suppression::Collection)? {
                (None, inherited)
            } else {
                (
                    Some(D::Collection {
                        kind,
                        minimum: bound(node, "min", ordinal, true, meter)?,
                        maximum: bound(node, "max", ordinal, true, meter)?,
                    }),
                    Suppression::Collection,
                )
            }
        }
        K::BoundedDomain(
            BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds,
        ) => {
            let source = descriptor_kind(graph, &node.semantic_type, ordinal, meter)?;
            let valid = matches!(
                (frame.kind, source),
                (
                    K::BoundedDomain(BoundedDomainForm::RationalRange),
                    K::ScalarType(ScalarTypeForm::Rational)
                ) | (
                    K::BoundedDomain(BoundedDomainForm::DecimalRange),
                    K::ScalarType(ScalarTypeForm::Decimal)
                ) | (
                    K::BoundedDomain(BoundedDomainForm::FloatRounding),
                    K::ScalarType(ScalarTypeForm::Float32 | ScalarTypeForm::Float64)
                ) | (
                    K::BoundedDomain(BoundedDomainForm::TextBounds),
                    K::ScalarType(ScalarTypeForm::Text)
                )
            );
            if !valid {
                return Err(malformed());
            }
            (
                (!suppress(Suppression::Whole)?).then_some(D::Whole { kind: frame.kind }),
                Suppression::Whole,
            )
        }
        K::BoundedDomain(BoundedDomainForm::ModelPopulation) => (
            (!suppress(Suppression::Whole)?).then_some(D::Whole { kind: frame.kind }),
            inherited,
        ),
        K::CompositeType(
            CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet,
        ) => {
            let kind = collection(frame.kind).ok_or_else(malformed)?;
            (
                (!suppress(Suppression::Collection)?).then_some(D::UnboundedCollection { kind }),
                inherited,
            )
        }
        K::CompositeType(CompositeTypeForm::Reference) => (
            (!suppress(Suppression::Whole)?).then_some(D::Whole { kind: frame.kind }),
            inherited,
        ),
        K::CompositeType(CompositeTypeForm::Union) => {
            return Err(Error::UnsupportedDomain {
                ordinal,
                type_node: node.node_id.clone(),
            });
        }
        K::CompositeType(CompositeTypeForm::Alias) => (None, inherited),
        K::CompositeType(
            CompositeTypeForm::Record | CompositeTypeForm::Tuple | CompositeTypeForm::Option,
        ) => {
            if inherited != Suppression::None {
                return Err(malformed());
            }
            (None, inherited)
        }
        _ => return Err(malformed()),
    };
    Ok(value)
}

fn next_edge(
    frame: &mut TypeFrame<'_>,
    ordinal: u64,
    meter: &mut Meter,
) -> Result<Option<CheckedCompositeChildEdge>> {
    use CheckedNodeKind as K;
    let node = frame.node;
    let malformed = || type_error(ordinal, Some(node.node_id.clone()));
    let index = frame.next;
    let indexed = match frame.kind {
        K::CompositeType(
            CompositeTypeForm::Record
            | CompositeTypeForm::Tuple
            | CompositeTypeForm::Option
            | CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet,
        ) => true,
        K::CompositeType(CompositeTypeForm::Alias) => false,
        K::BoundedDomain(
            BoundedDomainForm::IntegerRange
            | BoundedDomainForm::CollectionBounds
            | BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds,
        ) => false,
        _ => return Ok(None),
    };
    let members = if indexed || frame.kind == K::CompositeType(CompositeTypeForm::Alias) {
        Some(aggregate_members(&node.body).ok_or_else(malformed)?)
    } else {
        None
    };
    let many = matches!(
        frame.kind,
        K::CompositeType(CompositeTypeForm::Record | CompositeTypeForm::Tuple)
    );
    if many && members.is_some_and(|m| index >= m.len()) || !many && index != 0 {
        return Ok(None);
    }
    meter.charge(1)?;
    let edge_ordinal = indexed
        .then(|| child_ordinal(index, ordinal, &node.node_id))
        .transpose()?;
    let (target, name, optional_presence) = match frame.kind {
        K::CompositeType(CompositeTypeForm::Record) => {
            let member = members.and_then(|m| m.get(index)).ok_or_else(malformed)?;
            let name = member
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(malformed)?;
            let name = meter.text(name)?;
            let value = member.get("value").ok_or_else(malformed)?;
            if member.get("term").and_then(Value::as_str) != Some("binding") {
                return Err(malformed());
            }
            if value.get("term").and_then(Value::as_str) == Some("aggregate") {
                let [optional] = aggregate_members(value).ok_or_else(malformed)? else {
                    return Err(malformed());
                };
                let inner = binding(optional, "optional").ok_or_else(malformed)?;
                let target = target(inner).ok_or_else(malformed)?;
                (target, Some(name), true)
            } else {
                (target(value).ok_or_else(malformed)?, Some(name), false)
            }
        }
        K::CompositeType(_) => {
            let members = members.ok_or_else(malformed)?;
            if !many && members.len() != 1 {
                return Err(malformed());
            }
            (
                target(members.get(index).ok_or_else(malformed)?).ok_or_else(malformed)?,
                None,
                false,
            )
        }
        K::BoundedDomain(_) => (node.semantic_type.clone(), None, false),
        _ => return Err(malformed()),
    };
    frame.next = index
        .checked_add(1)
        .ok_or_else(|| Error::PositionOutOfRange {
            ordinal: Some(ordinal),
            type_node: Some(node.node_id.clone()),
        })?;
    Ok(Some(CheckedCompositeChildEdge {
        target,
        ordinal: edge_ordinal,
        name,
        optional_presence,
    }))
}

fn project_type(
    graph: &Graph<'_>,
    root: &CheckedNodeId,
    ty: &CheckedNodeId,
    ordinal: u64,
    meter: &mut Meter,
) -> Result<(
    Vec<CheckedCompositeShapeEntry>,
    Vec<CheckedCompositeDomainPosition>,
)> {
    let mut shape = Vec::new();
    let mut positions = Vec::new();
    let mut stack: Vec<TypeFrame<'_>> = Vec::new();
    let mut pending = Some((ty.clone(), Vec::new(), Suppression::None, false));
    loop {
        if let Some((id, path, suppression, optional_presence)) = pending.take() {
            let node = graph
                .node(&id, meter)?
                .ok_or_else(|| type_error(ordinal, Some(id.clone())))?;
            meter.charge(1)?;
            let kind = graph
                .kind(node)
                .ok_or_else(|| type_error(ordinal, Some(id.clone())))?;
            if optional_presence
                && kind != CheckedNodeKind::CompositeType(CompositeTypeForm::Option)
            {
                return Err(type_error(ordinal, Some(id)));
            }
            let shape_index = shape.len();
            shape.push(CheckedCompositeShapeEntry {
                type_node: id.clone(),
                kind,
                path: meter.path(&path)?,
                edges: Vec::new(),
            });
            if let Some(reentered) = stack.iter().position(|frame| frame.node.node_id == id) {
                let original = stack
                    .get(reentered)
                    .ok_or_else(|| type_error(ordinal, Some(id.clone())))?;
                if anchor(kind) {
                    depth_position(
                        &mut positions,
                        root,
                        &id,
                        &original.path,
                        &path,
                        ordinal,
                        meter,
                    )?;
                    continue;
                }
                if !stack.iter().skip(reentered).any(|frame| anchor(frame.kind)) {
                    return Err(type_error(ordinal, Some(id)));
                }
            }
            let mut frame = TypeFrame {
                node,
                kind,
                path,
                suppression,
                next: 0,
                shape: shape_index,
            };
            let (authored, suppression) = descriptor(graph, &frame, ordinal, meter)?;
            frame.suppression = suppression;
            if let Some(authored) = authored {
                position(&mut positions, root, &id, &frame.path, authored, meter)?;
            }
            stack.push(frame);
        }
        let Some(frame) = stack.last_mut() else {
            break;
        };
        match next_edge(frame, ordinal, meter)? {
            Some(edge) => {
                let mut path = frame.path.clone();
                let suppression = if let Some(index) = edge.ordinal {
                    path.push(index);
                    Suppression::None
                } else {
                    frame.suppression
                };
                let id = edge.target.clone();
                let optional_presence = edge.optional_presence;
                shape
                    .get_mut(frame.shape)
                    .ok_or_else(|| type_error(ordinal, Some(frame.node.node_id.clone())))?
                    .edges
                    .push(edge);
                pending = Some((id, path, suppression, optional_presence));
            }
            None => {
                stack.pop();
            }
        }
    }
    Ok((shape, positions))
}

fn closed_literal(
    graph: &Graph<'_>,
    root: &CheckedSemanticNodeV2,
    ordinal: u64,
    meter: &mut Meter,
) -> Result<()> {
    if !matches!(
        graph.kind(root),
        Some(CheckedNodeKind::Value(
            ValueForm::RecordValue
                | ValueForm::TupleValue
                | ValueForm::CollectionValue
                | ValueForm::OptionValue
                | ValueForm::UnionValue
        ))
    ) {
        return Err(unsupported(
            ordinal,
            Some(root.semantic_type.clone()),
            CheckedUnsupportedCompositeOperand::NonliteralGraphValue,
        ));
    }
    enum Step<'a> {
        Node(CheckedNodeId),
        Term(&'a Value, &'a CheckedSemanticNodeV2),
        Members(std::slice::Iter<'a, Value>, &'a CheckedSemanticNodeV2),
        Leave,
    }
    let mut stack = vec![Step::Node(root.node_id.clone())];
    let mut ancestors: Vec<&CheckedNodeId> = Vec::new();
    while let Some(step) = stack.pop() {
        match step {
            Step::Node(id) => {
                let node = graph.node(&id, meter)?.ok_or_else(|| Error::MissingChild {
                    ordinal,
                    child: id.clone(),
                })?;
                meter.charge(1)?;
                let nonliteral = || {
                    unsupported(
                        ordinal,
                        Some(node.semantic_type.clone()),
                        CheckedUnsupportedCompositeOperand::NonliteralGraphValue,
                    )
                };
                if ancestors.contains(&&node.node_id) {
                    return Err(nonliteral());
                }
                match graph.kind(node) {
                    Some(CheckedNodeKind::Value(ValueForm::UnionValue)) => {
                        let ty = graph
                            .node(&node.semantic_type, meter)?
                            .ok_or_else(|| type_error(ordinal, Some(node.semantic_type.clone())))?;
                        if graph.kind(ty)
                            != Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Union))
                        {
                            return Err(type_error(ordinal, Some(node.semantic_type.clone())));
                        }
                        return Err(Error::UnsupportedDomain {
                            ordinal,
                            type_node: node.semantic_type.clone(),
                        });
                    }
                    Some(CheckedNodeKind::Value(
                        ValueForm::Literal
                        | ValueForm::EnumValue
                        | ValueForm::RecordValue
                        | ValueForm::TupleValue
                        | ValueForm::CollectionValue
                        | ValueForm::OptionValue,
                    )) => {}
                    _ => return Err(nonliteral()),
                }
                ancestors.push(&node.node_id);
                stack.push(Step::Leave);
                stack.push(Step::Term(&node.body, node));
            }
            Step::Term(term, owner) => {
                let nonliteral = || {
                    unsupported(
                        ordinal,
                        Some(owner.semantic_type.clone()),
                        CheckedUnsupportedCompositeOperand::NonliteralGraphValue,
                    )
                };
                match term.get("term").and_then(Value::as_str) {
                    Some("literal") => {}
                    Some("reference") => {
                        meter.charge(1)?;
                        let id = target(term).ok_or(Error::MalformedChild { ordinal })?;
                        stack.push(Step::Node(id));
                    }
                    Some("aggregate") => {
                        let members = aggregate_members(term).ok_or_else(nonliteral)?;
                        stack.push(Step::Members(members.iter(), owner));
                    }
                    Some("binding") => {
                        meter.charge(1)?;
                        let value = term.get("value").ok_or_else(nonliteral)?;
                        stack.push(Step::Term(value, owner));
                    }
                    _ => return Err(nonliteral()),
                }
            }
            Step::Members(mut members, owner) => {
                if let Some(member) = members.next() {
                    meter.charge(1)?;
                    stack.push(Step::Members(members, owner));
                    stack.push(Step::Term(member, owner));
                }
            }
            Step::Leave => {
                ancestors.pop();
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::CheckedOccurrenceRole;
    use super::*;
    use crate::{CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2ReadResult};
    use ix_trace_rs::trace;
    use serde_json::json;
    use std::sync::OnceLock;

    fn raw_id(digit: char) -> CheckedNodeId {
        CheckedNodeId {
            domain: NODE_DOMAIN.into(),
            digest: digit.to_string().repeat(64).into(),
        }
    }

    fn fixture() -> &'static (Value, BTreeMap<char, CheckedNodeId>) {
        static FIXTURE: OnceLock<(Value, BTreeMap<char, CheckedNodeId>)> = OnceLock::new();
        FIXTURE.get_or_init(|| {
            let node = |digit, tag, form, ty, body| {
                json!({"node_id":raw_id(digit),"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":tag,"semantic_form":form,"semantic_type":raw_id(ty),"dependencies":[],
                "occurrences":[{"role":if matches!(tag,"value"|"expression") {"expression"} else {"type"},"ordinal":0}],"body":body})
            };
            let parameter_body = |name| json!({"term":"aggregate","members":[
                {"term":"binding","name":"name","value":{"term":"literal","type":raw_id('3'),"value_kind":"text","value":name}},
                {"term":"binding","name":"level","value":{"term":"literal","type":raw_id('2'),"value_kind":"integer","value":"0"}}]});
            let empty = json!({"term":"aggregate","members":[]});
            let mut wire = json!({
            "contract_version":"quire.checked-package/v2",
            "identity_preimage":{"version":"quire.checked-package-id/v2",
                "edition":{"role":"edition","definition":{"authority":"test","identity":"edition"}},
                "profile_selections":[],"definition_selections":[],"model_selections":[],"required_features":[],"dependency_selections":[],"identity_projection":[]},
            "package_id":{"domain":"quire.package.semantic/v2","algorithm":"sha256","digest":"a".repeat(64)},
            "lock":{"sources":[],"edition":{"role":"edition","definition":{"authority":"test","identity":"edition"}},
                "profile_selections":[],"definition_selections":[],"model_selections":[],"required_features":[],"dependency_selections":[]},
            "semantic_graph":{"graph_version":"quire.checked-semantic-graph/v2","nodes":[
                node('1',"scalar_type","boolean",'1',empty.clone()),
                node('2',"scalar_type","integer",'2',empty.clone()),
                node('3',"scalar_type","text",'3',empty.clone()),
                node('4',"composite_type","record",'4',json!({"term":"aggregate","members":[
                    {"term":"binding","name":"ready","value":{"term":"reference","target":raw_id('1')}}]})),
                node('5',"value","parameter",'4',parameter_body("left")),
                node('6',"expression","binary",'1',json!({"term":"application","operator":"binary",
                    "operation":{"identity":"quire.op.structural.eq","laws":[],"mode":null,"member":null,"leaves":[]},
                    "result_type":raw_id('1'),"arguments":[{"term":"reference","target":raw_id('5')},{"term":"reference","target":raw_id('8')}]})),
                node('7',"composite_type","option",'7',json!({"term":"aggregate","members":[{"term":"reference","target":raw_id('1')}]})),
                node('8',"value","parameter",'4',parameter_body("right")),
            ]},"source_map":[],"capability_report":[],
            "diagnostics":{"catalog":{"authority":"test","identity":"diagnostics"},"entries":[]}
        });
            // Author canonical node identities in dependency order. This is a
            // test producer; public assertions use independently named paths.
            let mut ids = BTreeMap::new();
            for (position, digit) in [(0,'1'),(1,'2'),(2,'3'),(3,'4'),(4,'5'),(6,'7'),(7,'8'),(5,'6')] {
                let node = &wire["semantic_graph"]["nodes"][position];
                let ty = if node["node_id"] == node["semantic_type"] { Value::Null } else { node["semantic_type"].clone() };
                let preimage = json!({"version":if digit=='6' {"quire.application-node/v1"} else {"quire.structural-node/v1"},
                    "node_tag":node["node_tag"],"semantic_form":node["semantic_form"],"semantic_type":ty,
                    "declaration":null,"recursion":null,"body":node["body"]});
                let fresh = CheckedNodeId { domain:NODE_DOMAIN.into(), digest:quire_canonical::sha256(&preimage,quire_canonical::Limits::new(1<<20)).expect("fixture key").to_string().into() };
                replace_id(&mut wire, &raw_id(digit), &fresh);
                ids.insert(digit,fresh);
            }
            let source = json!({"authority":"test","identity":"composite","digest_domain":"quire.source.bytes/v1","digest":"a".repeat(64)});
            wire["lock"]["sources"] = json!([source]);
            wire["semantic_graph"]["nodes"][3]["dependencies"] = json!([ids[&'1']]);
            wire["semantic_graph"]["nodes"][6]["dependencies"] = json!([ids[&'1']]);
            let mut application_dependencies = vec![ids[&'5'].clone(),ids[&'8'].clone()];
            application_dependencies.sort();
            wire["semantic_graph"]["nodes"][5]["dependencies"] = json!(application_dependencies);
            let nodes = wire["semantic_graph"]["nodes"].as_array().expect("nodes");
            wire["source_map"] = json!(nodes.iter().enumerate().map(|(position,node)| json!({
                "node_id":node["node_id"],"role":node["occurrences"][0]["role"],"ordinal":0,
                "regions":[{"source":source,"start":position,"end":position+1}]})).collect::<Vec<_>>());
            let mut projection = wire["semantic_graph"]["nodes"].as_array().expect("nodes").clone();
            for node in &mut projection { node.as_object_mut().expect("node").remove("occurrences"); }
            wire["identity_preimage"]["identity_projection"] = json!(projection);
            wire["package_id"]["digest"] = json!(quire_canonical::sha256(&wire["identity_preimage"],quire_canonical::Limits::new(1<<20)).expect("package key").to_string());
            (wire,ids)
        })
    }

    fn replace_id(value: &mut Value, old: &CheckedNodeId, new: &CheckedNodeId) {
        if *value == json!(old) {
            *value = json!(new);
            return;
        }
        match value {
            Value::Array(members) => {
                for member in members {
                    replace_id(member, old, new);
                }
            }
            Value::Object(members) => {
                for member in members.values_mut() {
                    replace_id(member, old, new);
                }
            }
            _ => {}
        }
    }

    fn id(digit: char) -> CheckedNodeId {
        fixture()
            .1
            .get(&digit)
            .cloned()
            .unwrap_or_else(|| raw_id(digit))
    }

    /// Every corruption below starts from a genuinely reader-admitted package.
    fn package() -> CheckedPackageV2 {
        match CheckedPackageV2::read(
            &serde_json::to_vec(&fixture().0).expect("fixture bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => *package,
            other => panic!("defensive fixture must admit before mutation: {other:?}"),
        }
    }

    fn read(package: &CheckedPackageV2) -> Result<CheckedCompositeOperands> {
        package.composite_application_operands(
            &id('6'),
            &CheckedOccurrence {
                role: CheckedOccurrenceRole::Expression,
                ordinal: 0,
            },
            100_000,
        )
    }

    #[trace("FR-038-AC-181")]
    #[test]
    fn defensive_target_failures_never_salvage_or_fabricate_a_child_identity() {
        for malformed in [
            Value::Null,
            json!({"domain":NODE_DOMAIN,"digest":"bad"}),
            json!({"domain":NODE_DOMAIN,"digest":"5".repeat(64),"extra":true}),
        ] {
            let mut p = package();
            p.wire.semantic_graph.nodes[5].body["arguments"][0]["target"] = malformed;
            assert_eq!(read(&p), Err(Error::MalformedChild { ordinal: 0 }));
        }
        let mut p = package();
        p.wire.semantic_graph.nodes[5].body["arguments"][0]
            .as_object_mut()
            .expect("argument")
            .remove("target");
        assert_eq!(read(&p), Err(Error::MalformedChild { ordinal: 0 }));
        let mut p = package();
        p.wire.semantic_graph.nodes[5].body["arguments"][0]["target"] = json!(id('9'));
        assert_eq!(
            read(&p),
            Err(Error::MissingChild {
                ordinal: 0,
                child: id('9')
            })
        );
        p.wire.semantic_graph.nodes[5].body["arguments"][1] =
            json!({"term":"literal","type":id('2'),"value_kind":"integer","value":"1"});
        assert_eq!(
            read(&p),
            Err(Error::MissingChild {
                ordinal: 0,
                child: id('9')
            })
        );
        assert_eq!(
            p.composite_application_operands(
                &id('6'),
                &CheckedOccurrence {
                    role: CheckedOccurrenceRole::Expression,
                    ordinal: 0
                },
                0
            ),
            Err(Error::WorkLimit {
                limit: 0,
                consumed: 1
            })
        );
    }

    #[trace("FR-038-AC-181", "FR-038-AC-178")]
    #[test]
    fn defensive_operand_and_catalog_failures_keep_their_original_typed_loci() {
        let mut p = package();
        p.wire.semantic_graph.nodes[5].body["operation"]["identity"] = json!("unknown.operation");
        assert_eq!(
            read(&p),
            Err(Error::UnknownOperator {
                application: id('6')
            })
        );
        let mut p = package();
        p.wire.semantic_graph.nodes[5].body["arguments"][0] =
            json!({"term":"literal","type":id('2'),"value_kind":"integer","value":"1"});
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('2')),
                CheckedUnsupportedCompositeOperand::InlineInteger
            ))
        );
        let mut p = package();
        p.wire.semantic_graph.nodes[4].semantic_type = id('1');
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('1')),
                CheckedUnsupportedCompositeOperand::NonStructuralType
            ))
        );
        let mut p = package();
        p.wire.semantic_graph.nodes[4].body = json!({"term":"application","result_type":id('1')});
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('1')),
                CheckedUnsupportedCompositeOperand::ApplicationSubterm
            ))
        );
        let mut p = package();
        p.wire.semantic_graph.nodes[4].semantic_type = id('9');
        assert_eq!(read(&p), Err(type_error(0, Some(id('9')))));
    }

    // A valid first operand makes these independent second-argument defects
    // kill any implementation that hard-codes the enclosing ordinal to zero.
    #[trace("FR-038-AC-181", "FR-038-AC-178", "FR-038-AC-180")]
    #[test]
    fn tc_048_second_operand_refusals_retain_nonzero_ordinals_and_type_loci() {
        let mut p = package();
        p.wire.semantic_graph.nodes[5].body["arguments"][1]["target"] = json!(id('9'));
        assert_eq!(
            read(&p),
            Err(Error::MissingChild {
                ordinal: 1,
                child: id('9')
            })
        );
        let mut p = package();
        p.wire.semantic_graph.nodes[5].body["arguments"][1] =
            json!({"term":"application","result_type":id('2')});
        assert_eq!(
            read(&p),
            Err(unsupported(
                1,
                Some(id('2')),
                CheckedUnsupportedCompositeOperand::ApplicationSubterm
            ))
        );
        let mut p = package();
        p.wire.semantic_graph.nodes[5].body["arguments"][1] =
            json!({"term":"literal","type":id('2'),"value_kind":"integer","value":"1"});
        assert_eq!(
            read(&p),
            Err(unsupported(
                1,
                Some(id('2')),
                CheckedUnsupportedCompositeOperand::InlineInteger
            ))
        );
        let mut p = package();
        p.wire.semantic_graph.nodes[6].semantic_form = "union".into();
        p.kinds[6] = CheckedNodeKind::CompositeType(CompositeTypeForm::Union);
        p.wire.semantic_graph.nodes[7].semantic_type = id('7');
        assert_eq!(
            read(&p),
            Err(Error::UnsupportedDomain {
                ordinal: 1,
                type_node: id('7')
            })
        );
        let mut p = package();
        p.wire.semantic_graph.nodes[6].body["members"][0]["target"] = json!(id('7'));
        p.wire.semantic_graph.nodes[7].semantic_type = id('7');
        assert_eq!(read(&p), Err(type_error(1, Some(id('7')))));
        let mut p = package();
        p.wire.semantic_graph.nodes[6].semantic_form = "record".into();
        p.kinds[6] = CheckedNodeKind::CompositeType(CompositeTypeForm::Record);
        p.wire.semantic_graph.nodes[6].body = json!({"term":"aggregate","members":[{"term":"binding","name":"bad","value":{"term":"aggregate","members":[]}}]});
        p.wire.semantic_graph.nodes[7].semantic_type = id('7');
        assert_eq!(read(&p), Err(type_error(1, Some(id('7')))));
    }

    #[trace("FR-038-AC-181", "FR-038-AC-180")]
    #[test]
    fn malformed_optional_wrappers_unanchored_cycles_and_union_metadata_refuse_exactly() {
        let mut p = package();
        p.wire.semantic_graph.nodes[3].body["members"][0]["value"] =
            json!({"term":"aggregate","members":[]});
        assert_eq!(read(&p), Err(type_error(0, Some(id('4')))));
        let mut p = package();
        p.wire.semantic_graph.nodes[6].body["members"][0]["target"] = json!(id('7'));
        p.wire.semantic_graph.nodes[3].body["members"][0]["value"] =
            json!({"term":"reference","target":id('7')});
        assert_eq!(read(&p), Err(type_error(0, Some(id('7')))));
        let mut p = package();
        p.wire.semantic_graph.nodes[4].semantic_form = "union_value".into();
        p.kinds[4] = CheckedNodeKind::Value(ValueForm::UnionValue);
        assert_eq!(read(&p), Err(type_error(0, Some(id('4')))));
        let mut p = package();
        p.wire.semantic_graph.nodes[4].semantic_form = "record_value".into();
        p.kinds[4] = CheckedNodeKind::Value(ValueForm::RecordValue);
        p.wire.semantic_graph.nodes[4].body =
            json!({"term":"aggregate","members":[{"term":"reference","target":id('5')}]});
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('4')),
                CheckedUnsupportedCompositeOperand::NonliteralGraphValue
            ))
        );
    }

    #[trace("FR-038-AC-181")]
    #[test]
    fn attempted_meter_overflow_is_a_refusal_even_at_the_maximum_limit() {
        let mut meter = Meter {
            limit: u64::MAX,
            consumed: u64::MAX,
        };
        assert_eq!(
            meter.charge(1),
            Err(Error::WorkLimit {
                limit: u64::MAX,
                consumed: u64::MAX
            })
        );
        assert_eq!(meter.consumed, u64::MAX);
    }

    #[cfg(target_pointer_width = "64")]
    #[trace("FR-038-AC-181")]
    #[test]
    fn path_width_refusal_keeps_actual_ordinal_and_containing_type() {
        let large = usize::try_from(u64::from(u32::MAX) + 1).expect("64-bit input");
        assert_eq!(
            child_ordinal(large, 1, &id('4')),
            Err(Error::PositionOutOfRange {
                ordinal: Some(1),
                type_node: Some(id('4'))
            })
        );
    }
}
