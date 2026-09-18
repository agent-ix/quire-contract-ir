//! Exact, independent per-item lowering of an admitted checked package.
//!
//! Each requested key is lowered alone against its own work budget over the
//! closure reachable through `semantic_type`, `dependencies` and body
//! reference targets. A non-lowered record carries no node, and no record
//! depends on a sibling request.

use super::{CheckedNodeTag, CheckedPackageV2, CheckedSemanticNodeV2};
use crate::checked_package::common::{digest_json, validate_term, TermGrammar, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageIncomplete, CheckedPackageRefusal, CheckedSemanticId,
    CheckedSourceMapEntry,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Identity domain of a lowered Contract IR node.
pub const CONTRACT_IR_SEMANTIC_DOMAIN: &str = "quire.contract-ir.semantic/v1";
const LOWERED_NODE_PREIMAGE: &str = "quire.contract-ir.lowered-node/v1";

/// What a caller's backend can lower.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteLoweringProfileV2 {
    /// Node families the backend supports.
    pub supported_tags: BTreeSet<CheckedNodeTag>,
    /// Whether unbounded numeric, text and collection types must be bounded.
    pub require_bounds: bool,
    /// Work per request: one for the request, plus for each visited node one
    /// unit, one per body term and one per successor edge followed.
    pub work_limit: u64,
}

/// A Contract IR node lowered exactly from one admitted V2 node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteContractNodeV2 {
    /// Exact admitted node.
    pub node: CheckedSemanticNodeV2,
    /// Parsed family.
    pub node_tag: CheckedNodeTag,
    /// Exact source correspondence for this node.
    pub source_map: Vec<CheckedSourceMapEntry>,
    /// Semantic type key.
    pub semantic_type: CheckedNodeId,
    /// Every key reachable from the node, excluding itself, ascending.
    pub dependencies: Vec<CheckedNodeId>,
    /// Reachable `bounded_domain` keys, excluding the node itself, ascending.
    pub bounds: Vec<CheckedNodeId>,
    /// Reachable `claim` keys, excluding the node itself, ascending.
    pub claims: Vec<CheckedNodeId>,
    /// `quire.contract-ir.semantic/v1` identity of this lowered node.
    pub ir_id: CheckedSemanticId,
}

/// One terminal per-request V2 lowering outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompleteLoweringRecordV2 {
    /// The node and its closure lowered exactly.
    Lowered {
        /// Lowered node.
        node: Box<CompleteContractNodeV2>,
    },
    /// A reachable node's family is outside the profile.
    Unsupported {
        /// Requested key.
        node_id: CheckedNodeId,
        /// First unsupported reachable key, ascending.
        unsupported_node_id: CheckedNodeId,
        /// Its family.
        node_tag: CheckedNodeTag,
    },
    /// A reachable unbounded type has no reachable bounding domain.
    RequiresBound {
        /// Requested key.
        node_id: CheckedNodeId,
        /// First unbounded reachable type key, ascending.
        unbounded_type: CheckedNodeId,
    },
    /// The requested key is not in the admitted graph.
    InvalidInput {
        /// Requested key.
        node_id: CheckedNodeId,
    },
    /// A reachable node body refused re-validation during the walk. A package
    /// the V2 reader admitted never yields this; it is reported rather than
    /// guessed so the work count and closure stay exact.
    InvalidBody {
        /// Requested key.
        node_id: CheckedNodeId,
        /// Key of the node whose body refused.
        body_node_id: CheckedNodeId,
        /// The term validator's refusal.
        refusal: CheckedPackageRefusal,
    },
    /// A reachable node body stopped at a validation limit during the walk.
    /// Like `InvalidBody`, an admitted package never yields this.
    BodyIncomplete {
        /// Requested key.
        node_id: CheckedNodeId,
        /// Key of the node whose body stopped.
        body_node_id: CheckedNodeId,
        /// The term validator's limit stop.
        incomplete: CheckedPackageIncomplete,
    },
    /// This request exceeded its work budget.
    Failed {
        /// Requested key.
        node_id: CheckedNodeId,
        /// Caller-selected ceiling.
        limit: u64,
        /// Counter at the failed charge.
        consumed: u64,
    },
}

