//! Occurrence-preserving QSL temporal graph to `tl-syntax` construction.

use std::collections::{BTreeMap, BTreeSet};

use quire_spec_language::protocol_artifact::{
    temporal_subject::ValidatedTemporalSubject,
    wire::{
        Handle, Interval as NativeInterval, Nullable, TemporalBinary, TemporalOperation,
        TemporalUnary,
    },
    ProtocolNumber,
};
use serde::{Deserialize, Serialize};
use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
    SyntaxArtifactLimits,
};

use crate::{bridge::BridgeLimits, predicate::ValidatedPredicateProjection};

use super::decision::{TemporalCause, TemporalCauseCode as Code, TemporalCauseDimension as Dim};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Lane {
    Future,
    Past,
}

impl Lane {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Future => "future",
            Self::Past => "past",
        }
    }
}

/// One native occurrence and the distinct TL node it produced.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FormulaOccurrence {
    pub(crate) native_declaration: u32,
    pub(crate) native_node: u32,
    pub(crate) tl_node: u32,
}

pub(crate) struct BuiltFormula {
    pub(crate) lane: Lane,
    pub(crate) document: FormulaDocument,
    pub(crate) bytes: Vec<u8>,
    pub(crate) identity: String,
    pub(crate) occurrences: Vec<FormulaOccurrence>,
}

#[derive(Clone, Copy)]
struct Frame<'a> {
    handle: &'a Handle,
    depth: usize,
    expanded: bool,
}

pub(crate) fn build(
    subject: &ValidatedTemporalSubject,
    predicates: &ValidatedPredicateProjection,
    closed: bool,
    limits: BridgeLimits,
) -> Result<BuiltFormula, TemporalCause> {
    let limits = limits.effective();
    let predicate_map = predicate_map(subject, predicates)?;
    let temporal: BTreeMap<u32, _> = subject.temporal_nodes().collect();
    let mut stack = Vec::new();
    let mut results = Vec::new();
    let mut nodes = Vec::new();
    let mut occurrences = Vec::new();
    stack
        .try_reserve(limits.formula_depth)
        .map_err(|_| resource("formula.stack"))?;
    nodes
        .try_reserve(temporal.len().min(limits.formula_nodes))
        .map_err(|_| resource("formula.nodes"))?;
    stack.push(Frame {
        handle: subject.root(),
        depth: 1,
        expanded: false,
    });
    let mut saw_future = false;
    let mut saw_past = false;
    let mut visits = 0usize;

    while let Some(frame) = stack.pop() {
        visits = visits.saturating_add(1);
        if visits > limits.visited_work || frame.depth > limits.formula_depth {
            return Err(resource("formula.traversal"));
        }
        if frame.handle.declaration != subject.declaration() {
            return Err(cause(
                Dim::NativeSubject,
                Code::TemporalSubjectMismatch,
                format!("{}:{}", frame.handle.declaration, frame.handle.index),
                None,
            ));
        }
        let native = temporal.get(&frame.handle.index).ok_or_else(|| {
            cause(
                Dim::NativeSubject,
                Code::TemporalSubjectMismatch,
                format!("{}:{}", frame.handle.declaration, frame.handle.index),
                None,
            )
        })?;
        if !frame.expanded {
            stack.push(Frame {
                expanded: true,
                ..frame
            });
            match &native.operation {
                TemporalOperation::Constant { .. } | TemporalOperation::Holds { .. } => {}
                TemporalOperation::Group { value } | TemporalOperation::Unary { value, .. } => {
                    stack.push(Frame {
                        handle: value,
                        depth: frame.depth.saturating_add(1),
                        expanded: false,
                    });
                }
                TemporalOperation::Binary { left, right, .. } => {
                    stack.push(Frame {
                        handle: right,
                        depth: frame.depth.saturating_add(1),
                        expanded: false,
                    });
                    stack.push(Frame {
                        handle: left,
                        depth: frame.depth.saturating_add(1),
                        expanded: false,
                    });
                }
            }
            continue;
        }

        if let TemporalOperation::Group { .. } = native.operation {
            if results.is_empty() {
                return Err(cause(
                    Dim::NativeSubject,
                    Code::TemporalSubjectMismatch,
                    frame.handle.index.to_string(),
                    None,
                ));
            }
            continue;
        }
        if nodes.len() == limits.formula_nodes {
            return Err(resource("formula.nodes"));
        }
        let kind = match &native.operation {
            TemporalOperation::Constant { value } => {
                if *value {
                    NodeKind::True
                } else {
                    NodeKind::False
                }
            }
            TemporalOperation::Holds { value } => {
                let proposition = predicate_map
                    .get(&(value.declaration, value.index))
                    .copied()
                    .ok_or_else(|| {
                        cause(
                            Dim::PredicateProjection,
                            Code::TemporalPredicateProjectionIncomplete,
                            format!("{}:{}", value.declaration, value.index),
                            None,
                        )
                    })?;
                NodeKind::Proposition {
                    proposition: PropositionId(proposition),
                }
            }
            TemporalOperation::Group { .. } => return Err(malformed(frame.handle.index)),
            TemporalOperation::Unary {
                operator, interval, ..
            } => {
                let operand = results.pop().ok_or_else(|| malformed(frame.handle.index))?;
                unary(*operator, interval, operand, &mut saw_future, &mut saw_past)?
            }
            TemporalOperation::Binary {
                operator, interval, ..
            } => {
                let right = results.pop().ok_or_else(|| malformed(frame.handle.index))?;
                let left = results.pop().ok_or_else(|| malformed(frame.handle.index))?;
                binary(
                    *operator,
                    interval,
                    left,
                    right,
                    &mut saw_future,
                    &mut saw_past,
                )?
            }
        };
        let id = NodeId(u32::try_from(nodes.len()).map_err(|_| resource("formula.nodes"))?);
        nodes.push(Node::new(kind));
        occurrences.push(FormulaOccurrence {
            native_declaration: frame.handle.declaration,
            native_node: frame.handle.index,
            tl_node: id.0,
        });
        results.push(id);
    }
    if results.len() != 1 || (saw_future && saw_past) {
        return Err(cause(
            Dim::TemporalProfile,
            Code::TemporalProfileUnsupported,
            subject.document().identity(),
            Some("mixed-or-non-tree".to_owned()),
        ));
    }
    let lane = if saw_past { Lane::Past } else { Lane::Future };
    if lane == Lane::Past && subject.history_boundary() != Some("execution-origin") {
        return Err(cause(
            Dim::TemporalProfile,
            Code::TemporalProfileUnsupported,
            subject.document().identity(),
            subject.history_boundary().map(str::to_owned),
        ));
    }
    let profile = match lane {
        Lane::Past => SemanticProfile::OriginCompleteHistoryV1,
        Lane::Future if closed => SemanticProfile::ClosedTraceV1,
        Lane::Future => SemanticProfile::OnlinePrefixV1,
    };
    let root = results
        .pop()
        .ok_or_else(|| malformed(subject.root().index))?;
    let document = match lane {
        Lane::Future => FormulaDocument::new(profile, root, nodes),
        Lane::Past => FormulaDocument::new_v2(profile, root, nodes),
    }
    .map_err(|_| {
        cause(
            Dim::Formula,
            Code::TemporalFormulaRejected,
            subject.document().identity(),
            None,
        )
    })?;
    let bytes = document
        .canonical_json_bytes()
        .map_err(|_| resource("formula.bytes"))?;
    let document =
        FormulaDocument::from_json_bytes(&bytes, syntax_limits(limits)).map_err(|_| {
            cause(
                Dim::Formula,
                Code::TemporalFormulaRejected,
                subject.document().identity(),
                None,
            )
        })?;
    let identity = document
        .content_identity()
        .map_err(|_| resource("formula.identity"))?;
    Ok(BuiltFormula {
        lane,
        document,
        bytes,
        identity,
        occurrences,
    })
}

