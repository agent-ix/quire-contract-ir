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

use super::encode::{elements, encoded, member};
use super::{
    BoundedDomainForm, CheckedNodeKind, CheckedNodeTag, CheckedPackageV2, CheckedSemanticNodeV2,
    ClaimForm, CompositeTypeForm, CorrespondenceForm, ExpressionForm, FunctionForm, ModelForm,
    ProtocolForm, RelationForm, ScalarTypeForm, StateForm, TemporalForm, ValueForm,
};
use crate::checked_package::common::{
    on_stack_for, ReferenceMember, Step, Trail, ValidationFailure,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageRefusal,
    CheckedSemanticId, CheckedSourceMapEntry,
};
use quire_canonical::{Encode, Error, LimitKind, Limits, Sink, Writer};
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Identity domain of a lowered Contract IR node.
pub const CONTRACT_IR_SEMANTIC_DOMAIN: &str = "quire.contract-ir.semantic/v1";
const LOWERED_NODE_PREIMAGE: &str = "quire.contract-ir.lowered-node/v1";
/// Schema version and identity domain of a complete-V1 `ContractPackage`.
pub const CONTRACT_PACKAGE_VERSION: &str = "quire.contract-ir.contract-package/v1";

/// How an edge of the lowering closure reaches its target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EdgeRole {
    /// The target is a type or dependency of the source node.
    Typing,
    /// The target is only a `literal.type` annotation in the source's body.
    Annotation,
}

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
    /// This request exceeded a budget: its work budget, or the byte limit the
    /// package was read under (FR-038 "Every identity digest is computed
    /// through quire-canonical").
    Failed {
        /// Requested key.
        node_id: CheckedNodeId,
        /// The limit that failed: [`CheckedPackageLimit::Work`] or
        /// [`CheckedPackageLimit::Bytes`].
        limit_kind: CheckedPackageLimit,
        /// The ceiling: the caller's work limit, or the retained byte limit.
        limit: u64,
        /// The counter at the failed charge: the work consumed, or the
        /// canonical byte count the encoder needed, which is above `limit`.
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
    /// Absent exactly when the package's canonical bytes exceed the byte
    /// limit the lowering ran under: such a package has neither bytes nor id.
    encoding: Option<PackageEncoding>,
}

/// The canonical bytes of a package and the identity they hash to.
#[derive(Clone, Debug, Eq, PartialEq)]
struct PackageEncoding {
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
    /// reader requires of the checked package itself; `None` when they exceed
    /// the byte limit the lowering ran under (every record of that call is
    /// then `failed`).
    pub fn canonical_bytes(&self) -> Option<&[u8]> {
        self.encoding
            .as_ref()
            .map(|encoding| encoding.canonical_bytes.as_ref())
    }

    /// `quire.contract-ir.contract-package/v1` digest of the canonical bytes;
    /// `None` exactly when [`Self::canonical_bytes`] is.
    pub fn package_id(&self) -> Option<&CheckedSemanticId> {
        self.encoding.as_ref().map(|encoding| &encoding.package_id)
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

/// The whole package as it is encoded. It holds every lowered node and
/// dependency node, whose bodies are `Value`s, so it implements
/// [`quire_canonical::Encode`] and not `FixedShape`.
struct ContractPackagePreimage<'a> {
    version: &'static str,
    source_package_id: &'a CheckedSemanticId,
    lowered: &'a [CompleteContractNodeV2],
    dependencies: &'a [ContractPackageDependencyV2],
}

impl Encode for ContractPackagePreimage<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("version")?;
        writer.string(self.version)?;
        member(writer, "source_package_id", self.source_package_id)?;
        elements(writer, "lowered", self.lowered)?;
        elements(writer, "dependencies", self.dependencies)?;
        writer.end_object()
    }
}

