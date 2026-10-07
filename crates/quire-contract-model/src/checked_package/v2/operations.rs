//! FR-038/QSpec #76 operation-law validation: an `application` term's
//! `operation` member is admitted opaquely nowhere past this module — its
//! identity, laws, mode, member and leading operand shape are checked
//! against the upstream `quire.checked-operation-catalog/v1`
//! ([`operation_catalog`]) and the package's own lock selections.
//!
//! Two stages, run in the upstream contract's normative order (its
//! "Operation identity rules" and reader-boundary sections). The copy of
//! that description this tree used to carry was removed with the rest of the
//! private-sourced fixture tree; see agent-ix/quire-contract-ir#166:
//!
//! 1. [`validate_application_keys`]: every node whose body is an
//!    `application` term re-derives its own `node_id` from
//!    `{version: "quire.application-node/v1", node_tag, semantic_form,
//!    semantic_type, declaration, recursion, body}`; a retained stale key is
//!    `invalid_package`/`stale-node-key`. This runs before declaration
//!    (`validate_nominal_nodes`) and frame (`validate_frame_semantics`)
//!    checks, exactly like those two stages' own key/identity work.
//! 2. [`validate_operations`]: once declaration and frame checks pass, each
//!    application node's `operation` is checked against its catalogued
//!    entry, in ascending node-id digest order, reporting the first node
//!    that fails.
//!
//! A node's `recursion` preimage member is FR-322's `{size, ordinal}` of the
//! node within its `recursion_group` in graph order (`null` outside a group),
//! and each body `reference` to a member of that group is keyed as
//! `{term: "group_reference", ordinal}` (see [`application_preimage`]).
//!
//! Scope this reader does not cover, narrower than the upstream contract
//! description's full generality and not exercised by an admitted fixture or
//! upstream vector. Each item is a silent no-op (never a false refusal), not
//! a silent admission of something the upstream corpus requires rejected: an
//! `operation.member` of kind `position`, `element`, `relationship_end`,
//! `type_argument` or `operation` is checked for presence and kind only, not
//! that its `declaration` resolves to a real, eligible node (`field` is the
//! one kind an upstream mutation exercises, so it alone is checked in full,
//! including that the named field is actually declared; the `temporal_interval`
//! and `fairness` members are checked for their closed shape here and by
//! [`super::temporal`] for their meaning); a `constraints`
//! entry other than `same_family`/`same_type`/`conforming_reference`/
//! `reference_edge`/`union_arms` is not enforced; leaf-path
//! resolution covers exactly one shape, `["field:<name>"]` against the first
//! operand's record type, the one the upstream vectors exercise;
//! [`validate_application_keys`] re-derives a key for a node whose own
//! `body` is an application term at its root. An application stands nowhere
//! else: the flat wire refuses one nested in another term, ahead of every
//! identity check (`flat_wire`), so every application's key and its own
//! `operation` are checked here; `operation.mode`
//! is checked for `kind` only — its `value` is never checked against the
//! catalog's closed `modes` vocabulary; a catalogued entry's `result` is
//! never checked against the catalog's `result_forms`; `argument_family`
//! resolves only `reference` and `binding` argument terms, so a `literal` or
//! `aggregate` argument resolves to no family and
//! silently bypasses every operand-family check ([`check_operands`],
//! [`check_mode_type`], [`check_leaf_count`]) that consults it.

use super::dependency_references::{DependencyReferences, Referrer, SuppliedDependencies};
use super::encode::{ApplicationNodePreimage, GroupPlace};
use super::model_members::{
    declaration_key, Budget, CollectionKind, DeclarationForm, MemberKind, MemberType, ModelFailure,
    ModelOwners, ModelRefusal, Resolved,
};
use super::operation_catalog::{operation_catalog, OperationCatalog, OperationCatalogEntry};
use super::structural;
use super::structural::{
    aggregate_members, binding, union_type_body, union_value_body, UnionMember,
};
use super::temporal::member_is_well_formed;
use super::{
    ApplicationOperator, BodyTerm, BoundedDomainForm, CheckedArtifactRef, CheckedNodeId,
    CheckedNodeKind, CheckedNodeTag, CheckedPackageLockV2, CheckedSemanticNodeV2, ClaimForm,
    CompositeTypeForm, CorrespondenceForm, ExpressionForm, FunctionForm, LawRole, ModelForm,
    NominalIdentityPreimage, OperationConstraintKind, OperationMemberKind, OperationModeKind,
    ProtocolForm, RelationForm, ScalarTypeForm, StateForm, TemporalForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::common::{
    application_operator, body_term, decoder_pointer, node_pointer, NODE_DOMAIN,
};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use quire_walk::{walk, Children, Walk};
use serde::Deserialize;
use serde_json::{json, Value};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::ops::ControlFlow;
use std::rc::Rc;

/// One application node under validation, and pointers into its body.
#[derive(Clone, Copy)]
struct Application<'a> {
    node: &'a CheckedSemanticNodeV2,
    position: usize,
}

impl Application<'_> {
    fn node_id(self) -> JsonPointer {
        node_pointer(self.position).key("node_id")
    }

    /// `/semantic_graph/nodes/{n}/body` extended by `members`.
    fn body(self, members: &[&str]) -> JsonPointer {
        members.iter().fold(
            node_pointer(self.position).key("body"),
            |pointer, member| pointer.key(member),
        )
    }

    fn refuse(
        self,
        code: CheckedPackageRefusalCode,
        path: JsonPointer,
        cause: CheckedPackageRefusalCause,
    ) -> ValidationFailure {
        ValidationFailure::refused_at(code, path, Some(cause), self.node.node_id.clone())
    }
}

fn is_application(body: &Value) -> bool {
    body_term(body) == Some(BodyTerm::Application)
}

/// Every application node's `node_id` re-derived from its own visible
/// members, in ascending digest order, reporting the first stale one.
///
/// Each key is hashed through `quire-canonical` under `bytes`, the reader's
/// byte limit; a preimage whose canonical bytes exceed it refuses
/// `invalid_semantic_graph` at the node's `node_id`.
pub(super) fn validate_application_keys(
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
    bytes: u64,
) -> Result<(), ValidationFailure> {
    // Each recursion group's members, in graph order.
    let mut groups: BTreeMap<&str, Vec<&CheckedNodeId>> = BTreeMap::new();
    for node in nodes {
        if let Some(label) = node.recursion_group.as_deref() {
            groups.entry(label).or_default().push(&node.node_id);
        }
    }
    for (&node_id, &position) in index {
        let node = &nodes[position];
        if !is_application(&node.body) {
            continue;
        }
        let application = Application { node, position };
        meter.charge(1, || application.node_id())?;
        let group = node
            .recursion_group
            .as_deref()
            .and_then(|label| groups.get(label))
            .map_or(&[][..], Vec::as_slice);
        let preimage = application_preimage(application, group)?;
        let computed = quire_canonical::sha256(&preimage, quire_canonical::Limits::new(bytes))
            .map_err(|_| {
                ValidationFailure::refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    application.node_id(),
                )
            })?;
        let computed_digest = computed.to_string();
        if computed_digest != node_id.digest.as_ref() {
            return Err(ValidationFailure::refused_stale_node_key(
                application.node_id(),
                node_id.clone(),
                CheckedNodeId {
                    domain: NODE_DOMAIN.into(),
                    digest: computed_digest.into(),
                },
            ));
        }
    }
    Ok(())
}

/// QSpec FR-322's `application_node_preimage` for `node`, given its
/// recursion group's members in graph order (empty outside a group):
/// `{version, node_tag, semantic_form, semantic_type, declaration, recursion,
/// body}`, where `recursion` is `{size, ordinal}` of the node within the group
/// or `null`, and each body `reference` to a group member becomes
/// `{term: "group_reference", ordinal}`.
fn application_preimage<'a>(
    application: Application<'a>,
    group: &[&CheckedNodeId],
) -> Result<ApplicationNodePreimage<'a>, ValidationFailure> {
    let node = application.node;
    let recursion = match node.recursion_group {
        None => None,
        Some(_) => {
            let ordinal = group
                .iter()
                .position(|member| **member == node.node_id)
                .ok_or_else(|| {
                    ValidationFailure::refused(
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        application.node_id(),
                    )
                })?;
            Some(GroupPlace {
                size: group.len(),
                ordinal,
            })
        }
    };
    Ok(ApplicationNodePreimage {
        node_tag: &node.node_tag,
        semantic_form: &node.semantic_form,
        semantic_type: &node.semantic_type,
        declaration: node.declaration.as_ref(),
        recursion,
        body: group_references(&node.body, group),
    })
}

/// `term` with every `reference` to a recursion-group member rewritten as
/// `{term: "group_reference", ordinal}`, walking application arguments,
/// aggregate members and binding values, the SemanticTerm positions that hold
/// terms. The body grammar is read here as the wire's JSON, like every body
/// validator.
fn group_references<'a>(term: &'a Value, group: &[&CheckedNodeId]) -> Cow<'a, Value> {
    if group.is_empty() {
        return Cow::Borrowed(term);
    }
    // One copy of the body, rewritten in place by one visit of each of its
    // terms: no enclosing term is copied again at any level.
    let mut rewritten = term.clone();
    let ControlFlow::Continue(()) = walk(&mut GroupRewrite { group }, &mut rewritten);
    Cow::Owned(rewritten)
}

/// The walk of [`group_references`], over the terms of a copy of the body: each
/// `reference` to a member of the recursion group becomes
/// `{term: "group_reference", ordinal}`, in the application arguments,
/// aggregate members and binding values that hold terms. A dependency reference
/// names a node of another package, never a member of this recursion group,
/// and enters the preimage as it stands on the wire.
struct GroupRewrite<'t> {
    group: &'t [&'t CheckedNodeId],
}

impl<'t> Walk for GroupRewrite<'t> {
    type Node = &'t mut Value;
    type Frame = ();
    type Stop = Infallible;

    fn enter(
        &mut self,
        term: &'t mut Value,
        children: &mut Children<'_, &'t mut Value>,
    ) -> ControlFlow<Infallible> {
        match body_term(term) {
            Some(BodyTerm::Reference) => {
                let ordinal = structural::reference_target(term)
                    .and_then(|target| self.group.iter().position(|member| **member == target));
                if let Some(ordinal) = ordinal {
                    *term = json!({ "term": "group_reference", "ordinal": ordinal });
                }
            }
            Some(BodyTerm::Application) => {
                if let Some(Value::Array(arguments)) = term.get_mut("arguments") {
                    children.extend(arguments.iter_mut());
                }
            }
            Some(BodyTerm::Aggregate) => {
                if let Some(Value::Array(members)) = term.get_mut("members") {
                    children.extend(members.iter_mut());
                }
            }
            Some(BodyTerm::Binding) => {
                children.extend(term.get_mut("value"));
            }
            Some(
                BodyTerm::Literal
                | BodyTerm::DependencyReference
                | BodyTerm::Frame
                | BodyTerm::AbstractionRelation,
            )
            | None => {}
        }
        ControlFlow::Continue(())
    }

    fn exit(&mut self, (): ()) -> ControlFlow<Infallible> {
        ControlFlow::Continue(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperationWire {
    pub(super) identity: Box<str>,
    pub(super) laws: Vec<OperationLawWire>,
    mode: Option<OperationModeWire>,
    pub(super) member: Option<Value>,
    leaves: Vec<OperationLeafWire>,
}

impl OperationWire {
    /// The wire member's decoded `kind`: `None` when the operation carries no
    /// member (or one without a string `kind`), `Some(None)` when the `kind`
    /// is outside the catalog's vocabulary.
    // Decodes the wire member kind.
    fn member_kind_class(&self) -> Option<Option<OperationMemberKind>> {
        let kind = self.member.as_ref()?.get("kind")?.as_str()?;
        Some(OperationMemberKind::from_wire(kind))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperationLawWire {
    role: Box<str>,
    pub(super) definition: CheckedArtifactRef,
}

impl OperationLawWire {
    /// The law's decoded role; `None` outside the catalog's vocabulary.
    // Decodes the wire law role.
    pub(super) fn role_class(&self) -> Option<LawRole> {
        LawRole::from_wire(&self.role)
    }
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct OperationModeWire {
    kind: Box<str>,
    value: Box<str>,
}

impl OperationModeWire {
    /// The mode's decoded kind; `None` outside the catalog's vocabulary.
    // Decodes the wire mode kind.
    fn kind_class(&self) -> Option<OperationModeKind> {
        OperationModeKind::from_wire(&self.kind)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationLeafWire {
    path: Vec<Box<str>>,
    /// Read by `check_leaf_count`, which requires exactly one `text_profile`
    /// law per leaf.
    laws: Vec<OperationLawWire>,
    mode: Option<OperationModeWire>,
}

/// Every application node's `operation`, in ascending node-id digest order,
/// checked against [`operation_catalog`] and `lock`. Reports the first node
/// that fails, at the first check it fails, in this order: `unknown-operation`,
/// `operation-class-mismatch`, `operation-law-missing`,
/// `operation-law-mismatch`, `operation-law-unselected`,
/// `operation-mode-mismatch`, `operation-member-mismatch` — these seven match
/// the upstream contract description's own order — then
/// `operator-ineligible` (arity,
/// operand family, named-member existence), then `operation-mode-type-mismatch`
/// (an operand or leaf whose own type pins a value the wire's mode disagrees
/// with). The upstream description instead orders
/// `operation-mode-type-mismatch` before
/// `operator-ineligible`; this implementation runs the reverse, and no
/// upstream vector distinguishes the two orders, so this one adjacent pair's
/// relative order is not pinned by any vector — do not read it as validated
/// against the upstream description.
pub(super) fn validate_operations(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    lock: &CheckedPackageLockV2,
    owners: &ModelOwners<'_>,
    dependencies: &SuppliedDependencies<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = Graph {
        nodes,
        kinds,
        index,
    };
    let references = DependencyReferences::new(dependencies);
    for &position in index.values() {
        let node = &nodes[position];
        if let Some(failure) = case_placement_defect(position, &graph) {
            return Err(failure);
        }
        if !is_application(&node.body) {
            // No application check applies, but a union type or union value
            // node has its own joins (QSpec FR-440), and the terms of the
            // body are walked for `dependency_reference` (FR-322, step 7).
            if let Some(failure) = union_defect(position, &graph) {
                return Err(failure);
            }
            references.walk_body(Referrer { node, position }, meter)?;
            continue;
        }
        let application = Application { node, position };
        if let Some(failure) = operation_defect(
            application,
            &graph,
            lock,
            owners,
            references,
            operation_catalog(),
            meter,
        )? {
            return Err(failure);
        }
    }
    Ok(())
}

/// The admitted graph an application's checks read.
#[derive(Clone, Copy)]
struct Graph<'g> {
    nodes: &'g [CheckedSemanticNodeV2],
    kinds: &'g [CheckedNodeKind],
    index: &'g BTreeMap<&'g CheckedNodeId, usize>,
}

fn operation_defect(
    application: Application<'_>,
    graph: &Graph<'_>,
    lock: &CheckedPackageLockV2,
    owners: &ModelOwners<'_>,
    references: DependencyReferences<'_>,
    catalog: &OperationCatalog,
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let Graph {
        nodes,
        kinds,
        index,
    } = *graph;
    let node = application.node;
    let Some(body) = node.body.as_object() else {
        return Ok(None);
    };
    let operator = application_operator(&node.body);
    let Some(operation_value) = body.get("operation") else {
        return Ok(None);
    };
    let operation = match serde_path_to_error::deserialize::<_, OperationWire>(operation_value) {
        Ok(operation) => operation,
        Err(error) => {
            return Ok(Some(ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                decoder_pointer(application.body(&["operation"]), error.path()),
            )))
        }
    };
    meter.charge(
        1 + u64::try_from(operation.laws.len()).unwrap_or(u64::MAX)
            + u64::try_from(operation.leaves.len()).unwrap_or(u64::MAX),
        || application.body(&["operation"]),
    )?;
    let refuse = |path: JsonPointer, cause| {
        Ok(Some(application.refuse(
            CheckedPackageRefusalCode::InvalidPackage,
            path,
            cause,
        )))
    };
    let law_at = |law: usize, member: &str| {
        application
            .body(&["operation", "laws"])
            .index(law)
            .key(member)
    };

    let Some(entry) = catalog.entry(&operation.identity) else {
        return refuse(
            application.body(&["operation", "identity"]),
            CheckedPackageRefusalCause::UnknownOperation,
        );
    };
    if operator != Some(entry.operator) {
        return refuse(
            application.body(&["operator"]),
            CheckedPackageRefusalCause::OperationClassMismatch,
        );
    }
    if operation.laws.len() < entry.laws.len() {
        return refuse(
            application.body(&["operation", "laws"]),
            CheckedPackageRefusalCause::OperationLawMissing,
        );
    }
    if operation.laws.len() > entry.laws.len() {
        // Too many, not too few: a law the catalogued entry admits no role
        // for is a mismatch against what it declares, not a shortfall. The
        // first law past the catalogued roles is the one at fault.
        return refuse(
            application
                .body(&["operation", "laws"])
                .index(entry.laws.len()),
            CheckedPackageRefusalCause::OperationLawMismatch,
        );
    }
    // QSpec FR-322: an entry that names no leaf source lists no leaves; the
    // reference reader refuses a supplied one as a law mismatch, ahead of any
    // selection check.
    if entry.leaves.is_none() && !operation.leaves.is_empty() {
        return refuse(
            application.body(&["operation", "leaves"]).index(0),
            CheckedPackageRefusalCause::OperationLawMismatch,
        );
    }
    for (law_index, (law, role)) in operation.laws.iter().zip(entry.laws.iter()).enumerate() {
        let role = *role;
        if law.role_class() != Some(role) {
            return refuse(
                law_at(law_index, "role"),
                CheckedPackageRefusalCause::OperationLawMismatch,
            );
        }
        // A clause/profile role (`temporal_profile`, `protocol_profile`) has
        // no fixed catalogued definition list of its own — any published
        // profile definition of that kind is eligible — so its legitimacy
        // is exactly whether the lock's own `profile_selections` selected it
        // under this role; a value role (`integer_division`, `ieee_profile`,
        // `text_profile`) is closed over `law_roles`, so a definition
        // outside that fixed list is a mismatch before the lock is
        // consulted at all.
        let selected = if let Some(selection_role) = role.selection_role() {
            lock.profile_selections.iter().any(|selection| {
                selection.role == selection_role && selection.definition == law.definition
            })
        } else {
            let catalogued = catalog.law_role_definitions(role);
            let known = catalogued.is_some_and(|definitions| definitions.contains(&law.definition));
            if !known {
                return refuse(
                    law_at(law_index, "definition"),
                    CheckedPackageRefusalCause::OperationLawMismatch,
                );
            }
            lock.definition_selections.contains(&law.definition)
        };
        if !selected {
            return refuse(
                law_at(law_index, "definition"),
                CheckedPackageRefusalCause::OperationLawUnselected,
            );
        }
    }
    // A member present on the wire is the value at fault; an absent one is
    // a defect of the `operation` object that lacks it.
    let member_or_operation = |member: &str| {
        if operation_value.get(member).is_some() {
            application.body(&["operation", member])
        } else {
            application.body(&["operation"])
        }
    };
    match (&entry.mode, &operation.mode) {
        (None, None) => {}
        (Some(kind), Some(mode)) if mode.kind_class() == Some(*kind) => {}
        (Some(_), Some(_)) => {
            return refuse(
                application.body(&["operation", "mode", "kind"]),
                CheckedPackageRefusalCause::OperationModeMismatch,
            )
        }
        (None, Some(_)) | (Some(_), None) => {
            return refuse(
                member_or_operation("mode"),
                CheckedPackageRefusalCause::OperationModeMismatch,
            )
        }
    }
    // Outer `Some` is a member on the wire; inner `None` is a `kind` outside
    // the catalog's vocabulary, which no entry's member matches.
    let wire_member_kind = operation.member_kind_class();
    let member_mismatch = || {
        refuse(
            member_or_operation("member"),
            CheckedPackageRefusalCause::OperationMemberMismatch,
        )
    };
    match (entry.member, wire_member_kind) {
        (None, None) => {}
        (Some(kind), Some(Some(wire_kind))) => {
            // The kind agrees; the member must also be the closed shape that
            // kind has (QSpec FR-370: `temporal_interval`, `fairness`).
            let shaped = operation
                .member
                .as_ref()
                .is_some_and(|member| member_is_well_formed(wire_kind, member));
            if kind != wire_kind || !shaped {
                return member_mismatch();
            }
        }
        // A `null` member on an interval-capable operator is refused at the
        // application, not at `operation.member` (merged QSpec FR-370).
        (Some(OperationMemberKind::TemporalInterval), None) if operation.member.is_none() => {
            return refuse(
                application.body(&[]),
                CheckedPackageRefusalCause::OperationMemberMismatch,
            )
        }
        (Some(_), Some(None)) | (Some(_), None) | (None, Some(_)) => return member_mismatch(),
    }

    // FR-322 step 7: the application's own identity checks are done, so its
    // arguments are next, in pre-order; a `dependency_reference` among them
    // is checked here, before the operand checks below.
    references.walk_arguments(
        Referrer {
            node,
            position: application.position,
        },
        meter,
    )?;

    let arguments = body
        .get("arguments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(failure) = check_operands(
        application,
        &operation,
        entry,
        &arguments,
        graph,
        owners,
        catalog,
        meter,
    )? {
        return Ok(Some(failure));
    }
    if let Some(failure) = check_inner_result(application, entry, &arguments, graph) {
        return Ok(Some(failure));
    }
    if let Some(failure) = check_state_clause_result(application, entry, graph) {
        return Ok(Some(failure));
    }
    // A reference-edge operation's `field` member is resolved by
    // `check_reference_edge`, over a `Reference` operand, not as a field read.
    let edge_operation = entry
        .constraints
        .iter()
        .any(|constraint| constraint.kind == OperationConstraintKind::ReferenceEdge);
    let member_kind = match wire_member_kind.flatten() {
        Some(OperationMemberKind::Field) if edge_operation => None,
        Some(OperationMemberKind::Field) => Some(MemberKind::Field),
        Some(OperationMemberKind::Operation) => Some(MemberKind::Operation),
        // The `temporal_interval` and `fairness` members are checked for their
        // shape above and by the temporal step (profile fit, bounds, the
        // fairness operation's resolution); nothing here reads them further.
        Some(
            OperationMemberKind::Position
            | OperationMemberKind::Element
            | OperationMemberKind::RelationshipEnd
            | OperationMemberKind::TypeArgument
            | OperationMemberKind::ProfileOperator
            | OperationMemberKind::StateClause
            | OperationMemberKind::TemporalInterval
            | OperationMemberKind::Fairness,
        )
        | None => None,
    };
    if let Some(kind) = member_kind {
        let declaring = member_declaration(&operation).and_then(|declaration| {
            let position = *index.get(&declaration)?;
            Some((&nodes[position], kinds[position].tag()))
        });
        match declaring {
            Some((declaring, tag)) if owners.is_model_declaration_node(declaring, tag) => {
                if let Some(failure) = check_model_member(
                    application,
                    &operation,
                    kind,
                    declaring,
                    &arguments,
                    graph,
                    owners,
                    meter,
                )? {
                    return Ok(Some(failure));
                }
            }
            _ if kind == MemberKind::Field => {
                if let Some(failure) = check_field_member(application, &operation, nodes, index) {
                    return Ok(Some(failure));
                }
            }
            _ => {}
        }
    }
    if matches!(
        operation.identity.as_ref(),
        "quire.op.model.navigate" | "quire.op.model.reaches"
    ) {
        if let Some(failure) =
            check_relationship_member(application, &operation, &arguments, graph, owners, meter)?
        {
            return Ok(Some(failure));
        }
    }
    if let Some(failure) = check_mode_type(
        application,
        entry,
        &operation,
        &arguments,
        nodes,
        kinds,
        index,
        catalog,
    ) {
        return Ok(Some(failure));
    }
    check_leaf_count(
        application,
        entry,
        &operation,
        &arguments,
        graph,
        lock,
        catalog,
        meter,
    )
}

/// Resolves an argument term's family: `reference` resolves its target node
/// directly (a `population` declaration, a `scalar_type`/`composite_type`
/// node, or an `expression`/`reference` node — the catalog's own `reference`
/// family, an identity/pointer expression, not `composite_type`'s
/// same-named nullable-wrapper form) and, only when the target itself is not
/// one of those (a `value` node — a literal, a `record_value`, ...), falls
/// back to its own `semantic_type` (recursing through `bounded_domain`
/// refinements, e.g. a `float_rounding`/`text_bounds` wrapper, to the family
/// it refines). A `literal` resolves through its declared `type` and an
/// `application` through its `result_type`, except that a clause application
/// (a catalogued `clause` result) has family `clause`. `binding` is the
/// catalog's `binder` family (`quire.op.collection.sum.*`'s bound
/// variable). Every other term shape, and any term whose type does not
/// resolve, is not resolved (`None`), so a caller skips the check it would
/// otherwise support rather than guess.
fn argument_family(
    argument: &Value,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    catalog: &OperationCatalog,
) -> Option<&'static str> {
    match body_term(argument) {
        Some(BodyTerm::Application) if is_clause_application(argument, catalog) => Some("clause"),
        Some(BodyTerm::Reference | BodyTerm::Literal | BodyTerm::Application) => {
            let type_node = operand_type_node(argument, nodes, kinds, index)?;
            resolve_family(&type_node, nodes, kinds, index)
        }
        Some(BodyTerm::Binding) => Some("binder"),
        // A `dependency_reference` callee has family `function` (FR-322), and
        // the dependency-reference walk (step 7), which runs before the
        // operand checks, has already refused it anywhere but a
        // `quire.op.function.call` callee, so no operand check reads it.
        Some(
            BodyTerm::Aggregate
            | BodyTerm::DependencyReference
            | BodyTerm::Frame
            | BodyTerm::AbstractionRelation,
        )
        | None => None,
    }
}

/// Whether an `application` term's catalogued operation has result `clause`.
fn is_clause_application(argument: &Value, catalog: &OperationCatalog) -> bool {
    argument
        .get("operation")
        .and_then(|operation| operation.get("identity"))
        .and_then(Value::as_str)
        .and_then(|identity| catalog.entry(identity))
        .is_some_and(|entry| &*entry.result == "clause")
}

/// The node a `reference` term names, unresolved; `None` for any other term.
fn reference_term_target(argument: &Value) -> Option<CheckedNodeId> {
    if body_term(argument) != Some(BodyTerm::Reference) {
        return None;
    }
    serde_json::from_value(argument.get("target")?.clone()).ok()
}

/// The operation catalog's operand family a node of this kind denotes
/// directly, or `None` when it has none of its own (a `bounded_domain` then
/// resolves through its semantic type). Exhaustive over every form. An enum
/// is reported as `enum` here; [`resolve_family`] refines it to
/// `ordered_enum` from the node's nominal preimage.
fn operand_family(kind: CheckedNodeKind) -> Option<&'static str> {
    use CheckedNodeKind as K;
    match kind {
        K::ScalarType(
            form @ (ScalarTypeForm::Boolean
            | ScalarTypeForm::Integer
            | ScalarTypeForm::Rational
            | ScalarTypeForm::Decimal
            | ScalarTypeForm::Float32
            | ScalarTypeForm::Float64
            | ScalarTypeForm::Text
            | ScalarTypeForm::Enum),
        ) => Some(form.as_wire()),
        K::ScalarType(
            ScalarTypeForm::Dimension | ScalarTypeForm::Unit | ScalarTypeForm::CompoundUnit,
        ) => None,
        K::CompositeType(
            form @ (CompositeTypeForm::Option
            | CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet
            | CompositeTypeForm::Record
            | CompositeTypeForm::Tuple
            | CompositeTypeForm::Union
            | CompositeTypeForm::Reference),
        ) => Some(form.as_wire()),
        K::CompositeType(CompositeTypeForm::Alias) => None,
        K::Relation(RelationForm::Population) => Some("population"),
        K::Relation(
            RelationForm::Relationship | RelationForm::Membership | RelationForm::CausalRelation,
        ) => None,
        K::Function(
            FunctionForm::PureFunction | FunctionForm::Predicate | FunctionForm::RecursiveFunction,
        ) => Some("function"),
        // The catalog's `reference` family: an identity/pointer expression
        // (a `reference` term naming a relationship end or declaration),
        // never `composite_type`'s same-named nullable-wrapper form.
        K::Expression(ExpressionForm::Reference) => Some("reference"),
        K::Expression(
            ExpressionForm::Call
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
        ) => None,
        K::BoundedDomain(
            BoundedDomainForm::IntegerRange
            | BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds
            | BoundedDomainForm::CollectionBounds
            | BoundedDomainForm::ModelPopulation,
        ) => None,
        K::Value(
            ValueForm::Literal
            | ValueForm::EnumValue
            | ValueForm::CollectionValue
            | ValueForm::RecordValue
            | ValueForm::TupleValue
            | ValueForm::UnionValue
            | ValueForm::OptionValue
            | ValueForm::Parameter,
        ) => None,
        // Implements: FR-322 (STD-102). a model declaration node of an object type or a
        // systems interface is the `object` family, the `inner:0` result of
        // `quire.op.model.deref`, which only a `field_owner` position admits.
        K::Model(ModelForm::ObjectType | ModelForm::SystemsInterface) => Some("object"),
        K::Model(
            ModelForm::ModelImport
            | ModelForm::ValueType
            | ModelForm::VariantType
            | ModelForm::RecordValueType
            | ModelForm::EventType
            | ModelForm::StateMachine
            | ModelForm::Process
            | ModelForm::PersistenceInterface
            | ModelForm::Namespace
            | ModelForm::SystemsPart
            | ModelForm::SystemsPort
            | ModelForm::SystemsConnection
            | ModelForm::SystemsAllocation,
        ) => None,
        K::State(
            StateForm::StateClause
            | StateForm::Frame
            | StateForm::Transition
            | StateForm::OperationAnchor
            | StateForm::Snapshot,
        ) => None,
        // The catalog's `temporal` family: a temporal formula (QSpec FR-370).
        K::Temporal(TemporalForm::Formula) => Some("temporal"),
        K::Temporal(
            TemporalForm::TemporalClause
            | TemporalForm::Fairness
            | TemporalForm::Clock
            | TemporalForm::Window
            | TemporalForm::Activation
            | TemporalForm::Deadline,
        ) => None,
        K::Protocol(
            ProtocolForm::ProtocolClause
            | ProtocolForm::Role
            | ProtocolForm::Channel
            | ProtocolForm::Queue
            | ProtocolForm::Control
            | ProtocolForm::Obligation
            | ProtocolForm::Compensation,
        ) => None,
        K::Claim(
            ClaimForm::VerificationClaim
            | ClaimForm::AnalysisClaim
            | ClaimForm::Hyperproperty
            | ClaimForm::SynthesisRequest,
        ) => None,
        K::Correspondence(
            CorrespondenceForm::SourceLocus
            | CorrespondenceForm::ModelCorrespondence
            | CorrespondenceForm::BindingRole
            | CorrespondenceForm::ProfileCorrespondence
            | CorrespondenceForm::AbstractionRelation,
        ) => None,
    }
}

/// Whether an argument naming a node of this kind names a type itself
/// rather than a value of its semantic type: every form of the five type
/// families, and of the expressions only `reference`. Every form is listed.
fn is_type_shaped(kind: CheckedNodeKind) -> bool {
    use CheckedNodeKind as K;
    match kind {
        K::ScalarType(
            ScalarTypeForm::Boolean
            | ScalarTypeForm::Integer
            | ScalarTypeForm::Rational
            | ScalarTypeForm::Decimal
            | ScalarTypeForm::Float32
            | ScalarTypeForm::Float64
            | ScalarTypeForm::Text
            | ScalarTypeForm::Dimension
            | ScalarTypeForm::Unit
            | ScalarTypeForm::Enum
            | ScalarTypeForm::CompoundUnit,
        ) => true,
        K::CompositeType(
            CompositeTypeForm::Option
            | CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet
            | CompositeTypeForm::Record
            | CompositeTypeForm::Tuple
            | CompositeTypeForm::Union
            | CompositeTypeForm::Alias
            | CompositeTypeForm::Reference,
        ) => true,
        K::BoundedDomain(
            BoundedDomainForm::IntegerRange
            | BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds
            | BoundedDomainForm::CollectionBounds
            | BoundedDomainForm::ModelPopulation,
        ) => true,
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
        K::Expression(ExpressionForm::Reference) => true,
        K::Expression(
            ExpressionForm::Call
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
        ) => true,
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
        ) => true,
        K::State(
            StateForm::StateClause
            | StateForm::Frame
            | StateForm::Transition
            | StateForm::OperationAnchor
            | StateForm::Snapshot,
        ) => false,
        // A formula is named as a type-like operand of the `temporal` family:
        // a `reference` to one resolves to that node itself.
        K::Temporal(TemporalForm::Formula) => true,
        K::Temporal(
            TemporalForm::TemporalClause
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
            | CorrespondenceForm::ProfileCorrespondence
            | CorrespondenceForm::AbstractionRelation,
        ) => false,
    }
}

/// The operand family of the type `type_id` names: its own, or, for a
/// `bounded_domain`, that of the type its `semantic_type` names, however long
/// the chain of bounded domains is. The walk is iterative and follows each node
/// once, so a chain has no length limit and a cycle of bounded domains, which
/// ends at no type, names no family (`None`), as an unresolved type does.
fn resolve_family(
    type_id: &CheckedNodeId,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<&'static str> {
    let mut seen = BTreeSet::new();
    let mut current = type_id;
    loop {
        let position = *index.get(current)?;
        if !seen.insert(position) {
            return None;
        }
        let node = &nodes[position];
        let kind = *kinds.get(position)?;
        // An enum is `ordered_enum` when its nominal preimage is ordered and
        // `enum` otherwise (QSpec FR-322), which the node's kind alone cannot say.
        if kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Enum) {
            let ordered = matches!(
                &node.nominal_identity_preimage,
                Some(NominalIdentityPreimage::EnumDeclaration(declaration)) if declaration.ordered
            );
            return Some(if ordered { "ordered_enum" } else { "enum" });
        }
        let direct = operand_family(kind);
        if direct.is_some() {
            return direct;
        }
        if kind.tag() != CheckedNodeTag::BoundedDomain {
            return None;
        }
        current = &node.semantic_type;
    }
}

fn check_operands(
    application: Application<'_>,
    operation: &OperationWire,
    entry: &OperationCatalogEntry,
    arguments: &[Value],
    graph: &Graph<'_>,
    owners: &ModelOwners<'_>,
    catalog: &OperationCatalog,
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let Graph {
        nodes,
        kinds,
        index,
    } = *graph;
    let required = entry.operands.len();
    // `None` names the `arguments` array itself (an arity defect); `Some`
    // names the one argument at fault.
    let ineligible = |argument: Option<usize>| {
        let arguments_at = application.body(&["arguments"]);
        Ok(Some(application.refuse(
            CheckedPackageRefusalCode::IllTyped,
            argument.map_or_else(|| arguments_at.clone(), |at| arguments_at.clone().index(at)),
            CheckedPackageRefusalCause::OperatorIneligible,
        )))
    };
    if entry.rest.is_none() {
        if arguments.len() != required {
            return ineligible(None);
        }
    } else if arguments.len() < required {
        return ineligible(None);
    }
    for (position, expected) in entry.operands.iter().enumerate() {
        // QSpec FR-370: a temporal clause's first argument is a `reference`
        // to the `value`/`parameter` node of its `over` parameter. It is not
        // family-resolved (resolving it would give the parameter's own
        // family); it fits the `reference` operand exactly when it is a
        // `reference` term.
        if entry.operator == ApplicationOperator::Temporal
            && position == 0
            && &**expected == "reference"
        {
            if body_term(&arguments[0]) != Some(BodyTerm::Reference) {
                return ineligible(Some(0));
            }
            continue;
        }
        if let Some(actual) = argument_family(&arguments[position], nodes, kinds, index, catalog) {
            if !catalog.family_fits(actual, expected) {
                return ineligible(Some(position));
            }
        }
    }
    if let Some(rest_family) = &entry.rest {
        for (offset, argument) in arguments[required..].iter().enumerate() {
            if let Some(actual) = argument_family(argument, nodes, kinds, index, catalog) {
                if !catalog.family_fits(actual, rest_family) {
                    return ineligible(Some(required.saturating_add(offset)));
                }
            }
        }
    }
    for constraint in &entry.constraints {
        let indices: Option<Vec<usize>> = constraint
            .operands
            .iter()
            .map(|position| usize::try_from(*position).ok())
            .collect();
        let Some(indices) = indices else { continue };
        if indices.iter().any(|position| *position >= arguments.len()) {
            continue;
        }
        match constraint.kind {
            OperationConstraintKind::SameFamily => {
                let families: Option<Vec<&str>> = indices
                    .iter()
                    .map(|position| {
                        argument_family(&arguments[*position], nodes, kinds, index, catalog)
                    })
                    .collect();
                if let Some(families) = families {
                    if let Some(pair) = families.windows(2).position(|pair| pair[0] != pair[1]) {
                        return ineligible(indices.get(pair.saturating_add(1)).copied());
                    }
                }
            }
            OperationConstraintKind::SameType => {
                let types: Option<Vec<CheckedNodeId>> = indices
                    .iter()
                    .map(|position| operand_type_node(&arguments[*position], nodes, kinds, index))
                    .collect();
                if let Some(types) = types {
                    if let Some(pair) = types.windows(2).position(|pair| pair[0] != pair[1]) {
                        return ineligible(indices.get(pair.saturating_add(1)).copied());
                    }
                }
            }
            OperationConstraintKind::ConformingReference => {
                if let [first, second] = indices[..] {
                    if let Some(failure) = check_conforming_reference(
                        application,
                        [first, second],
                        arguments,
                        graph,
                        owners,
                        meter,
                    )? {
                        return Ok(Some(failure));
                    }
                }
            }
            // The catalog declares these, and this reader enforces none of
            // them (see the module documentation); each is named so a member
            // added to the vocabulary is a compile error here.
            OperationConstraintKind::SameDimension
            | OperationConstraintKind::InnerType
            | OperationConstraintKind::MemberOf
            | OperationConstraintKind::MemberFamily
            | OperationConstraintKind::BoundFamily
            | OperationConstraintKind::BoundIsMember
            | OperationConstraintKind::ExactConversion
            | OperationConstraintKind::RangeNarrowing
            | OperationConstraintKind::RationalNarrowing
            | OperationConstraintKind::ScaleReduction
            | OperationConstraintKind::PromotesExact
            | OperationConstraintKind::UniformRest => {}
            // QSpec FR-440: only `quire.op.control.case` catalogues this. A
            // constraint that does not hold, or that names anything but one
            // scrutinee, refuses at the `case` node: it is never skipped.
            OperationConstraintKind::UnionArms => {
                let held = matches!(
                    indices[..],
                    [scrutinee] if union_arms_hold(application, scrutinee, required, arguments, graph)
                );
                if !held {
                    return Ok(Some(application.refuse(
                        CheckedPackageRefusalCode::IllTyped,
                        node_pointer(application.position),
                        CheckedPackageRefusalCause::OperatorIneligible,
                    )));
                }
            }
            OperationConstraintKind::ReferenceEdge => {
                if let [first, second] = indices[..] {
                    if let Some(failure) = check_reference_edge(
                        application,
                        operation,
                        [first, second],
                        arguments,
                        graph,
                        owners,
                        meter,
                    )? {
                        return Ok(Some(failure));
                    }
                }
            }
        }
    }
    Ok(None)
}

/// The members of the union type node `type_id` names, through aliases and
/// bounded domains; `None` when it names no union type node.
fn union_type<'g>(type_id: &CheckedNodeId, graph: &Graph<'g>) -> Option<Vec<UnionMember<'g>>> {
    let (position, kind) = structural_type(type_id, graph.nodes, graph.kinds, graph.index)?;
    if kind != CheckedNodeKind::CompositeType(CompositeTypeForm::Union) {
        return None;
    }
    union_type_body(&graph.nodes[position].body)
}