fn predicate_map(
    subject: &ValidatedTemporalSubject,
    projection: &ValidatedPredicateProjection,
) -> Result<BTreeMap<(u32, u32), u32>, TemporalCause> {
    let mut map = BTreeMap::new();
    for correspondence in projection.correspondences() {
        let definition = projection
            .definition(correspondence.predicate_ref())
            .ok_or_else(|| {
                cause(
                    Dim::PredicateProjection,
                    Code::TemporalPredicateProjectionMismatch,
                    correspondence.predicate_ref().to_string(),
                    None,
                )
            })?;
        if map
            .insert(
                (definition.leaf_declaration(), definition.leaf_index()),
                correspondence.proposition_id(),
            )
            .is_some()
        {
            return Err(cause(
                Dim::PredicateProjection,
                Code::TemporalPredicateProjectionConflict,
                correspondence.predicate_ref().to_string(),
                None,
            ));
        }
    }
    let required: BTreeSet<_> = subject
        .predicate_leaves()
        .unwrap_or_default()
        .iter()
        .map(|value| (value.declaration, value.index))
        .collect();
    let mut allowed = required.clone();
    if let Some(quire_spec_language::protocol_artifact::wire::Activation::Each {
        guard: Nullable(Some(guard)),
        ..
    }) = subject.activation()
    {
        allowed.insert((guard.declaration, guard.index));
    }
    if required.iter().any(|leaf| !map.contains_key(leaf)) {
        return Err(cause(
            Dim::PredicateProjection,
            Code::TemporalPredicateProjectionIncomplete,
            subject.document().identity(),
            None,
        ));
    }
    if map.keys().any(|leaf| !allowed.contains(leaf)) {
        return Err(cause(
            Dim::PredicateProjection,
            Code::TemporalPredicateProjectionMismatch,
            subject.document().identity(),
            None,
        ));
    }
    Ok(map)
}

