// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Borrowed pre-read structural keys and recursion-group labels. The reader
//! owns the preimage, body mapping and canonical encoder used here.

use super::application_identity::{application_digest, ApplicationGroup, ApplicationIdentityError};
use super::derived_keys::{node_preimage, valid_derived_shape, Group, LocalDigests, Shape};
use super::model_members::{structural_key, MemberType};
use super::operations::has_application_key;
use super::structural::reference_selector;
use super::{
    BodyTerm, CheckedNodeKind, CheckedNodeTag, CheckedTerm, CheckedTermView, CorrespondenceForm,
    RawSemanticNodeV2 as CheckedSemanticNodeV2,
};
use crate::checked_package::common::{TermField, TermTag, WireTerm, NODE_DOMAIN};
use crate::checked_package::shared::CheckedNodeId;
use serde_json::Value;
use std::collections::BTreeMap;

/// Candidate identities and labels in supplied graph order.
///
/// ```compile_fail,E0451
/// use quire_contract_model::StructuralCandidates;
/// let _ = StructuralCandidates { nodes: vec![], groups: vec![] };
/// ```
#[derive(Debug)]
pub struct StructuralCandidates {
    nodes: Vec<CandidateStructuralNodeId>,
    groups: Vec<CandidateRecursionGroupLabel>,
}

impl StructuralCandidates {
    /// Structural candidates in supplied graph order.
    pub fn nodes(&self) -> &[CandidateStructuralNodeId] {
        &self.nodes
    }

    /// Group candidates in first-member graph order.
    pub fn groups(&self) -> &[CandidateRecursionGroupLabel] {
        &self.groups
    }
}

/// A structural key tied to its draft's graph position, without admission.
///
/// ```compile_fail,E0451
/// use quire_contract_model::{CandidateStructuralNodeId, CheckedNodeId};
/// let _ = CandidateStructuralNodeId {
///     position: 0,
///     node_id: CheckedNodeId { domain: "d".into(), digest: "x".into() },
/// };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateStructuralNodeId {
    position: u64,
    node_id: CheckedNodeId,
}

impl CandidateStructuralNodeId {
    /// Position in the supplied graph.
    pub const fn position(&self) -> u64 {
        self.position
    }

    /// Key to install during assembly before submitting to the reader.
    pub fn candidate_node_id(&self) -> &CheckedNodeId {
        &self.node_id
    }
}

/// A group label tied to the label that selected its draft members.
///
/// ```compile_fail,E0451
/// use quire_contract_model::CandidateRecursionGroupLabel;
/// let _ = CandidateRecursionGroupLabel {
///     retained_label: "old".into(), candidate_label: "new".into(),
/// };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateRecursionGroupLabel {
    retained_label: Box<str>,
    candidate_label: Box<str>,
}

impl CandidateRecursionGroupLabel {
    /// Label carried by the supplied drafts, used only for member selection.
    pub fn retained_label(&self) -> &str {
        &self.retained_label
    }

    /// Label derived from every member-local digest in graph order.
    pub fn candidate_label(&self) -> &str {
        &self.candidate_label
    }
}

/// Typed refusal to derive a complete candidate set.
#[derive(Debug, thiserror::Error)]
pub enum StructuralCandidateError {
    /// Earliest repeated provisional selector and its first predecessor.
    #[error("repeated provisional key at graph position {repeated_position}")]
    RepeatedKey {
        /// Repeated selector.
        node_id: CheckedNodeId,
        /// First matching graph position.
        first_position: u64,
        /// Earliest repeated graph position.
        repeated_position: u64,
    },
    /// The tag or form is outside the V2 closed vocabulary.
    #[error("invalid tag or form at graph position {position}")]
    InvalidTagForm {
        /// Graph position.
        position: usize,
        /// Provisional selector.
        node_id: CheckedNodeId,
    },
    /// A reader-authenticated closed shape is malformed.
    #[error("invalid closed shape at graph position {position}")]
    InvalidClosedShape {
        /// Graph position.
        position: usize,
        /// Provisional selector.
        node_id: CheckedNodeId,
    },
    /// A labelled member has no local digest in this candidate API.
    #[error("unsupported group member at graph position {position}")]
    UnsupportedGroupMember {
        /// Graph position.
        position: usize,
        /// Provisional selector.
        node_id: CheckedNodeId,
    },
    /// A graph position, group count, ordinal or work count cannot be represented.
    #[error("candidate counter overflow at graph position {position}")]
    CounterOverflow {
        /// Graph position of the failed conversion.
        position: usize,
    },
    /// The next graph comparison, membership visit, term visit or digest is unpaid.
    #[error("candidate work exhausted at graph position {position}")]
    WorkExhausted {
        /// Supplied work ceiling.
        limit: u64,
        /// Work successfully consumed before the failed charge.
        consumed: u64,
        /// Position of the failed operation.
        position: usize,
    },
    /// The authoritative canonical encoder refused a preimage or digest array.
    #[error("canonical encoding refused at graph position {position}: {source}")]
    CanonicalEncoding {
        /// Position of the failed encoding.
        position: usize,
        /// Typed canonical error, including byte-limit refusals.
        #[source]
        source: quire_canonical::Error,
    },
    /// The shared application-local preimage rejected a grouped member.
    #[error("application-local identity refused at graph position {position}: {source}")]
    ApplicationLocal {
        /// Position of the grouped application member.
        position: usize,
        /// Typed refusal from the reader-owned application preimage.
        #[source]
        source: ApplicationIdentityError,
    },
}