/// QSpec FR-440's `union_arms` constraint: the arguments past the scrutinee
/// are exactly one arm per member of the scrutinee's union, in member
/// declaration order, each a `binding` named by its member whose value is an
/// `aggregate` of a binder aggregate and the arm body; the binder aggregate
/// holds as many `reference`s to `value`/`parameter` nodes as the member's
/// payload arity, each binder's `semantic_type` the payload type at its
/// position; and every arm body has the application's `result_type`.
fn union_arms_hold(
    application: Application<'_>,
    scrutinee: usize,
    arms_from: usize,
    arguments: &[Value],
    graph: &Graph<'_>,
) -> bool {
    let Graph {
        nodes,
        kinds,
        index,
    } = *graph;
    let Some(members) = arguments
        .get(scrutinee)
        .and_then(|argument| operand_type_node(argument, nodes, kinds, index))
        .and_then(|type_id| union_type(&type_id, graph))
    else {
        return false;
    };
    let result_type: Option<CheckedNodeId> = application
        .node
        .body
        .get("result_type")
        .and_then(|value| serde_json::from_value(value.clone()).ok());
    let arms = arguments.get(arms_from..).unwrap_or_default();
    arms.len() == members.len()
        && members.iter().zip(arms).all(|(member, arm)| {
            let Some([binders, body]) = binding(arm, member.name).and_then(aggregate_members)
            else {
                return false;
            };
            let binders_hold = aggregate_members(binders).is_some_and(|binders| {
                binders.len() == member.payload.len()
                    && binders
                        .iter()
                        .zip(&member.payload)
                        .all(|(binder, payload)| {
                            reference_term_target(binder)
                                .and_then(|target| index.get(&target).copied())
                                .is_some_and(|position| {
                                    kinds[position] == CheckedNodeKind::Value(ValueForm::Parameter)
                                        && nodes[position].semantic_type == *payload
                                })
                        })
            });
            binders_hold
                && result_type.as_ref().is_some_and(|result| {
                    operand_type_node(body, nodes, kinds, index).as_ref() == Some(result)
                })
        })
}

/// Whether a node of this kind is a type node: a `scalar_type`,
/// `composite_type` or `bounded_domain`.
fn is_type_node(kind: CheckedNodeKind) -> bool {
    matches!(
        kind.tag(),
        CheckedNodeTag::ScalarType | CheckedNodeTag::CompositeType | CheckedNodeTag::BoundedDomain
    )
}

/// Merged QSpec FR-440 join 1, the first check of a node's joins at the
/// operation step: a `case` application stands only at the body root of an
/// `expression`/`case` node. An `expression` node whose form contradicts its root
/// application's operator class (an `expression`/`case` node whose body is no
/// `case` application, an `expression` node of another form whose body root is
/// one) is `invalid_semantic_graph` at the node's `body`; a `case` application
/// as the body root of a node that is no `expression` node is
/// `ill_typed`/`operator-ineligible` at the node. A `case` nested in another term
/// was refused ahead of every identity check, with every other nested
/// application (`flat_wire`).
fn case_placement_defect(position: usize, graph: &Graph<'_>) -> Option<ValidationFailure> {
    let node = &graph.nodes[position];
    let kind = graph.kinds[position];
    let root_is_case = is_application(&node.body)
        && application_operator(&node.body) == Some(ApplicationOperator::Case);
    let is_case_node = kind == CheckedNodeKind::Expression(ExpressionForm::Case);
    let refuse = |code, path, cause| {
        Some(ValidationFailure::refused_at(
            code,
            path,
            cause,
            node.node_id.clone(),
        ))
    };
    match (is_case_node, root_is_case) {
        (true, true) | (false, false) => None,
        (true, false) => refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(position).key("body"),
            None,
        ),
        (false, true) if kind.tag() == CheckedNodeTag::Expression => refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(position).key("body"),
            None,
        ),
        (false, true) => refuse(
            CheckedPackageRefusalCode::IllTyped,
            node_pointer(position),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
        ),
    }
}

/// QSpec FR-440's reader joins 1 and 2, checked at the operation step in
/// node-id digest order: a union type holds no member name twice
/// (`invalid_package`/`duplicate-member`) and every payload reference names a
/// type node (`ill_typed`/`operator-ineligible`); a union value's
/// `semantic_type` is a union type node, its binding names a member of it, and
/// its payload terms are that member's payload in count and in type
/// (`ill_typed`/`type-mismatch`). Each is located at the node. A body that is
/// not its form's closed shape was refused by the structural stage.
fn union_defect(position: usize, graph: &Graph<'_>) -> Option<ValidationFailure> {
    let Graph {
        nodes,
        kinds,
        index,
    } = *graph;
    let node = &nodes[position];
    let refuse = |code, cause| {
        ValidationFailure::refused_at(
            code,
            node_pointer(position),
            Some(cause),
            node.node_id.clone(),
        )
    };
    let kind = kinds[position];
    if kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Union) {
        let members = union_type_body(&node.body)?;
        let mut names = BTreeSet::new();
        if !members.iter().all(|member| names.insert(member.name)) {
            return Some(refuse(
                CheckedPackageRefusalCode::InvalidPackage,
                CheckedPackageRefusalCause::DuplicateMember,
            ));
        }
        let payload_is_type = members
            .iter()
            .flat_map(|member| &member.payload)
            .all(|target| {
                index
                    .get(target)
                    .is_none_or(|&target| is_type_node(kinds[target]))
            });
        return (!payload_is_type).then(|| {
            refuse(
                CheckedPackageRefusalCode::IllTyped,
                CheckedPackageRefusalCause::OperatorIneligible,
            )
        });
    }
    if kind == CheckedNodeKind::Value(ValueForm::UnionValue) {
        let (name, payload) = union_value_body(&node.body)?;
        // A `semantic_type` that is no node of the graph is the generic edge
        // resolution's refusal, later.
        let declared = *index.get(&node.semantic_type)?;
        let mismatch = || {
            refuse(
                CheckedPackageRefusalCode::IllTyped,
                CheckedPackageRefusalCause::TypeMismatch,
            )
        };
        if kinds[declared] != CheckedNodeKind::CompositeType(CompositeTypeForm::Union) {
            return Some(mismatch());
        }
        let members = union_type_body(&nodes[declared].body)?;
        let holds = members
            .iter()
            .find(|member| member.name == name)
            .is_some_and(|member| {
                member.payload.len() == payload.len()
                    && payload.iter().zip(&member.payload).all(|(term, expected)| {
                        operand_type_node(term, nodes, kinds, index).as_ref() == Some(expected)
                    })
            });
        return (!holds).then(mismatch);
    }
    None
}

/// FR-322 "Reaches over a field": the `reference_edge` constraint of
/// `quire.op.model.reaches_field` holds exactly when, in order, operand 0 is
/// `Reference<D>` for the member's declaring node `D`; the member resolves
/// on `D` to a field owned by object type `T`; the field's type is
/// `Reference<T>`, `Option<Reference<T>>` or a bounded
/// `Sequence<Reference<T>>`; and operand 1 is a `Reference<B>` whose object
/// type `B` is `T` or a declared subtype of `T`. An operand whose type cannot be resolved cannot
/// be shown to satisfy the edge and is refused.
fn check_reference_edge(
    application: Application<'_>,
    operation: &OperationWire,
    operands: [usize; 2],
    arguments: &[Value],
    graph: &Graph<'_>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let [source, target] = operands;
    let member_at = |member: &str| application.body(&["operation", "member", member]);
    let argument_at = |position: usize| application.body(&["arguments"]).index(position);
    let ineligible = |path: JsonPointer| {
        Ok(Some(model_refusal(
            application,
            path,
            ModelRefusal::ineligible(),
        )))
    };
    let reference_operand = |position: usize| {
        let type_id =
            operand_type_node(&arguments[position], graph.nodes, graph.kinds, graph.index)?;
        reference_target(&type_id, graph)
    };
    let declaring = member_declaration(operation)
        .and_then(|declaration| graph.index.get(&declaration).copied())
        .map(|position| (&graph.nodes[position], graph.kinds[position].tag()));
    let Some((declaring, tag)) = declaring else {
        return ineligible(member_at("declaration"));
    };
    if !owners.is_model_declaration_node(declaring, tag) {
        return ineligible(member_at("declaration"));
    }
    if reference_operand(source).as_ref() != Some(&declaring.node_id) {
        return ineligible(argument_at(source));
    }
    let owner = match owners.recover(declaring) {
        Ok(owner) => owner,
        Err(refusal) => {
            return Ok(Some(model_refusal(
                application,
                member_at("declaration"),
                refusal,
            )))
        }
    };
    if owner.object_type().is_none() {
        return ineligible(member_at("name"));
    }
    let name = operation
        .member
        .as_ref()
        .and_then(|member| member.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let resolved = {
        let mut budget = Budget::new(meter, owner.selection, owner.package.bytes);
        match owner
            .package
            .resolve(owner.node, MemberKind::Field, name, &mut budget)
        {
            Ok(resolved) => resolved,
            Err(ModelFailure::Refused(refusal)) => {
                return Ok(Some(model_refusal(application, member_at("name"), refusal)))
            }
            Err(ModelFailure::Limit(failure)) => return Err(failure),
        }
    };
    let Resolved::Field(field) = resolved else {
        return ineligible(member_at("name"));
    };
    // `T`: the object type that declares the resolved field.
    let Some(edge_owner) = owner
        .package
        .object_types
        .iter()
        .find(|(_, declared)| declared.fields.iter().any(|f| f.identity == field.identity))
        .map(|(node, _)| node.as_ref())
    else {
        return ineligible(member_at("name"));
    };
    // `ModelOwners::new` derived this key (or a longer one for an interface)
    // under the same limit, so the encoder cannot refuse it here; were it to,
    // the refusal is `invalid_semantic_graph` at the node, as a refused node
    // key is.
    let Ok(edge_key) = declaration_key(
        &owner.package.identity,
        DeclarationForm::ObjectType,
        edge_owner,
        owner.package.bytes,
    ) else {
        return Ok(Some(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            application.node_id(),
        )));
    };
    let edge = MemberType::Reference(edge_key.into());
    let is_edge = match owner.package.field_type(field) {
        Some(MemberType::Option(inner)) => *inner == edge,
        Some(MemberType::Collection {
            kind: CollectionKind::Sequence,
            element,
            bounds: Some(_),
        }) => *element == edge,
        Some(reference) => reference == edge,
        None => false,
    };
    if !is_edge {
        return ineligible(member_at("name"));
    }
    let Some(end) = reference_operand(target) else {
        return ineligible(argument_at(target));
    };
    let Some(end_position) = graph.index.get(&end).copied() else {
        return ineligible(argument_at(target));
    };
    let end_node = &graph.nodes[end_position];
    if !owners.is_model_declaration_node(end_node, graph.kinds[end_position].tag()) {
        return ineligible(argument_at(target));
    }
    let end_owner = match owners.recover(end_node) {
        Ok(end_owner) => end_owner,
        Err(refusal) => {
            return Ok(Some(model_refusal(
                application,
                argument_at(target),
                refusal,
            )))
        }
    };
    if !std::ptr::eq(end_owner.package, owner.package) || end_owner.object_type().is_none() {
        return ineligible(argument_at(target));
    }
    let mut budget = Budget::new(meter, owner.selection, owner.package.bytes);
    if owner
        .package
        .conforms_to(end_owner.node, edge_owner, &mut budget)?
    {
        Ok(None)
    } else {
        ineligible(argument_at(target))
    }
}

/// The `operation.member.declaration` an application names.
fn member_declaration(operation: &OperationWire) -> Option<CheckedNodeId> {
    serde_json::from_value(operation.member.as_ref()?.get("declaration")?.clone()).ok()
}

/// The selected relationship end of a `navigate` or `reaches` application.
fn check_relationship_member(
    application: Application<'_>,
    operation: &OperationWire,
    arguments: &[Value],
    graph: &Graph<'_>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let member_at = |member: &str| application.body(&["operation", "member", member]);
    let argument_at = |position: usize| application.body(&["arguments"]).index(position);
    let refuse = |path: JsonPointer, refusal: ModelRefusal| {
        Ok(Some(model_refusal(application, path, refusal)))
    };
    let ineligible = |path: JsonPointer| refuse(path, ModelRefusal::ineligible());
    let key_refused = || {
        Ok(Some(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            application.node_id(),
        )))
    };
    let Some(declaring) =
        member_declaration(operation).and_then(|id| graph.index.get(&id).copied())
    else {
        return refuse(member_at("declaration"), ModelRefusal::unselected());
    };
    let node = &graph.nodes[declaring];
    if !owners.is_model_declaration_node(node, graph.kinds[declaring].tag()) {
        return refuse(member_at("declaration"), ModelRefusal::unselected());
    }
    let owner = match owners.recover(node) {
        Ok(owner) => owner,
        Err(refusal) => return refuse(member_at("declaration"), refusal),
    };
    if owner.form != DeclarationForm::Relationship {
        return ineligible(member_at("declaration"));
    }
    let Some(relationship) = owner.package.relationships.get(owner.node) else {
        return refuse(member_at("declaration"), ModelRefusal::unselected());
    };
    let mut budget = Budget::new(meter, owner.selection, owner.package.bytes);
    budget.charge(1)?;
    let name = operation
        .member
        .as_ref()
        .and_then(|member| member.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let forward = match (
        relationship.source.role.as_deref() == Some(name),
        relationship.target.role.as_deref() == Some(name),
    ) {
        (true, false) => true,
        (false, true) => false,
        (false, false) => return refuse(member_at("name"), ModelRefusal::missing_name()),
        (true, true) => return refuse(member_at("name"), ModelRefusal::ambiguous()),
    };
    let (receiver, destination) = if forward {
        (&relationship.source, &relationship.target)
    } else {
        (&relationship.target, &relationship.source)
    };
    let operand_owner = |position: usize| {
        let Some(argument) = arguments.get(position) else {
            return Ok(None);
        };
        let Some(type_id) = operand_type_node(argument, graph.nodes, graph.kinds, graph.index)
        else {
            return Ok(None);
        };
        let Some(target) = reference_target(&type_id, graph) else {
            return Ok(None);
        };
        let Some(&at) = graph.index.get(&target) else {
            return Ok(None);
        };
        let node = &graph.nodes[at];
        if !owners.is_model_declaration_node(node, graph.kinds[at].tag()) {
            return Ok(None);
        }
        owners.recover(node).map(Some)
    };
    let actual_receiver = match operand_owner(0) {
        Ok(Some(owner)) => owner,
        Ok(None) => return ineligible(argument_at(0)),
        Err(refusal) => return refuse(argument_at(0), refusal),
    };
    if !std::ptr::eq(actual_receiver.package, owner.package)
        || actual_receiver.object_type().is_none()
        || !owner
            .package
            .conforms_to(actual_receiver.node, &receiver.type_ref, &mut budget)?
    {
        return ineligible(argument_at(0));
    }
    if !relationship.direction.admits(forward) {
        return ineligible(member_at("name"));
    }
    let Some(destination_object) = owner.package.object_types.get(&destination.type_ref) else {
        return ineligible(member_at("name"));
    };
    let form = if destination_object.interface {
        DeclarationForm::SystemsInterface
    } else {
        DeclarationForm::ObjectType
    };
    let Ok(destination_key) = declaration_key(
        &owner.package.identity,
        form,
        &destination.type_ref,
        owner.package.bytes,
    ) else {
        return key_refused();
    };
    let member_type = match relationship.navigation_type(forward, destination_key.into()) {
        Ok(member_type) => member_type,
        Err(refusal) => return refuse(member_at("name"), refusal),
    };
    if operation.identity.as_ref() == "quire.op.model.reaches" {
        if relationship.source.type_ref != relationship.target.type_ref
            || !matches!(
                member_type,
                MemberType::Reference(_) | MemberType::Option(_)
            )
        {
            return ineligible(member_at("name"));
        }
        let actual_target = match operand_owner(1) {
            Ok(Some(owner)) => owner,
            Ok(None) => return ineligible(argument_at(1)),
            Err(refusal) => return refuse(argument_at(1), refusal),
        };
        if !std::ptr::eq(actual_target.package, owner.package)
            || actual_target.object_type().is_none()
            || !owner
                .package
                .conforms_to(actual_target.node, &destination.type_ref, &mut budget)?
        {
            return ineligible(argument_at(1));
        }
    }
    let expected = if operation.identity.as_ref() == "quire.op.model.reaches" {
        MemberType::Boolean
    } else {
        member_type
    };
    let Ok(expected_key) = expected.node_key(owner.package.bytes) else {
        return key_refused();
    };
    let result = application
        .node
        .body
        .get("result_type")
        .and_then(|value| value.get("digest"))
        .and_then(Value::as_str);
    if result != Some(expected_key.as_str()) {
        return ineligible(application.body(&["result_type"]));
    }
    Ok(None)
}

/// The node a `composite_type`/`reference` type node's body references: `X`
/// of `Reference<X>`. `None` for any other node.
fn reference_target(type_id: &CheckedNodeId, graph: &Graph<'_>) -> Option<CheckedNodeId> {
    let position = *graph.index.get(type_id)?;
    if *graph.kinds.get(position)? != CheckedNodeKind::CompositeType(CompositeTypeForm::Reference) {
        return None;
    }
    let members = graph.nodes[position].body.get("members")?.as_array()?;
    let [member] = members.as_slice() else {
        return None;
    };
    serde_json::from_value(member.get("target")?.clone()).ok()
}

/// A model-owned member refusal at `path`.
fn model_refusal(
    application: Application<'_>,
    path: JsonPointer,
    refusal: ModelRefusal,
) -> ValidationFailure {
    application.refuse(refusal.code, path, refusal.cause)
}

/// FR-322 "Reference conformance" (STD-101): the two `reference` operands'
/// object types, recovered by step 2 even when both name one type node, are
/// object type declarations of one selected document, one conforming to the
/// other along declared supertypes. An operand whose type resolves to no
/// node is left to the checks that report an unresolved argument.
fn check_conforming_reference(
    application: Application<'_>,
    operands: [usize; 2],
    arguments: &[Value],
    graph: &Graph<'_>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let at = |position: usize| application.body(&["arguments"]).index(position);
    let ineligible = |position: usize| {
        Ok(Some(model_refusal(
            application,
            at(position),
            ModelRefusal::ineligible(),
        )))
    };
    let mut recovered = Vec::with_capacity(2);
    for position in operands {
        let Some(type_id) =
            operand_type_node(&arguments[position], graph.nodes, graph.kinds, graph.index)
        else {
            return Ok(None);
        };
        let Some(target) = reference_target(&type_id, graph) else {
            return ineligible(position);
        };
        let Some(target_position) = graph.index.get(&target) else {
            return Ok(None);
        };
        let target_node = &graph.nodes[*target_position];
        if !owners.is_model_declaration_node(target_node, graph.kinds[*target_position].tag()) {
            return ineligible(position);
        }
        match owners.recover(target_node) {
            Ok(owner) => recovered.push(owner),
            Err(refusal) => {
                return Ok(Some(model_refusal(application, at(position), refusal)));
            }
        }
    }
    let [a, b] = recovered.as_slice() else {
        return Ok(None);
    };
    // A declaration that is no object type is the fault of its own operand;
    // two documents, or two types that do not conform, fault the pair, whose
    // refusal is at the second operand.
    for (position, owner) in operands.into_iter().zip([a, b]) {
        if owner.object_type().is_none() {
            return ineligible(position);
        }
    }
    if !std::ptr::eq(a.package, b.package) {
        return ineligible(operands[1]);
    }
    let mut budget = Budget::new(meter, a.selection, a.package.bytes);
    if a.package.conforms(a.node, b.node, &mut budget)? {
        Ok(None)
    } else {
        ineligible(operands[1])
    }
}

/// The `inner:<n>` result form over a `reference` operand
/// (`quire.op.model.deref`): the application's `result_type` is the node
/// the operand's `Reference<X>` names, and that node has a family, so a
/// `deref` of a relationship reference types nothing an operand admits.
/// Other `inner:<n>` operands are not checked here.
// Parses the catalog's `inner:<n>` result-form text.
fn check_inner_result(
    application: Application<'_>,
    entry: &OperationCatalogEntry,
    arguments: &[Value],
    graph: &Graph<'_>,
) -> Option<ValidationFailure> {
    let operand: usize = entry.result.strip_prefix("inner:")?.parse().ok()?;
    let type_id = operand_type_node(
        arguments.get(operand)?,
        graph.nodes,
        graph.kinds,
        graph.index,
    )?;
    let inner = reference_target(&type_id, graph)?;
    let result_type: Option<CheckedNodeId> = application
        .node
        .body
        .get("result_type")
        .and_then(|value| serde_json::from_value(value.clone()).ok());
    let typed = result_type.as_ref() == Some(&inner)
        && resolve_family(&inner, graph.nodes, graph.kinds, graph.index).is_some();
    (!typed).then(|| {
        application.refuse(
            CheckedPackageRefusalCode::IllTyped,
            application.body(&["result_type"]),
            CheckedPackageRefusalCause::OperatorIneligible,
        )
    })
}

/// The `clause` result of `quire.op.state.clause` is declared Boolean
/// (QSpec FR-341): its `result_type` names the `scalar_type`/`boolean` node,
/// else `ill_typed`/`operator-ineligible` at the `result_type`.
fn check_state_clause_result(
    application: Application<'_>,
    entry: &OperationCatalogEntry,
    graph: &Graph<'_>,
) -> Option<ValidationFailure> {
    if entry.operator != ApplicationOperator::StateClause {
        return None;
    }
    let boolean = application
        .node
        .body
        .get("result_type")
        .and_then(|value| serde_json::from_value::<CheckedNodeId>(value.clone()).ok())
        .and_then(|result_type| graph.index.get(&result_type).copied())
        .and_then(|position| graph.kinds.get(position))
        == Some(&CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean));
    (!boolean).then(|| {
        application.refuse(
            CheckedPackageRefusalCode::IllTyped,
            application.body(&["result_type"]),
            CheckedPackageRefusalCause::OperatorIneligible,
        )
    })
}

