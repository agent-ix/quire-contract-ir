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
//! entry other than `same_family`/`same_type` is not enforced; leaf-path
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
//! [`check_mode_type`], [`check_leaves`]) that consults it.

use super::model_members::{
    Budget, MemberKind, MemberType, ModelFailure, ModelOwners, ModelRefusal, Resolved,
};
use super::operation_catalog::{operation_catalog, OperationCatalog, OperationCatalogEntry};
use super::{
    BodyTerm, BoundedDomainForm, CheckedArtifactRef, CheckedNodeId, CheckedNodeKind,
    CheckedNodeTag, CheckedPackageLockV2, CheckedSemanticNodeV2, ClaimForm, CompositeTypeForm,
    CorrespondenceForm, ExpressionForm, FunctionForm, LawRole, ModelForm, OperationConstraintKind,
    OperationMemberKind, OperationModeKind, ProtocolForm, RelationForm, ScalarTypeForm, StateForm,
    TemporalForm, ValueForm, WorkMeter,
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
use std::collections::BTreeMap;

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
        Some(BodyTerm::Literal | BodyTerm::Frame) | None => {}
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
    // string-edge: decodes the wire member kind.
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
    // string-edge: decodes the wire law role.
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
    // string-edge: decodes the wire mode kind.
    fn kind_class(&self) -> Option<OperationModeKind> {
        OperationModeKind::from_wire(&self.kind)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationLeafWire {
    path: Vec<Box<str>>,
    /// Parsed for `deny_unknown_fields` fidelity with the wire shape; not
    /// read by `check_leaves`, which resolves a leaf's pinned value from its
    /// own record-field type rather than from any law this member declares.
    #[allow(dead_code)]
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
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = Graph {
        nodes,
        kinds,
        index,
    };
    for &position in index.values() {
        let node = &nodes[position];
        if !is_application(&node.body) {
            continue;
        }
        let application = Application { node, position };
        if let Some(failure) = operation_defect(
            application,
            &graph,
            lock,
            owners,
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
    if entry.leaves.is_some() && operation.leaves.is_empty() {
        return refuse(
            application.body(&["operation", "leaves"]),
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
    match (entry.member, wire_member_kind) {
        (None, None) => {}
        (Some(kind), Some(Some(wire_kind))) if kind == wire_kind => {}
        _ => {
            return refuse(
                member_or_operation("member"),
                CheckedPackageRefusalCause::OperationMemberMismatch,
            )
        }
    }

    let arguments = body
        .get("arguments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(failure) = check_operands(
        application,
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
    let member_kind = match wire_member_kind.flatten() {
        Some(OperationMemberKind::Field) => Some(MemberKind::Field),
        Some(OperationMemberKind::Operation) => Some(MemberKind::Operation),
        Some(
            OperationMemberKind::Position
            | OperationMemberKind::Element
            | OperationMemberKind::RelationshipEnd
            | OperationMemberKind::TypeArgument
            | OperationMemberKind::ProfileOperator,
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
    if let Some(failure) = check_leaves(application, &operation, &arguments, nodes, kinds, index) {
        return Ok(Some(failure));
    }
    Ok(None)
}

/// Resolves an argument term's family: `reference` resolves its target node
/// directly (a `population` declaration, a `scalar_type`/`composite_type`
/// node, or an `expression`/`reference` node — the catalog's own `reference`
/// family, an identity/pointer expression, not `composite_type`'s
/// same-named nullable-wrapper form) and, only when the target itself is not
/// one of those (a `value` node — a literal, a `record_value`, ...), falls
/// back to its own `semantic_type` (recursing through `bounded_domain`
/// refinements, e.g. a `float_rounding`/`text_bounds` wrapper, to the family
/// it refines). `binding` is the catalog's `binder` family
/// (`quire.op.collection.sum.*`'s bound variable). Every other term shape is
/// not resolved (`None`), so a caller skips the check it would otherwise
/// support rather than guess.
fn argument_family(
    argument: &Value,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<&'static str> {
    match body_term(argument) {
        Some(BodyTerm::Reference) => {
            let type_node = operand_type_node(argument, nodes, kinds, index)?;
            resolve_family(&type_node, nodes, kinds, index, 0)
        }
        Some(BodyTerm::Binding) => Some("binder"),
        Some(BodyTerm::Literal | BodyTerm::Application | BodyTerm::Aggregate | BodyTerm::Frame)
        | None => None,
    }
}

/// The direct (unreduced) type-node id an argument's target carries, used by
/// [`check_mode_type`]/[`check_leaves`] to find a type-pinned mode.
fn argument_type_id(argument: &Value) -> Option<CheckedNodeId> {
    if body_term(argument) != Some(BodyTerm::Reference) {
        return None;
    }
    serde_json::from_value(argument.get("target")?.clone()).ok()
}

/// The operation catalog's operand family a node of this kind denotes
/// directly, or `None` when it has none of its own (a `bounded_domain` then
/// resolves through its semantic type). Exhaustive over every form.
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
        // FR-322 (STD-102): a model declaration node of an object type or a
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
            | ModelForm::FieldDeclaration
            | ModelForm::OperationDeclaration
            | ModelForm::ClauseMemberDeclaration
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
        if let Some(actual) = argument_family(&arguments[position], nodes, kinds, index) {
            if !catalog.family_fits(actual, expected) {
                return ineligible(Some(position));
            }
        }
    }
    if let Some(rest_family) = &entry.rest {
        for (offset, argument) in arguments[required..].iter().enumerate() {
            if let Some(actual) = argument_family(argument, nodes, kinds, index) {
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
                    .map(|position| argument_family(&arguments[*position], nodes, kinds, index))
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
                    .map(|position| argument_type_id(&arguments[*position]))
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
        }
    }
    Ok(None)
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
// string-edge: parses the catalog's `inner:<n>` result-form text.
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

/// The type-node id an operand's own type-pinned mode (if any) is checked
/// against: a `reference` argument whose direct target is itself
/// type-shaped (a `scalar_type`, `composite_type` or `bounded_domain` node)
/// names that type directly; otherwise (a `value` node — a literal, a
/// `record_value`, ...) it is that node's own `semantic_type`. The same
/// two-tier resolution [`argument_family`] uses, kept separate because a
/// pin lookup needs the wrapper node itself (e.g. `float_rounding`), not
/// the family name [`resolve_family`] reduces it to.
fn operand_type_node(
    argument: &Value,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<CheckedNodeId> {
    let target = argument_type_id(argument)?;
    let position = *index.get(&target)?;
    let target_node = &nodes[position];
    if is_type_shaped(*kinds.get(position)?) {
        Some(target)
    } else {
        Some(target_node.semantic_type.clone())
    }
}

/// A record type's declared field's own type-node id, by field name.
fn record_field_type(
    record_type: &CheckedNodeId,
    field: &str,
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<CheckedNodeId> {
    let declaring = &nodes[*index.get(record_type)?];
    let members = declaring.body.get("members")?.as_array()?;
    let binding = members.iter().find(|entry| {
        body_term(entry) == Some(BodyTerm::Binding)
            && entry.get("name").and_then(Value::as_str) == Some(field)
    })?;
    let value = binding.get("value")?;
    if body_term(value) != Some(BodyTerm::Reference) {
        return None;
    }
    serde_json::from_value(value.get("target")?.clone()).ok()
}

/// The value a type node's own declaration pins for a type-pinned mode kind
/// (`rounding`, `text_profile`): a `bounded_domain` wrapper's `aggregate`
/// body carries exactly one `binding` named for the kind, e.g.
/// `float_rounding`'s `{"term":"binding","name":"rounding","value":{"term":
/// "literal", ..., "value": "nearest-even"}}`.
fn type_pin(
    type_id: &CheckedNodeId,
    kind: &str,
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<Box<str>> {
    let node = &nodes[*index.get(type_id)?];
    let members = node.body.get("members")?.as_array()?;
    let binding = members.iter().find(|entry| {
        body_term(entry) == Some(BodyTerm::Binding)
            && entry.get("name").and_then(Value::as_str) == Some(kind)
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
    let Some(mode_kind) = mode.kind_class() else {
        return None;
    };
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
        if let Some(pinned) = type_pin(&type_id, &mode.kind, nodes, index) {
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

/// The one leaf-path shape an upstream mutation exercises: `["field:<name>"]`
/// against the first operand's record type. Any other path is not resolved.
fn check_leaves(
    application: Application<'_>,
    operation: &OperationWire,
    arguments: &[Value],
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<ValidationFailure> {
    for (leaf_index, leaf) in operation.leaves.iter().enumerate() {
        let Some(mode) = &leaf.mode else { continue };
        let [segment] = leaf.path.as_slice() else {
            continue;
        };
        let Some(field) = segment.strip_prefix("field:") else {
            continue;
        };
        let Some(record_type) = arguments
            .first()
            .and_then(|argument| operand_type_node(argument, nodes, kinds, index))
        else {
            continue;
        };
        let Some(field_type) = record_field_type(&record_type, field, nodes, index) else {
            continue;
        };
        if let Some(pinned) = type_pin(&field_type, &mode.kind, nodes, index) {
            if pinned.as_ref() != mode.value.as_ref() {
                return Some(
                    application.refuse(
                        CheckedPackageRefusalCode::InvalidPackage,
                        application
                            .body(&["operation", "leaves"])
                            .index(leaf_index)
                            .key("mode")
                            .key("value"),
                        CheckedPackageRefusalCause::OperationModeTypeMismatch,
                    ),
                );
            }
        }
    }
    None
}

#[cfg(test)]
mod model_member_vectors;

#[cfg(test)]
mod tests {
    use super::super::CheckedSelectionRole;
    use super::{
        application_preimage, is_type_shaped, operand_family, operation_catalog, operation_defect,
        validate_application_keys, Application, CheckedNodeId, CheckedNodeKind, CheckedNodeTag,
        CheckedPackageLockV2, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
        CheckedSemanticNodeV2, ExpressionForm, Graph, ModelOwners, ValidationFailure, WorkMeter,
        APPLICATION_NODE_VERSION,
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
        let mut index: BTreeMap<&CheckedNodeId, usize> = BTreeMap::new();
        for (position, node) in nodes.iter().enumerate() {
            index.insert(&node.node_id, position);
        }
        let lock = empty_lock();
        let mut meter = WorkMeter::new(1_000);
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
            &lock,
            &ModelOwners::default(),
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
    /// `check_leaves`, independent of the operation's own top-level mode
    /// (left absent here, so `check_mode_type` never fires first).
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
            "decimal_range",
            &node_id('t'),
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

        let mut operation = plain_operation(STRUCTURAL_EQ_IDENTITY);
        operation["leaves"] = json!([
            {
                "path": ["field:name"],
                "mode": { "kind": "rounding", "value": "toward-zero" },
                "laws": [],
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

        let result = defect_for_graph(vec![root.clone(), record_node, field_type_node]);

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

    /// `stale-node-key`: agent-ix/quire-contract-ir#171. `application_node`'s
    /// `node_id.digest` is a placeholder, never the JCS SHA-256 of its own
    /// preimage — exactly the condition [`validate_application_keys`] exists
    /// to catch. Unlike every other test in this module, this one calls
    /// `validate_application_keys` directly rather than `operation_defect`:
    /// the two are separate stages (see the module doc), and nothing above
    /// reaches this one.
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