/// A lowered node as it is encoded: the node's identity projection and the
/// three key sets of its closure. It holds the node's `Value` body, so it
/// implements [`quire_canonical::Encode`] and not `FixedShape`.
struct LoweredNodePreimage<'a> {
    version: &'static str,
    node: super::CheckedNodeProjectionV2,
    dependencies: &'a [CheckedNodeId],
    bounds: &'a [CheckedNodeId],
    claims: &'a [CheckedNodeId],
}

impl Encode for LoweredNodePreimage<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("version")?;
        writer.string(self.version)?;
        encoded(writer, "node", &self.node)?;
        member(writer, "dependencies", self.dependencies)?;
        member(writer, "bounds", self.bounds)?;
        member(writer, "claims", self.claims)?;
        writer.end_object()
    }
}

/// The `consumed` of a `failed` record for the `bytes` limit. When the refusal
/// is the encoder's canonical-bytes limit it is that refusal's `required`, the
/// canonical output written when it refused, which is above `ceiling`. Every
/// other encoder refusal (the object-buffer limit, an allocation failure, a
/// number the encoder has no encoding for, and any refusal a later
/// `quire-canonical` adds) is `ceiling + 1`, the smallest count above the
/// ceiling, saturating at `u64::MAX`. So every refusal is a `failed` record,
/// none is an error beside the records, and none panics (FR-038 "Every
/// identity digest is computed through quire-canonical").
fn required_bytes(error: &Error, ceiling: u64) -> u64 {
    match error {
        Error::Limit(limit) if limit.kind == LimitKind::CanonicalBytes => limit.required,
        _ => ceiling.saturating_add(1),
    }
}

/// The SHA-256 of a lowered node preimage's canonical bytes, by
/// `quire-canonical`; the byte count the encoder needed when `ceiling` was
/// exceeded otherwise.
fn identify_node(preimage: &LoweredNodePreimage<'_>, ceiling: u64) -> Result<String, u64> {
    quire_canonical::sha256(preimage, Limits::new(ceiling))
        .map(|digest| digest.to_string())
        .map_err(|error| required_bytes(&error, ceiling))
}

/// A package's canonical bytes and id, by `quire-canonical`, in one pass; the
/// byte count the encoder needed when `ceiling` was exceeded otherwise. A
/// package over the ceiling has neither bytes nor id, and no encode here
/// panics.
fn encode_package(
    preimage: &ContractPackagePreimage<'_>,
    ceiling: u64,
) -> Result<PackageEncoding, u64> {
    let mut sink = (Vec::new(), Sha256::new());
    quire_canonical::encode(&mut sink, preimage, Limits::new(ceiling))
        .map_err(|error| required_bytes(&error, ceiling))?;
    let (bytes, hasher) = sink;
    Ok(PackageEncoding {
        package_id: CheckedSemanticId {
            domain: CONTRACT_PACKAGE_VERSION.into(),
            algorithm: "sha256".into(),
            digest: format!("{:x}", hasher.finalize()).into_boxed_str(),
        },
        canonical_bytes: bytes.into_boxed_slice(),
    })
}

/// Every requested record `failed` for the `bytes` limit, for a call whose
/// package is over the ceiling: `limit` is the retained byte limit and
/// `consumed` the byte count the encoder needed.
fn failed_records(
    requested: &[CheckedNodeId],
    ceiling: u64,
    required: u64,
) -> Vec<CompleteLoweringRecordV2> {
    requested
        .iter()
        .map(|request| CompleteLoweringRecordV2::Failed {
            node_id: request.clone(),
            limit_kind: CheckedPackageLimit::Bytes,
            limit: ceiling,
            consumed: required,
        })
        .collect()
}

impl Encode for CompleteContractNodeV2 {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        encoded(writer, "node", &self.node)?;
        writer.name("node_tag")?;
        writer.string(self.node_tag.as_wire())?;
        member(writer, "source_map", &self.source_map)?;
        member(writer, "semantic_type", &self.semantic_type)?;
        member(writer, "dependencies", &self.dependencies)?;
        member(writer, "bounds", &self.bounds)?;
        member(writer, "claims", &self.claims)?;
        member(writer, "ir_id", &self.ir_id)?;
        writer.end_object()
    }
}