/// FR-322 "Model-owned members" steps 2 to 4 for a `field` or `operation`
/// member whose declaring node is a model declaration node, at the
/// application's `operator-ineligible` check. A field is read over an
/// operand of the declaring node's type (`member_of`); an operation is
/// dispatched on a `Reference<X>` receiver of that node, with one argument
/// per parameter, each of its parameter's type node. The member type is
/// compared with the application's `result_type` by node key.
fn check_model_member(
    application: Application<'_>,
    operation: &OperationWire,
    kind: MemberKind,
    declaring: &CheckedSemanticNodeV2,
    arguments: &[Value],
    graph: &Graph<'_>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let member_at = |member: &str| application.body(&["operation", "member", member]);
    let argument_at = |position: usize| application.body(&["arguments"]).index(position);
    let refuse = |path: JsonPointer, refusal: ModelRefusal| {
        Ok(Some(model_refusal(application, path, refusal)))
    };
    let ineligible = |path: JsonPointer| refuse(path, ModelRefusal::ineligible());
    // A type key the encoder refuses (a preimage past the byte limit) is
    // `invalid_semantic_graph` at the node, as a refused node key is.
    let key_refused = || {
        Ok(Some(ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            application.node_id(),
        )))
    };
    let type_node = |position: usize| {
        arguments
            .get(position)
            .and_then(|argument| operand_type_node(argument, graph.nodes, graph.kinds, graph.index))
    };
    let owner = match owners.recover(declaring) {
        Ok(owner) => owner,
        Err(refusal) => return refuse(member_at("declaration"), refusal),
    };
    if kind == MemberKind::Field && type_node(0).as_ref() != Some(&declaring.node_id) {
        return ineligible(argument_at(0));
    }
    let name = operation
        .member
        .as_ref()
        .and_then(|member| member.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let Some(object) = owner.object_type() else {
        return ineligible(member_at("name"));
    };
    let mut budget = Budget::new(meter, owner.selection, owner.package.bytes);
    let resolved = match owner.package.resolve(owner.node, kind, name, &mut budget) {
        Ok(resolved) => resolved,
        Err(ModelFailure::Refused(refusal)) => return refuse(member_at("name"), refusal),
        Err(ModelFailure::Limit(failure)) => return Err(failure),
    };
    let member_type = match resolved {
        Resolved::Field(field) => owner.package.field_type(field),
        Resolved::Operation(resolved) => {
            if object.interface {
                return ineligible(member_at("name"));
            }
            let receiver = MemberType::Reference(declaring.node_id.digest.clone());
            let Ok(receiver_key) = receiver.node_key(owner.package.bytes) else {
                return key_refused();
            };
            if type_node(0).map(|id| id.digest).as_deref() != Some(receiver_key.as_str()) {
                return ineligible(argument_at(0));
            }
            if arguments.len().saturating_sub(1) != resolved.parameters.len() {
                return ineligible(application.body(&["arguments"]));
            }
            for (offset, parameter) in resolved.parameters.iter().enumerate() {
                let position = offset.saturating_add(1);
                let Some(expected) = owner.package.slot_type(parameter) else {
                    return ineligible(member_at("name"));
                };
                let Ok(expected_key) = expected.node_key(owner.package.bytes) else {
                    return key_refused();
                };
                if type_node(position).map(|id| id.digest).as_deref() != Some(expected_key.as_str())
                {
                    return ineligible(argument_at(position));
                }
            }
            resolved
                .result
                .as_ref()
                .and_then(|result| owner.package.slot_type(result))
        }
    };
    let Some(member_type) = member_type else {
        return ineligible(member_at("name"));
    };
    let result_type = application
        .node
        .body
        .get("result_type")
        .and_then(|value| value.get("digest"))
        .and_then(Value::as_str);
    let Ok(member_key) = member_type.node_key(owner.package.bytes) else {
        return key_refused();
    };
    if result_type != Some(member_key.as_str()) {
        return ineligible(application.body(&["result_type"]));
    }
    Ok(None)
}

/// The one `operation.member` shape an upstream mutation exercises in full: a
/// `field` member's `name` must actually be declared on the record type its
/// `declaration` names (that type's own `body` is an `aggregate` of
/// `binding` members, one per field, exactly the shape the record-type
/// fixture nodes carry).
fn check_field_member(
    application: Application<'_>,
    operation: &OperationWire,
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<ValidationFailure> {
    let member = operation.member.as_ref()?;
    let declaration: CheckedNodeId =
        serde_json::from_value(member.get("declaration")?.clone()).ok()?;
    let name = member.get("name")?.as_str()?;
    let declaring = &nodes[*index.get(&declaration)?];
    let members = declaring.body.get("members")?.as_array()?;
    let declared = members.iter().any(|entry| {
        body_term(entry) == Some(BodyTerm::Binding)
            && entry.get("name").and_then(Value::as_str) == Some(name)
    });
    if declared {
        None
    } else {
        Some(application.refuse(
            CheckedPackageRefusalCode::IllTyped,
            application.body(&["operation", "member", "name"]),
            CheckedPackageRefusalCause::OperatorIneligible,
        ))
    }
}

/// The type-node id an operand resolves to: a `literal` names its declared
/// `type` and an `application` its `result_type`; a `reference` argument whose direct target is itself
/// type-shaped (a `scalar_type`, `composite_type` or `bounded_domain` node)
/// names that type directly; otherwise (a `value` node — a literal, a
/// `record_value`, ...) it is that node's own `semantic_type`. The same
/// resolution [`argument_family`] uses, kept separate because a pin lookup
/// and `same_type` need the type node itself (e.g. `float_rounding`), not
/// the family name [`resolve_family`] reduces it to. Every other term
/// resolves to no type.
fn operand_type_node(
    argument: &Value,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<CheckedNodeId> {
    let typed = |member: &str| serde_json::from_value(argument.get(member)?.clone()).ok();
    match body_term(argument)? {
        BodyTerm::Literal => return typed("type"),
        BodyTerm::Application => return typed("result_type"),
        BodyTerm::Reference => {}
        BodyTerm::Binding
        | BodyTerm::Aggregate
        | BodyTerm::DependencyReference
        | BodyTerm::Frame
        | BodyTerm::AbstractionRelation => return None,
    }
    let target = reference_term_target(argument)?;
    let position = *index.get(&target)?;
    let target_node = &nodes[position];
    if is_type_shaped(*kinds.get(position)?) {
        Some(target)
    } else {
        Some(target_node.semantic_type.clone())
    }
}

/// The value a type node's own declaration pins for a type-pinned mode kind
/// (`rounding`, `text_profile`): a `bounded_domain` wrapper's `aggregate`
/// body carries exactly one `binding` named for the kind, e.g.
/// `float_rounding`'s `{"term":"binding","name":"rounding","value":{"term":
/// "literal", ..., "value": "nearest-even"}}`.
fn type_pin(
    type_id: &CheckedNodeId,
    kind: OperationModeKind,
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<Box<str>> {
    let node = &nodes[*index.get(type_id)?];
    let members = node.body.get("members")?.as_array()?;
    let binding = members.iter().find(|entry| {
        body_term(entry) == Some(BodyTerm::Binding)
            && entry.get("name").and_then(Value::as_str) == Some(kind.as_wire())
    })?;
    binding.get("value")?.get("value")?.as_str().map(Box::from)
}

fn check_mode_type(
    application: Application<'_>,
    entry: &OperationCatalogEntry,
    operation: &OperationWire,
    arguments: &[Value],
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    catalog: &super::operation_catalog::OperationCatalog,
) -> Option<ValidationFailure> {
    let mode = operation.mode.as_ref()?;
    let mode_kind = mode.kind_class()?;
    if !catalog.is_type_pinned_mode(mode_kind) {
        return None;
    }
    for (position, expected) in entry.operands.iter().enumerate() {
        let argument = arguments.get(position)?;
        let Some(type_id) = operand_type_node(argument, nodes, kinds, index) else {
            continue;
        };
        let Some(actual_family) = resolve_family(&type_id, nodes, kinds, index) else {
            continue;
        };
        if !catalog.family_fits(actual_family, expected) {
            continue;
        }
        if let Some(pinned) = type_pin(&type_id, mode_kind, nodes, index) {
            if pinned.as_ref() != mode.value.as_ref() {
                return Some(application.refuse(
                    CheckedPackageRefusalCode::InvalidPackage,
                    application.body(&["operation", "mode", "value"]),
                    CheckedPackageRefusalCause::OperationModeTypeMismatch,
                ));
            }
        }
    }
    None
}

/// The structural type node behind `type_id`: an alias or a non-population
/// bounded domain continues through its own `semantic_type` (QSpec FR-322).
/// `None` when the chain does not resolve or does not end within the graph.
fn structural_type(
    type_id: &CheckedNodeId,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<(usize, CheckedNodeKind)> {
    let mut position = *index.get(type_id)?;
    for _ in 0..=nodes.len() {
        let kind = *kinds.get(position)?;
        let node = &nodes[position];
        let forwards = kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Alias)
            || (kind.tag() == CheckedNodeTag::BoundedDomain
                && node.semantic_form.as_ref() != "model_population");
        if !forwards || node.semantic_type == node.node_id {
            return Some((position, kind));
        }
        position = *index.get(&node.semantic_type)?;
    }
    None
}

/// The `reference` term's target in `term`, as a node id.
fn referenced_type(term: &Value) -> Option<CheckedNodeId> {
    if body_term(term) != Some(BodyTerm::Reference) {
        return None;
    }
    serde_json::from_value(term.get("target")?.clone()).ok()
}

/// The inner type of an `option`, `sequence`, `set`, `bag` or `ordered_set`
/// type node: its first member's reference target.
fn inner_type(
    type_id: &CheckedNodeId,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<CheckedNodeId> {
    let (position, kind) = structural_type(type_id, nodes, kinds, index)?;
    let CheckedNodeKind::CompositeType(
        CompositeTypeForm::Option
        | CompositeTypeForm::Sequence
        | CompositeTypeForm::Set
        | CompositeTypeForm::Bag
        | CompositeTypeForm::OrderedSet,
    ) = kind
    else {
        return None;
    };
    referenced_type(nodes[position].body.get("members")?.as_array()?.first()?)
}

/// Why the leaves of a compared type could not be derived.
enum LeafWalkEnd {
    /// A node on the walk does not resolve or is not shaped as its form
    /// requires, a text leaf pins no profile, or an option or collection
    /// leads back to itself with no record or tuple between the two visits
    /// (a record-free cycle, which has no composite to anchor a recursion
    /// leaf).
    Unresolved,
    /// The work budget ran out.
    Work(ValidationFailure),
}

/// The `text_profile` a text type pins: the first node of its alias and
/// bounded-domain chain that binds one (QSpec FR-322). Every node of a chain
/// walked has the same answer as the node it forwards to, so each is memoised
/// in `memo`, and each step not already known is charged to `meter`: a long
/// alias chain named by many fields costs its length once, not once per field.
fn text_profile_pin(
    type_id: &CheckedNodeId,
    walk: &mut LeafWalk<'_, '_>,
) -> Result<Option<Box<str>>, LeafWalkEnd> {
    let (nodes, kinds, index) = (walk.nodes, walk.kinds, walk.index);
    let Some(mut position) = index.get(type_id).copied() else {
        return Ok(None);
    };
    let mut walked = Vec::new();
    let mut seen = BTreeSet::new();
    let found = loop {
        if let Some(known) = walk.pins.get(&position) {
            break known.clone();
        }
        let at = &walk.at;
        walk.meter
            .charge(1, || at.clone())
            .map_err(LeafWalkEnd::Work)?;
        let node = &nodes[position];
        walked.push(position);
        seen.insert(position);
        if let Some(pin) = type_pin(&node.node_id, OperationModeKind::TextProfile, nodes, index) {
            break Some(pin);
        }
        let Some(kind) = kinds.get(position).copied() else {
            break None;
        };
        let forwards = kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Alias)
            || (kind.tag() == CheckedNodeTag::BoundedDomain
                && node.semantic_form.as_ref() != "model_population");
        if !forwards || node.semantic_type == node.node_id {
            break None;
        }
        match index.get(&node.semantic_type) {
            // A chain that revisits a node is a cycle: no pin.
            Some(next) if !seen.contains(next) => position = *next,
            _ => break None,
        }
    };
    for position in walked {
        walk.pins.insert(position, found.clone());
    }
    Ok(found)
}

/// The outcome of comparing the supplied leaves with the derived ones: the
/// `text_profile` each text leaf's type pins (`None` for a recursion leaf),
/// or the fault.
type LeafShape = Result<Vec<Option<Box<str>>>, LeafFault>;

/// Where the supplied leaves depart from the derived ones.
enum LeafFault {
    /// Fewer leaves supplied than derived: `operation-law-missing` at
    /// `operation.leaves`.
    Missing,
    /// A text leaf with no place left, at `operation.leaves/<i>`.
    Unplaced(usize),
    /// Entry `i` is not the leaf derived for its place, or carries the wrong
    /// laws: `operation-law-mismatch` at its `path` or `laws`.
    Member(usize, &'static str),
}

/// What a type node anchors on the walk.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Anchor {
    /// A `record` or `tuple`: open between entering its fields or positions
    /// and leaving them. An edge to an open composite is a reentry.
    Composite,
    /// An option or collection: noted per path, so that reaching it again
    /// with no composite entered between the two visits is refused.
    Wrapper,
    /// Any other node, which has no children.
    Leaf,
}

/// One edge of the type and the structural node it leads to: the path
/// segments it adds (`field:<name>`, `position:<n>` or `inner`, or, through a
/// union, `member:<Name>` then `position:<i>`) and where it leads.
struct LeafChild {
    segments: Vec<Box<str>>,
    /// The edge's target as the type names it. A text type's profile is read
    /// through it, since the pin lives in a wrapper above the shared `text`
    /// scalar.
    target: CheckedNodeId,
    /// The node `target` resolves to through aliases and bounded domains.
    position: usize,
    kind: CheckedNodeKind,
}

/// A type node's edges as its body names them: the segments and the target
/// node key, in declaration order.
type RawEdges = Vec<(Vec<Box<str>>, CheckedNodeId)>;

/// A type node's anchor and its outgoing edges, in declaration order.
struct Expansion {
    anchor: Anchor,
    children: Vec<LeafChild>,
}

/// One type node being walked: how many of its edges are done and, when
/// counting, the running count.
struct LeafFrame {
    position: usize,
    component: usize,
    expansion: Rc<Expansion>,
    next: usize,
    total: usize,
    /// No composite reachable from the node was open where it was entered, so
    /// its count is its own and is memoised when the frame completes.
    closed: bool,
    /// How many path segments the path held when the node was entered, so
    /// leaving one of its edges restores exactly that path.
    path_len: usize,
}

/// A node's own contribution: a count, or edges still to walk.
enum LeafEntry {
    Count(usize),
    Pushed,
}

/// What visiting one edge of the derivation left to do.
enum Visited {
    /// Nothing below the edge to walk: a leaf was placed, a reentry was
    /// placed or skipped, or the subtree holds no leaf.
    Done,
    /// A frame was pushed for the edge's node.
    Descended,
    Fault(LeafFault),
}

/// The composites on the current path: the `record` and `tuple` nodes the
/// walk is inside, between entering their fields or positions and leaving
/// them. The set is the path only, never every composite visited.
#[derive(Default)]
struct OpenComposites {
    /// Each open composite and the number of path segments the path held when
    /// it was entered (the `d` of a recursion leaf).
    entered_at: BTreeMap<usize, usize>,
    /// How many composites of each strongly connected component are open.
    per_component: BTreeMap<usize, usize>,
}

impl OpenComposites {
    fn open(&mut self, position: usize, component: usize, depth: usize) {
        self.entered_at.insert(position, depth);
        *self.per_component.entry(component).or_default() += 1;
    }

    fn close(&mut self, position: usize, component: usize) {
        self.entered_at.remove(&position);
        if let Some(open) = self.per_component.get_mut(&component) {
            *open = open.saturating_sub(1);
        }
    }

    fn depth_of(&self, position: usize) -> Option<usize> {
        self.entered_at.get(&position).copied()
    }

    /// Whether a composite of `component` is open. An open composite is an
    /// ancestor of the node being entered, so one reachable from that node
    /// shares its component, and a memoised count applies only when none is
    /// open.
    fn touches(&self, component: usize) -> bool {
        self.per_component
            .get(&component)
            .is_some_and(|open| *open > 0)
    }

    fn len(&self) -> usize {
        self.entered_at.len()
    }
}

/// Whether `leaf` is a recursion leaf, `recursion:<d>` as its last segment.
fn is_recursion_leaf(leaf: &OperationLeafWire) -> bool {
    leaf.path
        .last()
        .and_then(|segment| segment.strip_prefix("recursion:"))
        .is_some_and(|depth| !depth.is_empty() && depth.bytes().all(|byte| byte.is_ascii_digit()))
}

/// Whether `leaf`'s path is `prefix` followed, when `last` is given, by that
/// one segment.
fn path_is(leaf: &OperationLeafWire, prefix: &[Box<str>], last: Option<&str>) -> bool {
    let (head, tail) = match leaf.path.split_last() {
        Some((tail, head)) if last.is_some() => (head, Some(tail.as_ref())),
        _ => (leaf.path.as_slice(), None),
    };
    tail == last && head == prefix
}

/// Derives the leaves of a compared type (QSpec FR-322: one entry for each
/// `text` leaf, plus, for a type that reaches itself, one recursion leaf at
/// each reentry into a composite from which text is reachable), walking
/// `record` fields, `tuple` positions and the inner type of an `option`,
/// `sequence`, `set`, `bag` or `ordered_set`. [`LeafWalk::count`] counts the
/// derived leaves; [`LeafWalk::first_leaf_fault`] derives them one at a time
/// in declaration order and compares each with the supplied one.
///
/// The open composites are the `record` and `tuple` nodes on the current
/// path. An edge to one is a reentry: it is not followed, it contributes no
/// text leaf, and it derives one recursion leaf where text is reachable from
/// that composite. A node's memoised count is its count with no composite
/// reachable from it open, so it applies at a use only when none is open
/// there (see [`OpenComposites::touches`]). An option or collection is noted
/// while it is on the path; reaching it again with no composite entered
/// between the two visits is a record-free cycle, which is refused. Every
/// edge entered, a reentry included, is charged to the work meter. The walks
/// are iterative over an explicit heap stack, so nesting and cycles cost
/// heap, never call stack, and the work budget alone bounds them.
///
/// The expected paths are never listed up front, since a shared-field type
/// has exponentially many. Once the count has settled, and the supplied list
/// is not shorter, the pass skips every subtree the memo says derives no
/// leaf, so its cost is bounded by the supplied leaves times the depth times
/// the width of a node's fields (a sibling holding no text is entered and
/// skipped) for a type that reaches no open composite, and by the work
/// budget for one that does.
struct LeafWalk<'g, 'm> {
    nodes: &'g [CheckedSemanticNodeV2],
    kinds: &'g [CheckedNodeKind],
    index: &'g BTreeMap<&'g CheckedNodeId, usize>,
    meter: &'m mut WorkMeter,
    at: JsonPointer,
    /// Each node's derived-leaf count where no composite reachable from it
    /// was open.
    memo: BTreeMap<usize, usize>,
    /// The profile each alias or domain node of a text type's chain pins.
    pins: BTreeMap<usize, Option<Box<str>>>,
    /// Each node below the compared type, expanded once.
    expanded: BTreeMap<usize, Rc<Expansion>>,
    /// The strongly connected component of each node.
    component_of: BTreeMap<usize, usize>,
    /// Whether a `text` type is reachable from each component.
    component_text: Vec<bool>,
    open: OpenComposites,
    /// The open-composite count at each held visit of an option or
    /// collection, dropped when the walk leaves the node.
    noted: BTreeMap<usize, Vec<usize>>,
    stack: Vec<LeafFrame>,
}

/// One node of the component search and how many of its edges are done.
struct ComponentVisit {
    position: usize,
    expansion: Rc<Expansion>,
    next: usize,
}

impl<'g, 'm> LeafWalk<'g, 'm> {
    fn new(graph: Graph<'g>, meter: &'m mut WorkMeter, at: JsonPointer) -> LeafWalk<'g, 'm> {
        LeafWalk {
            nodes: graph.nodes,
            kinds: graph.kinds,
            index: graph.index,
            meter,
            at,
            memo: BTreeMap::new(),
            pins: BTreeMap::new(),
            expanded: BTreeMap::new(),
            component_of: BTreeMap::new(),
            component_text: Vec::new(),
            open: OpenComposites::default(),
            noted: BTreeMap::new(),
            stack: Vec::new(),
        }
    }

    /// One unit to the work budget, at `operation.leaves`.
    fn charge(&mut self) -> Result<(), LeafWalkEnd> {
        let at = &self.at;
        self.meter
            .charge(1, || at.clone())
            .map_err(LeafWalkEnd::Work)
    }

    /// The anchor and the raw edges of the node at `position`: a `record`'s
    /// fields, a `tuple`'s positions, the inner type of an option or
    /// collection, and a union's payload positions, each under its member.
    /// A union is an open composite like a record or tuple: a cycle through
    /// one is a recursion leaf, never a record-free cycle (QSpec FR-322
    /// "Structural leaf walk").
    fn children(&self, position: usize, kind: CheckedNodeKind) -> Option<(Anchor, RawEdges)> {
        let members = || self.nodes[position].body.get("members")?.as_array();
        match kind {
            CheckedNodeKind::CompositeType(CompositeTypeForm::Record) => members()?
                .iter()
                .map(|member| {
                    if body_term(member) != Some(BodyTerm::Binding) {
                        return None;
                    }
                    let name = member.get("name")?.as_str()?;
                    let value = member.get("value")?;
                    let target = if body_term(value) == Some(BodyTerm::Aggregate) {
                        let [optional] = value.get("members")?.as_array()?.as_slice() else {
                            return None;
                        };
                        if body_term(optional) != Some(BodyTerm::Binding)
                            || optional.get("name")?.as_str()? != "optional"
                        {
                            return None;
                        }
                        let target = referenced_type(optional.get("value")?)?;
                        let position = *self.index.get(&target)?;
                        if self.kinds.get(position)
                            != Some(&CheckedNodeKind::CompositeType(CompositeTypeForm::Option))
                        {
                            return None;
                        }
                        target
                    } else {
                        referenced_type(value)?
                    };
                    Some((vec![format!("field:{name}").into()], target))
                })
                .collect::<Option<Vec<_>>>()
                .map(|edges| (Anchor::Composite, edges)),
            CheckedNodeKind::CompositeType(CompositeTypeForm::Tuple) => members()?
                .iter()
                .enumerate()
                .map(|(at, member)| {
                    Some((
                        vec![format!("position:{at}").into()],
                        referenced_type(member)?,
                    ))
                })
                .collect::<Option<Vec<_>>>()
                .map(|edges| (Anchor::Composite, edges)),
            // QSpec FR-322 "Structural leaf walk": a leaf through a union is
            // `member:<Name>`, `position:<i>` (`i` the index within the
            // member's payload, even for a single payload), then the segments
            // into the payload type.
            CheckedNodeKind::CompositeType(CompositeTypeForm::Union) => {
                let edges = union_type_body(&self.nodes[position].body)?
                    .into_iter()
                    .flat_map(|member| {
                        member
                            .payload
                            .into_iter()
                            .enumerate()
                            .map(move |(at, target)| {
                                (
                                    vec![
                                        format!("member:{}", member.name).into(),
                                        format!("position:{at}").into(),
                                    ],
                                    target,
                                )
                            })
                    })
                    .collect();
                Some((Anchor::Composite, edges))
            }
            CheckedNodeKind::CompositeType(
                CompositeTypeForm::Option
                | CompositeTypeForm::Sequence
                | CompositeTypeForm::Set
                | CompositeTypeForm::Bag
                | CompositeTypeForm::OrderedSet,
            ) => Some((
                Anchor::Wrapper,
                vec![(vec!["inner".into()], referenced_type(members()?.first()?)?)],
            )),
            _ => Some((Anchor::Leaf, Vec::new())),
        }
    }

    /// The edge adding `segments` to `target`, resolved to its structural node.
    fn edge(
        &self,
        segments: Vec<Box<str>>,
        target: CheckedNodeId,
    ) -> Result<LeafChild, LeafWalkEnd> {
        let (position, kind) = structural_type(&target, self.nodes, self.kinds, self.index)
            .ok_or(LeafWalkEnd::Unresolved)?;
        Ok(LeafChild {
            segments,
            target,
            position,
            kind,
        })
    }

    fn expansion(&self, position: usize) -> Result<Rc<Expansion>, LeafWalkEnd> {
        self.expanded
            .get(&position)
            .cloned()
            .ok_or(LeafWalkEnd::Unresolved)
    }

    fn component(&self, position: usize) -> Result<usize, LeafWalkEnd> {
        self.component_of
            .get(&position)
            .copied()
            .ok_or(LeafWalkEnd::Unresolved)
    }

    /// Whether a `text` type is reachable from the composite at `component`.
    fn reaches_text(&self, component: usize) -> Result<bool, LeafWalkEnd> {
        self.component_text
            .get(component)
            .copied()
            .ok_or(LeafWalkEnd::Unresolved)
    }

    /// Finds the strongly connected components of the type graph below
    /// `root`, and for each whether a `text` type is reachable from it, one
    /// unit of work per node. The walks need both: an open composite that
    /// is reachable from a node shares the node's component (it is an
    /// ancestor on the path), and a reentry derives a recursion leaf only
    /// where text is reachable. Iterative (Tarjan), on a heap stack.
    fn analyse(&mut self, root: &LeafChild) -> Result<(), LeafWalkEnd> {
        // Each node's discovery number and lowest reachable number.
        let mut numbers: BTreeMap<usize, (usize, usize)> = BTreeMap::new();
        let mut members: Vec<usize> = Vec::new();
        let mut calls: Vec<ComponentVisit> = Vec::new();
        self.discover(root, &mut numbers, &mut members, &mut calls)?;
        while let Some(call) = calls.last_mut() {
            let expansion = Rc::clone(&call.expansion);
            let position = call.position;
            if let Some(child) = expansion.children.get(call.next) {
                call.next += 1;
                match numbers.get(&child.position).copied() {
                    Some((number, _)) if !self.component_of.contains_key(&child.position) => {
                        // An edge to a node still on the member stack.
                        let (_, low) = numbers.get_mut(&position).ok_or(LeafWalkEnd::Unresolved)?;
                        *low = (*low).min(number);
                    }
                    Some(_) => {}
                    None => self.discover(child, &mut numbers, &mut members, &mut calls)?,
                }
                continue;
            }
            calls.pop();
            let (number, low) = numbers
                .get(&position)
                .copied()
                .ok_or(LeafWalkEnd::Unresolved)?;
            if let Some(parent) = calls.last() {
                let (_, parent_low) = numbers
                    .get_mut(&parent.position)
                    .ok_or(LeafWalkEnd::Unresolved)?;
                *parent_low = (*parent_low).min(low);
            }
            if low == number {
                self.close_component(position, &mut members)?;
            }
        }
        Ok(())
    }

    /// Expands the node `edge` leads to and starts its component search.
    fn discover(
        &mut self,
        edge: &LeafChild,
        numbers: &mut BTreeMap<usize, (usize, usize)>,
        members: &mut Vec<usize>,
        calls: &mut Vec<ComponentVisit>,
    ) -> Result<(), LeafWalkEnd> {
        self.charge()?;
        let (anchor, raw) = self
            .children(edge.position, edge.kind)
            .ok_or(LeafWalkEnd::Unresolved)?;
        let children = raw
            .into_iter()
            .map(|(segment, target)| self.edge(segment, target))
            .collect::<Result<Vec<_>, _>>()?;
        let expansion = Rc::new(Expansion { anchor, children });
        self.expanded.insert(edge.position, Rc::clone(&expansion));
        let number = numbers.len();
        numbers.insert(edge.position, (number, number));
        members.push(edge.position);
        calls.push(ComponentVisit {
            position: edge.position,
            expansion,
            next: 0,
        });
        Ok(())
    }

    /// Pops the component rooted at `root` off the member stack and records
    /// whether a `text` type is reachable from it: one of its members is
    /// text, or an edge leaves it for a component that reaches text (every
    /// such component is already closed).
    fn close_component(
        &mut self,
        root: usize,
        members: &mut Vec<usize>,
    ) -> Result<(), LeafWalkEnd> {
        let component = self.component_text.len();
        let mut closed = Vec::new();
        while let Some(member) = members.pop() {
            self.component_of.insert(member, component);
            closed.push(member);
            if member == root {
                break;
            }
        }
        let mut text = false;
        for member in &closed {
            let kind = self.kinds.get(*member).copied();
            text |= kind == Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Text));
            for child in &self.expansion(*member)?.children {
                let other = self.component(child.position)?;
                if other != component {
                    text |= self.reaches_text(other)?;
                }
            }
        }
        self.component_text.push(text);
        Ok(())
    }

    /// The edge from the root of the compared type, which names no segment.
    fn root_edge(&self, root: &CheckedNodeId) -> Result<LeafChild, LeafWalkEnd> {
        self.edge(Vec::new(), root.clone())
    }

    /// One edge entered while counting: its own contribution, or the frame
    /// of its edges still to sum. Every entry, a reentry included, is one
    /// unit of work.
    fn enter(&mut self, edge: &LeafChild) -> Result<LeafEntry, LeafWalkEnd> {
        self.charge()?;
        // A text leaf whose type chain pins no profile cannot be decided
        // (QSpec FR-322: the type is ineligible). The pin lives in a wrapper
        // above the shared `text` scalar, so it is read at every visit.
        if edge.kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Text) {
            text_profile_pin(&edge.target, self)?.ok_or(LeafWalkEnd::Unresolved)?;
            return Ok(LeafEntry::Count(1));
        }
        let component = self.component(edge.position)?;
        if self.open.depth_of(edge.position).is_some() {
            // A reentry: no text leaf, and the recursion leaf where text is
            // reachable.
            return Ok(LeafEntry::Count(usize::from(self.reaches_text(component)?)));
        }
        let closed = !self.open.touches(component);
        if closed {
            if let Some(count) = self.memo.get(&edge.position) {
                return Ok(LeafEntry::Count(*count));
            }
        }
        let expansion = self.expansion(edge.position)?;
        if expansion.children.is_empty() {
            self.memo.insert(edge.position, 0);
            return Ok(LeafEntry::Count(0));
        }
        match expansion.anchor {
            Anchor::Composite => self.open.open(edge.position, component, 0),
            Anchor::Wrapper => {
                let composites = self.open.len();
                let held = self.noted.entry(edge.position).or_default();
                if held.last() == Some(&composites) {
                    // Reached again with no composite entered between the two
                    // visits: a record-free cycle.
                    return Err(LeafWalkEnd::Unresolved);
                }
                held.push(composites);
            }
            Anchor::Leaf => {}
        }
        self.stack.push(LeafFrame {
            position: edge.position,
            component,
            expansion,
            next: 0,
            total: 0,
            closed,
            // Counting derives no paths.
            path_len: 0,
        });
        Ok(LeafEntry::Pushed)
    }

    /// Leaves the node of a finished counting frame: it is no longer open or
    /// noted, and its count is memoised where it did not depend on an open
    /// composite.
    fn leave(&mut self, frame: &LeafFrame) {
        match frame.expansion.anchor {
            Anchor::Composite => self.open.close(frame.position, frame.component),
            Anchor::Wrapper => {
                if let Some(held) = self.noted.get_mut(&frame.position) {
                    held.pop();
                    if held.is_empty() {
                        self.noted.remove(&frame.position);
                    }
                }
            }
            Anchor::Leaf => {}
        }
        if frame.closed {
            self.memo.insert(frame.position, frame.total);
        }
    }

    /// The number of leaves the compared type derives: its text leaves and
    /// its recursion leaves.
    fn count(&mut self, root: &CheckedNodeId) -> Result<usize, LeafWalkEnd> {
        let root = self.root_edge(root)?;
        self.analyse(&root)?;
        let mut finished = match self.enter(&root)? {
            LeafEntry::Count(count) => Some(count),
            LeafEntry::Pushed => None,
        };
        loop {
            if let Some(count) = finished.take() {
                let Some(frame) = self.stack.last_mut() else {
                    return Ok(count);
                };
                frame.total = frame.total.saturating_add(count);
            }
            let Some(frame) = self.stack.last_mut() else {
                return Err(LeafWalkEnd::Unresolved);
            };
            let expansion = Rc::clone(&frame.expansion);
            let at = frame.next;
            if let Some(child) = expansion.children.get(at) {
                frame.next += 1;
                finished = match self.enter(child)? {
                    LeafEntry::Count(count) => Some(count),
                    LeafEntry::Pushed => None,
                };
            } else if let Some(frame) = self.stack.pop() {
                self.leave(&frame);
                finished = Some(frame.total);
            }
        }
    }

    /// The first supplied leaf, by index, that is not the one the type
    /// derives for its place, and how: the pass follows the derivation in
    /// order, and at each place the next supplied entry must be the leaf
    /// derived there. A text leaf must carry its expected path and exactly
    /// one catalogued `text_profile` law; a recursion leaf must sit at its
    /// reentry with the `d` the reentered composite was entered at, and carry
    /// no law. An entry left over once every derived leaf is placed is
    /// unplaced. Must run after [`LeafWalk::count`] of `root`, with at least
    /// as many leaves supplied as that count. When every leaf fits, returns
    /// the `text_profile` each text leaf's type pins, in order.
    fn first_leaf_fault(
        &mut self,
        root: &CheckedNodeId,
        supplied: &[OperationLeafWire],
        text_laws: &[CheckedArtifactRef],
    ) -> Result<LeafShape, LeafWalkEnd> {
        let root = self.root_edge(root)?;
        let mut pins: Vec<Option<Box<str>>> = Vec::new();
        let mut path: Vec<Box<str>> = Vec::new();
        self.open = OpenComposites::default();
        let mut visited = self.visit(&root, &path, supplied, text_laws, &mut pins)?;
        'walk: loop {
            match visited {
                Visited::Fault(fault) => return Ok(Err(fault)),
                Visited::Done => {
                    // Back to the path of the node whose edge was visited.
                    let Some(frame) = self.stack.last() else {
                        break 'walk;
                    };
                    path.truncate(frame.path_len);
                }
                Visited::Descended => {}
            }
            visited = loop {
                let Some(frame) = self.stack.last_mut() else {
                    break 'walk;
                };
                let expansion = Rc::clone(&frame.expansion);
                if let Some(child) = expansion.children.get(frame.next) {
                    frame.next += 1;
                    path.extend(child.segments.iter().cloned());
                    break self.visit(child, &path, supplied, text_laws, &mut pins)?;
                }
                if let Some(frame) = self.stack.pop() {
                    if frame.expansion.anchor == Anchor::Composite {
                        self.open.close(frame.position, frame.component);
                    }
                }
                if let Some(parent) = self.stack.last() {
                    path.truncate(parent.path_len);
                }
            };
        }
        Ok(match supplied.get(pins.len()) {
            None => Ok(pins),
            Some(leaf) if is_recursion_leaf(leaf) => Err(LeafFault::Member(pins.len(), "path")),
            Some(_) => Err(LeafFault::Unplaced(pins.len())),
        })
    }

    /// One edge of the derivation, entered with `path` ending in its segment:
    /// a text leaf or a reentry that derives a recursion leaf is compared
    /// with the next supplied entry; a node that derives nothing is skipped
    /// when its memoised count says so; any other is descended into. Every
    /// edge is one unit of work.
    fn visit(
        &mut self,
        edge: &LeafChild,
        path: &[Box<str>],
        supplied: &[OperationLeafWire],
        text_laws: &[CheckedArtifactRef],
        pins: &mut Vec<Option<Box<str>>>,
    ) -> Result<Visited, LeafWalkEnd> {
        self.charge()?;
        let placed = pins.len();
        if edge.kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Text) {
            let pin = text_profile_pin(&edge.target, self)?.ok_or(LeafWalkEnd::Unresolved)?;
            let Some(leaf) = supplied.get(placed) else {
                return Ok(Visited::Fault(LeafFault::Missing));
            };
            let lawful = matches!(
                leaf.laws.as_slice(),
                [law] if law.role_class() == Some(LawRole::TextProfile)
                    && text_laws.contains(&law.definition)
            );
            if is_recursion_leaf(leaf) || !path_is(leaf, path, None) {
                return Ok(Visited::Fault(LeafFault::Member(placed, "path")));
            }
            if !lawful {
                return Ok(Visited::Fault(LeafFault::Member(placed, "laws")));
            }
            pins.push(Some(pin));
            return Ok(Visited::Done);
        }
        let component = self.component(edge.position)?;
        if let Some(depth) = self.open.depth_of(edge.position) {
            if self.reaches_text(component)? {
                let Some(leaf) = supplied.get(placed) else {
                    return Ok(Visited::Fault(LeafFault::Missing));
                };
                let segment = format!("recursion:{depth}");
                if !is_recursion_leaf(leaf) || !path_is(leaf, path, Some(&segment)) {
                    return Ok(Visited::Fault(LeafFault::Member(placed, "path")));
                }
                if !leaf.laws.is_empty() {
                    return Ok(Visited::Fault(LeafFault::Member(placed, "laws")));
                }
                pins.push(None);
            }
            return Ok(Visited::Done);
        }
        let expansion = self.expansion(edge.position)?;
        let closed = !self.open.touches(component);
        if expansion.children.is_empty() || (closed && self.memo.get(&edge.position) == Some(&0)) {
            return Ok(Visited::Done);
        }
        if expansion.anchor == Anchor::Composite {
            self.open.open(edge.position, component, path.len());
        }
        self.stack.push(LeafFrame {
            position: edge.position,
            component,
            expansion,
            next: 0,
            total: 0,
            closed,
            path_len: path.len(),
        });
        Ok(Visited::Descended)
    }
}

