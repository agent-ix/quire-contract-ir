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
//! including that the named field is actually declared); a `constraints`
//! entry other than `same_family`/`same_type`/`conforming_reference`/
//! `reference_edge` is not enforced; leaf-path
//! resolution covers exactly one shape, `["field:<name>"]` against the first
//! operand's record type, the one the upstream vectors exercise;
//! [`validate_application_keys`] re-derives a key only for a node whose own
//! `body` is an application term at its root, never for a nested
//! `application` term inside `body.arguments[*]` — the upstream description
//! instead says
//! every node whose body *contains* an application gets a re-derived key, so
//! a nested application's own key is never checked here, and its own
//! `operation` is never validated by [`validate_operations`] either, since
//! that stage only visits root-bodied application nodes too; `operation.mode`
//! is checked for `kind` only — its `value` is never checked against the
//! catalog's closed `modes` vocabulary; a catalogued entry's `result` is
//! never checked against the catalog's `result_forms`; `argument_family`
//! resolves only `reference` and `binding` argument terms, so a `literal`,
//! `aggregate` or nested `application` argument resolves to no family and
//! silently bypasses every operand-family check ([`check_operands`],
//! [`check_mode_type`], [`check_leaf_count`]) that consults it.

use super::dependency_references::{DependencyReferences, Referrer, SuppliedDependencies};
use super::model_members::{
    declaration_key, Budget, CollectionKind, DeclarationForm, MemberKind, MemberType, ModelFailure,
    ModelOwners, ModelRefusal, Resolved,
};
use super::operation_catalog::{operation_catalog, OperationCatalog, OperationCatalogEntry};
use super::{
    ApplicationOperator, BodyTerm, BoundedDomainForm, CheckedArtifactRef, CheckedNodeId,
    CheckedNodeKind, CheckedNodeTag, CheckedPackageLockV2, CheckedSemanticNodeV2, ClaimForm,
    CompositeTypeForm, CorrespondenceForm, ExpressionForm, FunctionForm, LawRole, ModelForm,
    NominalIdentityPreimage, OperationConstraintKind, OperationMemberKind, OperationModeKind,
    ProtocolForm, RelationForm, ScalarTypeForm, StateForm, TemporalForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::common::{
    application_operator, body_term, decoder_pointer, digest_json, node_pointer,
};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

const APPLICATION_NODE_VERSION: &str = "quire.application-node/v1";

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
pub(super) fn validate_application_keys(
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
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
        let computed = digest_json(&preimage).map_err(|_| {
            ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                application.node_id(),
            )
        })?;
        if computed != node_id.digest.as_ref() {
            return Err(application.refuse(
                CheckedPackageRefusalCode::InvalidPackage,
                application.node_id(),
                CheckedPackageRefusalCause::StaleNodeKey,
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
fn application_preimage(
    application: Application<'_>,
    group: &[&CheckedNodeId],
) -> Result<Value, ValidationFailure> {
    let node = application.node;
    let invalid = || {
        ValidationFailure::refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            application.node_id(),
        )
    };
    let declaration = node
        .declaration
        .as_ref()
        .map(|declaration| json!({ "qualified_name": declaration.qualified_name }));
    let semantic_type = serde_json::to_value(&node.semantic_type).map_err(|_| invalid())?;
    let recursion = match node.recursion_group {
        None => Value::Null,
        Some(_) => {
            let ordinal = group
                .iter()
                .position(|member| **member == node.node_id)
                .ok_or_else(invalid)?;
            json!({ "size": group.len(), "ordinal": ordinal })
        }
    };
    Ok(json!({
        "version": APPLICATION_NODE_VERSION,
        "node_tag": node.node_tag.as_ref(),
        "semantic_form": node.semantic_form.as_ref(),
        "semantic_type": semantic_type,
        "declaration": declaration,
        "recursion": recursion,
        "body": group_references(&node.body, group),
    }))
}

/// `term` with every `reference` to a recursion-group member rewritten as
/// `{term: "group_reference", ordinal}`, walking application arguments,
/// aggregate members and binding values, the SemanticTerm positions that hold
/// terms. The body grammar is read here as the wire's JSON, like every body
/// validator.
fn group_references(term: &Value, group: &[&CheckedNodeId]) -> Value {
    if group.is_empty() {
        return term.clone();
    }
    let mut rewritten = term.clone();
    match body_term(term) {
        Some(BodyTerm::Reference) => {
            let target = term
                .get("target")
                .and_then(|target| serde_json::from_value::<CheckedNodeId>(target.clone()).ok());
            if let Some(ordinal) =
                target.and_then(|target| group.iter().position(|member| **member == target))
            {
                return json!({ "term": "group_reference", "ordinal": ordinal });
            }
        }
        Some(BodyTerm::Application) => {
            if let Some(arguments) = term.get("arguments").and_then(Value::as_array) {
                rewritten["arguments"] = Value::Array(
                    arguments
                        .iter()
                        .map(|argument| group_references(argument, group))
                        .collect(),
                );
            }
        }
        Some(BodyTerm::Aggregate) => {
            if let Some(members) = term.get("members").and_then(Value::as_array) {
                rewritten["members"] = Value::Array(
                    members
                        .iter()
                        .map(|member| group_references(member, group))
                        .collect(),
                );
            }
        }
        Some(BodyTerm::Binding) => {
            if let Some(value) = term.get("value") {
                rewritten["value"] = group_references(value, group);
            }
        }
        // A dependency reference names a node of another package, never a
        // member of this recursion group, and enters the preimage as it
        // stands on the wire.
        Some(BodyTerm::Literal | BodyTerm::DependencyReference | BodyTerm::Frame) | None => {}
    }
    rewritten
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationWire {
    identity: Box<str>,
    laws: Vec<OperationLawWire>,
    mode: Option<OperationModeWire>,
    member: Option<Value>,
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
struct OperationLawWire {
    role: Box<str>,
    definition: CheckedArtifactRef,
}

impl OperationLawWire {
    /// The law's decoded role; `None` outside the catalog's vocabulary.
    // Decodes the wire law role.
    fn role_class(&self) -> Option<LawRole> {
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
        if !is_application(&node.body) {
            // No application check applies: the terms of the body are still
            // walked for `dependency_reference` (FR-322, step 7).
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
            if kind != wire_kind {
                return member_mismatch();
            }
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
        Some(
            OperationMemberKind::Position
            | OperationMemberKind::Element
            | OperationMemberKind::RelationshipEnd
            | OperationMemberKind::TypeArgument
            | OperationMemberKind::ProfileOperator
            | OperationMemberKind::StateClause,
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
            resolve_family(&type_node, nodes, kinds, index, 0)
        }
        Some(BodyTerm::Binding) => Some("binder"),
        // A `dependency_reference` callee has family `function` (FR-322), and
        // the dependency-reference walk (step 7), which runs before the
        // operand checks, has already refused it anywhere but a
        // `quire.op.function.call` callee, so no operand check reads it.
        Some(BodyTerm::Aggregate | BodyTerm::DependencyReference | BodyTerm::Frame) | None => None,
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
        K::Temporal(
            TemporalForm::TemporalClause
            | TemporalForm::Formula
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
            | CorrespondenceForm::ProfileCorrespondence,
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
            | ValueForm::OptionValue
            | ValueForm::Parameter,
        ) => false,
        K::Expression(ExpressionForm::Reference) => true,
        K::Expression(
            ExpressionForm::Call
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

fn resolve_family(
    type_id: &CheckedNodeId,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    depth: u8,
) -> Option<&'static str> {
    if depth > 8 {
        return None;
    }
    let position = *index.get(type_id)?;
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
    if kind.tag() == CheckedNodeTag::BoundedDomain {
        return resolve_family(&node.semantic_type, nodes, kinds, index, depth + 1);
    }
    None
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
        let mut budget = Budget::new(meter, owner.selection);
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
    let edge = MemberType::Reference(
        declaration_key(
            &owner.package.identity,
            &owner.package.version,
            DeclarationForm::ObjectType,
            edge_owner,
        )
        .into(),
    );
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
    let mut budget = Budget::new(meter, owner.selection);
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
    let mut budget = Budget::new(meter, a.selection);
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
        && resolve_family(&inner, graph.nodes, graph.kinds, graph.index, 0).is_some();
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
    let mut budget = Budget::new(meter, owner.selection);
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
            if type_node(0).map(|id| id.digest) != Some(receiver.node_key().into()) {
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
                if type_node(position).map(|id| id.digest) != Some(expected.node_key().into()) {
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
    if result_type != Some(member_type.node_key().as_str()) {
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
        | BodyTerm::Frame => return None,
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
        let Some(actual_family) = resolve_family(&type_id, nodes, kinds, index, 0) else {
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

/// Why a text-leaf count could not be decided.
enum LeafWalkEnd {
    /// The type reaches itself through its fields or inner types.
    Cycle,
    /// A node on the walk does not resolve, or is not shaped as its form
    /// requires.
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
/// `text_profile` each leaf's type pins, or the first leaf at fault and the
/// member of it (`path` or `laws`).
type LeafShape = Result<Vec<Box<str>>, (usize, &'static str)>;

/// One path segment (`field:<name>`, `position:<n>`, `inner`) and the type
/// node it leads to.
type LeafChild = (Box<str>, CheckedNodeId);

/// One type node being summed: how many of its children are done, and the
/// running count. Its children are `LeafWalk::kids[position]`.
struct LeafFrame {
    position: usize,
    next: usize,
    total: usize,
}

/// A node's own contribution: a count, or children still to sum.
enum LeafEntry {
    Count(usize),
    Pushed,
}

/// Counts the `text` leaves of a type (QSpec FR-322: `leaves` has one entry
/// for each), walking `record` fields, `tuple` positions and the inner type
/// of an `option`, `sequence`, `set`, `bag` or `ordered_set`. Each type node
/// is counted once however many fields share it, and every node visit is
/// charged to the work meter, so a shared-field chain is linear and nesting
/// depth is bounded by the budget. The walk is iterative: depth costs heap,
/// not stack.
///
/// The expected paths are never listed up front, since a shared-field type
/// has exponentially many. Once [`LeafWalk::count`] has settled the count and
/// it equals the number of supplied leaves, [`LeafWalk::first_leaf_fault`]
/// derives them one at a time in declaration order, skipping every subtree
/// the memo says holds no text leaf, so its cost is bounded by the supplied
/// leaves times the depth times the width of a node's fields (a sibling
/// holding no text is entered and skipped), and is charged to the meter.
struct LeafWalk<'g, 'm> {
    nodes: &'g [CheckedSemanticNodeV2],
    kinds: &'g [CheckedNodeKind],
    index: &'g BTreeMap<&'g CheckedNodeId, usize>,
    meter: &'m mut WorkMeter,
    at: JsonPointer,
    memo: BTreeMap<usize, usize>,
    /// The profile each alias or domain node of a text type's chain pins.
    pins: BTreeMap<usize, Option<Box<str>>>,
    /// Each summed non-leaf type node's children, in declaration order.
    kids: BTreeMap<usize, Vec<LeafChild>>,
    stack: Vec<LeafFrame>,
    on_stack: BTreeSet<usize>,
}

impl LeafWalk<'_, '_> {
    fn children(&self, position: usize, kind: CheckedNodeKind) -> Option<Vec<LeafChild>> {
        let members = || self.nodes[position].body.get("members")?.as_array();
        match kind {
            CheckedNodeKind::CompositeType(CompositeTypeForm::Record) => members()?
                .iter()
                .map(|member| {
                    if body_term(member) != Some(BodyTerm::Binding) {
                        return None;
                    }
                    let name = member.get("name")?.as_str()?;
                    Some((
                        format!("field:{name}").into(),
                        referenced_type(member.get("value")?)?,
                    ))
                })
                .collect(),
            CheckedNodeKind::CompositeType(CompositeTypeForm::Tuple) => members()?
                .iter()
                .enumerate()
                .map(|(at, member)| {
                    Some((format!("position:{at}").into(), referenced_type(member)?))
                })
                .collect(),
            CheckedNodeKind::CompositeType(
                CompositeTypeForm::Option
                | CompositeTypeForm::Sequence
                | CompositeTypeForm::Set
                | CompositeTypeForm::Bag
                | CompositeTypeForm::OrderedSet,
            ) => Some(vec![(
                "inner".into(),
                referenced_type(members()?.first()?)?,
            )]),
            _ => Some(Vec::new()),
        }
    }

    fn enter(&mut self, type_id: &CheckedNodeId) -> Result<LeafEntry, LeafWalkEnd> {
        let at = &self.at;
        self.meter
            .charge(1, || at.clone())
            .map_err(LeafWalkEnd::Work)?;
        let (position, kind) = structural_type(type_id, self.nodes, self.kinds, self.index)
            .ok_or(LeafWalkEnd::Unresolved)?;
        // A text leaf whose type chain pins no profile cannot be decided
        // (QSpec FR-322: the type is ineligible). The pin lives in a wrapper
        // above the shared `text` scalar, so it is read at every visit.
        if kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Text) {
            text_profile_pin(type_id, self)?.ok_or(LeafWalkEnd::Unresolved)?;
            self.memo.insert(position, 1);
            return Ok(LeafEntry::Count(1));
        }
        if let Some(count) = self.memo.get(&position) {
            return Ok(LeafEntry::Count(*count));
        }
        if self.on_stack.contains(&position) {
            return Err(LeafWalkEnd::Cycle);
        }
        let children = self
            .children(position, kind)
            .ok_or(LeafWalkEnd::Unresolved)?;
        if children.is_empty() {
            self.memo.insert(position, 0);
            return Ok(LeafEntry::Count(0));
        }
        self.on_stack.insert(position);
        self.kids.insert(position, children);
        self.stack.push(LeafFrame {
            position,
            next: 0,
            total: 0,
        });
        Ok(LeafEntry::Pushed)
    }

    fn count(&mut self, root: &CheckedNodeId) -> Result<usize, LeafWalkEnd> {
        let mut finished = match self.enter(root)? {
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
            let child = self
                .kids
                .get(&frame.position)
                .and_then(|kids| kids.get(frame.next))
                .map(|(_, child)| child.clone());
            if let Some(child) = child {
                frame.next += 1;
                finished = match self.enter(&child)? {
                    LeafEntry::Count(count) => Some(count),
                    LeafEntry::Pushed => None,
                };
            } else {
                let total = frame.total;
                let position = frame.position;
                self.stack.pop();
                self.on_stack.remove(&position);
                self.memo.insert(position, total);
                finished = Some(total);
            }
        }
    }

    /// The first supplied leaf, by index, that is not the one the type
    /// derives for its place, and the member of it at fault (`path` or
    /// `laws`): the leaves are compared in declaration order against the
    /// expected path and the one `text_profile` law each carries. Must run
    /// after [`LeafWalk::count`] of `root`, with `supplied.len()` equal to
    /// that count. When every leaf fits, returns the `text_profile` each
    /// leaf's type pins, in order.
    fn first_leaf_fault(
        &mut self,
        root: &CheckedNodeId,
        supplied: &[OperationLeafWire],
        text_laws: &[CheckedArtifactRef],
    ) -> Result<LeafShape, LeafWalkEnd> {
        let mut pins: Vec<Box<str>> = Vec::new();
        let mut path: Vec<Box<str>> = Vec::new();
        let mut frames: Vec<(usize, usize)> = Vec::new();
        let mut emitted = 0;
        let mut entering = Some(root.clone());
        loop {
            if let Some(type_id) = entering.take() {
                let at = &self.at;
                self.meter
                    .charge(1, || at.clone())
                    .map_err(LeafWalkEnd::Work)?;
                let (position, kind) =
                    structural_type(&type_id, self.nodes, self.kinds, self.index)
                        .ok_or(LeafWalkEnd::Unresolved)?;
                let count = *self.memo.get(&position).ok_or(LeafWalkEnd::Unresolved)?;
                if count > 0 && kind != CheckedNodeKind::ScalarType(ScalarTypeForm::Text) {
                    frames.push((position, 0));
                } else {
                    if count > 0 {
                        let leaf = supplied.get(emitted).ok_or(LeafWalkEnd::Unresolved)?;
                        if !leaf
                            .path
                            .iter()
                            .map(AsRef::as_ref)
                            .eq(path.iter().map(AsRef::as_ref))
                        {
                            return Ok(Err((emitted, "path")));
                        }
                        let lawful = matches!(
                            leaf.laws.as_slice(),
                            [law] if law.role_class() == Some(LawRole::TextProfile)
                                && text_laws.contains(&law.definition)
                        );
                        if !lawful {
                            return Ok(Err((emitted, "laws")));
                        }
                        pins.push(
                            text_profile_pin(&type_id, self)?.ok_or(LeafWalkEnd::Unresolved)?,
                        );
                        emitted += 1;
                    }
                    if frames.is_empty() {
                        return Ok(Ok(pins));
                    }
                    path.pop();
                }
            }
            let Some(top) = frames.last_mut() else {
                return Ok(Ok(pins));
            };
            let child = self
                .kids
                .get(&top.0)
                .ok_or(LeafWalkEnd::Unresolved)?
                .get(top.1);
            if let Some((segment, child)) = child {
                top.1 += 1;
                path.push(segment.clone());
                entering = Some(child.clone());
            } else {
                frames.pop();
                if !frames.is_empty() {
                    path.pop();
                }
            }
        }
    }
}

/// QSpec FR-322: for an entry naming a leaf source, `operation.leaves` lists,
/// in declaration order, one entry for every `text` leaf of the compared
/// type, each with that leaf's path (`field:<name>`, `position:<n>`, `inner`)
/// and one `text_profile` law. A type with none takes an empty list; fewer
/// entries than text leaves is `operation-law-missing`, and more entries, a
/// wrong or misordered path, or a leaf whose laws are not exactly one
/// catalogued `text_profile` definition is `operation-law-mismatch`; a leaf
/// law the lock does not select is `operation-law-unselected`. Each leaf's
/// `mode` must be a catalogued `text_profile` mode (`operation-mode-mismatch`
/// when absent, of another kind or of an uncatalogued value) and equal the
/// profile its text type pins (`operation-mode-type-mismatch`).
/// `result_inner` expects leaves only for a `set`, `bag` or `ordered_set`
/// result; any other result expects none, so a supplied leaf is a mismatch. A
/// compared type that reaches itself, has a node that does not resolve, or
/// has a text leaf pinning no profile is `ill_typed`/`operator-ineligible`;
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
    let mut walk = LeafWalk {
        nodes,
        kinds,
        index,
        meter,
        at: at.clone(),
        memo: BTreeMap::new(),
        pins: BTreeMap::new(),
        kids: BTreeMap::new(),
        stack: Vec::new(),
        on_stack: BTreeSet::new(),
    };
    let refuse = |code, path: JsonPointer, cause| Ok(Some(application.refuse(code, path, cause)));
    let supplied = operation.leaves.len();
    let walked = walk.count(&compared).and_then(|expected| {
        if supplied != expected {
            return Ok(Err(expected));
        }
        let text_laws = catalog
            .law_role_definitions(LawRole::TextProfile)
            .unwrap_or_default();
        walk.first_leaf_fault(&compared, &operation.leaves, text_laws)
            .map(Ok)
    });
    match walked {
        Ok(Err(expected)) if supplied < expected => refuse(
            CheckedPackageRefusalCode::InvalidPackage,
            at,
            CheckedPackageRefusalCause::OperationLawMissing,
        ),
        Ok(Err(expected)) => refuse(
            CheckedPackageRefusalCode::InvalidPackage,
            at.index(expected),
            CheckedPackageRefusalCause::OperationLawMismatch,
        ),
        Ok(Ok(Err((leaf, member)))) => refuse(
            CheckedPackageRefusalCode::InvalidPackage,
            at.index(leaf).key(member),
            CheckedPackageRefusalCause::OperationLawMismatch,
        ),
        Ok(Ok(Ok(pins))) => {
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
            for (at_leaf, leaf) in operation.leaves.iter().enumerate() {
                let leaf_at = at.clone().index(at_leaf);
                let admitted = match &leaf.mode {
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
        Err(LeafWalkEnd::Cycle | LeafWalkEnd::Unresolved) => refuse(
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
    use super::{
        application_preimage, is_type_shaped, operand_family, operation_catalog, operation_defect,
        validate_application_keys, Application, CheckedNodeId, CheckedNodeKind, CheckedNodeTag,
        CheckedPackageLockV2, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
        CheckedSemanticNodeV2, DependencyReferences, ExpressionForm, Graph, LawRole, ModelOwners,
        SuppliedDependencies, ValidationFailure, WorkMeter, APPLICATION_NODE_VERSION,
    };
    use crate::checked_package::common::{digest_json, NODE_DOMAIN};
    use crate::checked_package::shared::{
        CheckedArtifactRef, CheckedRevision, CheckedSelection, JsonPointer,
    };

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
        let digest = digest_json(&preimage).expect("preimage digests");
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
            "identity": "test",
            "revision": { "namespace": "test", "value": "test" },
            "digest_domain": "sha256-jcs",
            "digest": dummy_digest(marker),
        })
    }

    /// The exact bytes of the catalog's own first `integer_division`
    /// law-role definition (`quire.value.integer-division.truncating/v1`),
    /// copied from `checked-operation-catalog-v1.json` so `catalog.entry(...)`
    /// recognizes it as catalogued while the empty lock leaves it
    /// unselected.
    fn real_integer_division_truncating_definition() -> Value {
        json!({
            "authority": "agent-ix",
            "identity": "quire.value.integer-division.truncating/v1",
            "revision": { "namespace": "quire-draft", "value": "1-draft.1" },
            "digest_domain": "quire.definition.bytes/v1",
            "digest": "9998507608e4885b314d5dcc59a88bb3d04ef3c263d2d8ae5810f92ae1893364",
        })
    }

    /// A lock with nothing selected; every test identity here carries zero
    /// laws, so nothing in `operation_defect` ever consults its selections.
    fn empty_lock() -> CheckedPackageLockV2 {
        let placeholder = CheckedArtifactRef {
            authority: Box::from("test"),
            identity: Box::from("test"),
            revision: CheckedRevision {
                namespace: Box::from("test"),
                value: Box::from("test"),
            },
            digest_domain: Box::from("sha256-jcs"),
            digest: Box::from(dummy_digest('a').as_str()),
            export: None,
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
    /// Tracing: TC-048
    #[test]
    fn application_preimage_matches_the_qsl_pinned_group_vector() {
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
        assert_eq!(
            String::from_utf8(serde_json::to_vec(&preimage).expect("bytes")).expect("utf-8"),
            expected
        );
    }

    /// Group references are rewritten in every nested term position, as in
    /// quire-spec-language's `group_references_are_rewritten_in_every_nested_term`.
    ///
    /// Tracing: TC-048
    #[test]
    fn group_references_are_rewritten_in_every_nested_term() {
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
        let members = &preimage["body"]["members"];
        assert_eq!(members[0]["value"], group_reference(1));
        assert_eq!(members[1]["arguments"][0], group_reference(0));
        assert_eq!(
            members[1]["arguments"][1]["arguments"][0],
            group_reference(1)
        );
        assert_eq!(members[1]["arguments"][1]["arguments"][1], reference(9));
        assert_eq!(preimage["recursion"], json!({ "size": 2, "ordinal": 0 }));
    }

    fn typed(digest: &str) -> CheckedNodeId {
        serde_json::from_value(json!({
            "domain": "quire.checked-semantic-node/v1",
            "digest": digest
        }))
        .expect("node id")
    }

    /// The operand classification over every kind the closed vocabularies
    /// produce: which catalog family a node denotes directly, and whether an
    /// argument naming it names a type. Pinned whole, so an edit to either
    /// exhaustive table that moves any one form is caught here.
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
                ("composite_type", "reference", "reference"),
                ("expression", "reference", "reference"),
                ("function", "pure_function", "function"),
                ("function", "predicate", "function"),
                ("function", "recursive_function", "function"),
                ("model", "object_type", "object"),
                ("model", "systems_interface", "object"),
                ("relation", "population", "population"),
            ]
        );
        // Type-shaped is the five type families, every form of each, plus
        // the `reference` expression; checked for every one of the 100 kinds,
        // so flipping any single form is caught.
        for kind in CheckedNodeKind::all() {
            let expected = matches!(
                kind.tag(),
                CheckedNodeTag::ScalarType
                    | CheckedNodeTag::CompositeType
                    | CheckedNodeTag::BoundedDomain
                    | CheckedNodeTag::Relation
                    | CheckedNodeTag::Function
            ) || kind == CheckedNodeKind::Expression(ExpressionForm::Reference);
            assert_eq!(is_type_shaped(kind), expected, "{kind:?}");
        }
    }

    /// An `application` node whose `operation.identity` is absent from the
    /// catalog is refused `invalid_package`/`unknown-operation`. This is the
    /// criterion the deleted private-sourced fixture tree used to carry (see
    /// agent-ix/quire-contract-ir#166): without it, the `catalog.entry(...)`
    /// lookup in `operation_defect` could be replaced by an always-`Some`
    /// admission and nothing in this crate's test suite would notice.
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

    /// `operation-mode-mismatch`: agent-ix/quire-contract-ir#171.
    /// [`DECIMAL_ADD_IDENTITY`] requires a `rounding` mode; a missing mode
    /// must be refused.
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
    /// `name` field's own type pins `rounding` to `"nearest-even"`; a
    /// `["field:name"]` leaf whose mode value disagrees must be refused by
    /// `check_leaf_count`, independent of the operation's own top-level mode
    /// (left absent here, so `check_mode_type` never fires first). The field
    /// is text and the leaf carries its `text_profile` law, so the leaf shape
    /// settles first.
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

    /// A type that reaches itself is `ill_typed`/`operator-ineligible`, and a
    /// field whose type is not in the graph is the same refusal.
    ///
    /// Tracing: TC-048, FR-038-AC-43
    #[test]
    fn tc_048_leaf_walk_refuses_a_cycle_and_an_unresolved_node() {
        let cyclic = vec![
            record_type_node('r', &[("name", 'x'), ("next", 'o')]),
            collection_type_node('o', "option", 'r'),
            scalar_type_node('x', "text"),
        ];
        let (_, locus) = leaves_defect("quire.op.structural.eq", json!([]), vec![], ['r', 'r']);
        assert_eq!(
            eq_over('r', cyclic),
            leaves_ineligible(locus.clone()),
            "cycle"
        );
        let dangling = vec![record_type_node('r', &[("f", 'm')])];
        assert_eq!(
            eq_over('r', dangling),
            leaves_ineligible(locus),
            "unresolved"
        );
    }

    /// Nesting is bounded by the work budget, not a depth cutoff: text under
    /// 40 options is still found, and 2000 options of integer exhaust the
    /// budget instead of being admitted or overflowing the stack.
    ///
    /// Tracing: TC-048, FR-038-AC-43
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
    /// `operation-law-missing` and a cyclic type is
    /// `ill_typed`/`operator-ineligible`, not `operation-mode-type-mismatch`.
    ///
    /// Tracing: TC-048, FR-038-AC-43
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
            vec![collection_type_node('o', "option", 'r')],
        );
        assert_eq!(result, leaves_ineligible(locus), "cyclic type");
    }

    /// Records sharing field types are counted once per type node: 12 levels
    /// of 4 fields all naming the next level (4^12 paths) finish inside the
    /// work budget instead of hanging.
    ///
    /// Tracing: TC-048, FR-038-AC-43
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

    /// `same_type` compares the type node each operand resolves to, not the
    /// operands' own nodes: [`STRUCTURAL_EQ_IDENTITY`] over two distinct
    /// parameters of one record type is admitted, and over parameters of two
    /// different record types is refused at the second operand.
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
    /// Tracing: TC-048
    #[test]
    fn validate_application_keys_refuses_a_stale_node_key() {
        let node = application_node(CATALOGUED_IDENTITY, "binary");
        let nodes = std::slice::from_ref(&node);
        let mut index: BTreeMap<&CheckedNodeId, usize> = BTreeMap::new();
        index.insert(&node.node_id, 0);
        let mut meter = WorkMeter::new(1_000);

        let result = validate_application_keys(nodes, &index, &mut meter);

        assert_eq!(
            result,
            Err(refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                "/semantic_graph/nodes/0/node_id",
                Some(CheckedPackageRefusalCause::StaleNodeKey),
                node.node_id.clone(),
            )),
            "a node_id that is not the JCS SHA-256 of the node's own preimage must be \
             refused as stale-node-key, got {result:?}"
        );
    }

    /// Control for the test above: a node whose `node_id.digest` genuinely
    /// is its own preimage's digest ([`correctly_keyed`]) is admitted, not
    /// refused. Without this control, a `validate_application_keys` that
    /// refused every node would satisfy the assertion above just as well as
    /// the real re-derivation does.
    ///
    /// Tracing: TC-048
    #[test]
    fn validate_application_keys_admits_a_correctly_keyed_node() {
        let node = correctly_keyed(application_node(CATALOGUED_IDENTITY, "binary"));
        let nodes = std::slice::from_ref(&node);
        let mut index: BTreeMap<&CheckedNodeId, usize> = BTreeMap::new();
        index.insert(&node.node_id, 0);
        let mut meter = WorkMeter::new(1_000);

        let result = validate_application_keys(nodes, &index, &mut meter);

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
}
