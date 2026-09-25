//! Exact, independent per-item lowering of an admitted checked package.
//!
//! Each requested key is lowered alone against its own work budget over the
//! closure reachable through `semantic_type`, `dependencies` and body
//! reference targets. A non-lowered record carries no node, and no record
//! depends on a sibling request.
//!
//! One call also assembles a single canonical `ContractPackage` (FR-035,
//! I12): every lowered node of the call plus the admitted nodes they reach.
//! A refused request contributes nothing to it, so the package never holds a
//! substitute for meaning it could not represent.

use super::{
    BoundedDomainForm, CheckedNodeKind, CheckedNodeTag, CheckedPackageV2, CheckedSemanticNodeV2,
    ClaimForm, CompositeTypeForm, CorrespondenceForm, ExpressionForm, FunctionForm, ModelForm,
    ProtocolForm, RelationForm, ScalarTypeForm, StateForm, TemporalForm, ValueForm,
};
use crate::checked_package::common::{digest_bytes, digest_json, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageIncomplete, CheckedPackageRefusal, CheckedSemanticId,
    CheckedSourceMapEntry,
};
use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Identity domain of a lowered Contract IR node.
pub const CONTRACT_IR_SEMANTIC_DOMAIN: &str = "quire.contract-ir.semantic/v1";
const LOWERED_NODE_PREIMAGE: &str = "quire.contract-ir.lowered-node/v1";
/// Schema version and identity domain of a complete-V1 `ContractPackage`.
pub const CONTRACT_PACKAGE_VERSION: &str = "quire.contract-ir.contract-package/v1";

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
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CompleteContractNodeV2 {
    /// Exact admitted node.
    pub node: CheckedSemanticNodeV2,
    /// Parsed family.
    #[serde(serialize_with = "wire_tag")]
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

/// An admitted node a lowered node reaches without itself being requested.
/// It carries no `ir_id` or closure of its own: it is represented only as
/// exact meaning some lowered node depends on.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ContractPackageDependencyV2 {
    /// Exact admitted node.
    pub node: CheckedSemanticNodeV2,
    /// Parsed family.
    #[serde(serialize_with = "wire_tag")]
    pub node_tag: CheckedNodeTag,
    /// Exact source correspondence for this node.
    pub source_map: Vec<CheckedSourceMapEntry>,
}

/// The canonical target-neutral package one lowering call emits.
///
/// It holds every `lowered` node of the call once, ascending by key, and the
/// admitted nodes those reach that were not themselves lowered, so every
/// reference inside it resolves inside it. Nodes refer to one another by key
/// only, so the package is cycle-free as a value. Only
/// [`CheckedPackageV2::lower`] builds one; its bytes and identity are fixed
/// at construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteContractPackageV2 {
    source_package_id: CheckedSemanticId,
    lowered: Vec<CompleteContractNodeV2>,
    dependencies: Vec<ContractPackageDependencyV2>,
    canonical_bytes: Box<[u8]>,
    package_id: CheckedSemanticId,
}

impl CompleteContractPackageV2 {
    /// Schema version of this package.
    pub const fn version(&self) -> &'static str {
        CONTRACT_PACKAGE_VERSION
    }

    /// Identity of the admitted package this was lowered from.
    pub fn source_package_id(&self) -> &CheckedSemanticId {
        &self.source_package_id
    }

    /// Every lowered node of the call, once each, ascending by key.
    pub fn lowered(&self) -> &[CompleteContractNodeV2] {
        &self.lowered
    }

    /// Reachable admitted nodes that were not lowered, ascending by key.
    pub fn dependencies(&self) -> &[ContractPackageDependencyV2] {
        &self.dependencies
    }

    /// RFC 8785 canonical bytes of the whole package, the encoding the V2
    /// reader requires of the checked package itself.
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// `quire.contract-ir.contract-package/v1` digest of the canonical bytes.
    pub fn package_id(&self) -> &CheckedSemanticId {
        &self.package_id
    }
}

/// One lowering call's output: the package and its per-item accounting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteLoweringResultV2 {
    /// One record per request, in request order.
    pub records: Vec<CompleteLoweringRecordV2>,
    /// The single canonical package holding every lowered node of the call.
    pub package: CompleteContractPackageV2,
}