/// QSpec FR-322: for an entry naming a leaf source, `operation.leaves` lists,
/// in declaration order, one entry for every `text` leaf of the compared
/// type, each with that leaf's path (`field:<name>`, `member:<Name>`,
/// `position:<n>`, `inner`) and one `text_profile` law. A type that reaches
/// itself through a record, tuple or union is admitted (FR-038, "Recursive
/// compared types"): its text leaves are followed by, at each reentry into a
/// composite from which text is reachable, a recursion leaf `{path: p +
/// "recursion:<d>", laws: [], mode: null}`, as QSpec FR-322 "Structural leaf
/// walk" states and both V2 schemas admit. A type with no leaf takes
/// an empty list; fewer entries than derived leaves is `operation-law-missing`,
/// and then one in-order pass places each supplied entry: a leaf that is not
/// the one derived for its place (a wrong or misordered path, a recursion leaf
/// anywhere it is not derived), an extra one, or a text leaf whose laws are not
/// exactly one catalogued `text_profile` definition is
/// `operation-law-mismatch`; a leaf law the lock does not select is
/// `operation-law-unselected`. Each text leaf's `mode` must be a catalogued
/// `text_profile` mode (`operation-mode-mismatch` when absent, of another kind
/// or of an uncatalogued value) and equal the profile its text type pins
/// (`operation-mode-type-mismatch`); a recursion leaf carries no mode.
/// `result_inner` expects leaves only for a `set`, `bag` or `ordered_set`
/// result; any other result expects none, so a supplied leaf is a mismatch. A
/// compared type that has a node that does not resolve, a text leaf pinning no
/// profile, or an option or collection that leads back to itself with no
/// record or tuple between the two visits is `ill_typed`/`operator-ineligible`;
/// one too large for the work budget is the budget's refusal. A `float32` or
/// `float64` leaf counts as no text leaf where the reference reader finds the
/// type undecidable. A compared type this reader cannot resolve from the first
/// operand is not decided, and its leaves are not checked.
fn check_leaf_count(
    application: Application<'_>,
    entry: &OperationCatalogEntry,
    operation: &OperationWire,
    arguments: &[Value],
    graph: &Graph<'_>,
    lock: &CheckedPackageLockV2,
    catalog: &OperationCatalog,
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let Graph {
        nodes,
        kinds,
        index,
    } = *graph;
    let Some(source) = entry.leaves.as_deref() else {
        return Ok(None);
    };
    let operand_type = || operand_type_node(arguments.first()?, nodes, kinds, index);
    // Outer `None`: the compared type is not decided. Inner `None`: the entry
    // expects no leaves.
    let compared: Option<Option<CheckedNodeId>> = match source {
        "operand:0" => operand_type().map(Some),
        "inner:0" => operand_type()
            .and_then(|ty| inner_type(&ty, nodes, kinds, index))
            .map(Some),
        "result_inner" => {
            let result = application
                .node
                .body
                .get("result_type")
                .and_then(|value| serde_json::from_value::<CheckedNodeId>(value.clone()).ok());
            let set_like = result.as_ref().and_then(|result| {
                let (_, kind) = structural_type(result, nodes, kinds, index)?;
                matches!(
                    kind,
                    CheckedNodeKind::CompositeType(
                        CompositeTypeForm::Set
                            | CompositeTypeForm::Bag
                            | CompositeTypeForm::OrderedSet
                    )
                )
                .then_some(result)
            });
            match set_like {
                Some(result) => inner_type(result, nodes, kinds, index).map(Some),
                None => Some(None),
            }
        }
        _ => None,
    };
    let Some(compared) = compared else {
        return Ok(None);
    };
    let at = application.body(&["operation", "leaves"]);
    let Some(compared) = compared else {
        return if operation.leaves.is_empty() {
            Ok(None)
        } else {
            Ok(Some(application.refuse(
                CheckedPackageRefusalCode::InvalidPackage,
                at.index(0),
                CheckedPackageRefusalCause::OperationLawMismatch,
            )))
        };
    };
    let mut walk = LeafWalk::new(*graph, meter, at.clone());
    let refuse = |code, path: JsonPointer, cause| Ok(Some(application.refuse(code, path, cause)));
    let walked = walk.count(&compared).and_then(|derived| {
        if operation.leaves.len() < derived {
            return Ok(Err(LeafFault::Missing));
        }
        let text_laws = catalog
            .law_role_definitions(LawRole::TextProfile)
            .unwrap_or_default();
        walk.first_leaf_fault(&compared, &operation.leaves, text_laws)
    });
    match walked {
        Ok(Err(LeafFault::Missing)) => refuse(
            CheckedPackageRefusalCode::InvalidPackage,
            at,
            CheckedPackageRefusalCause::OperationLawMissing,
        ),
        Ok(Err(LeafFault::Unplaced(leaf))) => refuse(
            CheckedPackageRefusalCode::InvalidPackage,
            at.index(leaf),
            CheckedPackageRefusalCause::OperationLawMismatch,
        ),
        Ok(Err(LeafFault::Member(leaf, member))) => refuse(
            CheckedPackageRefusalCode::InvalidPackage,
            at.index(leaf).key(member),
            CheckedPackageRefusalCause::OperationLawMismatch,
        ),
        Ok(Ok(pins)) => {
            // Every leaf now carries its one catalogued law; the lock must
            // select it, then each mode must be a catalogued `text_profile`
            // and the profile its leaf's type pins.
            let unselected = operation.leaves.iter().position(|leaf| {
                leaf.laws
                    .first()
                    .is_some_and(|law| !lock.definition_selections.contains(&law.definition))
            });
            if let Some(leaf) = unselected {
                return refuse(
                    CheckedPackageRefusalCode::InvalidPackage,
                    at.index(leaf).key("laws").index(0).key("definition"),
                    CheckedPackageRefusalCause::OperationLawUnselected,
                );
            }
            for (at_leaf, (leaf, pin)) in operation.leaves.iter().zip(&pins).enumerate() {
                let leaf_at = at.clone().index(at_leaf);
                let admitted = match &leaf.mode {
                    // A recursion leaf carries no mode.
                    Some(_) if pin.is_none() => Err(leaf_at.clone().key("mode")),
                    None if pin.is_none() => Ok(()),
                    None => Err(leaf_at.clone().key("mode")),
                    Some(mode) if mode.kind_class() != Some(OperationModeKind::TextProfile) => {
                        Err(leaf_at.clone().key("mode").key("kind"))
                    }
                    Some(mode)
                        if !catalog
                            .mode_value_admitted(OperationModeKind::TextProfile, &mode.value) =>
                    {
                        Err(leaf_at.clone().key("mode").key("value"))
                    }
                    Some(_) => Ok(()),
                };
                if let Err(path) = admitted {
                    return refuse(
                        CheckedPackageRefusalCode::InvalidPackage,
                        path,
                        CheckedPackageRefusalCause::OperationModeMismatch,
                    );
                }
            }
            for (at_leaf, (leaf, pin)) in operation.leaves.iter().zip(&pins).enumerate() {
                let Some(pin) = pin else { continue };
                if leaf.mode.as_ref().is_some_and(|mode| mode.value != *pin) {
                    return refuse(
                        CheckedPackageRefusalCode::InvalidPackage,
                        at.clone().index(at_leaf).key("mode").key("value"),
                        CheckedPackageRefusalCause::OperationModeTypeMismatch,
                    );
                }
            }
            Ok(None)
        }
        Err(LeafWalkEnd::Unresolved) => refuse(
            CheckedPackageRefusalCode::IllTyped,
            at,
            CheckedPackageRefusalCause::OperatorIneligible,
        ),
        Err(LeafWalkEnd::Work(failure)) => Err(failure),
    }
}

#[cfg(test)]
mod tests {
    use super::super::CheckedSelectionRole;
    use super::GroupPlace;
    use super::{
        application_preimage, is_type_shaped, operand_family, operation_catalog, operation_defect,
        validate_application_keys, Application, ApplicationOperator, CheckedNodeId,
        CheckedNodeKind, CheckedNodeTag, CheckedPackageLockV2, CheckedPackageRefusalCause,
        CheckedPackageRefusalCode, CheckedSemanticNodeV2, DependencyReferences, ExpressionForm,
        Graph, LawRole, ModelOwners, SuppliedDependencies, TemporalForm, ValidationFailure,
        WorkMeter,
    };
    use crate::checked_package::common::NODE_DOMAIN;
    use crate::checked_package::shared::{CheckedArtifactRef, CheckedSelection, JsonPointer};
    use crate::checked_package::v2::encode::APPLICATION_NODE_VERSION;
    use ix_trace_rs::trace;

    fn pointer(text: &str) -> JsonPointer {
        JsonPointer::parse(text).expect("test pointer")
    }

    /// The located refusal `operation_defect` builds, at `path`.
    fn refused_at(
        code: CheckedPackageRefusalCode,
        path: &str,
        cause: Option<CheckedPackageRefusalCause>,
        locus: CheckedNodeId,
    ) -> ValidationFailure {
        ValidationFailure::refused_at(code, pointer(path), cause, locus)
    }
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    fn relationship_member_outcome(
        relation: Value,
        role: &str,
        receiver_type: &str,
        operation_identity: &str,
        result_key: &str,
    ) -> Option<(
        CheckedPackageRefusalCode,
        CheckedPackageRefusalCause,
        String,
    )> {
        use crate::checked_package::v2::model_members::{
            declaration_key, tests as model_tests, DeclarationForm, MemberType,
        };
        let model = model_tests::read(&model_tests::relationship_document(vec![relation]))
            .expect("selected document admits");
        let mut meter = WorkMeter::new(100_000);
        let owners = ModelOwners::new(std::slice::from_ref(&model), |_| Ok(())).expect("owners");
        let identity = "acme/orders";
        let widget = "ix://acme/orders/Widget";
        let gadget = "ix://acme/orders/Gadget";
        let relationship = "ix://acme/orders/relationship/Widget-links-Gadget";
        let keys = [widget, gadget, relationship].map(|name| {
            let form = if name == relationship {
                DeclarationForm::Relationship
            } else {
                DeclarationForm::ObjectType
            };
            declaration_key(identity, form, name, model_tests::BYTES).expect("declaration key")
        });
        let [widget_key, gadget_key, relationship_key] = &keys;
        let receiver_key = if receiver_type == widget {
            widget_key
        } else {
            gadget_key
        };
        let reference_key = MemberType::Reference(receiver_key.as_str().into())
            .node_key(model_tests::BYTES)
            .expect("reference key");
        let id = |digest: &str| json!({"domain": NODE_DOMAIN, "digest": digest});
        let mut operation = plain_operation(operation_identity);
        operation["member"] = json!({
            "kind": "relationship_end", "declaration": id(relationship_key), "name": role,
        });
        let operator = if operation_identity == "quire.op.model.reaches" {
            "reaches"
        } else {
            "query"
        };
        let mut application = custom_application_node(
            operator,
            operation.clone(),
            vec![json!({"term": "reference", "target": id(&dummy_digest('r'))})],
        );
        if operation_identity == "quire.op.model.reaches" {
            application.body["arguments"]
                .as_array_mut()
                .expect("arguments")
                .push(json!({"term": "reference", "target": id(&dummy_digest('r'))}));
        }
        application.body["result_type"] = id(result_key);
        let object = |key: &str, name: &str| {
            let mut node = graph_node(
                'o',
                "model",
                "object_type",
                &node_id('o'),
                json!({"term": "aggregate", "members": []}),
            );
            node.node_id.digest = key.into();
            node.semantic_type = node.node_id.clone();
            node.owner = Some(
                serde_json::from_value(json!({
                    "kind": "model", "identity": identity, "node": name,
                }))
                .expect("owner"),
            );
            node
        };
        let mut relation_node = graph_node(
            'l',
            "relation",
            "relationship",
            &node_id('l'),
            json!({"term": "aggregate", "members": []}),
        );
        relation_node.node_id.digest = relationship_key.as_str().into();
        relation_node.semantic_type = relation_node.node_id.clone();
        relation_node.owner = Some(
            serde_json::from_value(json!({
                "kind": "model", "identity": identity, "node": relationship,
            }))
            .expect("owner"),
        );
        let mut reference_node = graph_node(
            'e',
            "composite_type",
            "reference",
            &node_id('e'),
            json!({"term": "aggregate", "members": [{"term": "reference", "target": id(receiver_key)}]}),
        );
        reference_node.node_id.digest = reference_key.into();
        reference_node.semantic_type = reference_node.node_id.clone();
        let mut parameter = graph_node(
            'r',
            "value",
            "parameter",
            &reference_node.node_id,
            json!({"term": "aggregate", "members": []}),
        );
        parameter.node_id.digest = dummy_digest('r').into();
        let nodes = [
            application,
            object(widget_key, widget),
            object(gadget_key, gadget),
            relation_node,
            reference_node,
            parameter,
        ];
        let kinds = kinds_of(&nodes);
        let index: BTreeMap<_, _> = nodes
            .iter()
            .enumerate()
            .map(|(at, node)| (&node.node_id, at))
            .collect();
        let graph = Graph {
            nodes: &nodes,
            kinds: &kinds,
            index: &index,
        };
        let operation: super::OperationWire = serde_json::from_value(operation).expect("operation");
        super::check_relationship_member(
            Application {
                node: &nodes[0],
                position: 0,
            },
            &operation,
            nodes[0].body["arguments"].as_array().expect("arguments"),
            &graph,
            &owners,
            &mut meter,
        )
        .expect("work budget")
        .map(|failure| match failure {
            ValidationFailure::Refused(refusal) => (
                refusal.code,
                refusal.cause.expect("typed cause"),
                refusal.path.expect("path").to_string(),
            ),
            other => panic!("expected refusal, got {other:?}"),
        })
    }