fn unary(
    operator: TemporalUnary,
    interval: &Nullable<NativeInterval>,
    operand: NodeId,
    future: &mut bool,
    past: &mut bool,
) -> Result<NodeKind, TemporalCause> {
    Ok(match operator {
        TemporalUnary::Not if interval.0.is_none() => NodeKind::Not { operand },
        TemporalUnary::Eventually => {
            *future = true;
            NodeKind::Future {
                interval: required_interval(interval)?,
                operand,
            }
        }
        TemporalUnary::Always => {
            *future = true;
            NodeKind::Globally {
                interval: required_interval(interval)?,
                operand,
            }
        }
        TemporalUnary::Once => {
            *past = true;
            NodeKind::Once {
                interval: required_interval(interval)?,
                operand,
            }
        }
        TemporalUnary::Historically => {
            *past = true;
            NodeKind::Historically {
                interval: required_interval(interval)?,
                operand,
            }
        }
        TemporalUnary::Not => {
            return Err(cause(
                Dim::Interval,
                Code::TemporalIntervalInvalid,
                "not",
                None,
            ))
        }
    })
}

fn binary(
    operator: TemporalBinary,
    interval: &Nullable<NativeInterval>,
    left: NodeId,
    right: NodeId,
    future: &mut bool,
    past: &mut bool,
) -> Result<NodeKind, TemporalCause> {
    Ok(match operator {
        TemporalBinary::Implies if interval.0.is_none() => NodeKind::Implies { left, right },
        TemporalBinary::Or if interval.0.is_none() => NodeKind::Or { left, right },
        TemporalBinary::And if interval.0.is_none() => NodeKind::And { left, right },
        TemporalBinary::Until => {
            *future = true;
            NodeKind::Until {
                interval: required_interval(interval)?,
                left,
                right,
            }
        }
        TemporalBinary::Release => {
            *future = true;
            NodeKind::Release {
                interval: required_interval(interval)?,
                left,
                right,
            }
        }
        TemporalBinary::Since => {
            *past = true;
            NodeKind::Since {
                interval: required_interval(interval)?,
                left,
                right,
            }
        }
        TemporalBinary::Triggered => {
            *past = true;
            NodeKind::Triggered {
                interval: required_interval(interval)?,
                left,
                right,
            }
        }
        TemporalBinary::Implies | TemporalBinary::Or | TemporalBinary::And => {
            return Err(cause(
                Dim::Interval,
                Code::TemporalIntervalInvalid,
                operator.as_str(),
                None,
            ))
        }
    })
}

fn required_interval(value: &Nullable<NativeInterval>) -> Result<Interval, TemporalCause> {
    let value = value.0.as_ref().ok_or_else(|| {
        cause(
            Dim::Interval,
            Code::TemporalIntervalInvalid,
            "missing",
            None,
        )
    })?;
    let lower = bound(&value.lower)?;
    let upper = bound(&value.upper)?;
    Interval::new(lower, upper).map_err(|_| {
        cause(
            Dim::Interval,
            Code::TemporalIntervalInvalid,
            format!("{lower}:{upper}"),
            None,
        )
    })
}

fn bound(
    value: &quire_spec_language::protocol_artifact::wire::Integer,
) -> Result<u32, TemporalCause> {
    match value.checked() {
        Ok(ProtocolNumber::Integer(value)) => u32::try_from(value.value()).map_err(|_| {
            cause(
                Dim::Interval,
                Code::TemporalIntervalInvalid,
                value.value().to_string(),
                None,
            )
        }),
        _ => Err(cause(
            Dim::Interval,
            Code::TemporalIntervalInvalid,
            "non-integer",
            None,
        )),
    }
}

pub(crate) fn syntax_limits(limits: BridgeLimits) -> SyntaxArtifactLimits {
    SyntaxArtifactLimits {
        document_bytes: limits.document_bytes,
        json_depth: limits.json_depth,
        string_bytes: limits.string_bytes,
        formula_nodes: limits.formula_nodes,
        formula_depth: limits.formula_depth,
        signals: limits.predicates,
        bindings: limits.predicates,
        propositions: limits.predicates,
        work: limits.visited_work,
    }
}

fn malformed(index: u32) -> TemporalCause {
    cause(
        Dim::NativeSubject,
        Code::TemporalSubjectMismatch,
        index.to_string(),
        None,
    )
}
fn resource(path: &str) -> TemporalCause {
    cause(
        Dim::Formula,
        Code::TemporalProjectionResourceExhausted,
        path,
        None,
    )
}
pub(crate) fn cause(
    dimension: Dim,
    code: Code,
    rejected: impl Into<String>,
    raw: Option<String>,
) -> TemporalCause {
    TemporalCause::new(dimension, code, rejected, raw)
}