impl Encode for ContractPackageDependencyV2 {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        encoded(writer, "node", &self.node)?;
        writer.name("node_tag")?;
        writer.string(self.node_tag.as_wire())?;
        member(writer, "source_map", &self.source_map)?;
        writer.end_object()
    }
}

impl CheckedPackageV2 {
    /// Lowers each requested key independently, on a stack sized for the depth
    /// this package was admitted at. Dropping the result is safe at that depth;
    /// its derived `Clone`, `Debug` and `PartialEq` recurse on the caller's
    /// stack.
    pub fn lower(
        &self,
        requested: &[CheckedNodeId],
        profile: &CompleteLoweringProfileV2,
    ) -> CompleteLoweringResultV2 {
        on_stack_for(self.depth, || {
            self.lower_on_stack(requested, profile, self.bytes)
        })
    }

    fn lower_on_stack(
        &self,
        requested: &[CheckedNodeId],
        profile: &CompleteLoweringProfileV2,
        ceiling: u64,
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
            .map(|request| self.lower_one(request, profile, &index, ceiling))
            .collect::<Vec<_>>();
        match self.assemble(&records, &index, ceiling) {
            Ok(package) => CompleteLoweringResultV2 { records, package },
            // The package is over the ceiling: no package exists, so no
            // lowered node, dependency node, bytes or id is returned and every
            // requested record is `failed` for `bytes`.
            Err(required) => CompleteLoweringResultV2 {
                records: failed_records(requested, ceiling, required),
                package: CompleteContractPackageV2 {
                    source_package_id: self.package_id().clone(),
                    lowered: Vec::new(),
                    dependencies: Vec::new(),
                    encoding: None,
                },
            },
        }
    }