/// The whole package as it is encoded; every member is a typed field, so a
/// field added to a node type is carried into the bytes by construction.
#[derive(Serialize)]
struct ContractPackagePreimage<'a> {
    version: &'static str,
    source_package_id: &'a CheckedSemanticId,
    lowered: &'a [CompleteContractNodeV2],
    dependencies: &'a [ContractPackageDependencyV2],
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
            .collect::<Vec<_>>();
        let package = self.assemble(&records, &index);
        CompleteLoweringResultV2 { records, package }
    }

    /// Builds the call's package from its `lowered` records alone.
    fn assemble(
        &self,
        records: &[CompleteLoweringRecordV2],
        index: &BTreeMap<&CheckedNodeId, usize>,
    ) -> CompleteContractPackageV2 {
        let mut lowered = BTreeMap::new();
        for record in records {
            if let CompleteLoweringRecordV2::Lowered { node } = record {
                lowered
                    .entry(node.node.node_id.clone())
                    .or_insert_with(|| (**node).clone());
            }
        }
        let reached = lowered
            .values()
            .flat_map(|node| node.dependencies.iter())
            .filter(|key| !lowered.contains_key(*key))
            .collect::<BTreeSet<_>>();
        let nodes = &self.graph().nodes;
        let kinds = self.node_kinds();
        let dependencies = reached
            .into_iter()
            .filter_map(|key| {
                let position = *index.get(key)?;
                let node = nodes.get(position)?;
                Some(ContractPackageDependencyV2 {
                    node: node.clone(),
                    node_tag: kinds.get(position)?.tag(),
                    source_map: self.node_source_map(&node.node_id),
                })
            })
            .collect::<Vec<_>>();
        let lowered = lowered.into_values().collect::<Vec<_>>();
        let preimage = ContractPackagePreimage {
            version: CONTRACT_PACKAGE_VERSION,
            source_package_id: self.package_id(),
            lowered: &lowered,
            dependencies: &dependencies,
        };
        // Every map in the preimage is string-keyed, so neither step can
        // fail. Encoding through `Value` sorts every object's keys, which is
        // the RFC 8785 form `CheckedPackageV2::read` requires of its input.
        let canonical_bytes = serde_json::to_value(&preimage)
            .and_then(|value| serde_json::to_vec(&value))
            .expect("the package preimage has only string-keyed maps");
        CompleteContractPackageV2 {
            source_package_id: self.package_id().clone(),
            package_id: CheckedSemanticId {
                domain: CONTRACT_PACKAGE_VERSION.into(),
                algorithm: "sha256".into(),
                digest: digest_bytes(&canonical_bytes).into_boxed_str(),
            },
            lowered,
            dependencies,
            canonical_bytes: canonical_bytes.into_boxed_slice(),
        }
    }

    fn node_source_map(&self, node_id: &CheckedNodeId) -> Vec<CheckedSourceMapEntry> {
        self.source_map()
            .iter()
            .filter(|entry| &entry.node_id == node_id)
            .cloned()
            .collect()
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
        let kinds = self.node_kinds();
        let mut visited = BTreeSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some(position) = queue.pop_front() {
            work = work.saturating_add(1);
            if work > profile.work_limit {
                return failed(work);
            }
            let Some((node, kind)) = nodes.get(position).zip(kinds.get(position).copied()) else {
                continue;
            };
            let mut successors = vec![node.semantic_type.clone()];
            successors.extend(node.dependencies.iter().cloned());
            // The walk reports the body's term count and every reference
            // target; a failure is terminal for this request. `validate_body`
            // is the same admission dispatch the reader used, so a package it
            // admitted re-walks identically here.
            let walked = super::validate_body(kind, &node.body, &mut |target, _path| {
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
                            cause: None,
                            locus: None,
                        },
                    };
                }
                // `validate_body` never returns `RefusedAt` — only
                // `validate_frame_semantics` (run separately by the reader,
                // never by this re-walk) constructs it — but the two share
                // one `ValidationFailure` type, so this arm carries the same
                // cause/locus through rather than asserting an impossibility
                // this match cannot itself guarantee.
                Err(ValidationFailure::RefusedAt(code, path, cause, locus)) => {
                    return CompleteLoweringRecordV2::InvalidBody {
                        node_id: request.clone(),
                        body_node_id: node.node_id.clone(),
                        refusal: CheckedPackageRefusal {
                            code,
                            path: path.into(),
                            cause,
                            locus: Some(locus),
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
            .filter_map(|position| Some((nodes.get(*position)?, *kinds.get(*position)?)))
            .collect::<Vec<_>>();
        let mut ordered = reachable.clone();
        ordered.sort_by(|left, right| left.0.node_id.cmp(&right.0.node_id));
        if let Some((node, kind)) = ordered
            .iter()
            .find(|(_, kind)| !profile.supported_tags.contains(&kind.tag()))
        {
            return CompleteLoweringRecordV2::Unsupported {
                node_id: request.clone(),
                unsupported_node_id: node.node_id.clone(),
                node_tag: kind.tag(),
            };
        }
        let Some((node, kind)) = nodes.get(start).zip(kinds.get(start).copied()) else {
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
            .filter(|(candidate, kind)| {
                kind.tag() == CheckedNodeTag::BoundedDomain && candidate.node_id != node.node_id
            })
            .map(|(candidate, _)| candidate.node_id.clone())
            .collect::<Vec<_>>();
        if profile.require_bounds {
            if let Some((node, _)) = ordered.iter().find(|(node, kind)| {
                requires_bound(*kind)
                    && !ordered.iter().any(|(domain, domain_kind)| {
                        domain_kind.tag() == CheckedNodeTag::BoundedDomain
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
            .filter(|(candidate, kind)| {
                kind.tag() == CheckedNodeTag::Claim && candidate.node_id != node.node_id
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
                node_tag: kind.tag(),
                source_map: self.node_source_map(&node.node_id),
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

fn wire_tag<S: Serializer>(tag: &CheckedNodeTag, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(tag.as_wire())
}

/// Whether a node of this kind is an unbounded numeric, text or collection
/// type that `require_bounds` needs a reachable bounding domain for.
/// Exhaustive over every form.
fn requires_bound(kind: CheckedNodeKind) -> bool {
    use CheckedNodeKind as K;
    match kind {
        K::ScalarType(
            ScalarTypeForm::Integer
            | ScalarTypeForm::Rational
            | ScalarTypeForm::Decimal
            | ScalarTypeForm::Text,
        ) => true,
        K::ScalarType(
            ScalarTypeForm::Boolean
            | ScalarTypeForm::Float32
            | ScalarTypeForm::Float64
            | ScalarTypeForm::Dimension
            | ScalarTypeForm::Unit
            | ScalarTypeForm::Enum
            | ScalarTypeForm::CompoundUnit,
        ) => false,
        K::CompositeType(
            CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet,
        ) => true,
        K::CompositeType(
            CompositeTypeForm::Option
            | CompositeTypeForm::Record
            | CompositeTypeForm::Tuple
            | CompositeTypeForm::Alias
            | CompositeTypeForm::Reference,
        ) => false,
        K::BoundedDomain(
            BoundedDomainForm::IntegerRange
            | BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds
            | BoundedDomainForm::CollectionBounds
            | BoundedDomainForm::ModelPopulation,
        ) => false,
        K::Value(
            ValueForm::Literal
            | ValueForm::EnumValue
            | ValueForm::CollectionValue
            | ValueForm::RecordValue
            | ValueForm::TupleValue
            | ValueForm::OptionValue
            | ValueForm::Parameter,
        ) => false,
        K::Expression(
            ExpressionForm::Reference
            | ExpressionForm::Call
            | ExpressionForm::Unary
            | ExpressionForm::Binary
            | ExpressionForm::Conditional
            | ExpressionForm::Let
            | ExpressionForm::Quantify
            | ExpressionForm::Collection
            | ExpressionForm::Conversion
            | ExpressionForm::Query
            | ExpressionForm::PreRead
            | ExpressionForm::PresenceRead
            | ExpressionForm::ValueRead
            | ExpressionForm::Deref
            | ExpressionForm::Reachability,
        ) => false,
        K::Function(
            FunctionForm::PureFunction | FunctionForm::Predicate | FunctionForm::RecursiveFunction,
        ) => false,
        K::Model(
            ModelForm::ModelImport
            | ModelForm::ObjectType
            | ModelForm::ValueType
            | ModelForm::VariantType
            | ModelForm::RecordValueType
            | ModelForm::EventType
            | ModelForm::StateMachine
            | ModelForm::Process
            | ModelForm::PersistenceInterface
            | ModelForm::Namespace
            | ModelForm::FieldDeclaration
            | ModelForm::OperationDeclaration
            | ModelForm::ClauseMemberDeclaration
            | ModelForm::SystemsInterface
            | ModelForm::SystemsPart
            | ModelForm::SystemsPort
            | ModelForm::SystemsConnection
            | ModelForm::SystemsAllocation,
        ) => false,
        K::Relation(
            RelationForm::Relationship
            | RelationForm::Population
            | RelationForm::Membership
            | RelationForm::CausalRelation,
        ) => false,
        K::State(
            StateForm::StateClause
            | StateForm::Frame
            | StateForm::Transition
            | StateForm::OperationAnchor
            | StateForm::Snapshot,
        ) => false,
        K::Temporal(
            TemporalForm::TemporalClause
            | TemporalForm::Formula
            | TemporalForm::Clock
            | TemporalForm::Window
            | TemporalForm::Activation
            | TemporalForm::Deadline,
        ) => false,
        K::Protocol(
            ProtocolForm::ProtocolClause
            | ProtocolForm::Role
            | ProtocolForm::Channel
            | ProtocolForm::Queue
            | ProtocolForm::Control
            | ProtocolForm::Obligation
            | ProtocolForm::Compensation,
        ) => false,
        K::Claim(
            ClaimForm::VerificationClaim
            | ClaimForm::AnalysisClaim
            | ClaimForm::Hyperproperty
            | ClaimForm::SynthesisRequest,
        ) => false,
        K::Correspondence(
            CorrespondenceForm::SourceLocus
            | CorrespondenceForm::ModelCorrespondence
            | CorrespondenceForm::BindingRole
            | CorrespondenceForm::ProfileCorrespondence,
        ) => false,
    }
}