struct Work {
    limit: u64,
    consumed: u64,
}

impl Work {
    fn charge(&mut self, position: usize) -> Result<(), StructuralCandidateError> {
        let next = self
            .consumed
            .checked_add(1)
            .ok_or(StructuralCandidateError::CounterOverflow { position })?;
        if next > self.limit {
            return Err(StructuralCandidateError::WorkExhausted {
                limit: self.limit,
                consumed: self.consumed,
                position,
            });
        }
        self.consumed = next;
        Ok(())
    }
}

fn index(position: usize) -> Result<u64, StructuralCandidateError> {
    u64::try_from(position).map_err(|_| StructuralCandidateError::CounterOverflow { position })
}

fn encoded(position: usize, source: quire_canonical::Error) -> StructuralCandidateError {
    StructuralCandidateError::CanonicalEncoding { position, source }
}

fn malformed(position: usize, node: &CheckedSemanticNodeV2) -> StructuralCandidateError {
    StructuralCandidateError::InvalidClosedShape {
        position,
        node_id: node.node_id.clone(),
    }
}

/// Charges every visited term without recursively walking caller-owned JSON.
fn charge_terms(
    body: &Value,
    position: usize,
    work: &mut Work,
) -> Result<(), StructuralCandidateError> {
    let mut pending = vec![body];
    while let Some(term) = pending.pop() {
        work.charge(position)?;
        let wire = WireTerm::decode(term);
        match CheckedTerm::view_from_wire(wire) {
            Some(CheckedTermView::Aggregate { members }) => pending.extend(members.iter().rev()),
            Some(CheckedTermView::Application { arguments, .. }) => {
                pending.extend(arguments.iter().rev());
            }
            Some(CheckedTermView::Binding { value, .. }) => pending.push(value),
            Some(CheckedTermView::Reference { .. } | CheckedTermView::Literal { .. }) => {}
            // Candidate charging precedes complete body admission. Keep the
            // historical work count for a malformed parent so exhaustion
            // still outranks its later closed-shape refusal.
            None => match wire.tag() {
                TermTag::Known(BodyTerm::Aggregate) => {
                    if let Some(items) = wire.array(TermField::Members) {
                        pending.extend(items.iter().rev());
                    }
                }
                TermTag::Known(BodyTerm::Application) => {
                    if let Some(items) = wire.array(TermField::Arguments) {
                        pending.extend(items.iter().rev());
                    }
                }
                TermTag::Known(BodyTerm::Binding) => pending.extend(wire.field(TermField::Value)),
                _ => {}
            },
        }
    }
    Ok(())
}

/// Derives structural identities and group labels from a complete borrowed
/// ordered graph. The result is assembly data and does not admit a package.
pub fn derive_structural_candidates<Body: super::CandidateBody>(
    nodes: &[super::CheckedSemanticNodeV2<Body>],
    byte_limit: u64,
    work_limit: u64,
) -> Result<StructuralCandidates, StructuralCandidateError> {
    let raw_nodes = Body::candidate_nodes(nodes);
    derive_structural_candidates_raw(&raw_nodes, byte_limit, work_limit)
}