    /// Builds the call's package from its `lowered` records alone, encoded
    /// under `ceiling` bytes; the byte count the encoder needed when the
    /// package is over it.
    fn assemble(
        &self,
        records: &[CompleteLoweringRecordV2],
        index: &BTreeMap<&CheckedNodeId, usize>,
        ceiling: u64,
    ) -> Result<CompleteContractPackageV2, u64> {
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
        let encoding = encode_package(&preimage, ceiling)?;
        Ok(CompleteContractPackageV2 {
            source_package_id: self.package_id().clone(),
            encoding: Some(encoding),
            lowered,
            dependencies,
        })
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
        ceiling: u64,
    ) -> CompleteLoweringRecordV2 {
        let failed = |consumed| CompleteLoweringRecordV2::Failed {
            node_id: request.clone(),
            limit_kind: CheckedPackageLimit::Work,
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
        // Implements: FR-038. The types a value or expression is typed at. A node joins
        // this set when another reachable node names it through
        // `semantic_type`, `dependencies` or any body reference other than a
        // `literal.type` annotation; the requested node is always in it.
        let mut typed = BTreeSet::from([request.clone()]);
        // FR-038: the types named at a position: a `composite_type` node's
        // element or field types and a parameter's own type. A position is
        // covered only by a type that needs no bound, never by a
        // `bounded_domain` over the same shared type node elsewhere in the
        // closure.
        let mut positions = BTreeSet::new();
        let mut queue = VecDeque::from([start]);
        while let Some(position) = queue.pop_front() {
            work = work.saturating_add(1);
            if work > profile.work_limit {
                return failed(work);
            }
            let Some((node, kind)) = nodes.get(position).zip(kinds.get(position).copied()) else {
                continue;
            };
            let mut successors = vec![(node.semantic_type.clone(), EdgeRole::Typing)];
            successors.extend(
                node.dependencies
                    .iter()
                    .map(|target| (target.clone(), EdgeRole::Typing)),
            );
            // The walk reports the body's term count and every reference
            // target; a failure is terminal for this request. `validate_body`
            // is the same admission dispatch the reader used, so a package it
            // admitted re-walks identically here.
            let body_steps = [
                Step::Key("semantic_graph"),
                Step::Key("nodes"),
                Step::Index(position),
                Step::Key("body"),
            ];
            let walked = super::validate_body(
                kind,
                &node.body,
                &Trail::Base(&body_steps),
                &mut |target, site, _at| {
                    let role = match site.member {
                        ReferenceMember::Type => EdgeRole::Annotation,
                        ReferenceMember::Target
                        | ReferenceMember::ResultType
                        | ReferenceMember::FrameEntry => EdgeRole::Typing,
                    };
                    successors.push((target.clone(), role));
                },
            );
            let terms = match walked {
                Ok(terms) => terms,
                Err(ValidationFailure::Refused(refusal)) => {
                    return CompleteLoweringRecordV2::InvalidBody {
                        node_id: request.clone(),
                        body_node_id: node.node_id.clone(),
                        refusal,
                    };
                }
                Err(ValidationFailure::Incomplete(incomplete)) => {
                    return CompleteLoweringRecordV2::BodyIncomplete {
                        node_id: request.clone(),
                        body_node_id: node.node_id.clone(),
                        incomplete,
                    };
                }
            };
            let edges = u64::try_from(successors.len()).unwrap_or(u64::MAX);
            work = work.saturating_add(terms).saturating_add(edges);
            if work > profile.work_limit {
                return failed(work);
            }
            let composite = kind.tag() == CheckedNodeTag::CompositeType;
            let parameter = matches!(kind, CheckedNodeKind::Value(ValueForm::Parameter));
            for (edge, (successor, role)) in successors.into_iter().enumerate() {
                if let Some(&next) = index.get(&successor) {
                    if role == EdgeRole::Typing && next != position {
                        // Edge 0 is the node's own `semantic_type`.
                        if composite || (parameter && edge == 0) {
                            positions.insert(successor.clone());
                        }
                        typed.insert(successor);
                    }
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
        // Implements: FR-038. `bounds` and `claims` are filtered views of the same
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
            let quantities = quantity_position_ends(&positions, &ordered);
            if let Some((node, _)) = ordered.iter().find(|(node, kind)| {
                is_recursive_type(node, *kind)
                    || quantities.contains(&node.node_id)
                    || (requires_bound(*kind)
                        && (positions.contains(&node.node_id)
                            || (typed.contains(&node.node_id) && !is_bounded(node, &ordered))))
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
        // A preimage over the ceiling is terminal for this request alone
        // rather than an unidentified node.
        let digest = match identify_node(&preimage, ceiling) {
            Ok(digest) => digest,
            Err(required) => {
                return CompleteLoweringRecordV2::Failed {
                    node_id: request.clone(),
                    limit_kind: CheckedPackageLimit::Bytes,
                    limit: ceiling,
                    consumed: required,
                }
            }
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

/// Whether the closure holds a `bounded_domain` whose `semantic_type` is
/// `node`.
fn is_bounded(
    node: &CheckedSemanticNodeV2,
    closure: &[(&CheckedSemanticNodeV2, CheckedNodeKind)],
) -> bool {
    closure.iter().any(|(domain, kind)| {
        kind.tag() == CheckedNodeTag::BoundedDomain && domain.semantic_type == node.node_id
    })
}

/// Whether a node is a scalar or composite type declared as a member of a
/// `recursion_group`. A recursive type has unbounded depth and no
/// `bounded_domain` form bounds depth, so no domain in the closure covers it.
fn is_recursive_type(node: &CheckedSemanticNodeV2, kind: CheckedNodeKind) -> bool {
    matches!(
        kind.tag(),
        CheckedNodeTag::ScalarType | CheckedNodeTag::CompositeType
    ) && node.recursion_group.is_some()
}

/// How a type node continues a quantity chain walk (FR-038-AC-73).
enum QuantityChain {
    /// A `unit` or `compound_unit`: the chain ends here, at a quantity.
    Quantity,
    /// A `bounded_domain` other than `model_population`: the chain continues
    /// through its own `semantic_type`, its base type.
    Base,
    /// Any other node: the chain ends and holds no quantity.
    Other,
}

/// Exhaustive over every form, so a new form must decide whether it is a
/// quantity or forwards to one.
fn quantity_chain(kind: CheckedNodeKind) -> QuantityChain {
    use CheckedNodeKind as K;
    match kind {
        K::ScalarType(ScalarTypeForm::Unit | ScalarTypeForm::CompoundUnit) => {
            QuantityChain::Quantity
        }
        K::ScalarType(
            ScalarTypeForm::Integer
            | ScalarTypeForm::Rational
            | ScalarTypeForm::Decimal
            | ScalarTypeForm::Text
            | ScalarTypeForm::Boolean
            | ScalarTypeForm::Float32
            | ScalarTypeForm::Float64
            | ScalarTypeForm::Dimension
            | ScalarTypeForm::Enum,
        ) => QuantityChain::Other,
        K::BoundedDomain(BoundedDomainForm::ModelPopulation) => QuantityChain::Other,
        K::BoundedDomain(
            BoundedDomainForm::IntegerRange
            | BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds
            | BoundedDomainForm::CollectionBounds,
        ) => QuantityChain::Base,
        K::CompositeType(_)
        | K::Value(_)
        | K::Expression(_)
        | K::Function(_)
        | K::Model(_)
        | K::Relation(_)
        | K::State(_)
        | K::Temporal(_)
        | K::Protocol(_)
        | K::Claim(_)
        | K::Correspondence(_) => QuantityChain::Other,
    }
}

/// The `unit` or `compound_unit` nodes that a position is typed at, directly
/// or through a `bounded_domain` base chain (FR-038-AC-73). A quantity is its
/// own class, tested by this position predicate and never by
/// [`requires_bound`], so a requested unit, a compound unit's `dependencies`
/// and a `literal.type` annotation never raise. The walk is iterative and
/// bounded by the closure's size, so a cyclic chain ends without a quantity.
fn quantity_position_ends(
    positions: &BTreeSet<CheckedNodeId>,
    closure: &[(&CheckedSemanticNodeV2, CheckedNodeKind)],
) -> BTreeSet<CheckedNodeId> {
    let by_key = closure
        .iter()
        .map(|(node, kind)| (&node.node_id, (*node, *kind)))
        .collect::<BTreeMap<_, _>>();
    let mut ends = BTreeSet::new();
    for position in positions {
        let mut current = position;
        for _ in 0..=closure.len() {
            let Some(&(node, kind)) = by_key.get(current) else {
                break;
            };
            match quantity_chain(kind) {
                QuantityChain::Quantity => {
                    ends.insert(node.node_id.clone());
                    break;
                }
                QuantityChain::Base => current = &node.semantic_type,
                QuantityChain::Other => break,
            }
        }
    }
    ends
}

/// Whether a node of this kind is an unbounded numeric, text or collection
/// type that `require_bounds` needs a reachable bounding domain for.
/// Unit and compound-unit quantities are not decided here: see
/// [`quantity_position_ends`].
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
            | CompositeTypeForm::Union
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
            | ValueForm::UnionValue
            | ValueForm::OptionValue
            | ValueForm::Parameter,
        ) => false,
        K::Expression(
            ExpressionForm::Reference
            | ExpressionForm::Call
            | ExpressionForm::Unary
            | ExpressionForm::Binary
            | ExpressionForm::Conditional
            | ExpressionForm::Case
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
            | TemporalForm::Fairness
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

#[cfg(test)]
mod ceiling_tests;