/// Independent per-item outcomes for one V2 package identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteLoweringResultV2 {
    /// Identity of the admitted source package.
    pub package_id: CheckedSemanticId,
    /// One record per request, in request order.
    pub records: Vec<CompleteLoweringRecordV2>,
}

#[derive(Serialize)]
struct LoweredNodePreimage<'a> {
    version: &'static str,
    node: super::CheckedNodeProjectionV2,
    dependencies: &'a [CheckedNodeId],
    bounds: &'a [CheckedNodeId],
    claims: &'a [CheckedNodeId],
}

impl CheckedPackageV2 {
    /// Lowers each requested key independently.
    pub fn lower(
        &self,
        requested: &[CheckedNodeId],
        profile: &CompleteLoweringProfileV2,
    ) -> CompleteLoweringResultV2 {
        let index = self
            .graph()
            .nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect::<BTreeMap<_, _>>();
        let records = requested
            .iter()
            .map(|request| self.lower_one(request, profile, &index))
            .collect();
        CompleteLoweringResultV2 {
            package_id: self.package_id().clone(),
            records,
        }
    }

    fn lower_one(
        &self,
        request: &CheckedNodeId,
        profile: &CompleteLoweringProfileV2,
        index: &BTreeMap<&CheckedNodeId, usize>,
    ) -> CompleteLoweringRecordV2 {
        let failed = |consumed| CompleteLoweringRecordV2::Failed {
            node_id: request.clone(),
            limit: profile.work_limit,
            consumed,
        };
        let mut work = 1_u64;
        if work > profile.work_limit {
            return failed(work);
        }
        let Some(&start) = index.get(request) else {
            return CompleteLoweringRecordV2::InvalidInput {
                node_id: request.clone(),
            };
        };
        let nodes = &self.graph().nodes;
        let tags = self.node_tags();
        let mut visited = BTreeSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some(position) = queue.pop_front() {
            work = work.saturating_add(1);
            if work > profile.work_limit {
                return failed(work);
            }
            let Some(node) = nodes.get(position) else {
                continue;
            };
            let mut successors = vec![node.semantic_type.clone()];
            successors.extend(node.dependencies.iter().cloned());
            // The walk reports the body's term count and every reference
            // target; a failure is terminal for this request.
            let walked = validate_term(&node.body, TermGrammar::V2, &mut |target| {
                successors.push(target.clone())
            });
            let terms = match walked {
                Ok(terms) => terms,
                Err(ValidationFailure::Refused(code, path)) => {
                    return CompleteLoweringRecordV2::InvalidBody {
                        node_id: request.clone(),
                        body_node_id: node.node_id.clone(),
                        refusal: CheckedPackageRefusal {
                            code,
                            path: path.into(),
                        },
                    };
                }
                Err(ValidationFailure::Incomplete(limit_kind, limit, consumed)) => {
                    return CompleteLoweringRecordV2::BodyIncomplete {
                        node_id: request.clone(),
                        body_node_id: node.node_id.clone(),
                        incomplete: CheckedPackageIncomplete {
                            limit_kind,
                            limit,
                            consumed,
                        },
                    };
                }
            };
            let edges = u64::try_from(successors.len()).unwrap_or(u64::MAX);
            work = work.saturating_add(terms).saturating_add(edges);
            if work > profile.work_limit {
                return failed(work);
            }
            for successor in successors {
                if let Some(&next) = index.get(&successor) {
                    if visited.insert(next) {
                        queue.push_back(next);
                    }
                }
            }
        }
        let reachable = visited
            .iter()
            .filter_map(|position| Some((nodes.get(*position)?, *tags.get(*position)?)))
            .collect::<Vec<_>>();
        let mut ordered = reachable.clone();
        ordered.sort_by(|left, right| left.0.node_id.cmp(&right.0.node_id));
        if let Some((node, tag)) = ordered
            .iter()
            .find(|(_, tag)| !profile.supported_tags.contains(tag))
        {
            return CompleteLoweringRecordV2::Unsupported {
                node_id: request.clone(),
                unsupported_node_id: node.node_id.clone(),
                node_tag: *tag,
            };
        }
        let Some((node, tag)) = nodes.get(start).zip(tags.get(start).copied()) else {
            return CompleteLoweringRecordV2::InvalidInput {
                node_id: request.clone(),
            };
        };
        // FR-038: `bounds` and `claims` are filtered views of the same
        // reachable-excluding-self set that `dependencies` is drawn from, so
        // every key in `bounds` and every key in `claims` also appears in
        // `dependencies` — the requested node is never its own bound or claim.
        let bounds = ordered
            .iter()
            .filter(|(candidate, tag)| {
                *tag == CheckedNodeTag::BoundedDomain && candidate.node_id != node.node_id
            })
            .map(|(candidate, _)| candidate.node_id.clone())
            .collect::<Vec<_>>();
        if profile.require_bounds {
            if let Some((node, _)) = ordered.iter().find(|(node, tag)| {
                requires_bound(*tag, &node.semantic_form)
                    && !ordered.iter().any(|(domain, domain_tag)| {
                        *domain_tag == CheckedNodeTag::BoundedDomain
                            && domain.semantic_type == node.node_id
                    })
            }) {
                return CompleteLoweringRecordV2::RequiresBound {
                    node_id: request.clone(),
                    unbounded_type: node.node_id.clone(),
                };
            }
        }
        let dependencies = ordered
            .iter()
            .filter(|(candidate, _)| candidate.node_id != node.node_id)
            .map(|(candidate, _)| candidate.node_id.clone())
            .collect::<Vec<_>>();
        let claims = ordered
            .iter()
            .filter(|(candidate, tag)| {
                *tag == CheckedNodeTag::Claim && candidate.node_id != node.node_id
            })
            .map(|(candidate, _)| candidate.node_id.clone())
            .collect::<Vec<_>>();
        let preimage = LoweredNodePreimage {
            version: LOWERED_NODE_PREIMAGE,
            node: node.into(),
            dependencies: &dependencies,
            bounds: &bounds,
            claims: &claims,
        };
        // The preimage holds only string-keyed maps and strings, so encoding
        // cannot fail; the arm keeps the path panic-free and still terminal
        // for this request alone rather than emitting an unidentified node.
        let Some(digest) = serde_json::to_value(&preimage)
            .ok()
            .and_then(|value| digest_json(&value).ok())
        else {
            return failed(work);
        };
        CompleteLoweringRecordV2::Lowered {
            node: Box::new(CompleteContractNodeV2 {
                node: node.clone(),
                node_tag: tag,
                source_map: self
                    .source_map()
                    .iter()
                    .filter(|entry| entry.node_id == node.node_id)
                    .cloned()
                    .collect(),
                semantic_type: node.semantic_type.clone(),
                dependencies,
                bounds,
                claims,
                ir_id: CheckedSemanticId {
                    domain: CONTRACT_IR_SEMANTIC_DOMAIN.into(),
                    algorithm: "sha256".into(),
                    digest: digest.into_boxed_str(),
                },
            }),
        }
    }
}

fn requires_bound(tag: CheckedNodeTag, form: &str) -> bool {
    match tag {
        CheckedNodeTag::ScalarType => matches!(form, "integer" | "rational" | "decimal" | "text"),
        CheckedNodeTag::CompositeType => matches!(form, "sequence" | "set" | "bag" | "ordered_set"),
        CheckedNodeTag::BoundedDomain
        | CheckedNodeTag::Value
        | CheckedNodeTag::Expression
        | CheckedNodeTag::Function
        | CheckedNodeTag::Model
        | CheckedNodeTag::Relation
        | CheckedNodeTag::State
        | CheckedNodeTag::Temporal
        | CheckedNodeTag::Protocol
        | CheckedNodeTag::Claim
        | CheckedNodeTag::Correspondence => false,
    }
}