fn derive_structural_candidates_raw(
    nodes: &[CheckedSemanticNodeV2],
    byte_limit: u64,
    work_limit: u64,
) -> Result<StructuralCandidates, StructuralCandidateError> {
    let mut work = Work {
        limit: work_limit,
        consumed: 0,
    };
    let mut groups: BTreeMap<&str, Group<'_>> = BTreeMap::new();
    let mut kinds = Vec::with_capacity(nodes.len());
    for (position, node) in nodes.iter().enumerate() {
        let position_u64 = index(position)?;
        for (first, previous) in nodes.iter().take(position).enumerate() {
            work.charge(position)?;
            if previous.node_id == node.node_id {
                return Err(StructuralCandidateError::RepeatedKey {
                    node_id: node.node_id.clone(),
                    first_position: index(first)?,
                    repeated_position: position_u64,
                });
            }
        }
        let kind = CheckedNodeTag::from_wire(&node.node_tag)
            .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
            .ok_or_else(|| StructuralCandidateError::InvalidTagForm {
                position,
                node_id: node.node_id.clone(),
            })?;
        kinds.push(kind);
        if let Some(label) = node.recursion_group.as_deref() {
            work.charge(position)?;
            let group = groups.entry(label).or_insert_with(|| Group {
                members: Vec::new(),
                ordinals: BTreeMap::new(),
                least: position,
            });
            let ordinal = group.members.len();
            index(ordinal)?;
            index(
                ordinal
                    .checked_add(1)
                    .ok_or(StructuralCandidateError::CounterOverflow { position })?,
            )?;
            group.members.push(position);
            group.ordinals.insert(&node.node_id, ordinal);
        }
    }
    let mut candidates = Vec::new();
    for (position, node) in nodes.iter().enumerate() {
        let kind = kinds[position];
        if node.nominal_identity_preimage.is_some()
            || has_application_key(node, kind)
            || kind == CheckedNodeKind::Correspondence(CorrespondenceForm::AbstractionRelation)
        {
            continue;
        }
        let shape = Shape::of(kind);
        let integer = if matches!(shape, Some(Shape::IntegerRange | Shape::CollectionBounds)) {
            work.charge(position)?;
            MemberType::Integer
                .node_key(byte_limit)
                .map_err(|source| encoded(position, source))?
        } else {
            String::new()
        };
        let mut semantic_type_kind = None;
        if shape == Some(Shape::CollectionBounds) {
            for (target, &target_kind) in nodes.iter().zip(&kinds) {
                work.charge(position)?;
                if target.node_id == node.semantic_type {
                    semantic_type_kind = Some(target_kind);
                    break;
                }
            }
        }
        if !valid_derived_shape(
            node,
            kind,
            &integer,
            semantic_type_kind,
            false,
            reference_selector,
        ) {
            return Err(malformed(position, node));
        }
        charge_terms(&node.body, position, &mut work)?;
    }
    let mut labels = Vec::new();
    let mut derived_labels = BTreeMap::new();
    for (&retained, group) in &groups {
        let mut local = Vec::with_capacity(group.members.len());
        let member_ids: Vec<_> = group
            .members
            .iter()
            .map(|&position| &nodes[position].node_id)
            .collect();
        for &position in &group.members {
            let node = &nodes[position];
            let kind = kinds[position];
            if node.nominal_identity_preimage.is_some()
                || kind == CheckedNodeKind::Correspondence(CorrespondenceForm::AbstractionRelation)
            {
                return Err(StructuralCandidateError::UnsupportedGroupMember {
                    position,
                    node_id: node.node_id.clone(),
                });
            }
            charge_terms(&node.body, position, &mut work)?;
            work.charge(position)?;
            let digest = if has_application_key(node, kind) {
                application_digest(node, ApplicationGroup::Keys(&member_ids), byte_limit).map_err(
                    |error| match error {
                        ApplicationIdentityError::CanonicalEncoding(source) => {
                            encoded(position, source)
                        }
                        ApplicationIdentityError::NonApplication { .. }
                        | ApplicationIdentityError::MissingContext { .. }
                        | ApplicationIdentityError::MissingTarget { .. }
                        | ApplicationIdentityError::RepeatedKey { .. }
                        | ApplicationIdentityError::ContextMismatch { .. }
                        | ApplicationIdentityError::ContextIndexOutOfRange { .. } => {
                            StructuralCandidateError::ApplicationLocal {
                                position,
                                source: error,
                            }
                        }
                    },
                )?
            } else {
                structural_key(&node_preimage(node, Some(group), None), byte_limit)
                    .map_err(|source| encoded(position, source))?
            };
            local.push(digest);
        }
        work.charge(group.least)?;
        let candidate = quire_canonical::sha256(
            &LocalDigests(&local),
            quire_canonical::Limits::new(byte_limit),
        )
        .map_err(|source| encoded(group.least, source))?
        .to_string();
        derived_labels.insert(retained, candidate.clone());
        labels.push((
            group.least,
            CandidateRecursionGroupLabel {
                retained_label: retained.into(),
                candidate_label: candidate.into(),
            },
        ));
    }
    labels.sort_by_key(|(position, _)| *position);
    for (position, node) in nodes.iter().enumerate() {
        let kind = kinds[position];
        if node.nominal_identity_preimage.is_some()
            || has_application_key(node, kind)
            || kind == CheckedNodeKind::Correspondence(CorrespondenceForm::AbstractionRelation)
        {
            continue;
        }
        let group = node
            .recursion_group
            .as_deref()
            .and_then(|label| groups.get(label));
        let label = node
            .recursion_group
            .as_deref()
            .and_then(|label| derived_labels.get(label))
            .map(String::as_str);
        work.charge(position)?;
        let digest = structural_key(&node_preimage(node, group, label), byte_limit)
            .map_err(|source| encoded(position, source))?;
        candidates.push(CandidateStructuralNodeId {
            position: index(position)?,
            node_id: CheckedNodeId {
                domain: NODE_DOMAIN.into(),
                digest: digest.into(),
            },
        });
    }
    Ok(StructuralCandidates {
        nodes: candidates,
        groups: labels.into_iter().map(|(_, label)| label).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-048", "FR-048-AC-5")]
    #[test]
    fn tc_048_candidate_charges_a_malformed_parent_before_shape_refusal() {
        let body = json!({
            "term": "aggregate",
            "members": [{"term": "reference", "target": "provisional"}],
            "extra": true,
        });
        let mut work = Work {
            limit: 1,
            consumed: 0,
        };
        assert!(matches!(
            charge_terms(&body, 0, &mut work),
            Err(StructuralCandidateError::WorkExhausted {
                limit: 1,
                consumed: 1,
                position: 0,
            })
        ));
    }
}