    /// Trace: FR-038-AC-169, FR-038-AC-170, FR-038-AC-171, FR-038-AC-172
    #[trace(
        "TC-048",
        "FR-038-AC-169",
        "FR-038-AC-170",
        "FR-038-AC-171",
        "FR-038-AC-172"
    )]
    #[test]
    fn tc_048_relationship_end_binding_resolves_roles_before_navigation_and_reachability() {
        use crate::checked_package::v2::model_members::{
            declaration_key, tests::relationship, DeclarationForm, MemberType,
        };
        let widget = declaration_key(
            "acme/orders",
            DeclarationForm::ObjectType,
            "ix://acme/orders/Widget",
            1 << 20,
        )
        .expect("widget key");
        let gadget = declaration_key(
            "acme/orders",
            DeclarationForm::ObjectType,
            "ix://acme/orders/Gadget",
            1 << 20,
        )
        .expect("gadget key");
        let reference = MemberType::Reference(gadget.into())
            .node_key(1 << 20)
            .expect("reference");
        let option = MemberType::Option(Box::new(MemberType::Reference(widget.clone().into())))
            .node_key(1 << 20)
            .expect("option");
        let boolean = MemberType::Boolean.node_key(1 << 20).expect("boolean");
        let navigate = "quire.op.model.navigate";
        assert_eq!(
            relationship_member_outcome(
                relationship(),
                "links",
                "ix://acme/orders/Widget",
                navigate,
                &reference
            ),
            None
        );
        assert_eq!(
            relationship_member_outcome(
                relationship(),
                "linkedBy",
                "ix://acme/orders/Gadget",
                navigate,
                &option
            ),
            None
        );
        assert_eq!(
            relationship_member_outcome(
                relationship(),
                "absent",
                "ix://acme/orders/Gadget",
                navigate,
                "wrong"
            ),
            Some((
                CheckedPackageRefusalCode::MissingDeclaration,
                CheckedPackageRefusalCause::MissingName,
                "/semantic_graph/nodes/0/body/operation/member/name".into()
            ))
        );
        assert_eq!(
            relationship_member_outcome(
                relationship(),
                "links",
                "ix://acme/orders/Gadget",
                navigate,
                &reference
            ),
            Some((
                CheckedPackageRefusalCode::IllTyped,
                CheckedPackageRefusalCause::OperatorIneligible,
                "/semantic_graph/nodes/0/body/arguments/0".into()
            ))
        );
        assert_eq!(
            relationship_member_outcome(
                relationship(),
                "links",
                "ix://acme/orders/Widget",
                navigate,
                "wrong"
            ),
            Some((
                CheckedPackageRefusalCode::IllTyped,
                CheckedPackageRefusalCause::OperatorIneligible,
                "/semantic_graph/nodes/0/body/result_type".into()
            ))
        );
        let mut direction = relationship();
        direction["direction"] = json!("source-to-target");
        assert_eq!(
            relationship_member_outcome(
                direction,
                "linkedBy",
                "ix://acme/orders/Gadget",
                navigate,
                &option
            ),
            Some((
                CheckedPackageRefusalCode::IllTyped,
                CheckedPackageRefusalCause::OperatorIneligible,
                "/semantic_graph/nodes/0/body/operation/member/name".into()
            ))
        );
        for multiplicity in [
            json!({"lower": 0, "ordered": false, "unique": true}),
            json!({"lower": 0, "upper": 1, "ordered": true, "unique": true}),
        ] {
            let mut relation = relationship();
            relation["targetEnd"]["multiplicity"] = multiplicity;
            assert_eq!(
                relationship_member_outcome(
                    relation,
                    "links",
                    "ix://acme/orders/Widget",
                    navigate,
                    &reference
                ),
                Some((
                    CheckedPackageRefusalCode::UnsupportedConstruct,
                    CheckedPackageRefusalCause::ExpressionForm,
                    "/semantic_graph/nodes/0/body/operation/member/name".into()
                ))
            );
        }
        let mut self_relationship = relationship();
        self_relationship["targetEnd"]["type"] = json!("ix://acme/orders/Widget");
        assert_eq!(
            relationship_member_outcome(
                self_relationship.clone(),
                "links",
                "ix://acme/orders/Widget",
                "quire.op.model.reaches",
                &boolean
            ),
            None
        );
        assert_eq!(
            relationship_member_outcome(
                self_relationship,
                "links",
                "ix://acme/orders/Widget",
                "quire.op.model.reaches",
                "wrong"
            ),
            Some((
                CheckedPackageRefusalCode::IllTyped,
                CheckedPackageRefusalCause::OperatorIneligible,
                "/semantic_graph/nodes/0/body/result_type".into()
            ))
        );
        assert_eq!(
            relationship_member_outcome(
                relationship(),
                "links",
                "ix://acme/orders/Widget",
                "quire.op.model.reaches",
                &boolean
            ),
            Some((
                CheckedPackageRefusalCode::IllTyped,
                CheckedPackageRefusalCause::OperatorIneligible,
                "/semantic_graph/nodes/0/body/operation/member/name".into()
            ))
        );
    }

    /// A catalogued identity used correctly throughout this module's tests:
    /// zero laws, no mode, no member, two plain-literal operands (so the
    /// unexercised operand-family/type machinery never needs a second graph
    /// node to resolve against).
    const CATALOGUED_IDENTITY: &str = "quire.op.integer.add";
    /// Obviously synthetic; must never collide with a real catalogued
    /// identity.
    const UNCATALOGUED_IDENTITY: &str = "quire.op.test-only.not-a-real-operation";
    /// Catalogued with one required law role (`integer_division`), operator
    /// `binary`, two `integer` operands, no mode, no member.
    const INTEGER_DIV_IDENTITY: &str = "quire.op.integer.div";
    /// Catalogued with operator `binary`, two `decimal` operands, a required
    /// `rounding` mode, no laws, no member.
    const DECIMAL_ADD_IDENTITY: &str = "quire.op.decimal.add";
    /// Catalogued with operator `convert`, one `quantity` operand, a
    /// required `rounding` mode, a required `type_argument` member, no laws.
    const QUANTITY_CONVERT_IDENTITY: &str = "quire.op.quantity.convert";
    /// Catalogued with operator `query`, one `record` operand, a required
    /// `field` member, no laws, no mode.
    const RECORD_PROJECT_IDENTITY: &str = "quire.op.record.project";
    /// Catalogued with operator `binary`, two `structural_kind` operands, a
    /// `same_type` constraint and `leaves: "operand:0"`, no laws, no mode,
    /// no member.
    const STRUCTURAL_EQ_IDENTITY: &str = "quire.op.structural.eq";

    fn dummy_digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn node_id(byte: char) -> CheckedNodeId {
        CheckedNodeId {
            domain: Box::from(NODE_DOMAIN),
            digest: Box::from(dummy_digest(byte).as_str()),
        }
    }

    /// The default `operation` wire shape every fixture below starts from:
    /// `identity`, zero laws, no mode, no member, no leaves. Individual
    /// tests override exactly the field their refusal needs to be wrong.
    fn plain_operation(identity: &str) -> Value {
        json!({
            "identity": identity,
            "laws": [],
            "mode": null,
            "member": null,
            "leaves": [],
        })
    }

    /// A single-node `application` term with a caller-supplied `operator`,
    /// `operation` wire value and `arguments`. `node_id`/`semantic_type` are
    /// placeholders `operation_defect` never inspects for its own checks
    /// (only `validate_application_keys`'s own tests care whether a node's
    /// key is genuine).
    fn custom_application_node(
        operator: &str,
        operation: Value,
        arguments: Vec<Value>,
    ) -> CheckedSemanticNodeV2 {
        let value = json!({
            "node_id": { "domain": NODE_DOMAIN, "digest": dummy_digest('1') },
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "expression",
            "semantic_form": "call",
            "semantic_type": { "domain": NODE_DOMAIN, "digest": dummy_digest('2') },
            "dependencies": [],
            "occurrences": [],
            "body": {
                "term": "application",
                "operator": operator,
                "arguments": arguments,
                "operation": operation,
            },
        });
        serde_json::from_value(value).expect("test fixture node is well-formed")
    }

    /// A minimal single-node `application` term whose `operation.identity`
    /// is `identity` and whose `operator` is `operator`. Arguments are plain
    /// `literal` terms (not `reference`/`binding`), so `argument_family`
    /// resolves them to `None` and `check_operands` skips the family checks
    /// that would otherwise need a second, referenced graph node — the only
    /// thing this fixture needs to reach is the catalog lookup inside
    /// `operation_defect`.
    fn application_node(identity: &str, operator: &str) -> CheckedSemanticNodeV2 {
        custom_application_node(
            operator,
            plain_operation(identity),
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        )
    }

    /// `application_node`'s `node_id.digest` replaced by the genuine JCS
    /// SHA-256 of its own preimage — the same formula
    /// `validate_application_keys` re-derives — so the node passes that
    /// check rather than tripping the `stale-node-key` refusal every other
    /// fixture in this module deliberately does not care about.
    fn correctly_keyed(mut node: CheckedSemanticNodeV2) -> CheckedSemanticNodeV2 {
        let semantic_type =
            serde_json::to_value(&node.semantic_type).expect("semantic_type serializes");
        let preimage = json!({
            "version": APPLICATION_NODE_VERSION,
            "node_tag": node.node_tag.as_ref(),
            "semantic_form": node.semantic_form.as_ref(),
            "semantic_type": semantic_type,
            "declaration": Value::Null,
            "recursion": node.recursion_group.as_deref(),
            "body": node.body.clone(),
        });
        let digest = quire_canonical::sha256(&preimage, quire_canonical::Limits::new(1 << 20))
            .expect("preimage digests")
            .to_string();
        node.node_id.digest = Box::from(digest.as_str());
        node
    }

    /// A bare graph node with caller-supplied tag/form/semantic-type/body,
    /// keyed on `id_byte`. Used to build the second and third nodes
    /// `check_field_member`, `check_mode_type` and `check_leaves` resolve an
    /// operand's declared type through.
    fn graph_node(
        id_byte: char,
        node_tag: &str,
        semantic_form: &str,
        semantic_type: &CheckedNodeId,
        body: Value,
    ) -> CheckedSemanticNodeV2 {
        let value = json!({
            "node_id": { "domain": NODE_DOMAIN, "digest": dummy_digest(id_byte) },
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": node_tag,
            "semantic_form": semantic_form,
            "semantic_type": {
                "domain": semantic_type.domain.as_ref(),
                "digest": semantic_type.digest.as_ref(),
            },
            "dependencies": [],
            "occurrences": [],
            "body": body,
        });
        serde_json::from_value(value).expect("test fixture node is well-formed")
    }

    /// One `operation.laws[]` entry.
    fn law_json(role: &str, definition: Value) -> Value {
        json!({ "role": role, "definition": definition })
    }

    /// A syntactically valid `CheckedArtifactRef` guaranteed absent from
    /// every catalogued law-role definition list.
    fn dummy_law_definition(marker: char) -> Value {
        json!({
            "authority": "test",
            "identity": format!("test-{marker}"),
        })
    }

    /// The `{authority, identity}` of the catalog's own first
    /// `integer_division` law-role definition
    /// (`quire.value.integer-division.truncating/v1`), so
    /// `catalog.entry(...)` recognizes it as catalogued while the empty lock
    /// leaves it unselected.
    fn real_integer_division_truncating_definition() -> Value {
        json!({
            "authority": "agent-ix",
            "identity": "quire.value.integer-division.truncating/v1",
        })
    }

    /// A lock with nothing selected; every test identity here carries zero
    /// laws, so nothing in `operation_defect` ever consults its selections.
    fn empty_lock() -> CheckedPackageLockV2 {
        let placeholder = CheckedArtifactRef {
            authority: Box::from("test"),
            identity: Box::from("test"),
        };
        CheckedPackageLockV2 {
            sources: Vec::new(),
            edition: CheckedSelection {
                role: CheckedSelectionRole::Edition,
                definition: placeholder,
            },
            profile_selections: Vec::new(),
            definition_selections: Vec::new(),
            model_selections: Vec::new(),
            required_features: Vec::new(),
            dependency_selections: Vec::new(),
        }
    }

    fn defect_for(
        node: &CheckedSemanticNodeV2,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        let nodes = std::slice::from_ref(node);
        let mut index: BTreeMap<&CheckedNodeId, usize> = BTreeMap::new();
        index.insert(&node.node_id, 0);
        let lock = empty_lock();
        let mut meter = WorkMeter::new(1_000);
        let kinds = kinds_of(nodes);
        operation_defect(
            Application { node, position: 0 },
            &Graph {
                nodes,
                kinds: &kinds,
                index: &index,
            },
            &lock,
            &ModelOwners::default(),
            DependencyReferences::new(&SuppliedDependencies::default()),
            operation_catalog(),
            &mut meter,
        )
    }

    /// Each node's kind, decoded as intake decodes it.
    fn kinds_of(nodes: &[CheckedSemanticNodeV2]) -> Vec<CheckedNodeKind> {
        nodes
            .iter()
            .map(|node| {
                let tag = CheckedNodeTag::from_wire(&node.node_tag).expect("test node tag");
                CheckedNodeKind::decode(tag, &node.semantic_form).expect("test node form")
            })
            .collect()
    }

    /// Like [`defect_for`], but for a multi-node graph: `nodes[0]` is the
    /// `application` node under test; the rest are the type/declaration
    /// nodes its `operation` or `arguments` reference by id.
    fn defect_for_graph(
        nodes: Vec<CheckedSemanticNodeV2>,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        defect_for_graph_locked(nodes, &empty_lock())
    }

    /// The first catalogued `text_profile` law definition, as the wire
    /// carries it.
    fn text_law_definition() -> Value {
        let definitions = operation_catalog()
            .law_role_definitions(LawRole::TextProfile)
            .expect("the catalog lists text_profile definitions");
        serde_json::to_value(&definitions[0]).expect("definition serializes")
    }

    /// A lock selecting the catalogued `text_profile` definition.
    fn text_selecting_lock() -> CheckedPackageLockV2 {
        let mut lock = empty_lock();
        lock.definition_selections =
            vec![serde_json::from_value(text_law_definition()).expect("definition deserializes")];
        lock
    }

    fn defect_for_graph_locked(
        nodes: Vec<CheckedSemanticNodeV2>,
        lock: &CheckedPackageLockV2,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        defect_for_graph_metered(nodes, lock, 1_000)
    }

    /// [`defect_for_graph_locked`] under a work budget of `limit` units.
    fn defect_for_graph_metered(
        nodes: Vec<CheckedSemanticNodeV2>,
        lock: &CheckedPackageLockV2,
        limit: u64,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        let mut index: BTreeMap<&CheckedNodeId, usize> = BTreeMap::new();
        for (position, node) in nodes.iter().enumerate() {
            index.insert(&node.node_id, position);
        }
        let mut meter = WorkMeter::new(limit);
        let kinds = kinds_of(&nodes);
        operation_defect(
            Application {
                node: &nodes[0],
                position: 0,
            },
            &Graph {
                nodes: &nodes,
                kinds: &kinds,
                index: &index,
            },
            lock,
            &ModelOwners::default(),
            DependencyReferences::new(&SuppliedDependencies::default()),
            operation_catalog(),
            &mut meter,
        )
    }

    fn key(fill: u8) -> String {
        format!("{fill:02x}").repeat(32)
    }

    fn node_ref(fill: u8) -> serde_json::Value {
        json!({ "domain": "quire.checked-semantic-node/v1", "digest": key(fill) })
    }

    fn reference(fill: u8) -> serde_json::Value {
        json!({ "term": "reference", "target": node_ref(fill) })
    }

    fn add(arguments: Vec<serde_json::Value>) -> serde_json::Value {
        json!({
            "term": "application",
            "operator": "binary",
            "operation": {
                "identity": "quire.op.integer.add",
                "laws": [], "mode": null, "member": null, "leaves": []
            },
            "result_type": node_ref(3),
            "arguments": arguments,
        })
    }

    fn grouped_node(id: u8, body: serde_json::Value) -> CheckedSemanticNodeV2 {
        serde_json::from_value(json!({
            "node_id": node_ref(id),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "function",
            "semantic_form": "function",
            "semantic_type": node_ref(3),
            "dependencies": [],
            "occurrences": [],
            "recursion_group": "g",
            "declaration": { "qualified_name": ["pkg", "total"] },
            "body": body,
        }))
        .expect("node")
    }

    /// QSpec FR-322's application preimage for a recursion-group member,
    /// byte for byte the vector quire-spec-language's `application_key`
    /// pins (`preimage_bytes_are_pinned`): the node is member 1 of a group
    /// of 2, its reference to member 0 becomes a `group_reference`, and its
    /// reference outside the group stays a reference.
    ///
    /// Tracing: TC-048, FR-038-AC-88
    #[trace("TC-048", "FR-038-AC-88")]
    #[test]
    fn tc_048_application_preimage_matches_the_qsl_pinned_group_vector() {
        let node = grouped_node(2, add(vec![reference(1), reference(9)]));
        let one = typed(&key(1));
        let group = [&one, &node.node_id];
        let preimage = application_preimage(
            Application {
                node: &node,
                position: 0,
            },
            &group,
        )
        .expect("preimage");
        // Spelled literally, as quire-spec-language's own vector does, rather
        // than produced by the serializer under test.
        let literal_ref = |fill: u8| {
            format!(
                r#"{{"digest":"{}","domain":"quire.checked-semantic-node/v1"}}"#,
                key(fill)
            )
        };
        let three = literal_ref(3);
        let nine = literal_ref(9);
        let expected = format!(
            concat!(
                r#"{{"body":{{"arguments":[{{"ordinal":0,"term":"group_reference"}},"#,
                r#"{{"target":{nine},"term":"reference"}}],"#,
                r#""operation":{{"identity":"quire.op.integer.add","laws":[],"leaves":[],"member":null,"mode":null}},"#,
                r#""operator":"binary","result_type":{three},"term":"application"}},"#,
                r#""declaration":{{"qualified_name":["pkg","total"]}},"#,
                r#""node_tag":"function","recursion":{{"ordinal":1,"size":2}},"#,
                r#""semantic_form":"function","semantic_type":{three},"#,
                r#""version":"quire.application-node/v1"}}"#,
            ),
            nine = nine,
            three = three,
        );
        let bytes = quire_canonical::to_vec(&preimage, quire_canonical::Limits::new(1 << 20))
            .expect("encodes");
        assert_eq!(String::from_utf8(bytes).expect("utf-8"), expected);
    }

    /// Group references are rewritten in every nested term position, as in
    /// quire-spec-language's `group_references_are_rewritten_in_every_nested_term`.
    ///
    /// Tracing: TC-048, FR-038-AC-88
    #[trace("TC-048", "FR-038-AC-88")]
    #[test]
    fn tc_048_group_references_are_rewritten_in_every_nested_term() {
        let body = json!({
            "term": "aggregate",
            "members": [
                { "term": "binding", "name": "x", "value": reference(2) },
                add(vec![reference(1), add(vec![reference(2), reference(9)])]),
            ],
        });
        let node = grouped_node(1, body);
        let two = typed(&key(2));
        let group = [&node.node_id, &two];
        let preimage = application_preimage(
            Application {
                node: &node,
                position: 0,
            },
            &group,
        )
        .expect("preimage");
        let group_reference =
            |ordinal: usize| json!({ "term": "group_reference", "ordinal": ordinal });
        let members = &preimage.body["members"];
        assert_eq!(members[0]["value"], group_reference(1));
        assert_eq!(members[1]["arguments"][0], group_reference(0));
        assert_eq!(
            members[1]["arguments"][1]["arguments"][0],
            group_reference(1)
        );
        assert_eq!(members[1]["arguments"][1]["arguments"][1], reference(9));
        assert_eq!(
            preimage.recursion,
            Some(GroupPlace {
                size: 2,
                ordinal: 0
            })
        );
    }

    fn typed(digest: &str) -> CheckedNodeId {
        serde_json::from_value(json!({
            "domain": "quire.checked-semantic-node/v1",
            "digest": digest
        }))
        .expect("node id")
    }

    /// The family of a type reached through a chain of `bounded_domain` nodes
    /// does not depend on the chain's length: one of 300 resolves to the family
    /// at its end, as one of 1 does; a cycle of bounded domains and a chain that
    /// ends at a node of no family name none; and a type outside the graph names
    /// none. The walk is iterative and follows each node once.
    ///
    /// Tracing: TC-048, FR-038-AC-117
    #[trace("TC-048", "FR-038-AC-117")]
    #[test]
    fn tc_048_a_bounded_domain_chain_resolves_whatever_its_length() {
        const CHAIN: usize = 300;
        let digest = |position: usize| format!("{position:064x}");
        let id = |position: usize| json!({"domain": NODE_DOMAIN, "digest": digest(position)});
        let node = |position: usize,
                    tag: &str,
                    form: &str,
                    semantic_type: usize|
         -> CheckedSemanticNodeV2 {
            serde_json::from_value(json!({
                "node_id": id(position),
                "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": tag,
                "semantic_form": form,
                "semantic_type": id(semantic_type),
                "dependencies": [],
                "occurrences": [],
                "body": {"term": "aggregate", "members": []},
            }))
            .expect("node")
        };
        // Node 0 is an integer; nodes 1..=CHAIN are bounded domains, each over
        // the one before it; nodes CHAIN+1 and CHAIN+2 are a cycle of bounded
        // domains; node CHAIN+3 is a bounded domain over a `claim`, which has
        // no operand family.
        let mut nodes = vec![node(0, "scalar_type", "integer", 0)];
        nodes.extend(
            (1..=CHAIN)
                .map(|position| node(position, "bounded_domain", "integer_range", position - 1)),
        );
        nodes.push(node(
            CHAIN + 1,
            "bounded_domain",
            "integer_range",
            CHAIN + 2,
        ));
        nodes.push(node(
            CHAIN + 2,
            "bounded_domain",
            "integer_range",
            CHAIN + 1,
        ));
        nodes.push(node(
            CHAIN + 3,
            "bounded_domain",
            "integer_range",
            CHAIN + 4,
        ));
        nodes.push(node(CHAIN + 4, "claim", "verification_claim", CHAIN + 4));
        let kinds = nodes
            .iter()
            .map(|node| {
                CheckedNodeKind::decode(
                    CheckedNodeTag::from_wire(&node.node_tag).expect("tag"),
                    &node.semantic_form,
                )
                .expect("kind")
            })
            .collect::<Vec<_>>();
        let index = nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect::<BTreeMap<_, _>>();
        let family = |position: usize| {
            let at: CheckedNodeId = serde_json::from_value(id(position)).expect("id");
            super::resolve_family(&at, &nodes, &kinds, &index)
        };
        assert_eq!(family(0), Some("integer"));
        assert_eq!(family(1), Some("integer"));
        assert_eq!(family(CHAIN), Some("integer"));
        assert_eq!(family(CHAIN + 1), None, "a cycle ends at no type");
        assert_eq!(family(CHAIN + 2), None);
        assert_eq!(family(CHAIN + 3), None, "a chain ending at no family");
        assert_eq!(family(CHAIN + 100), None, "a type outside the graph");
    }

    /// The operand classification over every kind the closed vocabularies
    /// produce: which catalog family a node denotes directly, and whether an
    /// argument naming it names a type. Pinned whole, so an edit to either
    /// exhaustive table that moves any one form is caught here.
    ///
    /// Tracing: TC-048, FR-038-AC-87
    #[trace("TC-048", "FR-038-AC-87")]
    #[test]
    fn operand_classification_is_exactly_the_catalog_mapping() {
        let families = CheckedNodeKind::all()
            .into_iter()
            .filter_map(|kind| {
                Some((
                    kind.tag().as_wire(),
                    kind.form_wire(),
                    operand_family(kind)?,
                ))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            families,
            [
                ("scalar_type", "boolean", "boolean"),
                ("scalar_type", "integer", "integer"),
                ("scalar_type", "rational", "rational"),
                ("scalar_type", "decimal", "decimal"),
                ("scalar_type", "float32", "float32"),
                ("scalar_type", "float64", "float64"),
                ("scalar_type", "text", "text"),
                ("scalar_type", "enum", "enum"),
                ("composite_type", "option", "option"),
                ("composite_type", "sequence", "sequence"),
                ("composite_type", "set", "set"),
                ("composite_type", "bag", "bag"),
                ("composite_type", "ordered_set", "ordered_set"),
                ("composite_type", "record", "record"),
                ("composite_type", "tuple", "tuple"),
                ("composite_type", "union", "union"),
                ("composite_type", "reference", "reference"),
                ("expression", "reference", "reference"),
                ("function", "pure_function", "function"),
                ("function", "predicate", "function"),
                ("function", "recursive_function", "function"),
                ("model", "object_type", "object"),
                ("model", "systems_interface", "object"),
                ("relation", "population", "population"),
                ("temporal", "formula", "temporal"),
            ]
        );
        // Type-shaped is the five type families, every form of each, plus
        // the `reference` expression and the temporal formula; checked for
        // every kind of the taxonomy, so flipping any single form is caught.
        for kind in CheckedNodeKind::all() {
            let expected = matches!(
                kind.tag(),
                CheckedNodeTag::ScalarType
                    | CheckedNodeTag::CompositeType
                    | CheckedNodeTag::BoundedDomain
                    | CheckedNodeTag::Relation
                    | CheckedNodeTag::Function
            ) || kind == CheckedNodeKind::Expression(ExpressionForm::Reference)
                || kind == CheckedNodeKind::Temporal(TemporalForm::Formula);
            assert_eq!(is_type_shaped(kind), expected, "{kind:?}");
        }
    }

    /// An `application` node whose `operation.identity` is absent from the
    /// catalog is refused `invalid_package`/`unknown-operation`. This is the
    /// criterion the deleted private-sourced fixture tree used to carry (see
    /// agent-ix/quire-contract-ir#166): without it, the `catalog.entry(...)`
    /// lookup in `operation_defect` could be replaced by an always-`Some`
    /// admission and nothing in this crate's test suite would notice.
    ///
    /// Tracing: TC-048, FR-038-AC-81
    #[trace("TC-048", "FR-038-AC-81")]
    #[test]
    fn operation_defect_refuses_uncatalogued_identity() {
        let node = application_node(UNCATALOGUED_IDENTITY, "binary");

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/identity",
                Some(CheckedPackageRefusalCause::UnknownOperation),
                node.node_id.clone(),
            ))),
            "an uncatalogued operation.identity must be refused as unknown-operation, got {result:?}"
        );
    }

    /// Control for the test above: the same node shape, with a real
    /// catalogued identity used exactly as its catalog entry requires
    /// (matching operator, arity and operand shape), is admitted — not
    /// refused for `unknown-operation` or anything else. Without this
    /// control, a reader that refused every node would satisfy the assertion
    /// above just as well as the real check does.
    ///
    /// Tracing: TC-048, FR-038-AC-81
    #[trace("TC-048", "FR-038-AC-81")]
    #[test]
    fn operation_defect_admits_catalogued_identity_used_correctly() {
        // Guard against catalog drift: fail loudly here, not by way of a
        // confusing assertion failure below, if this identity is ever
        // removed or reshaped upstream.
        let entry = operation_catalog()
            .entry(CATALOGUED_IDENTITY)
            .unwrap_or_else(|| {
                panic!("{CATALOGUED_IDENTITY} must be catalogued for this control to be meaningful")
            });
        assert_eq!(entry.operator.as_wire(), "binary");
        assert_eq!(entry.operands.len(), 2);

        let node = application_node(CATALOGUED_IDENTITY, "binary");

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(None),
            "a catalogued identity used correctly must not be refused, got {result:?}"
        );
    }

    /// `quire.op.model.reaches_field` whose member declaration names no node
    /// of the graph cannot satisfy `reference_edge`: it refuses as
    /// `ill_typed`/`operator-ineligible` at the member's declaration.
    ///
    /// Tracing: TC-048, FR-038-AC-84
    #[trace("TC-048", "FR-038-AC-84")]
    #[test]
    fn operation_defect_refuses_a_reference_edge_with_no_declaring_node() {
        let node = custom_application_node(
            "reaches",
            json!({
                "identity": "quire.op.model.reaches_field",
                "laws": [], "mode": null, "leaves": [],
                "member": {"kind": "field", "declaration": node_ref(3), "name": "next"},
            }),
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        );
        let node_id = node.node_id.clone();
        assert_eq!(
            defect_for(&node),
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::IllTyped,
                "/semantic_graph/nodes/0/body/operation/member/declaration",
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                node_id,
            )))
        );
    }

    /// `operator-class-mismatch`: agent-ix/quire-contract-ir#171. `binary` is
    /// catalogued for [`CATALOGUED_IDENTITY`]; supplying `unary` must be
    /// refused before arity or anything else is checked.
    ///
    /// Tracing: TC-048, FR-038-AC-81
    #[trace("TC-048", "FR-038-AC-81")]
    #[test]
    fn operation_defect_refuses_wrong_operator_class() {
        let node = application_node(CATALOGUED_IDENTITY, "unary");

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operator",
                Some(CheckedPackageRefusalCause::OperationClassMismatch),
                node.node_id.clone(),
            ))),
            "an operator that disagrees with the catalogued entry's operator class must be \
             refused as operation-class-mismatch, got {result:?}"
        );
    }

    /// `operation-law-missing`: agent-ix/quire-contract-ir#171.
    /// [`INTEGER_DIV_IDENTITY`] requires one `integer_division` law;
    /// supplying zero must be refused.
    ///
    /// Tracing: TC-048, FR-038-AC-82
    #[trace("TC-048", "FR-038-AC-82")]
    #[test]
    fn operation_defect_refuses_missing_laws() {
        let node = custom_application_node(
            "binary",
            plain_operation(INTEGER_DIV_IDENTITY),
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        );

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/laws",
                Some(CheckedPackageRefusalCause::OperationLawMissing),
                node.node_id.clone(),
            ))),
            "fewer laws than the catalogued entry requires must be refused as \
             operation-law-missing, got {result:?}"
        );
    }

    /// `operation-law-mismatch` (too many laws): agent-ix/quire-contract-ir#171.
    /// [`CATALOGUED_IDENTITY`] requires zero laws; supplying one must be
    /// refused as a mismatch, not admitted as extra.
    ///
    /// Tracing: TC-048, FR-038-AC-82
    #[trace("TC-048", "FR-038-AC-82")]
    #[test]
    fn operation_defect_refuses_too_many_laws() {
        let mut operation = plain_operation(CATALOGUED_IDENTITY);
        operation["laws"] = json!([law_json("anything", dummy_law_definition('a'))]);
        let node = custom_application_node(
            "binary",
            operation,
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        );

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/laws/0",
                Some(CheckedPackageRefusalCause::OperationLawMismatch),
                node.node_id.clone(),
            ))),
            "more laws than the catalogued entry admits must be refused as \
             operation-law-mismatch, got {result:?}"
        );
    }

    /// `operation-law-mismatch` (role order): agent-ix/quire-contract-ir#171.
    /// [`INTEGER_DIV_IDENTITY`]'s one required role is `integer_division`; a
    /// law naming any other role at that position must be refused. The
    /// definition is [`real_integer_division_truncating_definition`] (a
    /// genuinely catalogued `integer_division` definition, not a dummy one)
    /// deliberately: if the role check is skipped, the catalog-membership
    /// and lock-selection checks further down key off the *catalogued*
    /// entry's role, not the wire's declared role, so a dummy definition
    /// would still be refused — just under a different cause
    /// (`operation-law-mismatch` for an uncatalogued definition) — and this
    /// test would not distinguish the role check being gone from it being
    /// present.
    ///
    /// Tracing: TC-048, FR-038-AC-82
    #[trace("TC-048", "FR-038-AC-82")]
    #[test]
    fn operation_defect_refuses_wrong_law_role() {
        let mut operation = plain_operation(INTEGER_DIV_IDENTITY);
        operation["laws"] = json!([law_json(
            "not_integer_division",
            real_integer_division_truncating_definition()
        )]);
        let node = custom_application_node(
            "binary",
            operation,
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        );

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/laws/0/role",
                Some(CheckedPackageRefusalCause::OperationLawMismatch),
                node.node_id.clone(),
            ))),
            "a law whose role does not match the catalogued entry's role order must be \
             refused as operation-law-mismatch, got {result:?}"
        );
    }

    /// `operation-law-mismatch` (uncatalogued definition):
    /// agent-ix/quire-contract-ir#171. `integer_division` is a value role
    /// closed over the catalog's own definition list; a definition outside
    /// that list must be refused before the lock is even consulted.
    ///
    /// Tracing: TC-048, FR-038-AC-56
    #[trace("TC-048", "FR-038-AC-56")]
    #[test]
    fn operation_defect_refuses_uncatalogued_law_definition() {
        let mut operation = plain_operation(INTEGER_DIV_IDENTITY);
        operation["laws"] = json!([law_json("integer_division", dummy_law_definition('c'))]);
        let node = custom_application_node(
            "binary",
            operation,
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        );

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/laws/0/definition",
                Some(CheckedPackageRefusalCause::OperationLawMismatch),
                node.node_id.clone(),
            ))),
            "a law definition absent from the catalogued role's own definition list must be \
             refused as operation-law-mismatch, got {result:?}"
        );
    }

    /// `operation-law-unselected`: agent-ix/quire-contract-ir#171. A
    /// catalogued `integer_division` definition
    /// ([`real_integer_division_truncating_definition`], copied byte-for-byte
    /// from the catalog) is known to `catalog.entry(...)`, so this exercises
    /// the *lock selection* check specifically, not the catalog-membership
    /// check the previous test covers: `empty_lock` selects nothing, so it
    /// must still be refused.
    ///
    /// Tracing: TC-048, FR-038-AC-56
    #[trace("TC-048", "FR-038-AC-56")]
    #[test]
    fn operation_defect_refuses_unselected_law_definition() {
        let mut operation = plain_operation(INTEGER_DIV_IDENTITY);
        operation["laws"] = json!([law_json(
            "integer_division",
            real_integer_division_truncating_definition()
        )]);
        let node = custom_application_node(
            "binary",
            operation,
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        );

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/laws/0/definition",
                Some(CheckedPackageRefusalCause::OperationLawUnselected),
                node.node_id.clone(),
            ))),
            "a catalogued law definition absent from the lock's own selections must be \
             refused as operation-law-unselected, got {result:?}"
        );
    }

    /// An `integer.div` application whose one law is `definition`.
    fn integer_division_node(definition: Value) -> CheckedSemanticNodeV2 {
        let mut operation = plain_operation(INTEGER_DIV_IDENTITY);
        operation["laws"] = json!([law_json("integer_division", definition)]);
        custom_application_node(
            "binary",
            operation,
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        )
    }

    /// FR-038-AC-56: a value-role law admits only when its `{authority,
    /// identity}` pair is catalogued for the role and is a
    /// `lock.definition_selections` row, compared by those two members alone.
    ///
    /// Tracing: TC-048, FR-038-AC-56
    #[trace("TC-048", "FR-038-AC-56")]
    #[test]
    fn tc_048_a_value_role_law_joins_the_catalog_and_the_lock_by_authority_and_identity() {
        let catalogued = real_integer_division_truncating_definition();
        let mut lock = empty_lock();
        lock.definition_selections =
            vec![serde_json::from_value(catalogued.clone()).expect("a definition reference")];
        let at = "/semantic_graph/nodes/0/body/operation/laws/0/definition";

        // The selected, catalogued pair admits.
        let node = integer_division_node(catalogued.clone());
        assert_eq!(defect_for_graph_locked(vec![node], &lock), Ok(None));

        // A pair the role does not catalogue refuses `operation-law-mismatch`,
        // an empty pair included, and so does the catalogued identity under
        // another authority: the pair is the whole comparison.
        for (name, definition) in [
            ("another pair", dummy_law_definition('c')),
            ("empty pair", json!({ "authority": "", "identity": "" })),
            (
                "another authority",
                json!({
                    "authority": "other",
                    "identity": catalogued["identity"],
                }),
            ),
        ] {
            let node = integer_division_node(definition);
            assert_eq!(
                defect_for_graph_locked(vec![node.clone()], &lock),
                Ok(Some(refused_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    at,
                    Some(CheckedPackageRefusalCause::OperationLawMismatch),
                    node.node_id.clone(),
                ))),
                "{name}"
            );
        }

        // A catalogued pair the lock does not select refuses
        // `operation-law-unselected`.
        let node = integer_division_node(catalogued);
        assert_eq!(
            defect_for_graph_locked(vec![node.clone()], &empty_lock()),
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                at,
                Some(CheckedPackageRefusalCause::OperationLawUnselected),
                node.node_id.clone(),
            )))
        );
    }

    /// `operation-mode-mismatch`: agent-ix/quire-contract-ir#171.
    /// [`DECIMAL_ADD_IDENTITY`] requires a `rounding` mode; a missing mode
    /// must be refused.
    ///
    /// Tracing: TC-048, FR-038-AC-83
    #[trace("TC-048", "FR-038-AC-83")]
    #[test]
    fn operation_defect_refuses_mode_mismatch() {
        let node = custom_application_node(
            "binary",
            plain_operation(DECIMAL_ADD_IDENTITY),
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        );

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/mode",
                Some(CheckedPackageRefusalCause::OperationModeMismatch),
                node.node_id.clone(),
            ))),
            "a missing mode where the catalogued entry requires one must be refused as \
             operation-mode-mismatch, got {result:?}"
        );
    }

    /// `operation-member-mismatch`: agent-ix/quire-contract-ir#171.
    /// [`QUANTITY_CONVERT_IDENTITY`] requires a `type_argument` member; the
    /// mode is set to match so the member check, not the mode check, is what
    /// fails.
    ///
    /// Tracing: TC-048, FR-038-AC-84
    #[trace("TC-048", "FR-038-AC-84")]
    #[test]
    fn operation_defect_refuses_member_mismatch() {
        let mut operation = plain_operation(QUANTITY_CONVERT_IDENTITY);
        operation["mode"] = json!({ "kind": "rounding", "value": "exact" });
        let node = custom_application_node(
            "convert",
            operation,
            vec![json!({ "term": "literal", "value": 1 })],
        );

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/member",
                Some(CheckedPackageRefusalCause::OperationMemberMismatch),
                node.node_id.clone(),
            ))),
            "a missing member where the catalogued entry requires one must be refused as \
             operation-member-mismatch, got {result:?}"
        );
    }

    /// `operator-ineligible` (arity): agent-ix/quire-contract-ir#171.
    /// [`CATALOGUED_IDENTITY`] takes exactly two operands and admits no
    /// `rest`; three arguments must be refused by `check_operands`.
    ///
    /// Tracing: TC-048, FR-038-AC-85
    #[trace("TC-048", "FR-038-AC-85")]
    #[test]
    fn operation_defect_refuses_wrong_arity() {
        let node = custom_application_node(
            "binary",
            plain_operation(CATALOGUED_IDENTITY),
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
                json!({ "term": "literal", "value": 3 }),
            ],
        );

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::IllTyped,
                "/semantic_graph/nodes/0/body/arguments",
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                node.node_id.clone(),
            ))),
            "an argument count that disagrees with the catalogued entry's fixed arity must be \
             refused as operator-ineligible, got {result:?}"
        );
    }

    /// `operator-ineligible` (field member): agent-ix/quire-contract-ir#171.
    /// [`RECORD_PROJECT_IDENTITY`] requires a `field` member; this exercises
    /// `check_field_member` specifically (reached only once the generic
    /// member-kind check above already passed) by naming a field its
    /// declared record type does not declare.
    ///
    /// Tracing: TC-048, FR-038-AC-84
    #[trace("TC-048", "FR-038-AC-84")]
    #[test]
    fn operation_defect_refuses_undeclared_field_member() {
        let record_type = node_id('7');
        let record_node = graph_node(
            '7',
            "composite_type",
            "record",
            &node_id('8'),
            json!({
                "term": "aggregate",
                "members": [
                    {
                        "term": "binding",
                        "name": "other_field",
                        "value": {
                            "term": "reference",
                            "target": { "domain": NODE_DOMAIN, "digest": dummy_digest('9') },
                        },
                    },
                ],
            }),
        );

        let mut operation = plain_operation(RECORD_PROJECT_IDENTITY);
        operation["member"] = json!({
            "kind": "field",
            "declaration": {
                "domain": record_type.domain.as_ref(),
                "digest": record_type.digest.as_ref(),
            },
            "name": "missing_field",
        });
        let root = custom_application_node(
            "query",
            operation,
            vec![json!({ "term": "literal", "value": 1 })],
        );

        let result = defect_for_graph(vec![root.clone(), record_node]);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::IllTyped,
                "/semantic_graph/nodes/0/body/operation/member/name",
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                root.node_id.clone(),
            ))),
            "a field member naming a field its declared record type does not declare must be \
             refused as operator-ineligible, got {result:?}"
        );
    }

    /// `operation-mode-type-mismatch` (operand): agent-ix/quire-contract-ir#171.
    /// [`DECIMAL_ADD_IDENTITY`]'s first operand is a `reference` to a
    /// `bounded_domain` node whose own body pins `rounding` to
    /// `"nearest-even"`; the operation's own mode value disagrees, so
    /// `check_mode_type` must refuse it.
    ///
    /// Tracing: TC-048, FR-038-AC-83
    #[trace("TC-048", "FR-038-AC-83")]
    #[test]
    fn operation_defect_refuses_mode_type_mismatch_on_operand() {
        let decimal_scalar = graph_node('5', "scalar_type", "decimal", &node_id('6'), json!({}));
        let decimal_range = graph_node(
            '4',
            "bounded_domain",
            "decimal_range",
            &node_id('5'),
            json!({
                "term": "aggregate",
                "members": [
                    {
                        "term": "binding",
                        "name": "rounding",
                        "value": { "term": "literal", "value": "nearest-even" },
                    },
                ],
            }),
        );

        let mut operation = plain_operation(DECIMAL_ADD_IDENTITY);
        operation["mode"] = json!({ "kind": "rounding", "value": "toward-zero" });
        let root = custom_application_node(
            "binary",
            operation,
            vec![
                json!({
                    "term": "reference",
                    "target": { "domain": NODE_DOMAIN, "digest": dummy_digest('4') },
                }),
                json!({ "term": "literal", "value": 0 }),
            ],
        );

        let result = defect_for_graph(vec![root.clone(), decimal_range, decimal_scalar]);

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/mode/value",
                Some(CheckedPackageRefusalCause::OperationModeTypeMismatch),
                root.node_id.clone(),
            ))),
            "a mode value that disagrees with what the first operand's own type pins must be \
             refused as operation-mode-type-mismatch, got {result:?}"
        );
    }

    /// `operation-mode-type-mismatch` (leaf): agent-ix/quire-contract-ir#171.
    /// [`STRUCTURAL_EQ_IDENTITY`]'s first operand is a `record` whose
    /// `name` field's own text type pins the `nfc` `text_profile`; a
    /// `["field:name"]` leaf whose `text_profile` mode value (`nfd`) is a
    /// catalogued value other than the pinned one must be refused by
    /// `check_leaf_count`, independent of the operation's own top-level mode
    /// (left absent here, so `check_mode_type` never fires first). The leaf
    /// carries its `text_profile` law, so the leaf shape settles first.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn operation_defect_refuses_mode_type_mismatch_on_leaf() {
        let record_node = graph_node(
            'r',
            "composite_type",
            "record",
            &node_id('t'),
            json!({
                "term": "aggregate",
                "members": [
                    {
                        "term": "binding",
                        "name": "name",
                        "value": {
                            "term": "reference",
                            "target": { "domain": NODE_DOMAIN, "digest": dummy_digest('f') },
                        },
                    },
                ],
            }),
        );
        let field_type_node = graph_node(
            'f',
            "bounded_domain",
            "text_bounds",
            &node_id('t'),
            json!({
                "term": "aggregate",
                "members": [
                    {
                        "term": "binding",
                        "name": "text_profile",
                        "value": { "term": "literal", "value": "nfc" },
                    },
                ],
            }),
        );

        let mut operation = plain_operation(STRUCTURAL_EQ_IDENTITY);
        operation["leaves"] = json!([
            {
                "path": ["field:name"],
                "mode": { "kind": "text_profile", "value": "nfd" },
                "laws": [law_json("text_profile", text_law_definition())],
            }
        ]);
        let root = custom_application_node(
            "binary",
            operation,
            vec![
                json!({
                    "term": "reference",
                    "target": { "domain": NODE_DOMAIN, "digest": dummy_digest('r') },
                }),
                json!({ "term": "literal", "value": 0 }),
            ],
        );

        let result = defect_for_graph_locked(
            vec![
                root.clone(),
                record_node,
                field_type_node,
                scalar_type_node('t', "text"),
            ],
            &text_selecting_lock(),
        );

        assert_eq!(
            result,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/leaves/0/mode/value",
                Some(CheckedPackageRefusalCause::OperationModeTypeMismatch),
                root.node_id.clone(),
            ))),
            "a leaf mode value that disagrees with what the named field's own type pins must \
             be refused as operation-mode-type-mismatch, got {result:?}"
        );
    }

    /// A scalar type node of `form` keyed on `id_byte`. A `text` node binds
    /// the `nfc` profile on itself, so a text type found through it pins one
    /// as the reference requires of every text leaf; [`unpinned_text_node`]
    /// is the one that pins none.
    fn scalar_type_node(id_byte: char, form: &str) -> CheckedSemanticNodeV2 {
        let members = if form == "text" {
            json!([{
                "term": "binding",
                "name": "text_profile",
                "value": { "term": "literal", "value": "nfc" },
            }])
        } else {
            json!([])
        };
        graph_node(
            id_byte,
            "scalar_type",
            form,
            &node_id('t'),
            json!({ "term": "aggregate", "members": members }),
        )
    }

    /// A `text` scalar that pins no profile.
    fn unpinned_text_node(id_byte: char) -> CheckedSemanticNodeV2 {
        graph_node(
            id_byte,
            "scalar_type",
            "text",
            &node_id('t'),
            json!({ "term": "aggregate", "members": [] }),
        )
    }

    /// A `reference` term naming the node keyed on `id_byte`.
    fn reference_to(id_byte: char) -> Value {
        json!({
            "term": "reference",
            "target": { "domain": NODE_DOMAIN, "digest": dummy_digest(id_byte) },
        })
    }

    /// A record type keyed on `id_byte` with one field per `(name, type)`.
    fn record_type_node(id_byte: char, fields: &[(&str, char)]) -> CheckedSemanticNodeV2 {
        let members: Vec<Value> = fields
            .iter()
            .map(|(name, type_byte)| {
                json!({ "term": "binding", "name": name, "value": reference_to(*type_byte) })
            })
            .collect();
        graph_node(
            id_byte,
            "composite_type",
            "record",
            &node_id(id_byte),
            json!({ "term": "aggregate", "members": members }),
        )
    }

    /// A one-member collection/option type keyed on `id_byte` over `inner`.
    fn collection_type_node(id_byte: char, form: &str, inner: char) -> CheckedSemanticNodeV2 {
        graph_node(
            id_byte,
            "composite_type",
            form,
            &node_id(id_byte),
            json!({ "term": "aggregate", "members": [reference_to(inner)] }),
        )
    }

    /// A `parameter` value keyed on `id_byte` whose type is `type_byte`.
    fn parameter_of(id_byte: char, type_byte: char) -> CheckedSemanticNodeV2 {
        graph_node(
            id_byte,
            "value",
            "parameter",
            &node_id(type_byte),
            json!({ "term": "aggregate", "members": [] }),
        )
    }

    /// `identity` over the two parameters keyed `a` and `b`, supplying
    /// `leaves` as the operation's leaf list, against `types`.
    fn leaves_defect(
        identity: &str,
        leaves: Value,
        types: Vec<CheckedSemanticNodeV2>,
        operands: impl IntoIterator<Item = char>,
    ) -> (
        Result<Option<ValidationFailure>, ValidationFailure>,
        CheckedNodeId,
    ) {
        leaves_defect_result(identity, leaves, types, operands, None)
    }

    /// [`leaves_defect`] for an application whose `result_type` names the
    /// node keyed `result`.
    fn leaves_defect_result(
        identity: &str,
        leaves: Value,
        types: Vec<CheckedSemanticNodeV2>,
        operands: impl IntoIterator<Item = char>,
        result: Option<char>,
    ) -> (
        Result<Option<ValidationFailure>, ValidationFailure>,
        CheckedNodeId,
    ) {
        leaves_defect_locked(
            identity,
            leaves,
            types,
            operands,
            result,
            &text_selecting_lock(),
        )
    }

    /// [`leaves_defect_result`] against `lock`.
    fn leaves_defect_locked(
        identity: &str,
        leaves: Value,
        types: Vec<CheckedSemanticNodeV2>,
        operands: impl IntoIterator<Item = char>,
        result: Option<char>,
        lock: &CheckedPackageLockV2,
    ) -> (
        Result<Option<ValidationFailure>, ValidationFailure>,
        CheckedNodeId,
    ) {
        leaves_defect_metered(identity, leaves, types, operands, result, lock, 1_000)
    }

    /// [`leaves_defect_locked`] under a work budget of `limit` units.
    fn leaves_defect_metered(
        identity: &str,
        leaves: Value,
        mut types: Vec<CheckedSemanticNodeV2>,
        operands: impl IntoIterator<Item = char>,
        result: Option<char>,
        lock: &CheckedPackageLockV2,
        limit: u64,
    ) -> (
        Result<Option<ValidationFailure>, ValidationFailure>,
        CheckedNodeId,
    ) {
        let mut operation = plain_operation(identity);
        operation["leaves"] = leaves;
        let mut root = custom_application_node(
            if identity.contains("structural") {
                "binary"
            } else {
                "collection"
            },
            operation,
            operands.into_iter().map(reference_to).collect(),
        );
        if let Some(result) = result {
            root.body["result_type"] = json!({
                "domain": NODE_DOMAIN,
                "digest": dummy_digest(result),
            });
        }
        let locus = root.node_id.clone();
        let mut graph = vec![root];
        graph.append(&mut types);
        (defect_for_graph_metered(graph, lock, limit), locus)
    }

    fn leaves_missing(
        locus: CheckedNodeId,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        Ok(Some(refused_at(
            CheckedPackageRefusalCode::InvalidPackage,
            "/semantic_graph/nodes/0/body/operation/leaves",
            Some(CheckedPackageRefusalCause::OperationLawMissing),
            locus,
        )))
    }

    /// QSpec FR-322: `leaves` lists one entry per text leaf of the compared
    /// type, so an all-integer record has none and `[]` is admitted for
    /// `structural.eq` and `structural.ne`.
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[trace("TC-048", "FR-038-AC-43")]
    #[test]
    fn tc_048_leaf_source_admits_empty_leaves_over_a_type_without_text() {
        for identity in ["quire.op.structural.eq", "quire.op.structural.ne"] {
            let types = vec![
                record_type_node('r', &[("x", 'i'), ("y", 'i')]),
                scalar_type_node('i', "integer"),
                parameter_of('a', 'r'),
                parameter_of('b', 'r'),
            ];
            let (result, _) = leaves_defect(identity, json!([]), types, ['a', 'b']);
            assert_eq!(
                result,
                Ok(None),
                "{identity} over an all-integer record has no text leaf, so `leaves: []` \
                 must be admitted, got {result:?}"
            );
        }
    }

    /// The `inner:0` leaf source (`collection.contains`) over a set of
    /// integers has no text leaf either.
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[trace("TC-048", "FR-038-AC-43")]
    #[test]
    fn tc_048_inner_leaf_source_admits_empty_leaves_over_a_type_without_text() {
        let types = vec![
            collection_type_node('s', "set", 'i'),
            scalar_type_node('i', "integer"),
            parameter_of('a', 's'),
            parameter_of('b', 'i'),
        ];
        let (result, _) =
            leaves_defect("quire.op.collection.contains", json!([]), types, ['a', 'b']);
        assert_eq!(
            result,
            Ok(None),
            "collection.contains over a set of integers has no text leaf, so `leaves: []` \
             must be admitted, got {result:?}"
        );
    }

    /// A text leaf anywhere in the compared type still demands its leaf: a
    /// record with a text field, nested one level down, with no leaves is
    /// `operation-law-missing`; so is a set of text.
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[trace("TC-048", "FR-038-AC-43")]
    #[test]
    fn tc_048_leaf_source_refuses_empty_leaves_over_a_type_with_text() {
        let nested = vec![
            record_type_node('r', &[("x", 'i'), ("o", 'q')]),
            record_type_node('q', &[("name", 'x')]),
            scalar_type_node('i', "integer"),
            scalar_type_node('x', "text"),
            parameter_of('a', 'r'),
            parameter_of('b', 'r'),
        ];
        let (result, locus) =
            leaves_defect("quire.op.structural.eq", json!([]), nested, ['a', 'b']);
        assert_eq!(
            result,
            leaves_missing(locus),
            "nested text field, no leaves"
        );

        let texts = vec![
            collection_type_node('s', "set", 'x'),
            scalar_type_node('x', "text"),
            parameter_of('a', 's'),
            parameter_of('b', 'x'),
        ];
        let (result, locus) =
            leaves_defect("quire.op.collection.contains", json!([]), texts, ['a', 'b']);
        assert_eq!(result, leaves_missing(locus), "set of text, no leaves");
    }

    /// Fewer leaves than text leaves is still law-missing: two text fields,
    /// one leaf supplied.
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[trace("TC-048", "FR-038-AC-43")]
    #[test]
    fn tc_048_leaf_source_refuses_too_few_leaves() {
        let types = vec![
            record_type_node('r', &[("a", 'x'), ("b", 'x')]),
            scalar_type_node('x', "text"),
            parameter_of('a', 'r'),
            parameter_of('b', 'r'),
        ];
        let (result, locus) = leaves_defect(
            "quire.op.structural.eq",
            json!([{ "path": ["field:a"], "laws": [] }]),
            types,
            ['a', 'b'],
        );
        assert_eq!(result, leaves_missing(locus));
    }

    /// One supplied leaf: `path` with one catalogued `text_profile` law and
    /// the `nfc` profile mode the test text types pin.
    fn text_leaf(path: &[&str]) -> Value {
        json!({
            "path": path,
            "laws": [law_json("text_profile", text_law_definition())],
            "mode": { "kind": "text_profile", "value": "nfc" },
        })
    }

    /// The refusal `check_leaf_count` raises at `operation.leaves` extended by
    /// `suffix`.
    fn leaves_refused(
        locus: CheckedNodeId,
        code: CheckedPackageRefusalCode,
        suffix: &str,
        cause: CheckedPackageRefusalCause,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        Ok(Some(refused_at(
            code,
            &format!("/semantic_graph/nodes/0/body/operation/leaves{suffix}"),
            Some(cause),
            locus,
        )))
    }

    fn leaves_mismatch(
        locus: CheckedNodeId,
        suffix: &str,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        leaves_refused(
            locus,
            CheckedPackageRefusalCode::InvalidPackage,
            suffix,
            CheckedPackageRefusalCause::OperationLawMismatch,
        )
    }

    /// `structural.eq` over a record of `a` and `b` text fields and `c`, an
    /// option of text, plus `i`, an integer: three text leaves.
    fn text_record_types() -> Vec<CheckedSemanticNodeV2> {
        let mut types = vec![
            record_type_node('r', &[("a", 'x'), ("n", 'i'), ("b", 'x'), ("c", 'o')]),
            collection_type_node('o', "option", 'x'),
            scalar_type_node('i', "integer"),
            parameter_of('a', 'r'),
            parameter_of('b', 'r'),
        ];
        types.extend(profiled_text());
        types
    }

    /// The text type keyed `x` as QSL emits it: a `text_bounds` domain that
    /// binds the `nfc` profile over a `text` scalar (keyed `T`) that pins none.
    fn profiled_text() -> [CheckedSemanticNodeV2; 2] {
        [
            graph_node(
                'x',
                "bounded_domain",
                "text_bounds",
                &node_id('T'),
                json!({
                    "term": "aggregate",
                    "members": [{
                        "term": "binding",
                        "name": "text_profile",
                        "value": { "term": "literal", "value": "nfc" },
                    }],
                }),
            ),
            unpinned_text_node('T'),
        ]
    }

    /// QSpec FR-322: the leaves are exactly the text leaves of the compared
    /// type in declaration order, each at its derived path with one
    /// `text_profile` law, and nothing else, whatever the leaf source.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_leaf_source_admits_exactly_the_derived_leaves() {
        let (result, _) = leaves_defect(
            "quire.op.structural.eq",
            json!([
                text_leaf(&["field:a"]),
                text_leaf(&["field:b"]),
                text_leaf(&["field:c", "inner"]),
            ]),
            text_record_types(),
            ['a', 'b'],
        );
        assert_eq!(result, Ok(None), "record, option");

        let mut tuple = vec![
            tuple_type_node('r', &['x', 'i', 's']),
            collection_type_node('s', "sequence", 'x'),
            scalar_type_node('i', "integer"),
            parameter_of('a', 'r'),
            parameter_of('b', 'r'),
        ];
        tuple.extend(profiled_text());
        let (result, _) = leaves_defect(
            "quire.op.structural.eq",
            json!([
                text_leaf(&["position:0"]),
                text_leaf(&["position:2", "inner"])
            ]),
            tuple,
            ['a', 'b'],
        );
        assert_eq!(result, Ok(None), "tuple, sequence");

        let mut texts = vec![
            collection_type_node('s', "set", 'x'),
            parameter_of('a', 's'),
            parameter_of('b', 'x'),
        ];
        texts.extend(profiled_text());
        let (result, _) = leaves_defect(
            "quire.op.collection.contains",
            json!([text_leaf(&[])]),
            texts,
            ['a', 'b'],
        );
        assert_eq!(
            result,
            Ok(None),
            "inner:0 over a set of text is the empty path"
        );
    }

    /// More leaves than text leaves is `operation-law-mismatch`, never
    /// admitted: two extra leaves over an all-integer record, one extra over
    /// the text record, and three unrelated entries over three text leaves.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_leaf_source_refuses_extra_and_unrelated_leaves() {
        let integers = vec![
            record_type_node('r', &[("x", 'i'), ("y", 'i')]),
            scalar_type_node('i', "integer"),
            parameter_of('a', 'r'),
            parameter_of('b', 'r'),
        ];
        let (result, locus) = leaves_defect(
            "quire.op.structural.eq",
            json!([text_leaf(&["field:x"]), text_leaf(&["field:y"])]),
            integers,
            ['a', 'b'],
        );
        assert_eq!(
            result,
            leaves_mismatch(locus, "/0"),
            "two leaves over an all-integer record"
        );

        let (result, locus) = leaves_defect(
            "quire.op.structural.eq",
            json!([
                text_leaf(&["field:a"]),
                text_leaf(&["field:b"]),
                text_leaf(&["field:c", "inner"]),
                text_leaf(&["field:a"]),
            ]),
            text_record_types(),
            ['a', 'b'],
        );
        assert_eq!(result, leaves_mismatch(locus, "/3"), "one extra leaf");

        let (result, locus) = leaves_defect(
            "quire.op.structural.eq",
            json!([
                text_leaf(&["field:p"]),
                text_leaf(&["field:q"]),
                text_leaf(&["field:z"]),
            ]),
            text_record_types(),
            ['a', 'b'],
        );
        assert_eq!(
            result,
            leaves_mismatch(locus, "/0/path"),
            "three unrelated leaves over three text leaves"
        );
    }

    /// A leaf at the wrong path, or the right leaves in the wrong order, is
    /// `operation-law-mismatch` at the first leaf that differs.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_leaf_source_refuses_a_wrong_path_and_a_wrong_order() {
        let run = |leaves: Value| {
            leaves_defect(
                "quire.op.structural.eq",
                leaves,
                text_record_types(),
                ['a', 'b'],
            )
        };
        let (result, locus) = run(json!([
            text_leaf(&["field:a"]),
            text_leaf(&["field:b"]),
            text_leaf(&["field:c"]),
        ]));
        assert_eq!(result, leaves_mismatch(locus, "/2/path"), "missing `inner`");

        let (result, locus) = run(json!([
            text_leaf(&["field:b"]),
            text_leaf(&["field:a"]),
            text_leaf(&["field:c", "inner"]),
        ]));
        assert_eq!(result, leaves_mismatch(locus, "/0/path"), "wrong order");

        let (result, locus) = run(json!([
            text_leaf(&["field:a"]),
            text_leaf(&["position:1"]),
            text_leaf(&["field:c", "inner"]),
        ]));
        assert_eq!(result, leaves_mismatch(locus, "/1/path"), "wrong segment");
    }

    /// Each leaf carries exactly one catalogued `text_profile` law: none, two,
    /// another role and an uncatalogued definition are all
    /// `operation-law-mismatch` at that leaf's `laws`.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_leaf_source_refuses_a_wrong_or_missing_leaf_law() {
        let with_laws = |laws: Value| {
            let mut leaf = text_leaf(&["field:b"]);
            leaf["laws"] = laws;
            leaves_defect(
                "quire.op.structural.eq",
                json!([
                    text_leaf(&["field:a"]),
                    leaf,
                    text_leaf(&["field:c", "inner"]),
                ]),
                text_record_types(),
                ['a', 'b'],
            )
        };
        let good = || law_json("text_profile", text_law_definition());
        let cases = [
            ("no law", json!([])),
            ("two laws", json!([good(), good()])),
            (
                "another role",
                json!([law_json("integer_division", text_law_definition())]),
            ),
            (
                "an uncatalogued definition",
                json!([law_json("text_profile", dummy_law_definition('9'))]),
            ),
        ];
        for (name, laws) in cases {
            let (result, locus) = with_laws(laws);
            assert_eq!(result, leaves_mismatch(locus, "/1/laws"), "{name}");
        }
    }

    /// A leaf law the lock does not select is `operation-law-unselected`, after
    /// the leaf shape is settled.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_leaf_source_refuses_an_unselected_leaf_law() {
        let (result, locus) = leaves_defect_locked(
            "quire.op.structural.eq",
            json!([
                text_leaf(&["field:a"]),
                text_leaf(&["field:b"]),
                text_leaf(&["field:c", "inner"]),
            ]),
            text_record_types(),
            ['a', 'b'],
            None,
            &empty_lock(),
        );
        assert_eq!(
            result,
            leaves_refused(
                locus,
                CheckedPackageRefusalCode::InvalidPackage,
                "/0/laws/0/definition",
                CheckedPackageRefusalCause::OperationLawUnselected,
            )
        );
    }

    /// Leaf law selection settles before any leaf mode, as the reference
    /// orders it: a leaf with an unselected law and no mode is
    /// `operation-law-unselected`, not `operation-mode-mismatch`.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_leaf_law_selection_is_settled_before_leaf_modes() {
        let mut bare = text_leaf(&["field:a"]);
        bare.as_object_mut().expect("leaf").remove("mode");
        let (result, locus) = leaves_defect_locked(
            "quire.op.structural.eq",
            json!([
                bare,
                text_leaf(&["field:b"]),
                text_leaf(&["field:c", "inner"]),
            ]),
            text_record_types(),
            ['a', 'b'],
            None,
            &empty_lock(),
        );
        assert_eq!(
            result,
            leaves_refused(
                locus,
                CheckedPackageRefusalCode::InvalidPackage,
                "/0/laws/0/definition",
                CheckedPackageRefusalCause::OperationLawUnselected,
            )
        );
    }

    /// The `text_profile` pin of an alias chain is memoised and charged, so a
    /// record whose 300 fields each name a different alias of one 300-link
    /// chain costs the chain once (about 600 work units, inside the 1 000 the
    /// test meter allows) rather than once per field, which would exhaust it.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_text_profile_pin_of_an_alias_chain_is_memoised_and_charged() {
        let id = |level: u32| char::from_u32(0x4e00 + level).expect("test id");
        let alias = |level: u32| id(1000 + level);
        let names: Vec<String> = (0..300).map(|level| format!("f{level}")).collect();
        let fields: Vec<(&str, char)> = names
            .iter()
            .enumerate()
            .map(|(level, name)| (name.as_str(), alias(u32::try_from(level).expect("level"))))
            .collect();
        let mut graph = vec![
            record_type_node('r', &fields),
            parameter_of('a', 'r'),
            parameter_of('b', 'r'),
        ];
        for level in 0..300 {
            let target = if level == 299 { 'x' } else { alias(level + 1) };
            graph.push(forwarding_node(
                alias(level),
                "composite_type",
                "alias",
                target,
            ));
        }
        graph.extend(profiled_text());
        let run = |limit: u64| {
            leaves_defect_metered(
                "quire.op.structural.eq",
                json!([]),
                graph.clone(),
                ['a', 'b'],
                None,
                &text_selecting_lock(),
                limit,
            )
        };
        let (result, locus) = run(1_000);
        assert_eq!(result, leaves_missing(locus));
        // The walk takes about 300 field visits plus 300 chain steps. A budget
        // of 450 holds the visits alone, so only charging the chain steps
        // exhausts it.
        let (result, _) = run(450);
        assert!(
            matches!(&result, Err(failure) if format!("{failure:?}").contains("Work")),
            "the chain steps must be charged to the work budget, got {result:?}"
        );
    }

    /// An entry that names no leaf source admits no leaves: a supplied one is
    /// `operation-law-mismatch`, as the reference reader refuses it.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_entry_without_a_leaf_source_refuses_a_supplied_leaf() {
        let mut operation = plain_operation(CATALOGUED_IDENTITY);
        operation["leaves"] = json!([text_leaf(&["field:a"])]);
        let node = custom_application_node(
            "binary",
            operation,
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        );
        assert_eq!(
            defect_for(&node),
            leaves_mismatch(node.node_id.clone(), "/0")
        );
    }

    /// The no-source check runs ahead of the per-law selection loop, as the
    /// reference orders it: `integer.div` carries a law the empty lock does not
    /// select, and a supplied leaf is still `operation-law-mismatch`.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_entry_without_a_leaf_source_refuses_a_leaf_before_law_selection() {
        let mut operation = plain_operation(INTEGER_DIV_IDENTITY);
        operation["laws"] = json!([law_json(
            "integer_division",
            real_integer_division_truncating_definition()
        )]);
        operation["leaves"] = json!([text_leaf(&["field:a"])]);
        let node = custom_application_node(
            "binary",
            operation,
            vec![
                json!({ "term": "literal", "value": 1 }),
                json!({ "term": "literal", "value": 2 }),
            ],
        );
        assert_eq!(
            defect_for(&node),
            leaves_mismatch(node.node_id.clone(), "/0")
        );
    }

    /// `result_inner` over a result that is not a set, bag or ordered set
    /// expects no leaves: `collection.flatten` to a `sequence` of text refuses
    /// one supplied leaf and admits none.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_result_inner_over_a_sequence_result_refuses_a_supplied_leaf() {
        let mut nested = vec![
            collection_type_node('q', "sequence", 'x'),
            collection_type_node('n', "sequence", 'q'),
            collection_type_node('R', "sequence", 'x'),
            parameter_of('a', 'n'),
        ];
        nested.extend(profiled_text());
        let run = |leaves: Value| {
            leaves_defect_result(
                "quire.op.collection.flatten",
                leaves,
                nested.clone(),
                ['a'],
                Some('R'),
            )
        };
        let (result, locus) = run(json!([text_leaf(&["inner"])]));
        assert_eq!(result, leaves_mismatch(locus, "/0"), "one leaf");
        let (result, _) = run(json!([]));
        assert_eq!(result, Ok(None), "no leaf");
    }

    /// A text leaf whose type pins no `text_profile` is undecidable, as in the
    /// reference: `ill_typed`/`operator-ineligible` at `operation.leaves`,
    /// even when the same `text` scalar is also reached through a pinning
    /// wrapper.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_text_leaf_pinning_no_profile_is_ineligible() {
        let types = vec![
            record_type_node('r', &[("a", 'x'), ("b", 'T')]),
            parameter_of('a', 'r'),
            parameter_of('b', 'r'),
        ]
        .into_iter()
        .chain(profiled_text())
        .collect::<Vec<_>>();
        for leaves in [
            json!([text_leaf(&["field:a"]), text_leaf(&["field:b"])]),
            json!([]),
        ] {
            let (result, locus) =
                leaves_defect("quire.op.structural.eq", leaves, types.clone(), ['a', 'b']);
            assert_eq!(result, leaves_ineligible(locus));
        }
    }

    /// Each leaf's mode must be a catalogued `text_profile` mode: absent,
    /// another kind and an uncatalogued value are `operation-mode-mismatch`
    /// at the member at fault; a catalogued value other than the type's pin is
    /// `operation-mode-type-mismatch`.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_leaf_mode_must_be_the_pinned_text_profile() {
        let with_mode = |mode: Option<Value>| {
            let mut leaf = text_leaf(&["field:b"]);
            match mode {
                Some(mode) => leaf["mode"] = mode,
                None => {
                    leaf.as_object_mut().expect("leaf").remove("mode");
                }
            }
            leaves_defect(
                "quire.op.structural.eq",
                json!([
                    text_leaf(&["field:a"]),
                    leaf,
                    text_leaf(&["field:c", "inner"]),
                ]),
                text_record_types(),
                ['a', 'b'],
            )
        };
        let refused = |locus, suffix: &str, cause| {
            leaves_refused(
                locus,
                CheckedPackageRefusalCode::InvalidPackage,
                suffix,
                cause,
            )
        };
        let mismatch = CheckedPackageRefusalCause::OperationModeMismatch;
        let (result, locus) = with_mode(None);
        assert_eq!(result, refused(locus, "/1/mode", mismatch), "no mode");
        let (result, locus) = with_mode(Some(json!({ "kind": "rounding", "value": "exact" })));
        assert_eq!(
            result,
            refused(locus, "/1/mode/kind", mismatch),
            "another kind"
        );
        let (result, locus) = with_mode(Some(json!({ "kind": "text_profile", "value": "x" })));
        assert_eq!(
            result,
            refused(locus, "/1/mode/value", mismatch),
            "uncatalogued value"
        );
        let (result, locus) = with_mode(Some(json!({ "kind": "text_profile", "value": "nfd" })));
        assert_eq!(
            result,
            refused(
                locus,
                "/1/mode/value",
                CheckedPackageRefusalCause::OperationModeTypeMismatch
            ),
            "catalogued but not the pin"
        );
    }

    /// A tuple type keyed on `id_byte` over `members`.
    fn tuple_type_node(id_byte: char, members: &[char]) -> CheckedSemanticNodeV2 {
        let members: Vec<Value> = members.iter().map(|byte| reference_to(*byte)).collect();
        graph_node(
            id_byte,
            "composite_type",
            "tuple",
            &node_id(id_byte),
            json!({ "term": "aggregate", "members": members }),
        )
    }

    /// An alias or bounded-domain node keyed on `id_byte` that continues
    /// through its semantic type `target`.
    fn forwarding_node(
        id_byte: char,
        tag: &str,
        form: &str,
        target: char,
    ) -> CheckedSemanticNodeV2 {
        graph_node(
            id_byte,
            tag,
            form,
            &node_id(target),
            json!({ "term": "aggregate", "members": [] }),
        )
    }

    fn leaves_ineligible(
        locus: CheckedNodeId,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        Ok(Some(refused_at(
            CheckedPackageRefusalCode::IllTyped,
            "/semantic_graph/nodes/0/body/operation/leaves",
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            locus,
        )))
    }

    /// `structural.eq` over parameters `a` and `b` of the type keyed `r`.
    fn eq_over(
        r: char,
        types: Vec<CheckedSemanticNodeV2>,
    ) -> Result<Option<ValidationFailure>, ValidationFailure> {
        let mut graph = types;
        graph.push(parameter_of('a', r));
        graph.push(parameter_of('b', r));
        leaves_defect("quire.op.structural.eq", json!([]), graph, ['a', 'b']).0
    }

    /// The text leaf is found through an alias, a bounded domain, a tuple
    /// position, an option and a collection inside a record, each with
    /// `leaves` empty; the same shapes over integers are admitted.
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[trace("TC-048", "FR-038-AC-43")]
    #[test]
    fn tc_048_leaf_walk_reaches_text_through_every_type_form() {
        let probe = |text_tag: &str, text_form: &str| {
            assert_eq!(text_tag, "scalar_type");
            let leaf = |id_byte: char| scalar_type_node(id_byte, text_form);
            let cases: [(&str, Vec<CheckedSemanticNodeV2>); 5] = [
                (
                    "alias",
                    vec![
                        record_type_node('r', &[("f", 'l')]),
                        forwarding_node('l', "composite_type", "alias", 'x'),
                    ],
                ),
                (
                    "bounded domain",
                    vec![
                        record_type_node('r', &[("f", 'd')]),
                        forwarding_node('d', "bounded_domain", "text_bounds", 'x'),
                    ],
                ),
                ("tuple", vec![tuple_type_node('r', &['i', 'x'])]),
                (
                    "option in a record",
                    vec![
                        record_type_node('r', &[("f", 'o')]),
                        collection_type_node('o', "option", 'x'),
                    ],
                ),
                (
                    "set in a record",
                    vec![
                        record_type_node('r', &[("f", 's')]),
                        collection_type_node('s', "set", 'x'),
                    ],
                ),
            ];
            cases.map(|(name, mut types)| {
                types.push(scalar_type_node('i', "integer"));
                types.push(leaf('x'));
                (name, types)
            })
        };
        for (name, types) in probe("scalar_type", "text") {
            let locus = {
                let (_, locus) =
                    leaves_defect("quire.op.structural.eq", json!([]), vec![], ['r', 'r']);
                locus
            };
            assert_eq!(
                eq_over('r', types),
                leaves_missing(locus),
                "{name} over text"
            );
        }
        for (name, types) in probe("scalar_type", "integer") {
            assert_eq!(eq_over('r', types), Ok(None), "{name} over integer");
        }
    }

    /// `result_inner` expects leaves only for a `set`, `bag` or
    /// `ordered_set` result: `collection.flatten` from a sequence of
    /// sequences of text to a sequence of text admits `[]`, as the
    /// reference reader does, while `collection.set` of text and a flatten
    /// whose result is a set of text are law-missing.
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[trace("TC-048", "FR-038-AC-43")]
    #[test]
    fn tc_048_result_inner_counts_leaves_only_for_set_like_results() {
        let nested = |result_form: &str| {
            vec![
                collection_type_node('q', "sequence", 'x'),
                collection_type_node('n', "sequence", 'q'),
                collection_type_node('R', result_form, 'x'),
                scalar_type_node('x', "text"),
                parameter_of('a', 'n'),
            ]
        };
        let (result, _) = leaves_defect_result(
            "quire.op.collection.flatten",
            json!([]),
            nested("sequence"),
            ['a'],
            Some('R'),
        );
        assert_eq!(result, Ok(None), "sequence result expects no leaves");
        for identity in ["quire.op.collection.flatten", "quire.op.collection.set"] {
            let operands: &[char] = if identity.ends_with("set") {
                &[]
            } else {
                &['a']
            };
            for form in ["set", "bag", "ordered_set"] {
                let (result, locus) = leaves_defect_result(
                    identity,
                    json!([]),
                    nested(form),
                    operands.iter().copied(),
                    Some('R'),
                );
                assert_eq!(
                    result,
                    leaves_missing(locus),
                    "{identity} to a {form} of text"
                );
            }
        }
    }

    /// A cycle through an option or collection alone, with no record or
    /// tuple between the two visits, is `ill_typed`/`operator-ineligible`
    /// (also under a work limit that would be exhausted by following it), a
    /// field whose type is not in the graph is the same refusal, and a type
    /// that reaches itself through a record is admitted (FR-038-AC-70).
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[trace("TC-048", "FR-038-AC-43")]
    #[test]
    fn tc_048_leaf_walk_refuses_a_record_free_cycle_and_an_unresolved_node() {
        let (_, locus) = leaves_defect("quire.op.structural.eq", json!([]), vec![], ['r', 'r']);
        let option_of_itself = vec![
            collection_type_node('o', "option", 'o'),
            scalar_type_node('x', "text"),
        ];
        assert_eq!(
            eq_over('o', option_of_itself),
            leaves_ineligible(locus.clone()),
            "option of itself"
        );
        let through_a_record = vec![
            record_type_node('r', &[("name", 'x'), ("next", 'o')]),
            collection_type_node('o', "option", 's'),
            collection_type_node('s', "sequence", 'o'),
            scalar_type_node('x', "text"),
        ];
        assert_eq!(
            eq_over('r', through_a_record),
            leaves_ineligible(locus.clone()),
            "a record holding a field of a record-free cycle"
        );
        let dangling = vec![record_type_node('r', &[("f", 'm')])];
        assert_eq!(
            eq_over('r', dangling),
            leaves_ineligible(locus),
            "unresolved"
        );
        let record_cycle = vec![
            record_type_node('r', &[("next", 'o')]),
            collection_type_node('o', "option", 'r'),
        ];
        assert_eq!(
            eq_over('r', record_cycle),
            Ok(None),
            "a record that reaches itself and no text admits with `leaves` empty"
        );
    }

    /// Nesting is bounded by the work budget, not a depth cutoff: text under
    /// 40 options is still found, and 2000 options of integer exhaust the
    /// budget instead of being admitted or overflowing the stack.
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[trace("TC-048", "FR-038-AC-43")]
    #[test]
    fn tc_048_leaf_walk_depth_is_bounded_by_the_work_budget() {
        let chain = |depth: u32, leaf: &str| {
            let id = |level: u32| char::from_u32(0x4e00 + level).expect("test id");
            let mut types = vec![scalar_type_node('x', leaf)];
            types.extend((0..depth).map(|level| {
                let inner = if level + 1 == depth {
                    'x'
                } else {
                    id(level + 1)
                };
                collection_type_node(id(level), "option", inner)
            }));
            let outer = if depth == 0 { 'x' } else { id(0) };
            let mut graph = types;
            graph.push(parameter_of('a', outer));
            graph.push(parameter_of('b', outer));
            leaves_defect("quire.op.structural.eq", json!([]), graph, ['a', 'b'])
        };
        let (result, locus) = chain(40, "text");
        assert_eq!(result, leaves_missing(locus), "text under 40 options");
        let (result, _) = chain(40, "integer");
        assert_eq!(result, Ok(None), "integer under 40 options");
        let (result, _) = chain(2000, "integer");
        assert!(
            matches!(&result, Err(failure) if format!("{failure:?}").contains("Work")),
            "2000 nested options must exhaust the work budget, got {result:?}"
        );
    }

    /// The leaf count settles before any leaf's mode: with a leaf whose mode
    /// disagrees with its field's pinned rounding, too few leaves are
    /// `operation-law-missing` and a record-free cycle is
    /// `ill_typed`/`operator-ineligible`, not `operation-mode-type-mismatch`.
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[trace("TC-048", "FR-038-AC-43")]
    #[test]
    fn tc_048_leaf_count_is_settled_before_leaf_modes() {
        let rounded = || {
            graph_node(
                'f',
                "bounded_domain",
                "decimal_range",
                &node_id('d'),
                json!({
                    "term": "aggregate",
                    "members": [{
                        "term": "binding",
                        "name": "rounding",
                        "value": { "term": "literal", "value": "nearest-even" },
                    }],
                }),
            )
        };
        let bad_leaf = json!([{
            "path": ["field:f"],
            "mode": { "kind": "rounding", "value": "toward-zero" },
            "laws": [],
        }]);
        let run = |fields: &[(&str, char)], mut types: Vec<CheckedSemanticNodeV2>| {
            types.push(record_type_node('r', fields));
            types.push(rounded());
            types.push(scalar_type_node('d', "decimal"));
            types.push(scalar_type_node('x', "text"));
            types.push(parameter_of('a', 'r'));
            types.push(parameter_of('b', 'r'));
            leaves_defect(
                "quire.op.structural.eq",
                bad_leaf.clone(),
                types,
                ['a', 'b'],
            )
        };
        let (result, locus) = run(&[("f", 'f'), ("a", 'x'), ("b", 'x')], vec![]);
        assert_eq!(result, leaves_missing(locus), "too few leaves");
        let (result, locus) = run(
            &[("f", 'f'), ("next", 'o')],
            vec![collection_type_node('o', "option", 'o')],
        );
        assert_eq!(result, leaves_ineligible(locus), "record-free cycle");
    }

    /// Records sharing field types are counted once per type node: 12 levels
    /// of 4 fields all naming the next level (4^12 paths) finish inside the
    /// work budget instead of hanging.
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[trace("TC-048", "FR-038-AC-43")]
    #[test]
    fn tc_048_leaf_walk_counts_each_shared_type_once() {
        let id = |level: u32| char::from_u32(0x4e00 + level).expect("test id");
        let mut graph = vec![scalar_type_node('i', "integer")];
        for level in 0..12 {
            let below = if level == 11 { 'i' } else { id(level + 1) };
            graph.push(record_type_node(
                id(level),
                &[("a", below), ("b", below), ("c", below), ("d", below)],
            ));
        }
        assert_eq!(eq_over(id(0), graph), Ok(None));
    }

    /// The expected paths are derived lazily, never listed: 16 levels of 10
    /// fields. When every field names the next level over text there are
    /// 10^16 leaves, so any supplied list is refused by count at once; when
    /// only the first field does and the others are integers there is one
    /// leaf at a 16-segment path, which is compared exactly, and a wrong
    /// segment deep in it is refused.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_leaf_paths_are_derived_lazily_over_a_shared_field_chain() {
        let id = |level: u32| char::from_u32(0x4e00 + level).expect("test id");
        let chain = |shared: bool| {
            let mut graph = vec![
                scalar_type_node('i', "integer"),
                scalar_type_node('x', "text"),
            ];
            let names = ["a", "b", "c", "d", "e", "f", "g", "h", "j", "k"];
            for level in 0..16 {
                let below = if level == 15 { 'x' } else { id(level + 1) };
                let fields: Vec<(&str, char)> = names
                    .iter()
                    .enumerate()
                    .map(|(at, name)| (*name, if shared || at == 0 { below } else { 'i' }))
                    .collect();
                graph.push(record_type_node(id(level), &fields));
            }
            graph.push(parameter_of('a', id(0)));
            graph.push(parameter_of('b', id(0)));
            graph
        };
        let run = |graph, leaves: Value| {
            leaves_defect("quire.op.structural.eq", leaves, graph, ['a', 'b'])
        };

        let (result, locus) = run(chain(true), json!([text_leaf(&["field:a"; 16])]));
        assert_eq!(result, leaves_missing(locus), "10^16 leaves, one supplied");

        let (result, _) = run(chain(false), json!([text_leaf(&["field:a"; 16])]));
        assert_eq!(result, Ok(None), "one leaf at the exact path");

        let mut path = vec!["field:a"; 16];
        path[9] = "field:b";
        let (result, locus) = run(chain(false), json!([text_leaf(&path)]));
        assert_eq!(
            result,
            leaves_mismatch(locus, "/0/path"),
            "wrong deep segment"
        );
    }

    /// Text under 40 options is one leaf at a 40-segment `inner` path.
    ///
    /// Tracing: TC-048, FR-038-AC-44
    #[trace("TC-048", "FR-038-AC-44")]
    #[test]
    fn tc_048_leaf_path_under_nested_options_is_exact() {
        let id = |level: u32| char::from_u32(0x4e00 + level).expect("test id");
        let mut graph = vec![scalar_type_node('x', "text")];
        for level in 0..40 {
            let inner = if level == 39 { 'x' } else { id(level + 1) };
            graph.push(collection_type_node(id(level), "option", inner));
        }
        graph.push(parameter_of('a', id(0)));
        graph.push(parameter_of('b', id(0)));
        let (result, _) = leaves_defect(
            "quire.op.structural.eq",
            json!([text_leaf(&["inner"; 40])]),
            graph.clone(),
            ['a', 'b'],
        );
        assert_eq!(result, Ok(None), "40 `inner` segments");
        let (result, locus) = leaves_defect(
            "quire.op.structural.eq",
            json!([text_leaf(&["inner"; 39])]),
            graph,
            ['a', 'b'],
        );
        assert_eq!(result, leaves_mismatch(locus, "/0/path"), "39 segments");
    }

    /// A text leaf at `path` whose type pins `profile`.
    fn text_leaf_pinning(path: &[&str], profile: &str) -> Value {
        let mut leaf = text_leaf(path);
        leaf["mode"]["value"] = json!(profile);
        leaf
    }

    /// A recursion leaf at `path`: no laws and no mode.
    fn recursion_leaf(path: &[&str]) -> Value {
        json!({ "path": path, "laws": [], "mode": null })
    }

    /// A text type keyed `id_byte` binding `profile` over the `text` scalar
    /// keyed `T` ([`unpinned_text_node`]).
    fn profiled_text_node(id_byte: char, profile: &str) -> CheckedSemanticNodeV2 {
        graph_node(
            id_byte,
            "bounded_domain",
            "text_bounds",
            &node_id('T'),
            json!({
                "term": "aggregate",
                "members": [{
                    "term": "binding",
                    "name": "text_profile",
                    "value": { "term": "literal", "value": profile },
                }],
            }),
        )
    }

    /// `structural.eq` over two parameters of the type keyed `root`, with
    /// `leaves` supplied, against `types`.
    fn eq_leaves(
        root: char,
        mut types: Vec<CheckedSemanticNodeV2>,
        leaves: Value,
    ) -> (
        Result<Option<ValidationFailure>, ValidationFailure>,
        CheckedNodeId,
    ) {
        types.push(parameter_of('a', root));
        types.push(parameter_of('b', root));
        leaves_defect("quire.op.structural.eq", leaves, types, ['a', 'b'])
    }

    /// `record Node { label: Text[nfc]; next?: Node; }`, keyed `N`, its
    /// option `o`, its text `x` and the unpinned scalar `T`.
    fn node_types() -> Vec<CheckedSemanticNodeV2> {
        vec![
            record_type_node('N', &[("label", 'x'), ("next", 'o')]),
            collection_type_node('o', "option", 'N'),
            profiled_text_node('x', "nfc"),
            unpinned_text_node('T'),
        ]
    }

    /// The leaves `Node` derives: its text leaf, then the recursion leaf at
    /// the reentry `next`, `inner`, the composite entered at depth 0.
    fn node_leaves() -> [Value; 2] {
        [
            text_leaf(&["field:label"]),
            recursion_leaf(&["field:next", "inner", "recursion:0"]),
        ]
    }

    /// Equality over a record that reaches itself is admitted with its text
    /// leaf followed by the recursion leaf at the reentry; the text leaf
    /// alone, the recursion leaf alone, and a second text leaf after both are
    /// each refused where the spec places them.
    ///
    /// Tracing: TC-048, FR-038-AC-70
    #[trace("TC-048", "FR-038-AC-70")]
    #[test]
    fn tc_048_equality_over_a_recursive_record_requires_its_recursion_leaf() {
        let [label, recursion] = node_leaves();
        let (result, _) = eq_leaves('N', node_types(), json!([label, recursion]));
        assert_eq!(result, Ok(None), "text leaf and recursion leaf");

        let (result, locus) = eq_leaves('N', node_types(), json!([label]));
        assert_eq!(result, leaves_missing(locus), "text leaf alone");
        let (result, locus) = eq_leaves('N', node_types(), json!([recursion]));
        assert_eq!(result, leaves_missing(locus), "recursion leaf alone");
        let (result, locus) = eq_leaves('N', node_types(), json!([]));
        assert_eq!(result, leaves_missing(locus), "no leaves");

        let (result, locus) = eq_leaves(
            'N',
            node_types(),
            json!([label, recursion, text_leaf(&["field:label"])]),
        );
        assert_eq!(
            result,
            leaves_mismatch(locus, "/2"),
            "a second text leaf has no place left"
        );
    }

    /// The recursion leaf's `d` is the number of path segments the reentered
    /// composite was entered at: `Option<Node>` enters `Node` one segment in.
    ///
    /// Tracing: TC-048, FR-038-AC-70
    #[trace("TC-048", "FR-038-AC-70")]
    #[test]
    fn tc_048_recursion_leaf_depth_counts_the_segments_the_composite_was_entered_at() {
        let mut types = node_types();
        types.push(collection_type_node('p', "option", 'N'));
        let (result, _) = eq_leaves(
            'p',
            types.clone(),
            json!([
                text_leaf(&["inner", "field:label"]),
                recursion_leaf(&["inner", "field:next", "inner", "recursion:1"]),
            ]),
        );
        assert_eq!(result, Ok(None), "Option<Node>");
        let (result, locus) = eq_leaves(
            'p',
            types,
            json!([
                text_leaf(&["inner", "field:label"]),
                recursion_leaf(&["inner", "field:next", "inner", "recursion:0"]),
            ]),
        );
        assert_eq!(
            result,
            leaves_mismatch(locus, "/1/path"),
            "d is 1 here, not 0"
        );
    }

    /// Two records that reach each other in one package: compared at either,
    /// each leaf carries the mode its own text type pins, and the recursion
    /// leaf is at the reentry into the composite compared.
    ///
    /// Tracing: TC-048, FR-038-AC-70
    #[trace("TC-048", "FR-038-AC-70")]
    #[test]
    fn tc_048_mutually_recursive_records_derive_a_recursion_leaf_at_either_end() {
        // A { name: Text[binary-utf8]; b?: B }, B { tag: Text[nfc]; a?: A }.
        let types = || {
            vec![
                record_type_node('P', &[("name", 'y'), ("b", 'u')]),
                collection_type_node('u', "option", 'Q'),
                record_type_node('Q', &[("tag", 'x'), ("a", 'v')]),
                collection_type_node('v', "option", 'P'),
                profiled_text_node('x', "nfc"),
                profiled_text_node('y', "binary-utf8"),
                unpinned_text_node('T'),
            ]
        };
        let (result, _) = eq_leaves(
            'P',
            types(),
            json!([
                text_leaf_pinning(&["field:name"], "binary-utf8"),
                text_leaf_pinning(&["field:b", "inner", "field:tag"], "nfc"),
                recursion_leaf(&["field:b", "inner", "field:a", "inner", "recursion:0"]),
            ]),
        );
        assert_eq!(result, Ok(None), "compared at A");
        let (result, _) = eq_leaves(
            'Q',
            types(),
            json!([
                text_leaf_pinning(&["field:tag"], "nfc"),
                text_leaf_pinning(&["field:a", "inner", "field:name"], "binary-utf8"),
                recursion_leaf(&["field:a", "inner", "field:b", "inner", "recursion:0"]),
            ]),
        );
        assert_eq!(result, Ok(None), "compared at B");
    }

    /// A record that two sibling fields both name is not a cycle: its leaves,
    /// recursion leaf included, are derived under each path, and a list
    /// lacking either recursion leaf is `operation-law-missing`.
    ///
    /// Tracing: TC-048, FR-038-AC-70
    #[trace("TC-048", "FR-038-AC-70")]
    #[test]
    fn tc_048_a_record_named_by_two_fields_derives_its_leaves_under_each() {
        let mut types = node_types();
        types.push(record_type_node('W', &[("x", 'N'), ("y", 'N')]));
        let leaves = [
            text_leaf(&["field:x", "field:label"]),
            recursion_leaf(&["field:x", "field:next", "inner", "recursion:1"]),
            text_leaf(&["field:y", "field:label"]),
            recursion_leaf(&["field:y", "field:next", "inner", "recursion:1"]),
        ];
        let (result, _) = eq_leaves('W', types.clone(), json!(leaves));
        assert_eq!(result, Ok(None), "Two");
        for lacking in [1, 3] {
            let mut without = leaves.to_vec();
            without.remove(lacking);
            let (result, locus) = eq_leaves('W', types.clone(), json!(without));
            assert_eq!(
                result,
                leaves_missing(locus),
                "lacking the recursion leaf at {lacking}"
            );
        }
    }

    /// Recursion through a collection and through a tuple.
    ///
    /// Tracing: TC-048, FR-038-AC-70
    #[trace("TC-048", "FR-038-AC-70")]
    #[test]
    fn tc_048_recursion_through_a_sequence_and_a_tuple_is_admitted() {
        let tree = vec![
            record_type_node('E', &[("label", 'y'), ("kids", 'k')]),
            collection_type_node('k', "sequence", 'E'),
            profiled_text_node('y', "binary-utf8"),
            unpinned_text_node('T'),
        ];
        let (result, _) = eq_leaves(
            'E',
            tree,
            json!([
                text_leaf_pinning(&["field:label"], "binary-utf8"),
                recursion_leaf(&["field:kids", "inner", "recursion:0"]),
            ]),
        );
        assert_eq!(result, Ok(None), "Tree2");

        // Pair = (Text[nfc], Option<Pair>).
        let pair = vec![
            tuple_type_node('Z', &['x', 'o']),
            collection_type_node('o', "option", 'Z'),
            profiled_text_node('x', "nfc"),
            unpinned_text_node('T'),
        ];
        let (result, _) = eq_leaves(
            'Z',
            pair,
            json!([
                text_leaf(&["position:0"]),
                recursion_leaf(&["position:1", "inner", "recursion:0"]),
            ]),
        );
        assert_eq!(result, Ok(None), "Pair");
    }

    /// `Wrap { y: Y; x: X }` over `X { t: Text; n?: Y }` and
    /// `Y { u: Text; x?: X }`: `Y` counts 2 text leaves under `field:y` and 1
    /// under `field:x`, where `X` is open and reachable from it, so a count
    /// memoised for `Y` once is not reused there. Four text leaves and two
    /// recursion leaves.
    ///
    /// Tracing: TC-048, FR-038-AC-70
    #[trace("TC-048", "FR-038-AC-70")]
    #[test]
    fn tc_048_a_memoised_count_is_not_reused_where_a_reachable_composite_is_open() {
        let types = || {
            vec![
                record_type_node('X', &[("t", 'x'), ("n", 'm')]),
                collection_type_node('m', "option", 'Y'),
                record_type_node('Y', &[("u", 'x'), ("x", 'n')]),
                collection_type_node('n', "option", 'X'),
                record_type_node('W', &[("y", 'Y'), ("x", 'X')]),
                profiled_text_node('x', "nfc"),
                unpinned_text_node('T'),
            ]
        };
        let derived = [
            text_leaf(&["field:y", "field:u"]),
            text_leaf(&["field:y", "field:x", "inner", "field:t"]),
            recursion_leaf(&[
                "field:y",
                "field:x",
                "inner",
                "field:n",
                "inner",
                "recursion:1",
            ]),
            text_leaf(&["field:x", "field:t"]),
            text_leaf(&["field:x", "field:n", "inner", "field:u"]),
            recursion_leaf(&[
                "field:x",
                "field:n",
                "inner",
                "field:x",
                "inner",
                "recursion:1",
            ]),
        ];
        let (result, _) = eq_leaves('W', types(), json!(derived));
        assert_eq!(result, Ok(None), "all six");

        let texts: Vec<Value> = [0, 1, 3, 4].iter().map(|at| derived[*at].clone()).collect();
        let (result, locus) = eq_leaves('W', types(), json!(texts));
        assert_eq!(result, leaves_missing(locus), "the four text leaves alone");

        // The derived count settles before any entry is placed: five entries,
        // one of them at a wrong path, are missing a leaf, not mismatched. A
        // count of five (one memo for `Y` regardless of the open set) would
        // place them and refuse the wrong path instead.
        let mut five = derived[..5].to_vec();
        five[1] = text_leaf(&["field:y", "field:x", "inner", "field:q"]);
        let (result, locus) = eq_leaves('W', types(), json!(five));
        assert_eq!(result, leaves_missing(locus), "five of six, one misplaced");

        let mut seven = derived.to_vec();
        seven.push(text_leaf(&["field:x", "field:t"]));
        let (result, locus) = eq_leaves('W', types(), json!(seven));
        assert_eq!(
            result,
            leaves_mismatch(locus, "/6"),
            "a further text leaf after all six"
        );
    }

    /// Two optional text fields name one option node, which is no revisit:
    /// the note on it is dropped when the walk leaves it.
    ///
    /// Tracing: TC-048, FR-038-AC-70
    #[trace("TC-048", "FR-038-AC-70")]
    #[test]
    fn tc_048_one_option_node_serving_two_fields_is_not_a_cycle() {
        let types = vec![
            record_type_node('R', &[("a", 'o'), ("b", 'o')]),
            collection_type_node('o', "option", 'x'),
            profiled_text_node('x', "nfc"),
            unpinned_text_node('T'),
        ];
        let (result, _) = eq_leaves(
            'R',
            types,
            json!([
                text_leaf(&["field:a", "inner"]),
                text_leaf(&["field:b", "inner"]),
            ]),
        );
        assert_eq!(result, Ok(None));

        // The same where no memoised count applies, since the option leads
        // back to the open record: `R { t: Text; a?: R; b?: R }`, one option
        // node for both fields, entered twice in turn.
        let types = vec![
            record_type_node('R', &[("t", 'x'), ("a", 'o'), ("b", 'o')]),
            collection_type_node('o', "option", 'R'),
            profiled_text_node('x', "nfc"),
            unpinned_text_node('T'),
        ];
        let (result, _) = eq_leaves(
            'R',
            types,
            json!([
                text_leaf(&["field:t"]),
                recursion_leaf(&["field:a", "inner", "recursion:0"]),
                recursion_leaf(&["field:b", "inner", "recursion:0"]),
            ]),
        );
        assert_eq!(
            result,
            Ok(None),
            "an option in the cycle serving two fields"
        );
    }

    /// A recursive record that reaches no `text` type expects nothing, not
    /// even a recursion leaf: `structural.eq` and `collection.contains`
    /// admit with `leaves` empty.
    ///
    /// Tracing: TC-048, FR-038-AC-70
    #[trace("TC-048", "FR-038-AC-70")]
    #[test]
    fn tc_048_a_recursive_record_without_text_expects_no_leaves() {
        let list = || {
            vec![
                record_type_node('L', &[("head", 'i'), ("tail", 'o')]),
                collection_type_node('o', "option", 'L'),
                scalar_type_node('i', "integer"),
            ]
        };
        let (result, _) = eq_leaves('L', list(), json!([]));
        assert_eq!(result, Ok(None), "structural.eq");

        let mut set = list();
        set.push(collection_type_node('s', "set", 'L'));
        set.push(parameter_of('a', 's'));
        set.push(parameter_of('b', 'L'));
        let (result, _) = leaves_defect("quire.op.collection.contains", json!([]), set, ['a', 'b']);
        assert_eq!(result, Ok(None), "collection.contains");
    }

    /// A recursion leaf is a leaf of its own place: before the text leaf, at
    /// a wrong path, with a wrong `d`, a second one, one at a reentry of a
    /// type with no text, one carrying a law and one carrying a mode are each
    /// refused where the spec places them.
    ///
    /// Tracing: TC-048, FR-038-AC-71
    #[trace("TC-048", "FR-038-AC-71")]
    #[test]
    fn tc_048_a_recursion_leaf_is_refused_wherever_it_is_not_derived() {
        let [label, recursion] = node_leaves();
        let run = |leaves: Value| eq_leaves('N', node_types(), leaves);
        let refused = |leaves: Value, suffix: &str| {
            let (result, locus) = run(leaves);
            (result, leaves_mismatch(locus, suffix))
        };

        let (result, expected) = refused(json!([recursion, label]), "/0/path");
        assert_eq!(result, expected, "before the text leaf");
        let (result, expected) = refused(
            json!([label, recursion_leaf(&["field:next", "recursion:0"])]),
            "/1/path",
        );
        assert_eq!(result, expected, "a wrong prefix");
        let (result, expected) = refused(
            json!([
                label,
                recursion_leaf(&["field:next", "inner", "recursion:1"])
            ]),
            "/1/path",
        );
        assert_eq!(result, expected, "a wrong d");
        let (result, expected) = refused(json!([label, recursion, recursion]), "/2/path");
        assert_eq!(result, expected, "a second recursion leaf");

        let mut lawful = recursion.clone();
        lawful["laws"] = json!([law_json("text_profile", text_law_definition())]);
        let (result, expected) = refused(json!([label, lawful]), "/1/laws");
        assert_eq!(result, expected, "a recursion leaf with a law");

        let mut moded = recursion.clone();
        moded["mode"] = json!({ "kind": "text_profile", "value": "nfc" });
        let (result, locus) = run(json!([label, moded]));
        assert_eq!(
            result,
            leaves_refused(
                locus,
                CheckedPackageRefusalCode::InvalidPackage,
                "/1/mode",
                CheckedPackageRefusalCause::OperationModeMismatch,
            ),
            "a recursion leaf with a mode"
        );

        let list = vec![
            record_type_node('L', &[("head", 'i'), ("tail", 'o')]),
            collection_type_node('o', "option", 'L'),
            scalar_type_node('i', "integer"),
        ];
        let (result, locus) = eq_leaves(
            'L',
            list,
            json!([recursion_leaf(&["field:tail", "inner", "recursion:0"])]),
        );
        assert_eq!(
            result,
            leaves_mismatch(locus, "/0/path"),
            "a reentry into a composite that reaches no text"
        );
    }

    /// A text leaf inside a recursive record whose type binds no profile is
    /// still ineligible, with the leaves supplied or not; a leaf law the lock
    /// does not select is still `operation-law-unselected`.
    ///
    /// Tracing: TC-048, FR-038-AC-71
    #[trace("TC-048", "FR-038-AC-71")]
    #[test]
    fn tc_048_a_recursive_record_keeps_the_text_leaf_refusals() {
        let unpinned = || {
            vec![
                record_type_node('N', &[("label", 'T'), ("next", 'o')]),
                collection_type_node('o', "option", 'N'),
                unpinned_text_node('T'),
            ]
        };
        for leaves in [json!([]), json!(node_leaves())] {
            let (result, locus) = eq_leaves('N', unpinned(), leaves);
            assert_eq!(result, leaves_ineligible(locus));
        }

        let mut types = node_types();
        types.push(parameter_of('a', 'N'));
        types.push(parameter_of('b', 'N'));
        let (result, locus) = leaves_defect_locked(
            "quire.op.structural.eq",
            json!(node_leaves()),
            types,
            ['a', 'b'],
            None,
            &empty_lock(),
        );
        assert_eq!(
            result,
            leaves_refused(
                locus,
                CheckedPackageRefusalCode::InvalidPackage,
                "/0/laws/0/definition",
                CheckedPackageRefusalCause::OperationLawUnselected,
            )
        );
    }

    /// The work `operation_defect` consumed to admit `structural.eq` over
    /// parameters of the type keyed `root`, with `leaves` supplied.
    fn eq_work(root: char, mut types: Vec<CheckedSemanticNodeV2>, leaves: Value) -> u64 {
        types.push(parameter_of('a', root));
        types.push(parameter_of('b', root));
        let mut operation = plain_operation("quire.op.structural.eq");
        operation["leaves"] = leaves;
        let application = custom_application_node(
            "binary",
            operation,
            ['a', 'b'].into_iter().map(reference_to).collect(),
        );
        let mut nodes = vec![application];
        nodes.append(&mut types);
        let mut index: BTreeMap<&CheckedNodeId, usize> = BTreeMap::new();
        for (position, node) in nodes.iter().enumerate() {
            index.insert(&node.node_id, position);
        }
        let mut meter = WorkMeter::new(1_000_000);
        let kinds = kinds_of(&nodes);
        let result = operation_defect(
            Application {
                node: &nodes[0],
                position: 0,
            },
            &Graph {
                nodes: &nodes,
                kinds: &kinds,
                index: &index,
            },
            &text_selecting_lock(),
            &ModelOwners::default(),
            DependencyReferences::new(&SuppliedDependencies::default()),
            operation_catalog(),
            &mut meter,
        );
        assert_eq!(result, Ok(None), "the comparison admits");
        meter.consumed()
    }

    /// Every edge entered is one unit of work, a reentry edge included: a
    /// record that holds one more optional field of itself costs the edge
    /// into the option and the edge from it back to the record, once in the
    /// count and once in the pass, and the budget is what bounds a cycle.
    ///
    /// Tracing: TC-048, FR-038-AC-72
    #[trace("TC-048", "FR-038-AC-72")]
    #[test]
    fn tc_048_each_reentry_edge_is_one_unit_of_work() {
        let work = |optionals: usize| {
            let names: Vec<String> = (0..optionals).map(|at| format!("f{at}")).collect();
            let mut fields = vec![("t", 'x')];
            fields.extend(names.iter().map(|name| (name.as_str(), 'o')));
            let types = vec![
                record_type_node('R', &fields),
                collection_type_node('o', "option", 'R'),
                profiled_text_node('x', "nfc"),
                unpinned_text_node('T'),
            ];
            let mut leaves = vec![text_leaf(&["field:t"])];
            leaves.extend(names.iter().map(|name| {
                let field = format!("field:{name}");
                recursion_leaf(&[field.as_str(), "inner", "recursion:0"])
            }));
            eq_work('R', types, json!(leaves))
        };
        // Each added field is four units in the walks (two edges, in the
        // count and again in the pass) and one for the recursion leaf it
        // adds to the operation's own charge per supplied leaf.
        let (one, two, three) = (work(1), work(2), work(3));
        assert_eq!(two - one, 5, "one more optional field of the record");
        assert_eq!(three - two, 5, "and one more");
    }

    /// The component search that decides where a memoised count applies is
    /// charged one unit per reachable type node: an integer field added to a
    /// record costs the node in the search, the edge in the count and the
    /// edge in the pass, three units, where two would mean the search is free.
    ///
    /// Tracing: TC-048, FR-038-AC-72
    #[trace("TC-048", "FR-038-AC-72")]
    #[test]
    fn tc_048_the_component_search_costs_one_unit_per_reachable_type_node() {
        let work = |with_integer: bool| {
            let mut fields = vec![("t", 'x')];
            let mut types = vec![
                profiled_text_node('x', "nfc"),
                unpinned_text_node('T'),
                scalar_type_node('i', "integer"),
            ];
            if with_integer {
                fields.push(("n", 'i'));
            }
            types.push(record_type_node('R', &fields));
            eq_work('R', types, json!([text_leaf(&["field:t"])]))
        };
        assert_eq!(work(true) - work(false), 3);
    }

    /// Only a cycle through a record or tuple is admitted: `T` as an
    /// `Option` of itself, a `Sequence` of itself and a record holding a
    /// field of such a type are `ill_typed`/`operator-ineligible` at a work
    /// limit of 1000, never `incomplete` for `work`, while `R { x: Option<R> }`
    /// admits.
    ///
    /// Tracing: TC-048, FR-038-AC-71
    #[trace("TC-048", "FR-038-AC-71")]
    #[test]
    fn tc_048_a_cycle_through_no_record_or_tuple_is_refused_not_exhausted() {
        let run = |root: char, types: Vec<CheckedSemanticNodeV2>| {
            let mut graph = types;
            graph.push(parameter_of('a', root));
            graph.push(parameter_of('b', root));
            leaves_defect_metered(
                "quire.op.structural.eq",
                json!([]),
                graph,
                ['a', 'b'],
                None,
                &text_selecting_lock(),
                1000,
            )
        };
        let (result, locus) = run('T', vec![collection_type_node('T', "option", 'T')]);
        assert_eq!(result, leaves_ineligible(locus), "Option of itself");
        let (result, locus) = run('T', vec![collection_type_node('T', "sequence", 'T')]);
        assert_eq!(result, leaves_ineligible(locus), "Sequence of itself");
        let (result, locus) = run(
            'R',
            vec![
                record_type_node('R', &[("x", 'T')]),
                collection_type_node('T', "option", 'T'),
            ],
        );
        assert_eq!(result, leaves_ineligible(locus), "a record holding one");
        let (result, _) = run(
            'R',
            vec![
                record_type_node('R', &[("x", 'o')]),
                collection_type_node('o', "option", 'R'),
            ],
        );
        assert_eq!(result, Ok(None), "R {{ x: Option<R> }}");
    }

    /// `same_type` compares the type node each operand resolves to, not the
    /// operands' own nodes: [`STRUCTURAL_EQ_IDENTITY`] over two distinct
    /// parameters of one record type is admitted, and over parameters of two
    /// different record types is refused at the second operand.
    ///
    /// Tracing: TC-048, FR-038-AC-86
    #[trace("TC-048", "FR-038-AC-86")]
    #[test]
    fn operation_defect_same_type_compares_operand_types_not_operand_nodes() {
        let record = |id_byte: char, field: &str| {
            graph_node(
                id_byte,
                "composite_type",
                "record",
                &node_id(id_byte),
                json!({
                    "term": "aggregate",
                    "members": [{
                        "term": "binding",
                        "name": field,
                        "value": {
                            "term": "reference",
                            "target": { "domain": NODE_DOMAIN, "digest": dummy_digest('i') },
                        },
                    }],
                }),
            )
        };
        let parameter = |id_byte: char, type_byte: char| {
            graph_node(
                id_byte,
                "value",
                "parameter",
                &node_id(type_byte),
                json!({ "term": "aggregate", "members": [] }),
            )
        };
        let operand = |id_byte: char| {
            json!({
                "term": "reference",
                "target": { "domain": NODE_DOMAIN, "digest": dummy_digest(id_byte) },
            })
        };
        let operation = plain_operation(STRUCTURAL_EQ_IDENTITY);
        let defect = |second: char| {
            let root = custom_application_node(
                "binary",
                operation.clone(),
                vec![operand('a'), operand(second)],
            );
            let graph = vec![
                root,
                record('r', "name"),
                record('s', "label"),
                scalar_type_node('i', "integer"),
                parameter('a', 'r'),
                parameter('b', 'r'),
                parameter('c', 's'),
            ];
            let locus = graph[0].node_id.clone();
            (defect_for_graph(graph), locus)
        };

        let (same, _) = defect('b');
        assert_eq!(
            same,
            Ok(None),
            "distinct parameters of one record type must be admitted, got {same:?}"
        );
        let (different, locus) = defect('c');
        assert_eq!(
            different,
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::IllTyped,
                "/semantic_graph/nodes/0/body/arguments/1",
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                locus,
            ))),
            "parameters of different record types must be refused at the second operand"
        );
    }

    /// `same_type` over terms that are not references: a `literal` resolves
    /// to its declared `type` and a nested `application` to its
    /// `result_type`, so two of either over one type node are admitted and
    /// over two different type nodes are refused at the second operand; an
    /// operand that resolves to no type leaves the constraint undecided, so
    /// it is admitted rather than guessed at.
    ///
    /// Tracing: TC-048, FR-038-AC-86
    #[trace("TC-048", "FR-038-AC-86")]
    #[test]
    fn operation_defect_same_type_resolves_literal_and_application_operands() {
        let record_type = |id_byte: char| {
            graph_node(
                id_byte,
                "composite_type",
                "record",
                &node_id(id_byte),
                json!({ "term": "aggregate", "members": [] }),
            )
        };
        let literal = |type_byte: char| {
            json!({
                "term": "literal",
                "type": { "domain": NODE_DOMAIN, "digest": dummy_digest(type_byte) },
                "value_kind": "record",
                "value": "x",
            })
        };
        let nested = |type_byte: char| {
            json!({
                "term": "application",
                "operator": "binary",
                "operation": plain_operation(CATALOGUED_IDENTITY),
                "result_type": { "domain": NODE_DOMAIN, "digest": dummy_digest(type_byte) },
                "arguments": [],
            })
        };
        let operation = plain_operation(STRUCTURAL_EQ_IDENTITY);
        let defect = |first: Value, second: Value| {
            let root = custom_application_node("binary", operation.clone(), vec![first, second]);
            let locus = root.node_id.clone();
            (
                defect_for_graph(vec![root, record_type('p'), record_type('q')]),
                locus,
            )
        };
        let refusal = |locus| {
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::IllTyped,
                "/semantic_graph/nodes/0/body/arguments/1",
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                locus,
            )))
        };

        assert_eq!(defect(literal('p'), literal('p')).0, Ok(None));
        let (result, locus) = defect(literal('p'), literal('q'));
        assert_eq!(result, refusal(locus), "literals of two types");
        assert_eq!(defect(nested('p'), nested('p')).0, Ok(None));
        let (result, locus) = defect(nested('p'), nested('q'));
        assert_eq!(result, refusal(locus), "applications of two result types");
        let untyped = json!({ "term": "literal", "value": 0 });
        assert_eq!(
            defect(literal('p'), untyped).0,
            Ok(None),
            "an operand that resolves to no type is not decided"
        );
    }

    /// Literal and nested-application operands take part in the family and
    /// mode-pin checks: a literal typed as another family than its operand
    /// position is refused at that argument, a literal whose type pins a
    /// different `rounding` than the operation's mode is refused as
    /// `operation-mode-type-mismatch`, and a clause application has family
    /// `clause`, so it fits no `boolean` position.
    ///
    /// Tracing: TC-048, FR-038-AC-83, FR-038-AC-85
    #[trace("TC-048", "FR-038-AC-83", "FR-038-AC-85")]
    #[test]
    fn operation_defect_checks_literal_and_application_operands() {
        let typed_literal = |type_byte: char| {
            json!({
                "term": "literal",
                "type": { "domain": NODE_DOMAIN, "digest": dummy_digest(type_byte) },
                "value": 1,
            })
        };
        let scalar_node = |id_byte: char, form: &str| {
            graph_node(
                id_byte,
                "scalar_type",
                form,
                &node_id(id_byte),
                json!({ "term": "aggregate", "members": [] }),
            )
        };

        // Wrong family: `integer.add` over a Boolean literal.
        let add = |first: Value| {
            custom_application_node(
                "binary",
                plain_operation(CATALOGUED_IDENTITY),
                vec![first, typed_literal('i')],
            )
        };
        let graph = |root: CheckedSemanticNodeV2| {
            vec![
                root,
                scalar_node('i', "integer"),
                scalar_node('b', "boolean"),
                scalar_node('5', "decimal"),
            ]
        };
        assert_eq!(defect_for_graph(graph(add(typed_literal('i')))), Ok(None));
        let root = add(typed_literal('b'));
        let locus = root.node_id.clone();
        assert_eq!(
            defect_for_graph(graph(root)),
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::IllTyped,
                "/semantic_graph/nodes/0/body/arguments/0",
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                locus,
            ))),
            "a Boolean literal at an integer position"
        );

        // Mode pin: a decimal literal whose type pins `nearest-even` under
        // a `toward-zero` operation mode.
        let pinned = graph_node(
            '4',
            "bounded_domain",
            "decimal_range",
            &node_id('5'),
            json!({
                "term": "aggregate",
                "members": [{
                    "term": "binding",
                    "name": "rounding",
                    "value": { "term": "literal", "value": "nearest-even" },
                }],
            }),
        );
        let mut operation = plain_operation(DECIMAL_ADD_IDENTITY);
        operation["mode"] = json!({ "kind": "rounding", "value": "toward-zero" });
        let root = custom_application_node(
            "binary",
            operation,
            vec![typed_literal('4'), json!({ "term": "literal", "value": 0 })],
        );
        let locus = root.node_id.clone();
        assert_eq!(
            defect_for_graph(vec![root, pinned, scalar_node('5', "decimal")]),
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/mode/value",
                Some(CheckedPackageRefusalCause::OperationModeTypeMismatch),
                locus,
            ))),
            "a literal's own type pins the rounding mode"
        );

        // A clause application is `clause`, not its Boolean `result_type`.
        let clause = json!({
            "term": "application",
            "operator": "state_clause",
            "operation": plain_operation("quire.op.state.clause"),
            "result_type": { "domain": NODE_DOMAIN, "digest": dummy_digest('b') },
            "arguments": [],
        });
        let root = custom_application_node(
            "unary",
            plain_operation("quire.op.boolean.not"),
            vec![clause],
        );
        let locus = root.node_id.clone();
        assert_eq!(
            defect_for_graph(graph(root)),
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::IllTyped,
                "/semantic_graph/nodes/0/body/arguments/0",
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                locus,
            ))),
            "a clause application at a boolean position"
        );
    }

    /// `stale-node-key`: agent-ix/quire-contract-ir#171. `application_node`'s
    /// `node_id.digest` is a placeholder, never the JCS SHA-256 of its own
    /// preimage — exactly the condition [`validate_application_keys`] exists
    /// to catch. Unlike every other test in this module, this one calls
    /// `validate_application_keys` directly rather than `operation_defect`:
    /// the two are separate stages (see the module doc), and nothing above
    /// reaches this one.
    ///
    /// Trace: TC-048, FR-038-AC-88, FR-038-AC-184
    #[trace("TC-048", "FR-038-AC-88", "FR-038-AC-184")]
    #[test]
    fn tc_048_validate_application_keys_refuses_a_stale_node_key() {
        let node = application_node(CATALOGUED_IDENTITY, "binary");
        let nodes = std::slice::from_ref(&node);
        let mut index: BTreeMap<&CheckedNodeId, usize> = BTreeMap::new();
        index.insert(&node.node_id, 0);
        let mut meter = WorkMeter::new(1_000);

        let result = validate_application_keys(nodes, &index, &mut meter, 1 << 20);

        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("a placeholder node key must refuse: {result:?}");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
        assert_eq!(
            refusal.path,
            Some(pointer("/semantic_graph/nodes/0/node_id"))
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::StaleNodeKey)
        );
        assert_eq!(refusal.locus, Some(node.node_id.clone()));
        assert_eq!(refusal.contract_version, None);
        assert_eq!(refusal.document_pointer, None);
        let expected = refusal.expected_node_id().cloned().expect("computed key");
        assert_eq!(expected.domain.as_ref(), NODE_DOMAIN);
        assert_ne!(expected, node.node_id);

        // The returned key is checked by the same production stage. The
        // fixture does not recompute an expected digest in test code.
        let mut rekeyed = node;
        rekeyed.node_id = expected;
        let mut index = BTreeMap::new();
        index.insert(&rekeyed.node_id, 0);
        let mut meter = WorkMeter::new(1_000);
        assert_eq!(
            validate_application_keys(std::slice::from_ref(&rekeyed), &index, &mut meter, 1 << 20),
            Ok(())
        );
    }

    /// Control for the test above: a node whose `node_id.digest` genuinely
    /// is its own preimage's digest ([`correctly_keyed`]) is admitted, not
    /// refused. Without this control, a `validate_application_keys` that
    /// refused every node would satisfy the assertion above just as well as
    /// the real re-derivation does.
    ///
    /// Tracing: TC-048, FR-038-AC-88
    #[trace("TC-048", "FR-038-AC-88")]
    #[test]
    fn tc_048_validate_application_keys_admits_a_correctly_keyed_node() {
        let node = correctly_keyed(application_node(CATALOGUED_IDENTITY, "binary"));
        let nodes = std::slice::from_ref(&node);
        let mut index: BTreeMap<&CheckedNodeId, usize> = BTreeMap::new();
        index.insert(&node.node_id, 0);
        let mut meter = WorkMeter::new(1_000);

        let result = validate_application_keys(nodes, &index, &mut meter, 1 << 20);

        assert_eq!(
            result,
            Ok(()),
            "a node_id that genuinely is the JCS SHA-256 of the node's own preimage must be \
             admitted, got {result:?}"
        );
    }

    /// `invalid_semantic_graph` (malformed wire): agent-ix/quire-contract-ir#171.
    /// `operation` must deserialize as `OperationWire`; a bare string is not
    /// one, so this must be refused before any catalog lookup runs.
    ///
    /// Tracing: TC-048, FR-038-AC-81
    #[trace("TC-048", "FR-038-AC-81")]
    #[test]
    fn operation_defect_refuses_malformed_operation_wire() {
        let node = custom_application_node("binary", json!("not-an-operation-object"), Vec::new());

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(Some(ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                pointer("/semantic_graph/nodes/0/body/operation"),
            ))),
            "an operation member that does not deserialize as OperationWire must be refused \
             as invalid_semantic_graph, got {result:?}"
        );
    }

    // ---- FR-038-AC-67 through AC-69, AC-96: the catalog words the reader admits.

    const OPERATOR_PATH: &str = "/semantic_graph/nodes/0/body/operator";

    /// Every catalogued entry of the three operator classes the reader once
    /// refused, read from the catalog: fifteen `temporal_formula` identities,
    /// `quire.op.temporal.fair` and `quire.op.control.case`, each with the
    /// member kind it catalogues and its fixed operand count.
    fn temporal_and_case_entries() -> Vec<(String, &'static str, Option<&'static str>, usize)> {
        let mut entries: Vec<_> = operation_catalog()
            .entries()
            .filter(|entry| {
                matches!(
                    entry.operator,
                    ApplicationOperator::Case
                        | ApplicationOperator::TemporalFormula
                        | ApplicationOperator::TemporalFairness
                )
            })
            .map(|entry| {
                (
                    entry.identity.to_string(),
                    entry.operator.as_wire(),
                    entry.member.map(|member| member.as_wire()),
                    entry.operands.len(),
                )
            })
            .collect();
        entries.sort();
        entries
    }

    /// The member a catalogued temporal entry takes, in its closed shape.
    fn catalogued_member(kind: &str) -> Value {
        match kind {
            "temporal_interval" => json!({
                "kind": "temporal_interval",
                "interval": {"lower": "0", "upper": "3"},
            }),
            _ => json!({
                "kind": "fairness",
                "fairness_kind": "weak",
                "granularity": "whole",
                "declaration": { "domain": NODE_DOMAIN, "digest": dummy_digest('f') },
                "name": "Step",
            }),
        }
    }

    /// FR-038-AC-96 (the operation step of the node): each temporal entry
    /// admits with no law, no mode, no leaves, its catalogued member and its
    /// catalogued operand count, and refuses `operation-law-mismatch`,
    /// `operation-mode-mismatch` and `operation-member-mismatch` where one of
    /// them disagrees with the entry, so no refusal is a refusal of every
    /// application. `quire.op.control.case` is held to the same rules; its
    /// `union_arms` constraint needs a union and is checked through a package.
    ///
    /// Tracing: TC-048, FR-038-AC-96
    #[trace("TC-048", "FR-038-AC-96")]
    #[test]
    fn tc_048_the_temporal_entries_admit_by_their_catalogued_shape() {
        let entries = temporal_and_case_entries();
        let temporal = entries
            .iter()
            .filter(|(_, operator, _, _)| *operator == "temporal_formula")
            .count();
        assert_eq!(
            (entries.len(), temporal),
            (17, 15),
            "the catalog carries fifteen formula identities, fair and case"
        );
        let operands = |count: usize| vec![json!({ "term": "literal", "value": 1 }); count];
        for (identity, operator, member, operand_count) in entries {
            let node_with = |edit: &dyn Fn(&mut Value)| {
                let mut operation = plain_operation(&identity);
                if let Some(kind) = member {
                    operation["member"] = catalogued_member(kind);
                }
                edit(&mut operation);
                custom_application_node(operator, operation, operands(operand_count))
            };
            if identity != "quire.op.control.case" {
                assert_eq!(defect_for(&node_with(&|_| {})), Ok(None), "{identity}");
            }
            let law = node_with(&|operation| {
                operation["laws"] = json!([law_json("text_profile", dummy_law_definition('a'))]);
            });
            assert_eq!(
                defect_for(&law),
                Ok(Some(refused_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    "/semantic_graph/nodes/0/body/operation/laws/0",
                    Some(CheckedPackageRefusalCause::OperationLawMismatch),
                    law.node_id.clone(),
                ))),
                "{identity} law"
            );
            let mode = node_with(&|operation| {
                operation["mode"] = json!({ "kind": "rounding", "value": "nearest-even" });
            });
            assert_eq!(
                defect_for(&mode),
                Ok(Some(refused_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    "/semantic_graph/nodes/0/body/operation/mode",
                    Some(CheckedPackageRefusalCause::OperationModeMismatch),
                    mode.node_id.clone(),
                ))),
                "{identity} mode"
            );
            let leaf = node_with(&|operation| {
                operation["leaves"] = json!([{
                    "path": ["field:x"],
                    "laws": [law_json("text_profile", dummy_law_definition('b'))],
                    "mode": { "kind": "rounding", "value": "nearest-even" },
                }]);
            });
            assert_eq!(
                defect_for(&leaf),
                Ok(Some(refused_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    "/semantic_graph/nodes/0/body/operation/leaves/0",
                    Some(CheckedPackageRefusalCause::OperationLawMismatch),
                    leaf.node_id.clone(),
                ))),
                "{identity} leaf"
            );
            let stray_member = node_with(&|operation| {
                operation["member"] = json!({ "kind": "state_clause" });
            });
            assert_eq!(
                defect_for(&stray_member),
                Ok(Some(refused_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    "/semantic_graph/nodes/0/body/operation/member",
                    Some(CheckedPackageRefusalCause::OperationMemberMismatch),
                    stray_member.node_id.clone(),
                ))),
                "{identity} member"
            );
        }
    }

    /// FR-038-AC-67: an unknown identity and a class mismatch refuse ahead of
    /// every other check of the entry, and of two defective root nodes the
    /// lower `node_id` digest is reported.
    ///
    /// Tracing: TC-048, FR-038-AC-67
    #[trace("TC-048", "FR-038-AC-67")]
    #[test]
    fn tc_048_identity_and_class_refuse_first_and_the_lowest_node_is_reported() {
        let case_under_unary = custom_application_node(
            "unary",
            plain_operation("quire.op.control.case"),
            Vec::new(),
        );
        assert_eq!(
            defect_for(&case_under_unary),
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                OPERATOR_PATH,
                Some(CheckedPackageRefusalCause::OperationClassMismatch),
                case_under_unary.node_id.clone(),
            )))
        );
        let unknown = custom_application_node(
            "case",
            plain_operation("quire.op.control.not-catalogued"),
            Vec::new(),
        );
        assert_eq!(
            defect_for(&unknown),
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/identity",
                Some(CheckedPackageRefusalCause::UnknownOperation),
                unknown.node_id.clone(),
            )))
        );

        // Two defective roots, in both digest orders: the lower digest wins,
        // whichever of the two operation defects it carries.
        let unknown_identity = |digest: char| {
            // Under `unary`: a `case` root in an `expression`/`call` node is
            // the form contradiction, refused ahead of identities.
            let mut node = custom_application_node(
                "unary",
                plain_operation("quire.op.control.not-catalogued"),
                vec![],
            );
            node.node_id = node_id(digest);
            node
        };
        let class_mismatch = |digest: char| {
            let mut node = custom_application_node(
                "unary",
                plain_operation("quire.op.control.case"),
                Vec::new(),
            );
            node.node_id = node_id(digest);
            node
        };
        for (low_is_unknown_identity, expected_cause) in [
            (true, CheckedPackageRefusalCause::UnknownOperation),
            (false, CheckedPackageRefusalCause::OperationClassMismatch),
        ] {
            let (low, high) = if low_is_unknown_identity {
                (unknown_identity('1'), class_mismatch('2'))
            } else {
                (class_mismatch('1'), unknown_identity('2'))
            };
            let nodes = vec![high.clone(), low.clone()];
            let kinds = kinds_of(&nodes);
            let index: BTreeMap<&CheckedNodeId, usize> = nodes
                .iter()
                .enumerate()
                .map(|(position, node)| (&node.node_id, position))
                .collect();
            let result = super::validate_operations(
                &nodes,
                &kinds,
                &index,
                &empty_lock(),
                &ModelOwners::default(),
                &SuppliedDependencies::default(),
                &mut WorkMeter::new(1_000),
            );
            let Err(ValidationFailure::Refused(refusal)) = result else {
                panic!("a defective graph refuses, got {result:?}");
            };
            assert_eq!(refusal.locus.as_ref(), Some(&low.node_id));
            assert_eq!(refusal.cause, Some(expected_cause));
        }
    }

    /// The `temporal.clause` graph: node 0 is the clause application over its
    /// `arguments`; nodes 1.. are the nodes those arguments name.
    fn clause_nodes(arguments: Vec<Value>, member: Option<Value>) -> Vec<CheckedSemanticNodeV2> {
        let mut operation = plain_operation("quire.op.temporal.clause");
        operation["laws"] = json!([law_json("temporal_profile", dummy_law_definition('p'))]);
        if let Some(member) = member {
            operation["member"] = member;
        }
        let boolean_type = node_id('b');
        vec![
            custom_application_node("temporal", operation, arguments),
            graph_node(
                '1',
                "value",
                "parameter",
                &node_id('2'),
                json!({"term": "aggregate", "members": []}),
            ),
            graph_node(
                '2',
                "composite_type",
                "record",
                &node_id('2'),
                json!({"term": "aggregate", "members": []}),
            ),
            graph_node(
                '3',
                "scalar_type",
                "text",
                &node_id('3'),
                json!({"term": "aggregate", "members": []}),
            ),
            graph_node(
                '4',
                "temporal",
                "formula",
                &boolean_type,
                json!({"term": "aggregate", "members": []}),
            ),
            graph_node(
                'b',
                "scalar_type",
                "boolean",
                &boolean_type,
                json!({"term": "aggregate", "members": []}),
            ),
        ]
    }

    fn node_ref_of(digest: char) -> Value {
        json!({ "domain": NODE_DOMAIN, "digest": dummy_digest(digest) })
    }

    fn clause_arguments(first: Value, sixth: Value) -> Vec<Value> {
        let aggregate = json!({ "term": "aggregate", "members": [] });
        vec![
            first,
            json!({ "term": "literal", "type": node_ref_of('3'),
                    "value_kind": "text", "value": "clause" }),
            aggregate.clone(),
            aggregate.clone(),
            aggregate,
            sixth,
        ]
    }

    fn clause_defect(
        arguments: Vec<Value>,
        member: Option<Value>,
    ) -> (
        CheckedSemanticNodeV2,
        Result<Option<ValidationFailure>, ValidationFailure>,
    ) {
        let nodes = clause_nodes(arguments, member);
        let clause = nodes[0].clone();
        let mut lock = empty_lock();
        let definition: CheckedArtifactRef =
            serde_json::from_value(dummy_law_definition('p')).expect("definition");
        lock.profile_selections = vec![CheckedSelection {
            role: LawRole::TemporalProfile
                .selection_role()
                .expect("a profile role"),
            definition,
        }];
        (clause, defect_for_graph_locked(nodes, &lock))
    }

    /// FR-038-AC-68: the six fixed operands of `quire.op.temporal.clause`.
    ///
    /// Tracing: TC-048, FR-038-AC-68
    #[trace("TC-048", "FR-038-AC-68")]
    #[test]
    fn tc_048_the_temporal_clause_is_checked_as_six_fixed_operands() {
        let good = || clause_arguments(reference_to('1'), reference_to('4'));
        let ineligible = |node: &CheckedSemanticNodeV2, path: &str| {
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::IllTyped,
                path,
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                node.node_id.clone(),
            )))
        };
        let body = "/semantic_graph/nodes/0/body";

        // The one selected law, no member, six arguments: passes, whatever
        // type the parameter has (here a record).
        assert_eq!(clause_defect(good(), None).1, Ok(None));

        // Five and seven arguments.
        let mut five = good();
        five.pop();
        let (node, result) = clause_defect(five, None);
        assert_eq!(result, ineligible(&node, &format!("{body}/arguments")));
        let mut seven = good();
        seven.push(json!({ "term": "aggregate", "members": [] }));
        let (node, result) = clause_defect(seven, None);
        assert_eq!(result, ineligible(&node, &format!("{body}/arguments")));

        // A first argument that is not a `reference` term, and a sixth that
        // references a Boolean node.
        let text_first = clause_arguments(
            json!({ "term": "literal", "type": node_ref_of('3'),
                    "value_kind": "text", "value": "x" }),
            reference_to('4'),
        );
        let (node, result) = clause_defect(text_first, None);
        assert_eq!(result, ineligible(&node, &format!("{body}/arguments/0")));
        let boolean_sixth = clause_arguments(reference_to('1'), reference_to('b'));
        let (node, result) = clause_defect(boolean_sixth, None);
        assert_eq!(result, ineligible(&node, &format!("{body}/arguments/5")));

        // A member of any kind refuses `operation-member-mismatch`.
        for kind in ["profile_operator", "fairness", "temporal_interval"] {
            let (node, result) = clause_defect(good(), Some(json!({ "kind": kind })));
            assert_eq!(
                result,
                Ok(Some(refused_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    &format!("{body}/operation/member"),
                    Some(CheckedPackageRefusalCause::OperationMemberMismatch),
                    node.node_id.clone(),
                ))),
                "{kind}"
            );
        }

        // A `reference` to a formula node resolves to `temporal`: it fits
        // that operand and an `any_term` position, and no `boolean` or
        // `any_value` one.
        let nodes = clause_nodes(good(), None);
        let kinds = kinds_of(&nodes);
        let index: BTreeMap<&CheckedNodeId, usize> = nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        let catalog = operation_catalog();
        let family = super::argument_family(&reference_to('4'), &nodes, &kinds, &index, catalog);
        assert_eq!(family, Some("temporal"));
        assert!(catalog.family_fits("temporal", "temporal"));
        assert!(catalog.family_fits("temporal", "any_term"));
        assert!(!catalog.family_fits("temporal", "boolean"));
        assert!(!catalog.family_fits("temporal", "any_value"));
    }

    /// FR-038-AC-69: a `temporal_interval` or `fairness` member on an entry
    /// that catalogues none, `quire.op.boolean.not` or a `temporal_formula`
    /// identity such as `holds`, is a member mismatch at the member, and
    /// FR-038-AC-98's `fairness` member on `quire.op.boolean.not`.
    ///
    /// Tracing: TC-048, FR-038-AC-69, FR-038-AC-98
    #[trace("TC-048", "FR-038-AC-69", "FR-038-AC-98")]
    #[test]
    fn tc_048_a_temporal_member_on_another_entry_is_a_member_mismatch() {
        for (identity, operator, kind) in [
            ("quire.op.boolean.not", "unary", "temporal_interval"),
            ("quire.op.boolean.not", "unary", "fairness"),
        ] {
            let mut operation = plain_operation(identity);
            operation["member"] = json!({ "kind": kind });
            let node = custom_application_node(
                operator,
                operation,
                vec![json!({ "term": "literal", "value": true })],
            );
            assert_eq!(
                defect_for(&node),
                Ok(Some(refused_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    "/semantic_graph/nodes/0/body/operation/member",
                    Some(CheckedPackageRefusalCause::OperationMemberMismatch),
                    node.node_id.clone(),
                ))),
                "{identity} {kind}"
            );
        }
        let mut operation = plain_operation("quire.op.temporal.holds");
        operation["member"] = json!({ "kind": "fairness" });
        let node = custom_application_node(
            "temporal_formula",
            operation,
            vec![json!({ "term": "literal", "value": true })],
        );
        assert_eq!(
            defect_for(&node),
            Ok(Some(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/body/operation/member",
                Some(CheckedPackageRefusalCause::OperationMemberMismatch),
                node.node_id.clone(),
            )))
        );
    }
}
