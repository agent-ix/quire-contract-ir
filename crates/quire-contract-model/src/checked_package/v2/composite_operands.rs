// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038: metered, package-authored domains of structural comparison operands.

use super::operation_catalog::operation_catalog;
use super::structural::{
    is_integer_bound, is_non_negative_integer, typed_aggregate, typed_binding, typed_reference,
};
use super::{
    BoundedDomainForm, CheckedCollectionKind, CheckedNodeId, CheckedNodeKind, CheckedOccurrence,
    CheckedPackageV2, CheckedScalarOperandChild, CheckedSemanticNodeV2, CheckedTerm,
    CompositeTypeForm, LiteralKind, ScalarTypeForm, ValueForm,
};
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
    /// The admitted structural comparison operation.
    pub operator: CheckedCompositeOperator,
    /// One entry per argument, without deduplication.
    pub operands: Vec<CheckedCompositeOperand>,
    /// The exact logical work consumed by this projection.
    pub consumed_work: u64,
}

/// The closed structural comparison operators supported by composite operand projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedCompositeOperator {
    /// Structural equality.
    Eq,
    /// Structural inequality.
    Ne,
}

impl CheckedCompositeOperator {
    #[ix_trace_rs::string_edge]
    fn from_identity(identity: &str) -> Option<Self> {
        match identity {
            "quire.op.structural.eq" => Some(Self::Eq),
            "quire.op.structural.ne" => Some(Self::Ne),
            _ => None,
        }
    }
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
        /// Whether authored member order is semantically ordered.
        ordered: bool,
        /// Admitted member order, byte-ascending when unordered.
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
    /// Catalog operation is neither structural.eq nor structural.ne.
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
    terms: &'a [CheckedTerm],
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

    fn term(&self, node: &CheckedSemanticNodeV2) -> Option<&'a CheckedTerm> {
        self.index
            .get(&node.node_id)
            .and_then(|position| self.terms.get(*position))
    }
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
    /// Returns authored domains in argument order with their typed operator:
    /// `Eq` for `structural.eq` or `Ne` for `structural.ne`.
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
            terms: &self.terms,
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
        let Some(CheckedTerm::Application {
            operation,
            arguments,
            ..
        }) = self.term(application)
        else {
            return Err(Error::NotApplication {
                node: application.clone(),
            });
        };
        if !node.occurrences.contains(occurrence) {
            return Err(Error::MissingOccurrence {
                application: application.clone(),
                occurrence: occurrence.clone(),
            });
        }
        let identity = operation
            .get("identity")
            .and_then(Value::as_str)
            .filter(|id| operation_catalog().entry(id).is_some())
            .ok_or_else(|| Error::UnknownOperator {
                application: application.clone(),
            })?;
        let operator = CheckedCompositeOperator::from_identity(identity).ok_or_else(|| {
            Error::IneligibleOperator {
                application: application.clone(),
            }
        })?;
        let mut operands = Vec::new();
        for (index, argument) in arguments.iter().enumerate() {
            let ordinal = argument_ordinal(index)?;
            meter.charge(1)?;
            if let CheckedTerm::Literal {
                type_id,
                value_kind,
                ..
            } = argument
            {
                let reason = if *value_kind == LiteralKind::Integer {
                    CheckedUnsupportedCompositeOperand::InlineInteger
                } else {
                    CheckedUnsupportedCompositeOperand::InlineNonInteger
                };
                return Err(unsupported(ordinal, Some(type_id.clone()), reason));
            }
            if let CheckedTerm::Application { result_type, .. } = argument {
                return Err(unsupported(
                    ordinal,
                    Some(result_type.clone()),
                    CheckedUnsupportedCompositeOperand::ApplicationSubterm,
                ));
            }
            let CheckedTerm::Reference { target: id } = argument else {
                return Err(Error::MalformedChild { ordinal });
            };
            let id = id.clone();
            let child = graph
                .node(&id, &mut meter)?
                .ok_or_else(|| Error::MissingChild {
                    ordinal,
                    child: id.clone(),
                })?;
            if let Some(CheckedTerm::Application { result_type, .. }) = self.term(&id) {
                return Err(unsupported(
                    ordinal,
                    Some(result_type.clone()),
                    CheckedUnsupportedCompositeOperand::ApplicationSubterm,
                ));
            }
            let resolution = super::operations::resolve_family_with_selected(
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
                |id| self.models.variant_family(id),
            )?;
            let family = match resolution {
                super::operations::FamilyResolution::Family(family) => Some(family),
                super::operations::FamilyResolution::NoFamily => None,
                super::operations::FamilyResolution::Malformed(type_node) => {
                    return Err(type_error(ordinal, Some(type_node)));
                }
            };
            if !family
                .is_some_and(|family| operation_catalog().family_fits(family, "structural_kind"))
            {
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
            operator,
            operands,
            consumed_work: meter.consumed,
        })
    }
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

fn forwarding_target(
    graph: &Graph<'_>,
    node: &CheckedSemanticNodeV2,
    kind: CheckedNodeKind,
) -> Option<CheckedNodeId> {
    match kind {
        CheckedNodeKind::CompositeType(CompositeTypeForm::Alias) => {
            let [inner] = typed_aggregate(graph.term(node)?)? else {
                return None;
            };
            typed_reference(inner).cloned()
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
        if let Some(next) = forwarding_target(graph, node, kind) {
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
    graph: &Graph<'_>,
    node: &CheckedSemanticNodeV2,
    name: &str,
    ordinal: u64,
    natural: bool,
    meter: &mut Meter,
) -> Result<CheckedCanonicalIntegerBound> {
    let malformed = || type_error(ordinal, Some(node.node_id.clone()));
    let Some(CheckedTerm::Aggregate { members }) = graph.term(node) else {
        return Err(malformed());
    };
    let term = members
        .iter()
        .find_map(|member| match member {
            CheckedTerm::Binding { name: found, value } if found.as_ref() == name => Some(value),
            _ => None,
        })
        .ok_or_else(malformed)?;
    let CheckedTerm::Literal {
        value_kind: LiteralKind::Integer,
        value,
        ..
    } = term.as_ref()
    else {
        return Err(malformed());
    };
    let text = value.as_str().ok_or_else(malformed)?;
    meter.length(text.len())?;
    let valid = if natural {
        is_non_negative_integer(text)
    } else {
        is_integer_bound(text)
    };
    if !valid {
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
            let CheckedTerm::NominalBody(body) = &node.body else {
                return Err(malformed());
            };
            let super::identity::NominalBodyKind::Enum {
                ordered,
                members: names,
            } = body.kind()
            else {
                return Err(malformed());
            };
            let members = names
                .iter()
                .map(|member| meter.text(member))
                .collect::<Result<Vec<_>>>()?;
            (
                Some(D::Enum {
                    ordered: *ordered,
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
            | ScalarTypeForm::CompoundUnit
            | ScalarTypeForm::CarriedQuantity,
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
                        lower: bound(graph, node, "min", ordinal, false, meter)?,
                        upper: bound(graph, node, "max", ordinal, false, meter)?,
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
                        minimum: bound(graph, node, "min", ordinal, true, meter)?,
                        maximum: bound(graph, node, "max", ordinal, true, meter)?,
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
    graph: &Graph<'_>,
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
        Some(
            graph
                .term(node)
                .and_then(typed_aggregate)
                .ok_or_else(malformed)?,
        )
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
            let CheckedTerm::Binding { name, value } = member else {
                return Err(malformed());
            };
            let name = meter.text(name.as_ref())?;
            if let CheckedTerm::Aggregate { members } = value.as_ref() {
                let [optional] = members.as_slice() else {
                    return Err(malformed());
                };
                let inner = typed_binding(optional, "optional").ok_or_else(malformed)?;
                let target = typed_reference(inner).cloned().ok_or_else(malformed)?;
                (target, Some(name), true)
            } else {
                (
                    typed_reference(value).cloned().ok_or_else(malformed)?,
                    Some(name),
                    false,
                )
            }
        }
        K::CompositeType(_) => {
            let members = members.ok_or_else(malformed)?;
            if !many && members.len() != 1 {
                return Err(malformed());
            }
            (
                typed_reference(members.get(index).ok_or_else(malformed)?)
                    .cloned()
                    .ok_or_else(malformed)?,
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
        match next_edge(graph, frame, ordinal, meter)? {
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

/// Compare already validated canonical signed decimal strings without narrowing.
#[ix_trace_rs::string_edge]
fn decimal_order(left: &str, right: &str) -> std::cmp::Ordering {
    let (left_negative, left_digits) = left.strip_prefix('-').map_or((false, left), |s| (true, s));
    let (right_negative, right_digits) = right
        .strip_prefix('-')
        .map_or((false, right), |s| (true, s));
    super::temporal::compare_integer_parts(left_negative, left_digits, right_negative, right_digits)
}

/// The one closed-value conversion exception. No value is evaluated or copied.
#[ix_trace_rs::string_edge]
fn closed_narrow(
    graph: &Graph<'_>,
    node: &CheckedSemanticNodeV2,
    ordinal: u64,
    meter: &mut Meter,
) -> Result<bool> {
    let Some(CheckedTerm::Application {
        operator,
        operation,
        result_type,
        arguments,
    }) = graph.term(node)
    else {
        return Ok(false);
    };
    if operation.identity() != Some("quire.op.numeric.narrow") {
        return Ok(false);
    }
    meter.charge(1)?;
    let malformed = || type_error(ordinal, Some(node.semantic_type.clone()));
    let record = operation.record();
    let member = record.get("member").ok_or_else(malformed)?;
    if graph.kind(node)
        != Some(CheckedNodeKind::Expression(
            super::ExpressionForm::Conversion,
        ))
        || *operator != super::ApplicationOperator::Convert
        || record
            .get("laws")
            .and_then(Value::as_array)
            .is_none_or(|a| !a.is_empty())
        || record
            .get("leaves")
            .and_then(Value::as_array)
            .is_none_or(|a| !a.is_empty())
        || !record.get("mode").is_some_and(Value::is_null)
        || member.get("kind").and_then(Value::as_str) != Some("type_argument")
        || member.get("declaration")
            != Some(&serde_json::to_value(result_type).expect("typed node identity"))
        || result_type != &node.semantic_type
        || arguments.len() != 1
    {
        return Err(malformed());
    }
    let range = graph.node(result_type, meter)?.ok_or_else(malformed)?;
    meter.charge(1)?;
    if graph.kind(range)
        != Some(CheckedNodeKind::BoundedDomain(
            BoundedDomainForm::IntegerRange,
        ))
    {
        return Err(malformed());
    }
    let range_error = || type_error(ordinal, Some(range.node_id.clone()));
    let integer = graph
        .node(&range.semantic_type, meter)?
        .ok_or_else(range_error)?;
    meter.charge(1)?;
    if graph.kind(integer) != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Integer)) {
        return Err(range_error());
    }
    let Some(CheckedTerm::Aggregate { members }) = graph.term(range) else {
        return Err(range_error());
    };
    if members.len() != 2 {
        return Err(range_error());
    }
    let mut endpoints = Vec::with_capacity(2);
    for (term, expected) in members.iter().zip(["min", "max"]) {
        meter.charge(1)?;
        let CheckedTerm::Binding { name, value } = term else {
            return Err(range_error());
        };
        meter.charge(1)?;
        let CheckedTerm::Literal {
            type_id,
            value_kind,
            value,
        } = value.as_ref()
        else {
            return Err(range_error());
        };
        let Some(text) = value.as_str() else {
            return Err(range_error());
        };
        meter.length(text.len())?;
        if name.as_ref() != expected
            || *value_kind != LiteralKind::Integer
            || type_id != &integer.node_id
            || !is_integer_bound(text)
        {
            return Err(range_error());
        }
        endpoints.push(text);
    }
    if decimal_order(endpoints[0], endpoints[1]).is_gt() {
        return Err(range_error());
    }
    meter.charge(1)?;
    let CheckedTerm::Reference { target } = &arguments[0] else {
        return Err(malformed());
    };
    let literal = graph.node(target, meter)?.ok_or_else(malformed)?;
    meter.charge(1)?;
    let nonliteral = || {
        unsupported(
            ordinal,
            Some(literal.semantic_type.clone()),
            CheckedUnsupportedCompositeOperand::NonliteralGraphValue,
        )
    };
    if graph.kind(literal) != Some(CheckedNodeKind::Value(ValueForm::Literal)) {
        return Err(nonliteral());
    }
    let Some(CheckedTerm::Literal {
        type_id,
        value_kind,
        value,
    }) = graph.term(literal)
    else {
        return Err(type_error(ordinal, Some(literal.semantic_type.clone())));
    };
    let literal_error = || type_error(ordinal, Some(type_id.clone()));
    let Some(text) = value.as_str() else {
        return Err(literal_error());
    };
    meter.length(text.len())?;
    if *value_kind != LiteralKind::Integer
        || type_id != &integer.node_id
        || literal.semantic_type != integer.node_id
        || !is_integer_bound(text)
    {
        return Err(literal_error());
    }
    if decimal_order(text, endpoints[0]).is_lt() || decimal_order(text, endpoints[1]).is_gt() {
        return Err(range_error());
    }
    Ok(true)
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
        Term(&'a CheckedTerm, &'a CheckedSemanticNodeV2),
        Members(std::slice::Iter<'a, CheckedTerm>, &'a CheckedSemanticNodeV2),
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
                if closed_narrow(graph, node, ordinal, meter)? {
                    continue;
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
                stack.push(Step::Term(graph.term(node).ok_or_else(nonliteral)?, node));
            }
            Step::Term(term, owner) => {
                let nonliteral = || {
                    unsupported(
                        ordinal,
                        Some(owner.semantic_type.clone()),
                        CheckedUnsupportedCompositeOperand::NonliteralGraphValue,
                    )
                };
                match term {
                    CheckedTerm::Literal { .. } => {}
                    CheckedTerm::Reference { target } => {
                        meter.charge(1)?;
                        stack.push(Step::Node(target.clone()));
                    }
                    CheckedTerm::Aggregate { members } => {
                        stack.push(Step::Members(members.iter(), owner));
                    }
                    CheckedTerm::Binding { value, .. } => {
                        meter.charge(1)?;
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
    use crate::{
        CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCode,
        CheckedPackageV2ReadResult,
    };
    use ix_trace_rs::trace;
    use serde_json::json;
    use std::sync::OnceLock;

    fn raw_id(digit: char) -> CheckedNodeId {
        CheckedNodeId {
            package: "placeholder".into(),
            path: vec![super::super::CheckedNodePathSegment::Name(
                digit.to_string().into(),
            )],
            ordinal: 0,
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
            "name":"composite",
            "lock":{"sources":[],"edition":{"role":"edition","definition":{"authority":"test","identity":"edition"}},
                "profile_selections":[],"definition_selections":[],"required_features":[],"dependency_selections":[]},
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
                node('a',"scalar_type","compound_unit",'a',empty.clone()),
            ]},"source_map":[],"capability_report":[],
            "diagnostics":{"catalog":{"authority":"test","identity":"diagnostics"},"entries":[]}
        });
            // Author canonical node identities in dependency order. This is a
            // test producer; public assertions use independently named paths.
            let mut ids = BTreeMap::new();
            for (_, digit) in [(0,'1'),(1,'2'),(2,'3'),(3,'4'),(4,'5'),(6,'7'),(7,'8'),(8,'a'),(5,'6')] {
                let fresh = match digit {
                    '1' => CheckedNodeId { package:"composite".into(), path:vec![super::super::CheckedNodePathSegment::Name("scalar_type".into()),super::super::CheckedNodePathSegment::Name("boolean".into())],ordinal:0 },
                    '2' => CheckedNodeId { package:"composite".into(), path:vec![super::super::CheckedNodePathSegment::Name("scalar_type".into()),super::super::CheckedNodePathSegment::Name("integer".into())],ordinal:0 },
                    '3' => CheckedNodeId { package:"composite".into(), path:vec![super::super::CheckedNodePathSegment::Name("scalar_type".into()),super::super::CheckedNodePathSegment::Name("text".into())],ordinal:0 },
                    _ => CheckedNodeId { package:"composite".into(), path:vec![super::super::CheckedNodePathSegment::Name("fixture".into()),super::super::CheckedNodePathSegment::Name(digit.to_string().into())],ordinal:0 },
                };
                replace_id(&mut wire, &raw_id(digit), &fresh);
                ids.insert(digit,fresh);
            }
            let source = json!({"authority":"test","identity":"composite"});
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

    // Defensive probes below mutate only after the original eq fixture admits.
    fn ne_package() -> CheckedPackageV2 {
        let mut package = package();
        application_operation(&mut package)["identity"] = json!("quire.op.structural.ne");
        package
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

    fn boolean_equality_wire() -> Value {
        let (original, ids) = fixture();
        let mut wire = original.clone();
        wire["semantic_graph"]["nodes"][5]["body"]["operation"]["identity"] =
            json!("quire.op.boolean.eq");
        wire["semantic_graph"]["nodes"][5]["body"]["arguments"] = json!([
            {"term":"reference","target":ids[&'1']},
            {"term":"reference","target":ids[&'1']}
        ]);
        wire["semantic_graph"]["nodes"][5]["dependencies"] = json!([ids[&'1']]);
        wire["checked_term_data"] = json!({
            "equality_decisions":[{
                "application":ids[&'6'],
                "occurrence":{"role":"expression","ordinal":0},
                "term_path":[],"operator":"equal","schedule":"plan",
                "left":{"operand":0,"source_type":ids[&'1'],"conversion_target":null},
                "right":{"operand":1,"source_type":ids[&'1'],"conversion_target":null},
                "units":[],"enum_members":[]
            }],
            "protocol_locations":[],"dispatch_tables":[],"probabilistic_claims":[]
        });
        wire
    }

    fn converted_integer_equality_wire() -> Value {
        let mut wire = boolean_equality_wire();
        let ids = &fixture().1;
        let converted = json!({"package":"composite","path":["fixture","converted"],"ordinal":0});
        for position in [4, 7] {
            wire["semantic_graph"]["nodes"][position]["semantic_type"] = json!(ids[&'2']);
        }
        wire["semantic_graph"]["nodes"][5]["body"]["operation"]["identity"] =
            json!("quire.op.integer.eq");
        wire["semantic_graph"]["nodes"][5]["body"]["arguments"][0] =
            json!({"term":"reference","target":converted});
        wire["semantic_graph"]["nodes"][5]["body"]["arguments"][1] =
            json!({"term":"reference","target":ids[&'8']});
        let converted_id: CheckedNodeId =
            serde_json::from_value(converted.clone()).expect("conversion id");
        let mut dependencies = vec![converted_id, ids[&'8'].clone()];
        dependencies.sort();
        wire["semantic_graph"]["nodes"][5]["dependencies"] = json!(dependencies);
        wire["checked_term_data"]["equality_decisions"][0]["left"]["source_type"] =
            json!(ids[&'2']);
        wire["checked_term_data"]["equality_decisions"][0]["left"]["conversion_target"] =
            json!(ids[&'2']);
        wire["checked_term_data"]["equality_decisions"][0]["right"]["source_type"] =
            json!(ids[&'2']);
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id":converted,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"expression","semantic_form":"unary","semantic_type":ids[&'2'],
                "dependencies":[ids[&'5'],ids[&'2']],
                "occurrences":[{"role":"expression","ordinal":0}],
                "body":{"term":"application","operator":"convert",
                    "operation":{"identity":"quire.op.numeric.convert","laws":[],"mode":null,
                        "member":{"kind":"type_argument","declaration":ids[&'2']},"leaves":[]},
                    "result_type":ids[&'2'],
                    "arguments":[{"term":"reference","target":ids[&'5']}]}
            }));
        let source = wire["lock"]["sources"][0].clone();
        wire["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(json!({
                "node_id":converted,"role":"expression","ordinal":0,
                "regions":[{"source":source,"start":9,"end":10}]
            }));
        wire
    }

    fn converted_decimal_equality_wire() -> Value {
        let mut wire = converted_integer_equality_wire();
        let integer = json!(fixture().1[&'2']);
        let text = json!(fixture().1[&'3']);
        let decimal = json!({"package":"composite","path":["scalar_type","decimal"],"ordinal":0});
        let bounded =
            json!({"package":"composite","path":["bounded_domain","decimal_range"],"ordinal":0});
        let empty = json!({"term":"aggregate","members":[]});
        let bound = |name: &str, ty: &Value, kind: &str, value: &str| {
            json!({
                "term":"binding","name":name,
                "value":{"term":"literal","type":ty,"value_kind":kind,"value":value}
            })
        };
        let nodes = wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes");
        nodes.push(json!({
            "node_id":decimal,"schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"scalar_type","semantic_form":"decimal","semantic_type":decimal,
            "dependencies":[],"occurrences":[{"role":"type","ordinal":0}],"body":empty
        }));
        nodes.push(json!({
            "node_id":bounded,"schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"bounded_domain","semantic_form":"decimal_range","semantic_type":decimal,
            "dependencies":[],"occurrences":[{"role":"type","ordinal":0}],
            "body":{"term":"aggregate","members":[
                bound("coefficient_min",&integer,"integer","-100"),
                bound("coefficient_max",&integer,"integer","100"),
                bound("scale_min",&integer,"integer","0"),
                bound("scale_max",&integer,"integer","0"),
                bound("rounding",&text,"text","nearest-even")
            ]}
        }));
        nodes[7]["semantic_type"] = bounded.clone();
        nodes[9]["semantic_type"] = bounded.clone();
        nodes[9]["body"]["result_type"] = bounded.clone();
        nodes[9]["body"]["operation"]["member"]["declaration"] = bounded.clone();
        nodes[9]["dependencies"] = json!([bounded, fixture().1[&'5']]);
        nodes[5]["body"]["operation"]["identity"] = json!("quire.op.decimal.eq");
        wire["checked_term_data"]["equality_decisions"][0]["left"]["conversion_target"] =
            bounded.clone();
        wire["checked_term_data"]["equality_decisions"][0]["right"]["source_type"] =
            bounded.clone();
        let source = wire["lock"]["sources"][0].clone();
        for (position, id) in [(10, decimal), (11, bounded)] {
            wire["source_map"]
                .as_array_mut()
                .expect("source map")
                .push(json!({
                    "node_id":id,"role":"type","ordinal":0,
                    "regions":[{"source":source,"start":position,"end":position+1}]
                }));
        }
        wire
    }

    fn bounded_text_equality_wire() -> Value {
        use super::super::operation_catalog::operation_catalog;
        use super::super::LawRole;

        let mut wire = boolean_equality_wire();
        let ids = &fixture().1;
        let bounded =
            json!({"package":"composite","path":["bounded_domain","text_bounds"],"ordinal":0});
        let definition = serde_json::to_value(
            &operation_catalog()
                .law_role_definitions(LawRole::TextProfile)
                .expect("text profile law")[0],
        )
        .expect("law definition");
        wire["lock"]["definition_selections"] = json!([definition]);
        let literal = |ty: &CheckedNodeId, kind: &str, value: &str| {
            json!({
                "term":"literal","type":ty,"value_kind":kind,"value":value
            })
        };
        let binding =
            |name: &str, value: Value| json!({"term":"binding","name":name,"value":value});
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id":bounded,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"bounded_domain","semantic_form":"text_bounds","semantic_type":ids[&'3'],
                "dependencies":[],"occurrences":[{"role":"type","ordinal":0}],
                "body":{"term":"aggregate","members":[
                    binding("min",literal(&ids[&'2'],"integer","0")),
                    binding("max",literal(&ids[&'2'],"integer","8")),
                    binding("text_profile",literal(&ids[&'3'],"text","nfc"))
                ]}
            }));
        for position in [4, 7] {
            wire["semantic_graph"]["nodes"][position]["semantic_type"] = bounded.clone();
        }
        wire["semantic_graph"]["nodes"][5]["body"]["operation"] = json!({
            "identity":"quire.op.text.eq",
            "laws":[{"role":"text_profile","definition":definition}],
            "mode":{"kind":"text_profile","value":"nfc"},"member":null,"leaves":[]
        });
        wire["semantic_graph"]["nodes"][5]["body"]["arguments"] = json!([
            {"term":"reference","target":ids[&'5']},
            {"term":"reference","target":ids[&'8']}
        ]);
        wire["semantic_graph"]["nodes"][5]["dependencies"] = json!([ids[&'5'], ids[&'8']]);
        for side in ["left", "right"] {
            wire["checked_term_data"]["equality_decisions"][0][side]["source_type"] =
                bounded.clone();
        }
        wire["checked_term_data"]["equality_decisions"][0]["schedule"] = json!("text");
        let source = wire["lock"]["sources"][0].clone();
        wire["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(json!({
                "node_id":bounded,"role":"type","ordinal":0,
                "regions":[{"source":source,"start":9,"end":10}]
            }));
        wire
    }

    /// Trace: FR-353-AC-7, FR-357-AC-6.
    #[test]
    fn bounded_text_equality_names_missing_text_identity() {
        use quire_exact::{Location, NodeKey, Origin, PathSegment, Role};

        let wire = bounded_text_equality_wire();
        let package = match CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("bounded Text equality must admit: {other:?}"),
        };
        let application = id('6');
        let occurrence = super::super::CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let original = NodeKey::new(
            "composite",
            vec![PathSegment::Text("text-equality".into())],
            0,
        )
        .expect("original key");
        let row = super::super::SelectedEqualityApplicationRow::new(
            application.clone(),
            occurrence.clone(),
            serde_json::from_value(wire["source_map"][5].clone()).expect("source row"),
            original.clone(),
            Location::new(original, Origin::new(Role::from("expression"), 0)),
        );
        let missing = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&row),
                &[],
                &[],
                1000,
            )
            .expect_err("Text identity row remains required");
        assert_eq!(
            missing.code,
            super::super::AdmittedEqualityReadErrorCode::UnavailableEqualityWitness
        );
        assert_eq!(
            missing.witness,
            Some(super::super::EqualityWitnessKind::TextType)
        );
        assert_eq!(
            *missing.at,
            serde_json::from_value(
                wire["checked_term_data"]["equality_decisions"][0]["left"]["source_type"].clone()
            )
            .expect("Text node")
        );
        let selected = super::super::SelectedFactEqualityId {
            application: application.clone(),
            occurrence: occurrence.clone(),
        };
        let selected_error = package
            .project_selected_facts(
                super::super::SelectedFactIds {
                    declarations: &[],
                    fields: &[],
                    equalities: std::slice::from_ref(&selected),
                },
                super::super::SelectedFactRows {
                    declarations: &[],
                    fields: &[],
                    source_units: &[],
                    equality_applications: &[row],
                    equality_facts: &[],
                },
                100_000,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
            .expect_err("selected-fact read preserves the native Text witness refusal");
        assert_eq!(
            selected_error.code,
            super::super::SelectedFactsErrorCode::Equality
        );
        assert_eq!(selected_error.at, missing.at);
        assert_eq!(selected_error.equality.as_deref(), Some(&missing));
    }

    /// Trace: FR-353-AC-1, FR-353-AC-7, FR-357-AC-5, FR-357-AC-6, FR-357-AC-7.
    /// Provenance: IR-1209 QSL per-fact Text type rows.
    #[test]
    fn bounded_text_equality_reads_joined_type_rows() {
        use quire_exact::{Location, NodeKey, Origin, PathSegment, Role, ValueType};
        use quire_semantic_value::declaration::EqualitySchedule;

        let wire = bounded_text_equality_wire();
        let package = match CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("bounded Text package must admit: {other:?}"),
        };
        let application = id('6');
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let original = NodeKey::new(
            "composite",
            vec![PathSegment::Text("text-equality".into())],
            0,
        )
        .expect("producer key");
        let application_row = super::super::SelectedEqualityApplicationRow::new(
            application.clone(),
            occurrence.clone(),
            serde_json::from_value(wire["source_map"][5].clone()).expect("source row"),
            original.clone(),
            Location::new(original.clone(), Origin::new(Role::from("expression"), 0)),
        );
        let type_id: CheckedNodeId = serde_json::from_value(
            wire["checked_term_data"]["equality_decisions"][0]["left"]["source_type"].clone(),
        )
        .expect("Text type id");
        let type_key = NodeKey::new(
            "composite",
            vec![PathSegment::Text("bounded-text".into())],
            0,
        )
        .expect("producer type key");
        let mut facts = Vec::new();
        for position in 0..2 {
            for fact in [
                super::super::EqualityFactWitness::OperandType {
                    position,
                    role: super::super::EqualityOperandTypeRole::Source,
                },
                super::super::EqualityFactWitness::TextPlanType {
                    position,
                    role: super::super::EqualityOperandTypeRole::Source,
                },
            ] {
                facts.push(super::super::SelectedEqualityFactRow {
                    application: original.clone(),
                    checked_application: application.clone(),
                    occurrence: occurrence.clone(),
                    source_map_entry: application_row.source_map_entry.clone(),
                    structural_key: type_key.clone(),
                    structural_node: type_id.clone(),
                    owner: package.node(&type_id).expect("type node").owner.clone(),
                    value_type: Some(ValueType::Text(
                        quire_exact::TextType::new(0, 8, quire_exact::TextProfile::Nfc)
                            .expect("Text bounds"),
                    )),
                    fact,
                });
            }
        }
        let admitted = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&application_row),
                &facts,
                &[],
                1000,
            )
            .expect("joined bounded Text decision");
        assert_eq!(admitted.schedule(), EqualitySchedule::Text);
        assert!(matches!(admitted.left().source(), ValueType::Text(_)));
        assert_eq!(admitted.left().source(), admitted.right().source());

        let selected = super::super::SelectedFactEqualityId {
            application: application.clone(),
            occurrence: occurrence.clone(),
        };
        let selected_read = |fact_rows: &[super::super::SelectedEqualityFactRow]| {
            package.project_selected_facts(
                super::super::SelectedFactIds {
                    declarations: &[],
                    fields: &[],
                    equalities: std::slice::from_ref(&selected),
                },
                super::super::SelectedFactRows {
                    declarations: &[],
                    fields: &[],
                    source_units: &[],
                    equality_applications: std::slice::from_ref(&application_row),
                    equality_facts: fact_rows,
                },
                100_000,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
        };
        let complete = selected_read(&facts).expect("selected Text facts join");
        assert_eq!(complete.equalities.len(), 1);
        assert_eq!(complete.equalities[0].decision, admitted);
        let mut oversized_facts = facts.clone();
        oversized_facts[0].structural_key = NodeKey::new(
            "composite",
            vec![PathSegment::Text("x".repeat(1_000_000))],
            0,
        )
        .unwrap();
        let denied =
            selected_read(&oversized_facts).expect_err("fact material is charged before copying");
        assert_eq!(
            denied.code,
            super::super::SelectedFactsErrorCode::ExhaustedWork
        );
        assert_eq!(denied.at.as_ref(), &type_id);
        assert!(denied.attempted_work > 0);

        facts.push(facts[0].clone());
        let duplicate_selected =
            selected_read(&facts).expect_err("selected duplicate fact refuses");
        assert_eq!(
            duplicate_selected.code,
            super::super::SelectedFactsErrorCode::Equality
        );
        assert_eq!(
            duplicate_selected.equality.as_ref().unwrap().code,
            super::super::AdmittedEqualityReadErrorCode::DuplicateEqualityWitness
        );
        assert_eq!(duplicate_selected.at.as_ref(), &type_id);
        let duplicate = package
            .admitted_equality(
                &application,
                &occurrence,
                &[application_row],
                &facts,
                &[],
                1000,
            )
            .expect_err("competing fact rows refuse");
        assert_eq!(
            duplicate.code,
            super::super::AdmittedEqualityReadErrorCode::DuplicateEqualityWitness
        );
        assert_eq!(*duplicate.at, type_id);
    }

    /// Trace: FR-353-AC-1, FR-353-AC-7.
    /// Provenance: IR-1209 checker-held Plan type and exact declaration key.
    #[test]
    fn declaration_plan_equality_reads_typed_rows() {
        use quire_exact::{Location, NodeKey, Origin, PathSegment, Role, ValueType};
        use quire_semantic_value::declaration::EqualitySchedule;

        let mut wire = boolean_equality_wire();
        let ids = &fixture().1;
        wire["semantic_graph"]["nodes"][5]["body"]["operation"]["identity"] =
            json!("quire.op.structural.eq");
        wire["semantic_graph"]["nodes"][5]["body"]["arguments"] = json!([
            {"term":"reference","target":ids[&'5']},
            {"term":"reference","target":ids[&'8']}
        ]);
        let mut dependencies = vec![ids[&'5'].clone(), ids[&'8'].clone()];
        dependencies.sort();
        wire["semantic_graph"]["nodes"][5]["dependencies"] = json!(dependencies);
        for side in ["left", "right"] {
            wire["checked_term_data"]["equality_decisions"][0][side]["source_type"] =
                json!(ids[&'4']);
        }
        let package = match CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("declaration Plan package must admit: {other:?}"),
        };
        let application = id('6');
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let original = NodeKey::new(
            "composite",
            vec![PathSegment::Text("plan-equality".into())],
            0,
        )
        .expect("application key");
        let declaration = NodeKey::new("composite", vec![PathSegment::Text("record".into())], 0)
            .expect("declaration key");
        let application_row = super::super::SelectedEqualityApplicationRow::new(
            application.clone(),
            occurrence.clone(),
            serde_json::from_value(wire["source_map"][5].clone()).expect("source row"),
            original.clone(),
            Location::new(original.clone(), Origin::new(Role::from("expression"), 0)),
        );
        let mut facts = Vec::new();
        for position in 0..2 {
            for fact in [
                super::super::EqualityFactWitness::OperandType {
                    position,
                    role: super::super::EqualityOperandTypeRole::Source,
                },
                super::super::EqualityFactWitness::TextPlanType {
                    position,
                    role: super::super::EqualityOperandTypeRole::Source,
                },
            ] {
                facts.push(super::super::SelectedEqualityFactRow {
                    application: original.clone(),
                    checked_application: application.clone(),
                    occurrence: occurrence.clone(),
                    source_map_entry: application_row.source_map_entry.clone(),
                    structural_key: declaration.clone(),
                    structural_node: ids[&'4'].clone(),
                    owner: package
                        .node(&ids[&'4'])
                        .expect("declaration node")
                        .owner
                        .clone(),
                    value_type: Some(ValueType::Composite(declaration.clone())),
                    fact,
                });
            }
        }
        let admitted = package
            .admitted_equality(
                &application,
                &occurrence,
                &[application_row],
                &facts,
                &[],
                1000,
            )
            .expect("typed declaration Plan decision");
        assert_eq!(admitted.schedule(), EqualitySchedule::Plan);
        assert_eq!(admitted.left().source(), &ValueType::Composite(declaration));
    }

    /// Trace: FR-353-AC-1, FR-353-AC-3, FR-353-AC-6.
    /// Provenance: IR-1209 compound quantity UnitId per-fact rows.
    #[test]
    fn compound_quantity_equality_reads_unit_rows() {
        use quire_exact::{Location, NodeKey, Origin, PathSegment, Role, UnitId, ValueType};
        use quire_semantic_value::declaration::EqualitySchedule;

        let mut wire = boolean_equality_wire();
        let ids = &fixture().1;
        let source_dimension =
            json!({"package":"composite","path":["fixture","source_dimension"],"ordinal":0});
        let source_root =
            json!({"package":"composite","path":["fixture","source_root"],"ordinal":0});
        let model_dimension_identity = "ix://composite/dimension/Area";
        let model_unit_identity = "ix://composite/unit/Area";
        let model_dimension = json!({"package":"composite","path":["scalar_type","dimension",model_dimension_identity],"ordinal":0});
        let model_root = json!({"package":"composite","path":["scalar_type","unit",model_unit_identity],"ordinal":0});
        let selected_model = json!({
            "contractVersion":"2.0.0",
            "package":{"identity":"composite","version":"1.0.0"},
            "constructs":[],"types":[],
            "dimensions":[{"identity":model_dimension_identity,"name":"Area","base":true,"terms":[]}],
            "units":[{"identity":model_unit_identity,"name":"Area","dimension":model_dimension_identity,
                "scale":"1","offset":"0"}]
        });
        let source = wire["lock"]["sources"][0].clone();
        let nodes = wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes");
        nodes.extend([
            json!({"node_id":{"package":"composite","path":["model","model_root","composite"],"ordinal":0},
                "schema_version":"quire.checked-semantic-graph/v2","node_tag":"model","semantic_form":"model_root",
                "semantic_type":{"package":"composite","path":["model","model_root","composite"],"ordinal":0},
                "dependencies":[],"occurrences":[{"role":"generated","ordinal":0}],
                "owner":{"kind":"model_root","identity":"composite"},"body":{"term":"aggregate","members":[]}}),
            json!({"node_id":source_dimension,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"scalar_type","semantic_form":"dimension","semantic_type":source_dimension,
                "dependencies":[],"occurrences":[{"role":"declaration","ordinal":0}],
                "owner":{"kind":"source","authority":"test","identity":"composite"},
                "declaration":{"qualified_name":["fixture","source_dimension"]},
                "body":{"term":"dimension","base":true,"terms":[]}}),
            json!({"node_id":model_dimension,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"scalar_type","semantic_form":"dimension","semantic_type":model_dimension,
                "dependencies":[],"occurrences":[{"role":"generated","ordinal":0}],
                "owner":{"kind":"model","identity":"composite","node":model_dimension_identity},
                "body":{"term":"aggregate","members":[]}}),
            json!({"node_id":source_root,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"scalar_type","semantic_form":"unit","semantic_type":source_dimension,
                "dependencies":[source_dimension],"occurrences":[{"role":"declaration","ordinal":0}],
                "owner":{"kind":"source","authority":"test","identity":"composite"},
                "declaration":{"qualified_name":["fixture","source_root"]},
                "body":{"term":"unit","dimension_node_id":source_dimension,"target_unit_node_id":null,
                    "scale":{"numerator":"1","denominator":"1"},"offset":{"numerator":"0","denominator":"1"}}}),
            json!({"node_id":model_root,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"scalar_type","semantic_form":"unit","semantic_type":model_dimension,
                "dependencies":[model_dimension],"occurrences":[{"role":"generated","ordinal":0}],
                "owner":{"kind":"model","identity":"composite","node":model_unit_identity},
                "body":{"term":"aggregate","members":[]}})
        ]);
        for (node, at, role) in [
            (&source_dimension, 10, "declaration"),
            (&source_root, 13, "declaration"),
        ] {
            wire["source_map"]
                .as_array_mut()
                .expect("source map")
                .push(json!({
                    "node_id":node,"role":role,"ordinal":0,
                    "regions":[{"source":source,"start":at,"end":at+1}]
                }));
        }
        wire["source_map"].as_array_mut().expect("source map").push(json!({
            "node_id":{"package":"composite","path":["model","model_root","composite"],"ordinal":0},
            "role":"generated","ordinal":0,
            "regions":[{"source":source,"start":9,"end":10}]
        }));
        for (node, at) in [(&model_dimension, 11), (&model_root, 14)] {
            wire["source_map"]
                .as_array_mut()
                .expect("source map")
                .push(json!({
                    "node_id":node,"role":"generated","ordinal":0,
                    "regions":[{"source":source,"start":at,"end":at+1}]
                }));
        }
        let mut roots: Vec<CheckedNodeId> = [source_root.clone(), model_root.clone()]
            .into_iter()
            .map(|value| serde_json::from_value(value).expect("root id"))
            .collect();
        roots.sort();
        wire["semantic_graph"]["nodes"][8]["dependencies"] = json!(roots);
        wire["semantic_graph"]["nodes"][8]["body"] = json!({"term":"aggregate","members":[
            {"term":"aggregate","members":[
                {"term":"binding","name":"unit","value":{"term":"reference","target":source_root}},
                {"term":"binding","name":"exponent","value":{"term":"literal","type":ids[&'2'],"value_kind":"integer","value":"2"}}]},
            {"term":"aggregate","members":[
                {"term":"binding","name":"unit","value":{"term":"reference","target":model_root}},
                {"term":"binding","name":"exponent","value":{"term":"literal","type":ids[&'2'],"value_kind":"integer","value":"-1"}}]}
        ]});
        wire["semantic_graph"]["nodes"][5]["body"]["operation"]["identity"] =
            json!("quire.op.quantity.eq");
        wire["semantic_graph"]["nodes"][5]["body"]["arguments"] = json!([
            {"term":"reference","target":ids[&'a']},
            {"term":"reference","target":ids[&'a']}
        ]);
        wire["semantic_graph"]["nodes"][5]["dependencies"] = json!([ids[&'a']]);
        wire["checked_term_data"]["equality_decisions"][0]["schedule"] = json!("quantity");
        for side in ["left", "right"] {
            wire["checked_term_data"]["equality_decisions"][0][side]["source_type"] =
                json!(ids[&'a']);
        }
        wire["checked_term_data"]["equality_decisions"][0]["units"] = json!([{
            "quantity_type":ids[&'a'], "unit":ids[&'a'],
            "form":{"kind":"compound","dimension_terms":[],"terms":[
                {"root_unit":source_root,"exponent":"2"},
                {"root_unit":model_root,"exponent":"-1"}]}
        }]);
        let mut evidence = CheckedPackageEvidence::new();
        evidence
            .insert_domain_package_document(
                "composite",
                serde_json::to_vec(&selected_model).expect("model bytes"),
            )
            .expect("selected model evidence");
        let package = match CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("compound quantity package must admit: {other:?}"),
        };
        let application = id('6');
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let original = NodeKey::new(
            "composite",
            vec![PathSegment::Text("quantity-equality".into())],
            0,
        )
        .expect("application key");
        let unit_key = NodeKey::new(
            "composite",
            vec![PathSegment::Text("dimensionless-unit".into())],
            0,
        )
        .expect("unit key");
        let application_row = super::super::SelectedEqualityApplicationRow::new(
            application.clone(),
            occurrence.clone(),
            serde_json::from_value(wire["source_map"][5].clone()).expect("source row"),
            original.clone(),
            Location::new(original.clone(), Origin::new(Role::from("expression"), 0)),
        );
        let source_key = NodeKey::new(
            "composite",
            vec![PathSegment::Text("a-source-root".into())],
            0,
        )
        .expect("source key");
        let model_key = NodeKey::new(
            "composite",
            vec![PathSegment::Text("b-model-root".into())],
            0,
        )
        .expect("model key");
        let unit_id = UnitId::compound(vec![
            (source_key.clone(), "2".parse().expect("source exponent")),
            (model_key.clone(), "-1".parse().expect("model exponent")),
        ]);
        let mut facts = Vec::new();
        for position in 0..2 {
            for (fact, value_type) in [
                (
                    super::super::EqualityFactWitness::OperandType {
                        position,
                        role: super::super::EqualityOperandTypeRole::Source,
                    },
                    Some(ValueType::Quantity(unit_id.clone())),
                ),
                (
                    super::super::EqualityFactWitness::Unit {
                        unit_id: unit_id.clone(),
                        quantity_type: ids[&'a'].clone(),
                        operand_position: position,
                        role: super::super::EqualityUnitRole::Source,
                    },
                    None,
                ),
            ] {
                facts.push(super::super::SelectedEqualityFactRow {
                    application: original.clone(),
                    checked_application: application.clone(),
                    occurrence: occurrence.clone(),
                    source_map_entry: application_row.source_map_entry.clone(),
                    structural_key: unit_key.clone(),
                    structural_node: ids[&'a'].clone(),
                    owner: package.node(&ids[&'a']).expect("unit node").owner.clone(),
                    value_type,
                    fact,
                });
            }
        }
        let source_row = super::super::SelectedSourceUnitRow::new(
            serde_json::from_value(source_root.clone()).expect("source root id"),
            "test",
            "composite",
            vec!["fixture".into(), "source_root".into()],
            UnitId::declared(source_key.clone()),
        );
        let admitted = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&application_row),
                &facts,
                std::slice::from_ref(&source_row),
                1000,
            )
            .expect("joined compound quantity decision");
        assert_eq!(admitted.schedule(), EqualitySchedule::Quantity);
        assert_eq!(admitted.left().source(), &ValueType::Quantity(unit_id));
        assert_eq!(admitted.units().len(), 1);
        assert_eq!(
            admitted.units()[0]
                .id
                .compound_terms()
                .expect("compound roots")
                .len(),
            2
        );
        let missing_source = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&application_row),
                &facts,
                &[],
                1000,
            )
            .expect_err("source-owned root requires its selected source row");
        assert_eq!(
            missing_source.witness,
            Some(super::super::EqualityWitnessKind::SourceUnit)
        );
        assert_eq!(
            *missing_source.at,
            serde_json::from_value(source_root.clone()).expect("source root id")
        );
        let mut reversed = facts.clone();
        for row in &mut reversed {
            if let super::super::EqualityFactWitness::Unit { unit_id, .. } = &mut row.fact {
                *unit_id = UnitId::compound(vec![
                    (model_key.clone(), "-1".parse().expect("model exponent")),
                    (source_key.clone(), "2".parse().expect("source exponent")),
                ]);
            }
        }
        let wrong_order = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&application_row),
                &reversed,
                std::slice::from_ref(&source_row),
                1000,
            )
            .expect_err("wrong ordered root identities refuse");
        assert_eq!(
            wrong_order.code,
            super::super::AdmittedEqualityReadErrorCode::MismatchedEqualityWitness
        );
        assert_eq!(
            *wrong_order.at,
            serde_json::from_value(source_root).expect("source root id")
        );
    }

    /// Trace: FR-353-AC-1, FR-353-AC-3, FR-353-AC-7, FR-353-AC-8.
    /// Provenance: IR-1209 declared unit and dimension identity rows.
    #[test]
    fn declared_quantity_equality_reads_dimension_rows() {
        use quire_exact::{Location, NodeKey, Origin, PathSegment, Role, UnitId, ValueType};
        use quire_semantic_value::declaration::EqualitySchedule;

        let mut wire = boolean_equality_wire();
        let ids = &fixture().1;
        let root_node = ids[&'a'].clone();
        let unit_node: CheckedNodeId = serde_json::from_value(json!({
            "package":"composite","path":["fixture","source"],"ordinal":0
        }))
        .expect("source unit id");
        let dimension_node: CheckedNodeId = serde_json::from_value(json!({
            "package":"composite","path":["fixture","dimension"],"ordinal":0
        }))
        .expect("dimension id");
        let base_node: CheckedNodeId = serde_json::from_value(json!({
            "package":"composite","path":["fixture","base"],"ordinal":0
        }))
        .expect("base dimension id");
        let source = wire["lock"]["sources"][0].clone();
        wire["semantic_graph"]["nodes"].as_array_mut().expect("nodes").insert(8, json!({
            "node_id":dimension_node,"schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"scalar_type","semantic_form":"dimension","semantic_type":dimension_node,
            "dependencies":[],"occurrences":[{"role":"declaration","ordinal":0}],
            "owner":{"kind":"source","authority":"test","identity":"composite"},
            "declaration":{"qualified_name":["fixture","dimension"]},
                "body":{"term":"dimension","base":true,"terms":[]}
        }));
        wire["source_map"]
            .as_array_mut()
            .expect("source map")
            .insert(
                8,
                json!({
                    "node_id":dimension_node,"role":"declaration","ordinal":0,
                    "regions":[{"source":source,"start":8,"end":9}]
                }),
            );
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .insert(
                8,
                json!({
                    "node_id":base_node,"schema_version":"quire.checked-semantic-graph/v2",
                    "node_tag":"scalar_type","semantic_form":"dimension","semantic_type":base_node,
                    "dependencies":[],"occurrences":[{"role":"declaration","ordinal":0}],
                    "owner":{"kind":"source","authority":"test","identity":"composite"},
                    "declaration":{"qualified_name":["fixture","base"]},
                    "body":{"term":"dimension","base":true,"terms":[]}
                }),
            );
        wire["source_map"]
            .as_array_mut()
            .expect("source map")
            .insert(
                8,
                json!({
                    "node_id":base_node,"role":"declaration","ordinal":0,
                    "regions":[{"source":source,"start":10,"end":11}]
                }),
            );
        wire["semantic_graph"]["nodes"][9]["dependencies"] = json!([base_node]);
        wire["semantic_graph"]["nodes"][9]["body"] = json!({
            "term":"dimension","base":false,"terms":[{"dimension_node_id":base_node,"exponent":"1"}]
        });
        wire["semantic_graph"]["nodes"][10]["semantic_form"] = json!("unit");
        wire["semantic_graph"]["nodes"][10]["semantic_type"] = json!(dimension_node);
        wire["semantic_graph"]["nodes"][10]["dependencies"] = json!([dimension_node]);
        wire["semantic_graph"]["nodes"][10]["occurrences"][0]["role"] = json!("declaration");
        wire["semantic_graph"]["nodes"][10]["owner"] = json!({
            "kind":"source","authority":"test","identity":"composite"
        });
        wire["semantic_graph"]["nodes"][10]["declaration"] = json!({
            "qualified_name":["fixture","a"]
        });
        wire["semantic_graph"]["nodes"][10]["body"] = json!({
            "term":"unit","dimension_node_id":dimension_node,
            "target_unit_node_id":null,
            "scale":{"numerator":"1","denominator":"1"},
            "offset":{"numerator":"0","denominator":"1"}
        });
        wire["source_map"][10]["role"] = json!("declaration");
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id":unit_node,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"scalar_type","semantic_form":"unit","semantic_type":dimension_node,
                "dependencies":[dimension_node,root_node],
                "occurrences":[{"role":"declaration","ordinal":0}],
                "owner":{"kind":"source","authority":"test","identity":"composite"},
                "declaration":{"qualified_name":["fixture","source"]},
                "body":{"term":"unit","dimension_node_id":dimension_node,
                    "target_unit_node_id":root_node,
                    "scale":{"numerator":"2","denominator":"1"},
                    "offset":{"numerator":"1","denominator":"1"}}
            }));
        let mut source_dependencies = vec![dimension_node.clone(), root_node.clone()];
        source_dependencies.sort();
        wire["semantic_graph"]["nodes"][11]["dependencies"] = json!(source_dependencies);
        wire["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(json!({
                "node_id":unit_node,"role":"declaration","ordinal":0,
                "regions":[{"source":source,"start":11,"end":12}]
            }));
        wire["semantic_graph"]["nodes"][5]["body"]["operation"]["identity"] =
            json!("quire.op.quantity.eq");
        wire["semantic_graph"]["nodes"][5]["body"]["arguments"] = json!([
            {"term":"reference","target":unit_node},
            {"term":"reference","target":unit_node}
        ]);
        wire["semantic_graph"]["nodes"][5]["dependencies"] = json!([unit_node]);
        wire["checked_term_data"]["equality_decisions"][0]["schedule"] = json!("quantity");
        for side in ["left", "right"] {
            wire["checked_term_data"]["equality_decisions"][0][side]["source_type"] =
                json!(unit_node);
        }
        wire["checked_term_data"]["equality_decisions"][0]["units"] = json!([{
            "quantity_type":unit_node,"unit":unit_node,
            "form":{"kind":"declared","dimension_node":dimension_node,
                "dimension_terms":[{"dimension":base_node,"exponent":"1"}],
                "root_unit":root_node,
                "path":[{"target":root_node,
                    "scale":{"numerator":"2","denominator":"1"},
                    "offset":{"numerator":"1","denominator":"1"}}]}
        }]);
        let package = match CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("declared quantity package must admit: {other:?}"),
        };
        let application = id('6');
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let original = NodeKey::new(
            "composite",
            vec![PathSegment::Text("declared-quantity-equality".into())],
            0,
        )
        .expect("app key");
        let unit_key =
            NodeKey::new("composite", vec![PathSegment::Text("unit".into())], 0).expect("unit key");
        let root_key = NodeKey::new("composite", vec![PathSegment::Text("root".into())], 0)
            .expect("root unit key");
        let dimension_key =
            NodeKey::new("composite", vec![PathSegment::Text("dimension".into())], 0)
                .expect("dimension key");
        let base_key = NodeKey::new("composite", vec![PathSegment::Text("base".into())], 0)
            .expect("base dimension key");
        let unit_id = UnitId::declared(unit_key.clone());
        let application_row = super::super::SelectedEqualityApplicationRow::new(
            application.clone(),
            occurrence.clone(),
            serde_json::from_value(wire["source_map"][5].clone()).expect("source row"),
            original.clone(),
            Location::new(original.clone(), Origin::new(Role::from("expression"), 0)),
        );
        let mut facts = Vec::new();
        for position in 0..2 {
            for (fact, key, node, value_type) in [
                (
                    super::super::EqualityFactWitness::OperandType {
                        position,
                        role: super::super::EqualityOperandTypeRole::Source,
                    },
                    unit_key.clone(),
                    unit_node.clone(),
                    Some(ValueType::Quantity(unit_id.clone())),
                ),
                (
                    super::super::EqualityFactWitness::Unit {
                        unit_id: unit_id.clone(),
                        quantity_type: unit_node.clone(),
                        operand_position: position,
                        role: super::super::EqualityUnitRole::Source,
                    },
                    unit_key.clone(),
                    unit_node.clone(),
                    None,
                ),
                (
                    super::super::EqualityFactWitness::DimensionDeclaration {
                        unit_id: unit_id.clone(),
                        operand_position: position,
                        role: super::super::EqualityDimensionRole::UnitDimension,
                        ordinal: 0,
                    },
                    dimension_key.clone(),
                    dimension_node.clone(),
                    None,
                ),
                (
                    super::super::EqualityFactWitness::DimensionDeclaration {
                        unit_id: unit_id.clone(),
                        operand_position: position,
                        role: super::super::EqualityDimensionRole::ConversionDimension,
                        ordinal: 1,
                    },
                    dimension_key.clone(),
                    dimension_node.clone(),
                    None,
                ),
                (
                    super::super::EqualityFactWitness::DimensionDeclaration {
                        unit_id: unit_id.clone(),
                        operand_position: position,
                        role: super::super::EqualityDimensionRole::BaseDimension,
                        ordinal: 1,
                    },
                    base_key.clone(),
                    base_node.clone(),
                    None,
                ),
                (
                    super::super::EqualityFactWitness::DimensionTerm {
                        unit_id: unit_id.clone(),
                        operand_position: position,
                        ordinal: 0,
                        base: base_key.clone(),
                        base_node: base_node.clone(),
                        exponent: "1".into(),
                    },
                    dimension_key.clone(),
                    dimension_node.clone(),
                    None,
                ),
                (
                    super::super::EqualityFactWitness::ConversionEdge {
                        unit_id: unit_id.clone(),
                        operand_position: position,
                        ordinal: 0,
                        target: root_key.clone(),
                        target_node: root_node.clone(),
                        scale: super::super::CheckedRational {
                            numerator: "2".into(),
                            denominator: "1".into(),
                        },
                        offset: super::super::CheckedRational {
                            numerator: "1".into(),
                            denominator: "1".into(),
                        },
                    },
                    unit_key.clone(),
                    unit_node.clone(),
                    None,
                ),
            ] {
                facts.push(super::super::SelectedEqualityFactRow {
                    application: original.clone(),
                    checked_application: application.clone(),
                    occurrence: occurrence.clone(),
                    source_map_entry: application_row.source_map_entry.clone(),
                    structural_key: key,
                    structural_node: node.clone(),
                    owner: package.node(&node).expect("fact node").owner.clone(),
                    value_type,
                    fact,
                });
            }
        }
        let source_row = super::super::SelectedSourceUnitRow::new(
            unit_node.clone(),
            "test",
            "composite",
            vec!["fixture".into(), "source".into()],
            unit_id.clone(),
        );
        let admitted = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&application_row),
                &facts,
                std::slice::from_ref(&source_row),
                1000,
            )
            .expect("joined declared quantity decision");
        assert_eq!(admitted.schedule(), EqualitySchedule::Quantity);
        assert_eq!(admitted.units()[0].id, unit_id);
        let quire_semantic_value::declaration::EqualityUnitForm::Declared { path, .. } =
            &admitted.units()[0].form
        else {
            panic!("declared unit form")
        };
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].target, root_key);
        assert_eq!(path[0].scale.numerator().to_string(), "2");
        assert_eq!(path[0].scale.denominator().to_string(), "1");
        assert_eq!(path[0].offset.numerator().to_string(), "1");
        assert_eq!(path[0].offset.denominator().to_string(), "1");
        let quire_semantic_value::quantity::QuantityUnit::Declared(resolved) = admitted
            .resolved_units()
            .get(unit_id.clone())
            .expect("QSV materialized the declared unit")
        else {
            panic!("QSV declared unit")
        };
        let canonical = resolved.canonical();
        assert_eq!(canonical.scale().numerator().to_string(), "2");
        assert_eq!(canonical.scale().denominator().to_string(), "1");
        assert_eq!(canonical.offset().numerator().to_string(), "1");
        assert_eq!(canonical.offset().denominator().to_string(), "1");
        let mut low = 0;
        let mut high = 1000;
        while low + 1 < high {
            let mid = low + (high - low) / 2;
            match package.admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&application_row),
                &facts,
                std::slice::from_ref(&source_row),
                mid,
            ) {
                Ok(_) => high = mid,
                Err(error)
                    if error.code == super::super::AdmittedEqualityReadErrorCode::WorkLimit =>
                {
                    low = mid;
                }
                Err(error) => panic!("unexpected bounded read refusal: {error:?}"),
            }
        }
        assert!(high > 1);
        let one_less = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&application_row),
                &facts,
                std::slice::from_ref(&source_row),
                high - 1,
            )
            .expect_err("one less than the exact work threshold refuses");
        assert_eq!(
            one_less.code,
            super::super::AdmittedEqualityReadErrorCode::WorkLimit
        );
        assert!(one_less.attempted_work > 0);
        let mut changed = facts.clone();
        let edge = changed
            .iter_mut()
            .find(|row| {
                matches!(
                    row.fact,
                    super::super::EqualityFactWitness::ConversionEdge {
                        operand_position: 0,
                        ..
                    }
                )
            })
            .expect("source edge row");
        let super::super::EqualityFactWitness::ConversionEdge { target_node, .. } = &mut edge.fact
        else {
            panic!("source edge fact");
        };
        *target_node = dimension_node.clone();
        let mismatch = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&application_row),
                &changed,
                std::slice::from_ref(&source_row),
                1000,
            )
            .expect_err("changed conversion target refuses");
        assert_eq!(
            mismatch.code,
            super::super::AdmittedEqualityReadErrorCode::MismatchedEqualityWitness
        );
        assert_eq!(*mismatch.at, unit_node);
        facts.retain(|row| {
            !matches!(
                row.fact,
                super::super::EqualityFactWitness::DimensionDeclaration { .. }
            )
        });
        let missing = package
            .admitted_equality(
                &application,
                &occurrence,
                &[application_row],
                &facts,
                &[source_row],
                1000,
            )
            .expect_err("missing dimension witness refuses");
        assert_eq!(
            missing.code,
            super::super::AdmittedEqualityReadErrorCode::UnavailableEqualityWitness
        );
        assert_eq!(
            missing.witness,
            Some(super::super::EqualityWitnessKind::DimensionDeclaration)
        );
        assert_eq!(*missing.at, dimension_node);
    }

    /// Trace: FR-353-AC-1, FR-353-AC-3, FR-353-AC-7.
    /// Provenance: IR-1209 QSL enum per-member structural rows.
    #[test]
    fn enum_equality_reads_member_rows() {
        use quire_exact::{
            EnumShape, Location, NodeKey, Origin, PathSegment, Role, ValueType, VariantId,
        };
        use quire_semantic_value::declaration::EqualitySchedule;

        let mut wire = boolean_equality_wire();
        let ids = &fixture().1;
        let enum_id = ids[&'a'].clone();
        let member_id: CheckedNodeId = serde_json::from_value(json!({
            "package":"composite","path":["fixture","enum_member"],"ordinal":0
        }))
        .expect("enum member id");
        wire["semantic_graph"]["nodes"][8]["semantic_form"] = json!("enum");
        wire["semantic_graph"]["nodes"][8]["body"] = json!({
            "term":"enum","ordered":false,"members":["Ready"]
        });
        wire["semantic_graph"]["nodes"][8]["owner"] = json!({
            "kind":"source","authority":"test","identity":"composite"
        });
        wire["semantic_graph"]["nodes"][8]["declaration"] = json!({
            "qualified_name":["fixture","a"]
        });
        wire["semantic_graph"]["nodes"][8]["occurrences"][0]["role"] = json!("declaration");
        wire["source_map"][8]["role"] = json!("declaration");
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id":member_id,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"value","semantic_form":"enum_value","semantic_type":enum_id,
                "dependencies":[enum_id],"occurrences":[{"role":"expression","ordinal":0}],
                "body":{"term":"literal","type":enum_id,"value_kind":"enum","value":"Ready"}
            }));
        let source = wire["lock"]["sources"][0].clone();
        wire["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(json!({
                "node_id":member_id,"role":"expression","ordinal":0,
                "regions":[{"source":source,"start":10,"end":11}]
            }));
        wire["semantic_graph"]["nodes"][5]["body"]["operation"]["identity"] =
            json!("quire.op.enum.eq");
        wire["semantic_graph"]["nodes"][5]["body"]["arguments"] = json!([
            {"term":"reference","target":member_id},
            {"term":"reference","target":member_id}
        ]);
        wire["semantic_graph"]["nodes"][5]["dependencies"] = json!([member_id]);
        wire["checked_term_data"]["equality_decisions"][0]["schedule"] = json!("enum");
        for side in ["left", "right"] {
            wire["checked_term_data"]["equality_decisions"][0][side]["source_type"] =
                json!(enum_id);
        }
        wire["checked_term_data"]["equality_decisions"][0]["enum_members"] = json!([{
            "enum_type":enum_id,"member":member_id,"case":"Ready","ordinal":0,"ordered":false
        }]);
        let package = match CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("enum equality package must admit: {other:?}"),
        };
        let application = id('6');
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let original = NodeKey::new(
            "composite",
            vec![PathSegment::Text("enum-equality".into())],
            0,
        )
        .expect("application key");
        let enum_key = NodeKey::new("composite", vec![PathSegment::Text("Choice".into())], 0)
            .expect("enum key");
        let member_key = NodeKey::new(
            "composite",
            vec![
                PathSegment::Text("Choice".into()),
                PathSegment::Text("Ready".into()),
            ],
            0,
        )
        .expect("member key");
        let shape = ValueType::Enum(EnumShape::new(
            false,
            [VariantId::new(enum_key.clone(), "Ready").expect("variant")],
        ));
        let application_row = super::super::SelectedEqualityApplicationRow::new(
            application.clone(),
            occurrence.clone(),
            serde_json::from_value(wire["source_map"][5].clone()).expect("source row"),
            original.clone(),
            Location::new(original.clone(), Origin::new(Role::from("expression"), 0)),
        );
        let mut facts = Vec::new();
        for position in 0..2 {
            facts.push(super::super::SelectedEqualityFactRow {
                application: original.clone(),
                checked_application: application.clone(),
                occurrence: occurrence.clone(),
                source_map_entry: application_row.source_map_entry.clone(),
                structural_key: enum_key.clone(),
                structural_node: enum_id.clone(),
                owner: package.node(&enum_id).expect("enum node").owner.clone(),
                value_type: Some(shape.clone()),
                fact: super::super::EqualityFactWitness::OperandType {
                    position,
                    role: super::super::EqualityOperandTypeRole::Source,
                },
            });
        }
        facts.push(super::super::SelectedEqualityFactRow {
            application: original.clone(),
            checked_application: application.clone(),
            occurrence: occurrence.clone(),
            source_map_entry: application_row.source_map_entry.clone(),
            structural_key: member_key.clone(),
            structural_node: member_id.clone(),
            owner: package.node(&member_id).expect("member node").owner.clone(),
            value_type: Some(shape.clone()),
            fact: super::super::EqualityFactWitness::EnumMember {
                source_declaration: enum_key.clone(),
                source_declaration_node: enum_id.clone(),
                declaration: enum_key.clone(),
                declaration_node: enum_id,
                member: member_key,
                member_node: member_id.clone(),
                case: "Ready".into(),
                ordinal: 0,
                ordered: false,
            },
        });
        let admitted = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&application_row),
                &facts,
                &[],
                1000,
            )
            .expect("joined enum decision");
        assert_eq!(admitted.schedule(), EqualitySchedule::Enum);
        assert_eq!(admitted.left().source(), &shape);
        assert_eq!(admitted.enum_members().len(), 1);
        let super::super::EqualityFactWitness::EnumMember { case, .. } = &mut facts[2].fact else {
            panic!("enum member fact");
        };
        *case = "Changed".into();
        let mismatch = package
            .admitted_equality(
                &application,
                &occurrence,
                &[application_row],
                &facts,
                &[],
                1000,
            )
            .expect_err("changed member case refuses at member node");
        assert_eq!(
            mismatch.code,
            super::super::AdmittedEqualityReadErrorCode::MismatchedEqualityWitness
        );
        assert_eq!(*mismatch.at, member_id);
    }

    /// Trace: FR-353-AC-2.
    #[test]
    fn converted_plan_equality_joins_original_source_and_selected_target() {
        let wire = converted_decimal_equality_wire();
        let admitted = CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        );
        assert!(
            matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
            "converted equality must admit: {admitted:?}"
        );

        let mut wrong_target = wire;
        wrong_target["checked_term_data"]["equality_decisions"][0]["left"]["conversion_target"] =
            json!(fixture().1[&'1']);
        let refused = CheckedPackageV2::read(
            &serde_json::to_vec(&wrong_target).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        );
        let CheckedPackageV2ReadResult::Refused(refusal) = refused else {
            panic!("mismatched conversion target must refuse: {refused:?}");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("/checked_term_data/equality_decisions/0/left/conversion_target")
        );
    }

    /// Trace: FR-353-AC-7.
    #[test]
    fn non_text_bounded_decimal_retains_plan_witness_refusal() {
        use quire_exact::{Location, NodeKey, Origin, PathSegment, Role};

        let wire = converted_decimal_equality_wire();
        let package = match CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("converted Decimal package must admit: {other:?}"),
        };
        let application = id('6');
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let original = NodeKey::new(
            "composite",
            vec![PathSegment::Text("decimal-equality".into())],
            0,
        )
        .expect("original key");
        let row = super::super::SelectedEqualityApplicationRow::new(
            application.clone(),
            occurrence.clone(),
            serde_json::from_value(wire["source_map"][5].clone()).expect("source row"),
            original.clone(),
            Location::new(original, Origin::new(Role::from("expression"), 0)),
        );
        let missing = package
            .admitted_equality(&application, &occurrence, &[row], &[], &[], 1000)
            .expect_err("Decimal bounds need a Plan witness");
        assert_eq!(
            missing.code,
            super::super::AdmittedEqualityReadErrorCode::UnavailableEqualityWitness
        );
        assert_eq!(
            missing.witness,
            Some(super::super::EqualityWitnessKind::PlanType)
        );
        assert_eq!(
            *missing.at,
            serde_json::from_value(
                wire["checked_term_data"]["equality_decisions"][0]["left"]["conversion_target"]
                    .clone()
            )
            .expect("Decimal bound node")
        );
    }

    /// Trace: FR-353-AC-2.
    #[test]
    fn unconverted_integer_equality_keeps_its_null_target() {
        let mut wire = converted_integer_equality_wire();
        let ids = &fixture().1;
        wire["semantic_graph"]["nodes"][5]["body"]["arguments"][0] =
            json!({"term":"reference","target":ids[&'5']});
        wire["semantic_graph"]["nodes"][5]["dependencies"] = json!([ids[&'5'], ids[&'8']]);
        wire["checked_term_data"]["equality_decisions"][0]["left"]["conversion_target"] =
            Value::Null;
        let admitted = CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        );
        assert!(
            matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
            "unconverted integer equality must admit: {admitted:?}"
        );
    }

    /// Trace: FR-353-AC-2, FR-353-AC-3, FR-353-AC-4.
    /// Provenance: IR-1209 scoped application and primitive Plan path.
    #[test]
    fn admitted_equality_joins_application_and_bounds_read() {
        use quire_exact::{Location, NodeKey, Origin, PathSegment, Role, ValueType};
        use quire_semantic_value::declaration::{
            CheckedEquality, EqualityOperator, EqualitySchedule,
        };

        let wire = boolean_equality_wire();
        let package = match CheckedPackageV2::read(
            &serde_json::to_vec(&wire).expect("wire bytes"),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("equality application must admit: {other:?}"),
        };
        let application = id('6');
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let original = NodeKey::new(
            "composite",
            vec![PathSegment::Text("original-equality".into())],
            0,
        )
        .expect("original producer key");
        let row = super::super::SelectedEqualityApplicationRow::new(
            application.clone(),
            occurrence.clone(),
            serde_json::from_value(wire["source_map"][5].clone()).expect("source row"),
            original.clone(),
            Location::new(original.clone(), Origin::new(Role::from("expression"), 0)),
        );
        let admitted = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&row),
                &[],
                &[],
                100,
            )
            .expect("joined Plan decision");
        assert_eq!(admitted.application(), &original);
        assert_eq!(admitted.operator(), EqualityOperator::Equal);
        assert_eq!(admitted.schedule(), EqualitySchedule::Plan);
        assert_eq!(admitted.left().source(), &ValueType::Boolean);
        assert_eq!(admitted.right().source(), &ValueType::Boolean);
        assert_eq!(
            CheckedEquality::from_admitted_decision(&admitted).schedule(),
            EqualitySchedule::Plan
        );

        let mut minimum = 0;
        for limit in 0..=100 {
            if package
                .admitted_equality(
                    &application,
                    &occurrence,
                    std::slice::from_ref(&row),
                    &[],
                    &[],
                    limit,
                )
                .is_ok()
            {
                minimum = limit;
                break;
            }
        }
        assert!(minimum > 0);
        assert_eq!(
            package.admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&row),
                &[],
                &[],
                minimum
            ),
            Ok(admitted.clone())
        );
        let short = package
            .admitted_equality(
                &application,
                &occurrence,
                std::slice::from_ref(&row),
                &[],
                &[],
                minimum - 1,
            )
            .expect_err("one less work unit refuses");
        assert_eq!(
            short.code,
            super::super::AdmittedEqualityReadErrorCode::WorkLimit
        );
        assert_eq!(short.attempted_work, 1);
        assert_eq!(short.at.as_ref(), &id('1'));
        assert_eq!(
            package
                .admitted_equality(&application, &occurrence, &[], &[], &[], 100)
                .unwrap_err()
                .code,
            super::super::AdmittedEqualityReadErrorCode::UnavailableEqualityWitness
        );
        let duplicate = package
            .admitted_equality(
                &application,
                &occurrence,
                &[row.clone(), row.clone()],
                &[],
                &[],
                100,
            )
            .expect_err("duplicate stage rows refuse");
        assert_eq!(
            duplicate.code,
            super::super::AdmittedEqualityReadErrorCode::DuplicateEqualityWitness
        );
        let mut changed_location = row.clone();
        changed_location.location = Location::new(
            NodeKey::new("composite", vec![PathSegment::Text("other".into())], 0)
                .expect("other original key"),
            Origin::new(Role::from("expression"), 0),
        );
        let mismatch = package
            .admitted_equality(
                &application,
                &occurrence,
                &[changed_location],
                &[],
                &[],
                100,
            )
            .expect_err("conflicting original location refuses");
        assert_eq!(
            mismatch.code,
            super::super::AdmittedEqualityReadErrorCode::MismatchedEqualityWitness
        );
        let mut foreign = row;
        foreign.application.package = "foreign".into();
        let foreign = package
            .admitted_equality(&application, &occurrence, &[foreign], &[], &[], 100)
            .expect_err("foreign stage row refuses");
        assert_eq!(
            foreign.code,
            super::super::AdmittedEqualityReadErrorCode::ForeignEqualityWitness
        );
        let wrong_occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 1,
        };
        assert_eq!(
            package
                .admitted_equality(&application, &wrong_occurrence, &[], &[], &[], 100)
                .expect_err("wrong occurrence refuses")
                .code,
            super::super::AdmittedEqualityReadErrorCode::WrongOccurrence
        );
        assert_eq!(
            self::package()
                .admitted_equality(&application, &occurrence, &[], &[], &[], 100)
                .expect_err("application without producer decision refuses")
                .code,
            super::super::AdmittedEqualityReadErrorCode::UnavailableEqualityDecision
        );
    }

    /// Trace: FR-357-AC-5, FR-357-AC-6, FR-357-AC-7.
    #[test]
    fn selected_facts_keep_opaque_equality_and_native_refusal() {
        use quire_exact::{Location, NodeKey, Origin, PathSegment, Role};

        let wire = boolean_equality_wire();
        let package = match CheckedPackageV2::read(
            &serde_json::to_vec(&wire).unwrap(),
            CheckedPackageReadLimits::bounded(),
            &CheckedPackageEvidence::new(),
        ) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("equality fixture must admit: {other:?}"),
        };
        let application = id('6');
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let original = NodeKey::new(
            "composite",
            vec![PathSegment::Text("selected-equality".into())],
            0,
        )
        .unwrap();
        let row = super::super::SelectedEqualityApplicationRow::new(
            application.clone(),
            occurrence.clone(),
            serde_json::from_value(wire["source_map"][5].clone()).unwrap(),
            original.clone(),
            Location::new(original.clone(), Origin::new(Role::from("expression"), 0)),
        );
        let id = super::super::SelectedFactEqualityId {
            application: application.clone(),
            occurrence: occurrence.clone(),
        };
        let ids = || super::super::SelectedFactIds {
            declarations: &[],
            fields: &[],
            equalities: std::slice::from_ref(&id),
        };
        let rows = |application_rows| super::super::SelectedFactRows {
            declarations: &[],
            fields: &[],
            source_units: &[],
            equality_applications: application_rows,
            equality_facts: &[],
        };
        let application_rows = [row.clone()];
        let complete = package
            .project_selected_facts(
                ids(),
                rows(&application_rows),
                100_000,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
            .unwrap();
        assert!(complete.declarations.is_empty());
        assert!(complete.fields.is_empty());
        assert_eq!(complete.equalities.len(), 1);
        assert_eq!(complete.equalities[0].id, id);
        assert_eq!(complete.equalities[0].decision.application(), &original);
        assert_eq!(
            package.clone().project_selected_facts(
                ids(),
                rows(&application_rows),
                complete.work,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX)
            ),
            Ok(complete.clone()),
        );
        let short = package
            .project_selected_facts(
                ids(),
                rows(&application_rows),
                complete.work - 1,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
            .expect_err("one below complete allowance refuses");
        assert_eq!(short.code, super::super::SelectedFactsErrorCode::Equality);
        assert_eq!(
            short.equality.as_ref().unwrap().code,
            super::super::AdmittedEqualityReadErrorCode::WorkLimit
        );
        assert_eq!(short.occurrence, Some(occurrence));
        let missing = package
            .project_selected_facts(
                ids(),
                rows(&[]),
                100_000,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
            .unwrap_err();
        assert_eq!(
            missing.code,
            super::super::SelectedFactsErrorCode::MissingRow
        );
        let mut wrong_location = row.clone();
        wrong_location.location = Location::new(
            NodeKey::new("other", vec![PathSegment::Text("wrong".into())], 0).unwrap(),
            Origin::new(Role::from("expression"), 0),
        );
        let mismatched_rows = [wrong_location];
        let mismatch = package
            .project_selected_facts(
                ids(),
                rows(&mismatched_rows),
                100_000,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
            .unwrap_err();
        assert_eq!(
            mismatch.code,
            super::super::SelectedFactsErrorCode::Equality
        );
        assert_eq!(
            mismatch.equality.as_ref().unwrap().code,
            super::super::AdmittedEqualityReadErrorCode::MismatchedEqualityWitness
        );
        let mut unrelated = row.clone();
        unrelated.application = self::id('1');
        let more_rows = [unrelated, row.clone()];
        let more = package
            .project_selected_facts(
                ids(),
                rows(&more_rows),
                100_000,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
            .unwrap();
        assert_eq!(more.equalities, complete.equalities);
        assert!(more.work > complete.work);
        let duplicate_rows = [row.clone(), row];
        let duplicate = package
            .project_selected_facts(
                ids(),
                rows(&duplicate_rows),
                100_000,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
            .unwrap_err();
        assert_eq!(
            duplicate.code,
            super::super::SelectedFactsErrorCode::DuplicateRow
        );
        let mut foreign = id.clone();
        foreign.application.package = "foreign".into();
        let foreign_error = package
            .project_selected_facts(
                super::super::SelectedFactIds {
                    declarations: &[],
                    fields: &[],
                    equalities: &[foreign.clone()],
                },
                rows(&application_rows),
                100_000,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
            .unwrap_err();
        assert_eq!(
            foreign_error.code,
            super::super::SelectedFactsErrorCode::ForeignNode
        );
        assert_eq!(foreign_error.at.as_ref(), &foreign.application);
        let mut wrong_occurrence = id.clone();
        wrong_occurrence.occurrence.ordinal = 1;
        let wrong_occurrence_error = package
            .project_selected_facts(
                super::super::SelectedFactIds {
                    declarations: &[],
                    fields: &[],
                    equalities: &[wrong_occurrence],
                },
                rows(&application_rows),
                100_000,
                &mut crate::NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
            .unwrap_err();
        assert_eq!(
            wrong_occurrence_error.code,
            super::super::SelectedFactsErrorCode::Equality
        );
        assert_eq!(
            wrong_occurrence_error.equality.as_ref().unwrap().code,
            super::super::AdmittedEqualityReadErrorCode::WrongOccurrence
        );
    }

    /// Trace: FR-353-AC-2.
    /// Provenance: IR-1209 application and operand admission join.
    #[test]
    fn equality_sidecar_refuses_first_mismatching_member() {
        use crate::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};

        let baseline = boolean_equality_wire();
        let cases: Vec<(&str, Value, &str, CheckedPackageRefusalCause)> = vec![
            (
                "nonempty path",
                {
                    let mut wire = baseline.clone();
                    wire["checked_term_data"]["equality_decisions"][0]["term_path"] =
                        json!([{"kind":"argument","index":0}]);
                    wire
                },
                "/checked_term_data/equality_decisions/0/term_path/0",
                CheckedPackageRefusalCause::InvalidValue,
            ),
            (
                "wrong operator",
                {
                    let mut wire = baseline.clone();
                    wire["checked_term_data"]["equality_decisions"][0]["operator"] =
                        json!("not_equal");
                    wire
                },
                "/checked_term_data/equality_decisions/0/operator",
                CheckedPackageRefusalCause::InvalidValue,
            ),
            (
                "wrong position",
                {
                    let mut wire = baseline.clone();
                    wire["checked_term_data"]["equality_decisions"][0]["left"]["operand"] =
                        json!(1);
                    wire
                },
                "/checked_term_data/equality_decisions/0/left/operand",
                CheckedPackageRefusalCause::InvalidValue,
            ),
            (
                "wrong source type",
                {
                    let mut wire = baseline.clone();
                    wire["checked_term_data"]["equality_decisions"][0]["right"]["source_type"] =
                        json!(id('2'));
                    wire
                },
                "/checked_term_data/equality_decisions/0/right/source_type",
                CheckedPackageRefusalCause::InvalidValue,
            ),
            (
                "wrong occurrence",
                {
                    let mut wire = baseline.clone();
                    wire["checked_term_data"]["equality_decisions"][0]["occurrence"]["ordinal"] =
                        json!(1);
                    wire
                },
                "/checked_term_data/equality_decisions/0/occurrence",
                CheckedPackageRefusalCause::InvalidValue,
            ),
            (
                "wrong application",
                {
                    let mut wire = baseline.clone();
                    wire["checked_term_data"]["equality_decisions"][0]["application"] =
                        json!(id('5'));
                    wire
                },
                "/checked_term_data/equality_decisions/0/application",
                CheckedPackageRefusalCause::InvalidValue,
            ),
            (
                "duplicate",
                {
                    let mut wire = baseline.clone();
                    let row = wire["checked_term_data"]["equality_decisions"][0].clone();
                    wire["checked_term_data"]["equality_decisions"]
                        .as_array_mut()
                        .expect("rows")
                        .push(row);
                    wire
                },
                "/checked_term_data/equality_decisions/1",
                CheckedPackageRefusalCause::ConflictingDefinition,
            ),
        ];
        for (name, wire, path, cause) in cases {
            let result = CheckedPackageV2::read(
                &serde_json::to_vec(&wire).expect("wire bytes"),
                CheckedPackageReadLimits::bounded(),
                &CheckedPackageEvidence::new(),
            );
            let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                panic!("{name} must refuse at admission: {result:?}");
            };
            assert_eq!(
                refusal.code,
                CheckedPackageRefusalCode::InvalidPackage,
                "{name}"
            );
            assert_eq!(refusal.cause, Some(cause), "{name}");
            assert_eq!(
                refusal.loci[0]
                    .path
                    .as_ref()
                    .expect("primary pointer")
                    .to_string(),
                path,
                "{name}"
            );
        }
    }

    // Start from a real reader admission; edit only at the expressly private
    // post-admission seam, where malformed wrapper controls are reachable.
    fn narrow_record(lower: &str, upper: &str, literal: &str) -> CheckedPackageV2 {
        let mut p = package();
        let set = |p: &mut CheckedPackageV2, position: usize, kind, semantic_type, body: Value| {
            std::sync::Arc::make_mut(&mut p.kinds)[position] = kind;
            std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[position].semantic_type =
                semantic_type;
            std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[position].body =
                CheckedTerm::decode_admitted(&body).expect("closed private control term");
            sync_term(p, position);
        };
        let integer_literal = |text: &str| json!({"term":"literal","type":id('2'),"value_kind":"integer","value":text});
        set(
            &mut p,
            6,
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange),
            id('2'),
            json!({"term":"aggregate","members":[
            {"term":"binding","name":"min","value":integer_literal(lower)},
            {"term":"binding","name":"max","value":integer_literal(upper)}]}),
        );
        set(
            &mut p,
            8,
            CheckedNodeKind::Value(ValueForm::Literal),
            id('2'),
            integer_literal(literal),
        );
        set(
            &mut p,
            7,
            CheckedNodeKind::Expression(super::super::ExpressionForm::Conversion),
            id('7'),
            json!({"term":"application","operator":"convert",
            "operation":{"identity":"quire.op.numeric.narrow","laws":[],"leaves":[],"mode":null,"member":{"kind":"type_argument","declaration":id('7')}},
            "result_type":id('7'),"arguments":[{"term":"reference","target":id('a')}]}),
        );
        set(
            &mut p,
            3,
            CheckedNodeKind::CompositeType(CompositeTypeForm::Record),
            id('4'),
            json!({"term":"aggregate","members":[{"term":"binding","name":"x","value":{"term":"reference","target":id('7')}}]}),
        );
        set(
            &mut p,
            4,
            CheckedNodeKind::Value(ValueForm::RecordValue),
            id('4'),
            json!({"term":"aggregate","members":[{"term":"binding","name":"x","value":{"term":"reference","target":id('8')}}]}),
        );
        application_arguments(&mut p)[1] = CheckedTerm::Reference { target: id('5') };
        p
    }

    #[trace("TC-048", "FR-038-AC-232")]
    #[test]
    fn narrow_closed_record_preserves_original_graph_child_and_signed_arbitrary_precision_bounds() {
        for (lower, upper, values) in [
            ("0", "9", vec!["0", "3", "9"]),
            (
                "340282366920938463463374607431768211456",
                "340282366920938463463374607431768211459",
                vec![
                    "340282366920938463463374607431768211456",
                    "340282366920938463463374607431768211459",
                ],
            ),
            (
                "-340282366920938463463374607431768211459",
                "-340282366920938463463374607431768211456",
                vec![
                    "-340282366920938463463374607431768211459",
                    "-340282366920938463463374607431768211456",
                ],
            ),
        ] {
            for value in values {
                let p = narrow_record(lower, upper, value);
                let operands = read(&p).expect("closed narrow field");
                for operand in operands.operands {
                    assert_eq!(
                        operand.child,
                        CheckedScalarOperandChild::GraphChild(id('5'))
                    );
                    assert_eq!(operand.semantic_type, id('4'));
                    assert_eq!(operand.domain, CheckedCompositeOperandDomain::Literal);
                }
            }
        }
    }

    #[trace("TC-048")]
    #[test]
    fn narrow_rejects_outside_signed_large_bounds_without_rounding() {
        for (lower, upper, outside) in [
            (
                "340282366920938463463374607431768211456",
                "340282366920938463463374607431768211459",
                [
                    "340282366920938463463374607431768211455",
                    "340282366920938463463374607431768211460",
                ],
            ),
            (
                "-340282366920938463463374607431768211459",
                "-340282366920938463463374607431768211456",
                [
                    "-340282366920938463463374607431768211460",
                    "-340282366920938463463374607431768211455",
                ],
            ),
        ] {
            for value in outside {
                assert_eq!(
                    read(&narrow_record(lower, upper, value)),
                    Err(type_error(0, Some(id('7'))))
                );
            }
        }
    }

    #[trace("TC-048")]
    #[test]
    fn narrow_exact_budget_and_one_less_preserve_first_failure() {
        let p = narrow_record("0", "9", "3");
        let success = read(&p).expect("closed narrow");
        // One selected application resolution, then per operand: argument,
        // child and family resolutions (3); root resolution+visit (2); the
        // record member, binding and reference edges (3); narrow resolution+
        // visit+wrapper (3); range and Integer resolution+visit (4); two range
        // member+binding edges (4); narrow argument edge (1); literal
        // resolution+visit (2); and each of the three decimal bytes (3).
        assert_eq!(
            success.consumed_work,
            1 + 2 * (3 + 2 + 3 + 3 + 4 + 4 + 1 + 2 + 3)
        );
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        assert_eq!(
            p.composite_application_operands(&id('6'), &occurrence, success.consumed_work),
            Ok(success.clone())
        );
        assert!(matches!(
            p.composite_application_operands(&id('6'), &occurrence, success.consumed_work - 1),
            Err(Error::WorkLimit { .. })
        ));
        assert_eq!(
            p.composite_application_operands(&id('6'), &occurrence, 0),
            Err(Error::WorkLimit {
                limit: 0,
                consumed: 1
            })
        );
    }

    #[trace("TC-048")]
    #[test]
    fn narrow_wrapper_range_and_literal_defects_keep_exact_typed_loci() {
        // Each candidate starts from the same admitted-then-mutated control;
        // only one independently named condition changes per iteration.
        for (pointer, replacement) in [
            ("/operator", json!("binary")),
            (
                "/operation/laws",
                json!([{"role":"integer_division","definition":{"authority":"test","identity":"law"}}]),
            ),
            ("/operation/leaves", json!([{}])),
            (
                "/operation/mode",
                json!({"kind":"rounding","value":"floor"}),
            ),
            ("/operation/member/kind", json!("element")),
            ("/operation/member/declaration", json!(id('2'))),
            ("/result_type", json!(id('2'))),
            ("/arguments", json!([])),
            (
                "/arguments",
                json!([{"term":"literal","type":id('2'),"value_kind":"integer","value":"3"}]),
            ),
        ] {
            let mut p = narrow_record("0", "9", "3");
            edit_body(&mut p, 7, |body| {
                *body.pointer_mut(pointer).expect("wrapper field") = replacement
            });
            sync_term(&mut p, 7);
            assert_eq!(read(&p), Err(type_error(0, Some(id('7')))), "{pointer}");
        }
        let mut p = narrow_record("0", "9", "3");
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[7].semantic_type = id('2');
        assert_eq!(read(&p), Err(type_error(0, Some(id('2')))));
        for (pointer, replacement) in [
            ("/members", json!([])),
            ("/members/0/name", json!("max")),
            ("/members/0/value", json!({"term":"aggregate","members":[]})),
            ("/members/0/value/type", json!(id('1'))),
            ("/members/0/value/value_kind", json!("text")),
            ("/members/0/value/value", json!("00")),
            ("/members/0/value/value", json!("10")),
        ] {
            let mut p = narrow_record("0", "9", "3");
            edit_body(&mut p, 6, |body| {
                *body.pointer_mut(pointer).expect("range field") = replacement
            });
            sync_term(&mut p, 6);
            assert_eq!(read(&p), Err(type_error(0, Some(id('7')))), "{pointer}");
        }
        let mut p = narrow_record("0", "9", "3");
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[6].semantic_type = id('1');
        assert_eq!(read(&p), Err(type_error(0, Some(id('7')))));
        for (pointer, replacement, locus) in [
            ("/value_kind", json!("text"), id('2')),
            ("/type", json!(id('1')), id('1')),
            ("/value", json!(3), id('2')),
            ("/value", json!("03"), id('2')),
            ("/value", json!("-0"), id('2')),
        ] {
            let mut p = narrow_record("0", "9", "3");
            edit_body(&mut p, 8, |body| {
                *body.pointer_mut(pointer).expect("literal field") = replacement
            });
            sync_term(&mut p, 8);
            assert_eq!(read(&p), Err(type_error(0, Some(locus))), "{pointer}");
        }
        let mut p = narrow_record("0", "9", "3");
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[8].semantic_type = id('1');
        assert_eq!(read(&p), Err(type_error(0, Some(id('2')))));
    }

    #[trace("TC-048", "FR-038-AC-234")]
    #[test]
    fn narrow_exception_never_evaluates_parameters_nested_applications_or_cycles() {
        for (target, expected_type) in [('5', '4'), ('8', '7'), ('6', '1')] {
            let mut p = narrow_record("0", "9", "3");
            edit_body(&mut p, 7, |body| {
                body["arguments"][0]["target"] = json!(id(target))
            });
            sync_term(&mut p, 7);
            assert_eq!(
                read(&p),
                Err(unsupported(
                    0,
                    Some(id(expected_type)),
                    CheckedUnsupportedCompositeOperand::NonliteralGraphValue
                ))
            );
        }
        let mut p = narrow_record("0", "9", "3");
        std::sync::Arc::make_mut(&mut p.kinds)[8] = CheckedNodeKind::Value(ValueForm::Parameter);
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('2')),
                CheckedUnsupportedCompositeOperand::NonliteralGraphValue
            ))
        );
        let mut p = narrow_record("0", "9", "3");
        edit_body(&mut p, 7, |body| {
            body["operation"]["identity"] = json!("quire.op.quantity.convert")
        });
        sync_term(&mut p, 7);
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('7')),
                CheckedUnsupportedCompositeOperand::NonliteralGraphValue
            ))
        );
    }

    #[trace("TC-048")]
    #[test]
    fn narrow_full_decimal_bytes_and_graph_order_have_independent_exact_work() {
        let lower = format!("-{}", "9".repeat(300));
        let upper = "8".repeat(400);
        let p = narrow_record(&lower, &upper, "0");
        let expected = 1 + 2 * (22 + lower.len() as u64 + upper.len() as u64 + 1);
        let result = read(&p).expect("long endpoints checked without narrowing");
        assert_eq!(result.consumed_work, expected);
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        assert_eq!(
            p.composite_application_operands(&id('6'), &occurrence, expected),
            Ok(result.clone())
        );
        assert!(matches!(
            p.composite_application_operands(&id('6'), &occurrence, expected - 1),
            Err(Error::WorkLimit { .. })
        ));
        let mut permuted = p;
        std::sync::Arc::make_mut(&mut permuted.wire)
            .semantic_graph
            .nodes
            .reverse();
        std::sync::Arc::make_mut(&mut permuted.kinds).reverse();
        std::sync::Arc::make_mut(&mut permuted.terms).reverse();
        permuted.node_index = permuted
            .wire
            .semantic_graph
            .nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (node.node_id.clone(), position))
            .collect::<std::collections::BTreeMap<_, _>>()
            .into();
        assert_eq!(read(&permuted), Ok(result));
    }

    #[trace("TC-048")]
    #[test]
    fn narrow_work_exhaustion_precedes_wrapper_membership_and_later_operand_errors() {
        let occurrence = CheckedOccurrence {
            role: CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let mut malformed = narrow_record("0", "9", "3");
        edit_body(&mut malformed, 7, |body| body["operator"] = json!("binary"));
        sync_term(&mut malformed, 7);
        assert_eq!(
            malformed.composite_application_operands(&id('6'), &occurrence, 11),
            Err(Error::WorkLimit {
                limit: 11,
                consumed: 12
            })
        );
        let mut outside = narrow_record("0", "9", "4");
        // Make the inclusive maximum smaller without changing decimal lengths.
        edit_body(&mut outside, 6, |body| {
            body["members"][1]["value"]["value"] = json!("3")
        });
        sync_term(&mut outside, 6);
        assert_eq!(
            outside.composite_application_operands(&id('6'), &occurrence, 25),
            Err(Error::WorkLimit {
                limit: 25,
                consumed: 26
            })
        );
        let mut later = narrow_record("0", "9", "3");
        application_arguments(&mut later)[1] = CheckedTerm::Literal {
            type_id: id('2'),
            value_kind: LiteralKind::Integer,
            value: json!("1"),
        };
        assert_eq!(
            later.composite_application_operands(&id('6'), &occurrence, 26),
            Err(Error::WorkLimit {
                limit: 26,
                consumed: 27
            })
        );
    }

    #[trace("TC-048", "FR-038-AC-233")]
    #[test]
    fn narrow_all_malformed_controls_preserve_signed_bounds_and_first_typed_locus() {
        narrow_wrapper_range_and_literal_defects_keep_exact_typed_loci();
        narrow_closed_record_preserves_original_graph_child_and_signed_arbitrary_precision_bounds();
        narrow_rejects_outside_signed_large_bounds_without_rounding();
        let mut absent = narrow_record("0", "9", "3");
        std::sync::Arc::make_mut(&mut absent.node_index).remove(&id('7'));
        assert_eq!(read(&absent), Err(type_error(0, Some(id('7')))));
        let mut wrong_kind = narrow_record("0", "9", "3");
        std::sync::Arc::make_mut(&mut wrong_kind.kinds)[6] =
            CheckedNodeKind::CompositeType(CompositeTypeForm::Option);
        assert_eq!(read(&wrong_kind), Err(type_error(0, Some(id('7')))));
    }

    fn application_arguments(package: &mut CheckedPackageV2) -> &mut Vec<CheckedTerm> {
        let CheckedTerm::Application { arguments, .. } =
            &mut std::sync::Arc::make_mut(&mut package.terms)[5]
        else {
            panic!("admitted composite application term");
        };
        arguments
    }

    fn application_operation(package: &mut CheckedPackageV2) -> &mut Value {
        let CheckedTerm::Application { operation, .. } =
            &mut std::sync::Arc::make_mut(&mut package.terms)[5]
        else {
            panic!("admitted composite application term");
        };
        operation.record_mut()
    }

    fn sync_term(package: &mut CheckedPackageV2, position: usize) {
        std::sync::Arc::make_mut(&mut package.terms)[position] =
            package.wire.semantic_graph.nodes[position].body.clone();
    }

    fn edit_body(package: &mut CheckedPackageV2, position: usize, edit: impl FnOnce(&mut Value)) {
        let mut body = serde_json::to_value(&package.wire.semantic_graph.nodes[position].body)
            .expect("typed fixture body");
        edit(&mut body);
        std::sync::Arc::make_mut(&mut package.wire)
            .semantic_graph
            .nodes[position]
            .body =
            CheckedTerm::decode_admitted(&body).expect("edited fixture body remains a closed term");
    }

    #[trace("FR-349-AC-2")]
    #[test]
    fn composite_uses_retained_application_after_raw_body_changes() {
        let mut package = package();
        let expected = read(&package).expect("admitted composite operands");
        std::sync::Arc::make_mut(&mut package.wire)
            .semantic_graph
            .nodes[5]
            .body = CheckedTerm::aggregate(Vec::new());
        assert_eq!(read(&package), Ok(expected));
    }

    #[trace("FR-349-AC-2")]
    #[test]
    fn alias_forwarding_uses_retained_term_after_raw_body_changes() {
        let mut package = package();
        std::sync::Arc::make_mut(&mut package.kinds)[6] =
            CheckedNodeKind::CompositeType(CompositeTypeForm::Alias);
        std::sync::Arc::make_mut(&mut package.wire)
            .semantic_graph
            .nodes[6]
            .body = CheckedTerm::aggregate(Vec::new());
        let graph = Graph {
            nodes: &package.wire.semantic_graph.nodes,
            kinds: &package.kinds,
            index: &package.node_index,
            terms: &package.terms,
        };
        let mut meter = Meter {
            limit: 100_000,
            consumed: 0,
        };
        assert_eq!(
            descriptor_kind(&graph, &id('7'), 0, &mut meter),
            Ok(CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
        );
    }

    #[trace("TC-048", "FR-038-AC-180")]
    #[test]
    fn ieee_type_walk_keeps_authored_whole_sources_without_claiming_comparison_admission() {
        // An actual reader-admitted package supplies the graph vocabulary;
        // the three extra types are exercised only at the private type-walk
        // seam because a structural comparison reaching IEEE cannot admit.
        let admitted = package();
        let mut nodes = admitted.wire.semantic_graph.nodes.clone();
        let mut kinds = admitted.kinds.as_ref().clone();
        let mut index = admitted.node_index.as_ref().clone();
        let template = nodes[1].clone();
        let mut append = |tag: &str,
                          form: &str,
                          semantic_type: Option<&CheckedNodeId>,
                          kind: CheckedNodeKind| {
            let id = CheckedNodeId {
                package: "composite".into(),
                path: vec![
                    super::super::CheckedNodePathSegment::Name(tag.into()),
                    super::super::CheckedNodePathSegment::Name(form.into()),
                ],
                ordinal: nodes.len() as u64,
            };
            let mut node = template.clone();
            node.node_id = id.clone();
            node.node_tag = tag.into();
            node.semantic_form = form.into();
            node.semantic_type = semantic_type.cloned().unwrap_or_else(|| id.clone());
            node.dependencies = semantic_type.into_iter().cloned().collect();
            node.body = CheckedTerm::aggregate(Vec::new());
            index.insert(id.clone(), nodes.len());
            nodes.push(node);
            kinds.push(kind);
            id
        };
        let float32 = append(
            "scalar_type",
            "float32",
            None,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Float32),
        );
        let float64 = append(
            "scalar_type",
            "float64",
            None,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Float64),
        );
        let rounding = append(
            "bounded_domain",
            "float_rounding",
            Some(&float32),
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::FloatRounding),
        );
        let terms = nodes
            .iter()
            .map(|node| node.body.clone())
            .collect::<Vec<_>>();
        let graph = Graph {
            nodes: &nodes,
            kinds: &kinds,
            index: &index,
            terms: &terms,
        };
        let root = id('5');
        for (ty, kind, expected_shape) in [
            (
                float32.clone(),
                CheckedNodeKind::ScalarType(ScalarTypeForm::Float32),
                1,
            ),
            (
                float64,
                CheckedNodeKind::ScalarType(ScalarTypeForm::Float64),
                1,
            ),
            (
                rounding.clone(),
                CheckedNodeKind::BoundedDomain(BoundedDomainForm::FloatRounding),
                2,
            ),
        ] {
            let mut meter = Meter {
                limit: 100_000,
                consumed: 0,
            };
            let (shape, positions) =
                project_type(&graph, &root, &ty, 0, &mut meter).expect("type-walk mapping");
            assert_eq!(shape.len(), expected_shape);
            assert_eq!(shape[0].type_node, ty);
            assert_eq!(shape[0].kind, kind);
            assert_eq!(shape[0].path, Vec::<u32>::new());
            assert_eq!(
                positions,
                vec![CheckedCompositeDomainPosition {
                    key: CheckedCompositeDomainKey::Node {
                        node: root.clone(),
                        path: vec![]
                    },
                    type_node: ty,
                    authored: CheckedAuthoredCompositeDomain::Whole { kind },
                }]
            );
            if kind == CheckedNodeKind::BoundedDomain(BoundedDomainForm::FloatRounding) {
                assert_eq!(shape[0].edges.len(), 1);
                assert_eq!(shape[0].edges[0].target, float32);
                assert_eq!(shape[0].edges[0].ordinal, None);
                assert_eq!(shape[1].type_node, float32);
                assert_eq!(
                    shape[1].kind,
                    CheckedNodeKind::ScalarType(ScalarTypeForm::Float32)
                );
                assert_eq!(shape[1].path, Vec::<u32>::new());
                assert!(shape[1].edges.is_empty());
            } else {
                assert!(shape[0].edges.is_empty());
            }
        }
    }

    #[trace("TC-048", "FR-038-AC-181")]
    #[test]
    fn defensive_target_failures_never_salvage_or_fabricate_a_child_identity() {
        let mut p = package();
        application_arguments(&mut p)[0] = CheckedTerm::Aggregate { members: vec![] };
        assert_eq!(read(&p), Err(Error::MalformedChild { ordinal: 0 }));
        let mut p = package();
        application_arguments(&mut p)[0] = CheckedTerm::Reference { target: id('9') };
        assert_eq!(
            read(&p),
            Err(Error::MissingChild {
                ordinal: 0,
                child: id('9')
            })
        );
        application_arguments(&mut p)[1] = CheckedTerm::Literal {
            type_id: id('2'),
            value_kind: LiteralKind::Integer,
            value: json!("1"),
        };
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

    #[trace("TC-048", "FR-038-AC-181", "FR-038-AC-178")]
    #[test]
    fn defensive_operand_and_catalog_failures_keep_their_original_typed_loci() {
        let mut p = package();
        application_operation(&mut p)["identity"] = json!("unknown.operation");
        assert_eq!(
            read(&p),
            Err(Error::UnknownOperator {
                application: id('6')
            })
        );
        let mut p = package();
        application_arguments(&mut p)[0] = CheckedTerm::Literal {
            type_id: id('2'),
            value_kind: LiteralKind::Integer,
            value: json!("1"),
        };
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('2')),
                CheckedUnsupportedCompositeOperand::InlineInteger
            ))
        );
        let mut p = package();
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[4].semantic_type = id('1');
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('1')),
                CheckedUnsupportedCompositeOperand::NonStructuralType
            ))
        );
        // The compound-unit node is present in the genuinely admitted fixture;
        // only the parameter's type is changed after admission.
        let mut p = package();
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[4].semantic_type = id('a');
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('a')),
                CheckedUnsupportedCompositeOperand::NonStructuralType
            ))
        );
        // A bounded-domain chain that enters a cycle reports the actual repeated
        // type, rather than the parameter's original entry type.
        let mut p = package();
        for index in [3, 6] {
            std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[index].semantic_form =
                "integer_range".into();
            std::sync::Arc::make_mut(&mut p.kinds)[index] =
                CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange);
            std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[index].semantic_type =
                id('7');
        }
        assert_eq!(read(&p), Err(type_error(0, Some(id('7')))));
        let mut p = package();
        let mut application = p.terms[5].clone();
        let CheckedTerm::Application { result_type, .. } = &mut application else {
            panic!("fixture application");
        };
        *result_type = id('1');
        std::sync::Arc::make_mut(&mut p.terms)[4] = application;
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('1')),
                CheckedUnsupportedCompositeOperand::ApplicationSubterm
            ))
        );
        let mut p = package();
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[4].semantic_type = id('9');
        assert_eq!(read(&p), Err(type_error(0, Some(id('9')))));
    }

    // A valid first operand makes these independent second-argument defects
    // kill any implementation that hard-codes the enclosing ordinal to zero.
    #[trace(
        "TC-048",
        "FR-038-AC-181",
        "FR-038-AC-178",
        "FR-038-AC-180",
        "FR-038-AC-199"
    )]
    #[test]
    fn tc_048_second_operand_refusals_retain_nonzero_ordinals_and_type_loci() {
        second_operand_refusals(package());
        second_operand_refusals(ne_package());
    }

    fn second_operand_refusals(original: CheckedPackageV2) {
        let mut p = original.clone();
        application_arguments(&mut p)[1] = CheckedTerm::Reference { target: id('9') };
        assert_eq!(
            read(&p),
            Err(Error::MissingChild {
                ordinal: 1,
                child: id('9')
            })
        );
        let mut p = original.clone();
        let mut application = p.terms[5].clone();
        let CheckedTerm::Application { result_type, .. } = &mut application else {
            panic!("fixture application");
        };
        *result_type = id('2');
        application_arguments(&mut p)[1] = application;
        assert_eq!(
            read(&p),
            Err(unsupported(
                1,
                Some(id('2')),
                CheckedUnsupportedCompositeOperand::ApplicationSubterm
            ))
        );
        let mut p = original.clone();
        application_arguments(&mut p)[1] = CheckedTerm::Literal {
            type_id: id('2'),
            value_kind: LiteralKind::Integer,
            value: json!("1"),
        };
        assert_eq!(
            read(&p),
            Err(unsupported(
                1,
                Some(id('2')),
                CheckedUnsupportedCompositeOperand::InlineInteger
            ))
        );
        let mut p = original.clone();
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[6].semantic_form =
            "union".into();
        std::sync::Arc::make_mut(&mut p.kinds)[6] =
            CheckedNodeKind::CompositeType(CompositeTypeForm::Union);
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[7].semantic_type = id('7');
        assert_eq!(
            read(&p),
            Err(Error::UnsupportedDomain {
                ordinal: 1,
                type_node: id('7')
            })
        );
        let mut p = original.clone();
        edit_body(&mut p, 6, |body| {
            body["members"][0]["target"] = json!(id('7'))
        });
        sync_term(&mut p, 6);
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[7].semantic_type = id('7');
        assert_eq!(read(&p), Err(type_error(1, Some(id('7')))));
        let mut p = original.clone();
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[7].semantic_type = id('1');
        assert_eq!(
            read(&p),
            Err(unsupported(
                1,
                Some(id('1')),
                CheckedUnsupportedCompositeOperand::NonStructuralType
            ))
        );
        let mut p = original.clone();
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[6].semantic_form =
            "record".into();
        std::sync::Arc::make_mut(&mut p.kinds)[6] =
            CheckedNodeKind::CompositeType(CompositeTypeForm::Record);
        edit_body(
            &mut p,
            6,
            |body| *body = json!({"term":"aggregate","members":[{"term":"binding","name":"bad","value":{"term":"aggregate","members":[]}}]}),
        );
        sync_term(&mut p, 6);
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[7].semantic_type = id('7');
        assert_eq!(read(&p), Err(type_error(1, Some(id('7')))));
        let mut p = original.clone();
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[4].semantic_type = id('9');
        application_arguments(&mut p)[1] = CheckedTerm::Literal {
            type_id: id('2'),
            value_kind: LiteralKind::Integer,
            value: json!("1"),
        };
        assert_eq!(read(&p), Err(type_error(0, Some(id('9')))));
    }

    #[trace("TC-048", "FR-038-AC-181", "FR-038-AC-180")]
    #[test]
    fn malformed_optional_wrappers_unanchored_cycles_and_union_metadata_refuse_exactly() {
        let mut p = package();
        edit_body(&mut p, 3, |body| {
            body["members"][0]["value"] = json!({"term":"aggregate","members":[]})
        });
        sync_term(&mut p, 3);
        assert_eq!(read(&p), Err(type_error(0, Some(id('4')))));
        let mut p = package();
        edit_body(&mut p, 6, |body| {
            body["members"][0]["target"] = json!(id('7'))
        });
        edit_body(&mut p, 3, |body| {
            body["members"][0]["value"] = json!({"term":"reference","target":id('7')})
        });
        sync_term(&mut p, 6);
        sync_term(&mut p, 3);
        assert_eq!(read(&p), Err(type_error(0, Some(id('7')))));
        let mut p = package();
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[4].semantic_form =
            "union_value".into();
        std::sync::Arc::make_mut(&mut p.kinds)[4] = CheckedNodeKind::Value(ValueForm::UnionValue);
        assert_eq!(read(&p), Err(type_error(0, Some(id('4')))));
        let mut p = package();
        std::sync::Arc::make_mut(&mut p.wire).semantic_graph.nodes[4].semantic_form =
            "record_value".into();
        std::sync::Arc::make_mut(&mut p.kinds)[4] = CheckedNodeKind::Value(ValueForm::RecordValue);
        edit_body(
            &mut p,
            4,
            |body| {
                *body =
                    json!({"term":"aggregate","members":[{"term":"reference","target":id('5')}]})
            },
        );
        sync_term(&mut p, 4);
        assert_eq!(
            read(&p),
            Err(unsupported(
                0,
                Some(id('4')),
                CheckedUnsupportedCompositeOperand::NonliteralGraphValue
            ))
        );
    }

    #[trace("TC-048", "FR-038-AC-181")]
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
    #[trace("TC-048", "FR-038-AC-181")]
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
