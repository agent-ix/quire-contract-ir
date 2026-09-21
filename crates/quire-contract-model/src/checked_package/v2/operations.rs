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
//! Scope this reader does not cover, narrower than the upstream contract
//! description's full generality and not exercised by an admitted fixture or
//! upstream vector,
//! except where noted below: a node's own `recursion` preimage member is
//! encoded here as the bare `recursion_group` label rather than the upstream
//! description's `group_reference` ordinal substitution (no upstream
//! application node carries one) — unlike every other item in this list, this one is a
//! **false-refusal risk, not a silent no-op**: the two encodings yield
//! different digests, so a legitimate application node inside a recursion
//! group is refused `invalid_package`/`stale-node-key` rather than admitted.
//! The remaining items are silent no-ops (never a false refusal), not a
//! silent admission of something the upstream corpus requires rejected: an
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

use super::operation_catalog::{operation_catalog, OperationCatalogEntry};
use super::{
    CheckedArtifactRef, CheckedNodeId, CheckedNodeTag, CheckedPackageLockV2, CheckedSemanticNodeV2,
    WorkMeter,
};
use crate::checked_package::common::digest_json;
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{CheckedPackageRefusalCause, CheckedPackageRefusalCode};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const NODE_ID_PATH: &str = "semantic_graph.nodes.node_id";
const OPERATION_PATH: &str = "semantic_graph.nodes.body.operation";
const OPERATOR_PATH: &str = "semantic_graph.nodes.body.operator";
const OPERATION_LAWS_PATH: &str = "semantic_graph.nodes.body.operation.laws";
const OPERATION_MODE_PATH: &str = "semantic_graph.nodes.body.operation.mode";
const OPERATION_MEMBER_PATH: &str = "semantic_graph.nodes.body.operation.member";
const OPERATION_ARGUMENTS_PATH: &str = "semantic_graph.nodes.body.arguments";

const APPLICATION_NODE_VERSION: &str = "quire.application-node/v1";

fn refuse_at(
    code: CheckedPackageRefusalCode,
    path: &'static str,
    cause: CheckedPackageRefusalCause,
    node: &CheckedSemanticNodeV2,
) -> ValidationFailure {
    ValidationFailure::RefusedAt(code, path, Some(cause), node.node_id.clone())
}

fn is_application(body: &Value) -> bool {
    body.get("term").and_then(Value::as_str) == Some("application")
}

/// Every application node's `node_id` re-derived from its own visible
/// members, in ascending digest order, reporting the first stale one. See
/// the module doc for the one narrowing from the upstream contract
/// description: a recursion-group member is encoded as its bare label, never
/// the `group_reference` ordinal substitution, because no upstream
/// application node carries one.
pub(super) fn validate_application_keys(
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for (&node_id, &position) in index {
        let node = &nodes[position];
        if !is_application(&node.body) {
            continue;
        }
        meter.charge(1)?;
        let declaration = node
            .declaration
            .as_ref()
            .map(|declaration| json!({ "qualified_name": declaration.qualified_name }));
        let semantic_type = serde_json::to_value(&node.semantic_type).map_err(|_| {
            ValidationFailure::Refused(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                NODE_ID_PATH,
            )
        })?;
        let preimage = json!({
            "version": APPLICATION_NODE_VERSION,
            "node_tag": node.node_tag.as_ref(),
            "semantic_form": node.semantic_form.as_ref(),
            "semantic_type": semantic_type,
            "declaration": declaration,
            "recursion": node.recursion_group.as_deref(),
            "body": node.body,
        });
        let computed = digest_json(&preimage).map_err(|_| {
            ValidationFailure::Refused(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                NODE_ID_PATH,
            )
        })?;
        if computed != node_id.digest.as_ref() {
            return Err(refuse_at(
                CheckedPackageRefusalCode::InvalidPackage,
                NODE_ID_PATH,
                CheckedPackageRefusalCause::StaleNodeKey,
                node,
            ));
        }
    }
    Ok(())
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationLawWire {
    role: Box<str>,
    definition: CheckedArtifactRef,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct OperationModeWire {
    kind: Box<str>,
    value: Box<str>,
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
    index: &BTreeMap<&CheckedNodeId, usize>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for &position in index.values() {
        let node = &nodes[position];
        if !is_application(&node.body) {
            continue;
        }
        if let Some(failure) = operation_defect(node, nodes, index, lock, meter)? {
            return Err(failure);
        }
    }
    Ok(())
}

fn operation_defect(
    node: &CheckedSemanticNodeV2,
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<Option<ValidationFailure>, ValidationFailure> {
    let Some(body) = node.body.as_object() else {
        return Ok(None);
    };
    let operator = body.get("operator").and_then(Value::as_str).unwrap_or("");
    let Some(operation_value) = body.get("operation") else {
        return Ok(None);
    };
    let Ok(operation) = serde_json::from_value::<OperationWire>(operation_value.clone()) else {
        return Ok(Some(ValidationFailure::Refused(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            OPERATION_PATH,
        )));
    };
    meter.charge(
        1 + u64::try_from(operation.laws.len()).unwrap_or(u64::MAX)
            + u64::try_from(operation.leaves.len()).unwrap_or(u64::MAX),
    )?;

    let catalog = operation_catalog();
    let Some(entry) = catalog.entry(&operation.identity) else {
        return Ok(Some(refuse_at(
            CheckedPackageRefusalCode::InvalidPackage,
            OPERATION_PATH,
            CheckedPackageRefusalCause::UnknownOperation,
            node,
        )));
    };
    if operator != entry.operator.as_ref() {
        return Ok(Some(refuse_at(
            CheckedPackageRefusalCode::InvalidPackage,
            OPERATOR_PATH,
            CheckedPackageRefusalCause::OperationClassMismatch,
            node,
        )));
    }
    let leaf_required = entry.leaves.is_some();
    if operation.laws.len() < entry.laws.len() || (leaf_required && operation.leaves.is_empty()) {
        return Ok(Some(refuse_at(
            CheckedPackageRefusalCode::InvalidPackage,
            OPERATION_LAWS_PATH,
            CheckedPackageRefusalCause::OperationLawMissing,
            node,
        )));
    }
    if operation.laws.len() > entry.laws.len() {
        // Too many, not too few: a law the catalogued entry admits no role
        // for is a mismatch against what it declares, not a shortfall.
        return Ok(Some(refuse_at(
            CheckedPackageRefusalCode::InvalidPackage,
            OPERATION_LAWS_PATH,
            CheckedPackageRefusalCause::OperationLawMismatch,
            node,
        )));
    }
    for (law, role) in operation.laws.iter().zip(entry.laws.iter()) {
        if law.role.as_ref() != role.as_ref() {
            return Ok(Some(refuse_at(
                CheckedPackageRefusalCode::InvalidPackage,
                OPERATION_LAWS_PATH,
                CheckedPackageRefusalCause::OperationLawMismatch,
                node,
            )));
        }
        // A clause/profile role (`temporal_profile`, `protocol_profile`) has
        // no fixed catalogued definition list of its own — any published
        // profile definition of that kind is eligible — so its legitimacy
        // is exactly whether the lock's own `profile_selections` selected it
        // under this role; a value role (`integer_division`, `ieee_profile`,
        // `text_profile`) is closed over `law_roles`, so a definition
        // outside that fixed list is a mismatch before the lock is
        // consulted at all.
        let selected = if catalog.is_profile_role(role) {
            lock.profile_selections.iter().any(|selection| {
                selection.role.as_ref() == role.as_ref() && selection.definition == law.definition
            })
        } else {
            let catalogued = catalog.law_role_definitions(role);
            let known = catalogued.is_some_and(|definitions| definitions.contains(&law.definition));
            if !known {
                return Ok(Some(refuse_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    OPERATION_LAWS_PATH,
                    CheckedPackageRefusalCause::OperationLawMismatch,
                    node,
                )));
            }
            lock.definition_selections.contains(&law.definition)
        };
        if !selected {
            return Ok(Some(refuse_at(
                CheckedPackageRefusalCode::InvalidPackage,
                OPERATION_LAWS_PATH,
                CheckedPackageRefusalCause::OperationLawUnselected,
                node,
            )));
        }
    }
    match (&entry.mode, &operation.mode) {
        (None, None) => {}
        (Some(kind), Some(mode)) if mode.kind.as_ref() == kind.as_ref() => {}
        _ => {
            return Ok(Some(refuse_at(
                CheckedPackageRefusalCode::InvalidPackage,
                OPERATION_MODE_PATH,
                CheckedPackageRefusalCause::OperationModeMismatch,
                node,
            )))
        }
    }
    let wire_member_kind = operation
        .member
        .as_ref()
        .and_then(|member| member.get("kind"))
        .and_then(Value::as_str);
    match (&entry.member, wire_member_kind) {
        (None, None) => {}
        (Some(kind), Some(wire_kind)) if kind.as_ref() == wire_kind => {}
        _ => {
            return Ok(Some(refuse_at(
                CheckedPackageRefusalCode::InvalidPackage,
                OPERATION_MEMBER_PATH,
                CheckedPackageRefusalCause::OperationMemberMismatch,
                node,
            )))
        }
    }

    let arguments = body
        .get("arguments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(failure) = check_operands(node, entry, &arguments, nodes, index, catalog) {
        return Ok(Some(failure));
    }
    if let Some(name) = wire_member_kind {
        if name == "field" {
            if let Some(failure) = check_field_member(node, &operation, nodes, index) {
                return Ok(Some(failure));
            }
        }
    }
    if let Some(failure) =
        check_mode_type(node, entry, &operation, &arguments, nodes, index, catalog)
    {
        return Ok(Some(failure));
    }
    if let Some(failure) = check_leaves(node, &operation, &arguments, nodes, index) {
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
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<&'static str> {
    match argument.get("term").and_then(Value::as_str) {
        Some("reference") => {
            let type_node = operand_type_node(argument, nodes, index)?;
            resolve_family(&type_node, nodes, index, 0)
        }
        Some("binding") => Some("binder"),
        _ => None,
    }
}

/// The direct (unreduced) type-node id an argument's target carries, used by
/// [`check_mode_type`]/[`check_leaves`] to find a type-pinned mode.
fn argument_type_id(argument: &Value) -> Option<CheckedNodeId> {
    if argument.get("term").and_then(Value::as_str) != Some("reference") {
        return None;
    }
    serde_json::from_value(argument.get("target")?.clone()).ok()
}

fn resolve_family(
    type_id: &CheckedNodeId,
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
    depth: u8,
) -> Option<&'static str> {
    if depth > 8 {
        return None;
    }
    let node = &nodes[*index.get(type_id)?];
    let tag = CheckedNodeTag::from_wire(&node.node_tag)?;
    let form = node.semantic_form.as_ref();
    let direct = match (tag, form) {
        (CheckedNodeTag::ScalarType, "boolean") => Some("boolean"),
        (CheckedNodeTag::ScalarType, "integer") => Some("integer"),
        (CheckedNodeTag::ScalarType, "rational") => Some("rational"),
        (CheckedNodeTag::ScalarType, "decimal") => Some("decimal"),
        (CheckedNodeTag::ScalarType, "float32") => Some("float32"),
        (CheckedNodeTag::ScalarType, "float64") => Some("float64"),
        (CheckedNodeTag::ScalarType, "text") => Some("text"),
        (CheckedNodeTag::ScalarType, "enum") => Some("enum"),
        (CheckedNodeTag::CompositeType, "option") => Some("option"),
        (CheckedNodeTag::CompositeType, "sequence") => Some("sequence"),
        (CheckedNodeTag::CompositeType, "set") => Some("set"),
        (CheckedNodeTag::CompositeType, "bag") => Some("bag"),
        (CheckedNodeTag::CompositeType, "ordered_set") => Some("ordered_set"),
        (CheckedNodeTag::CompositeType, "record") => Some("record"),
        (CheckedNodeTag::CompositeType, "tuple") => Some("tuple"),
        (CheckedNodeTag::CompositeType, "reference") => Some("reference"),
        (CheckedNodeTag::Relation, "population") => Some("population"),
        (CheckedNodeTag::Function, "pure_function" | "predicate" | "recursive_function") => {
            Some("function")
        }
        // The catalog's `reference` family: an identity/pointer expression
        // (a `reference` term naming a relationship end or declaration),
        // never `composite_type`'s same-named nullable-wrapper form.
        (CheckedNodeTag::Expression, "reference") => Some("reference"),
        _ => None,
    };
    if direct.is_some() {
        return direct;
    }
    if tag == CheckedNodeTag::BoundedDomain {
        return resolve_family(&node.semantic_type, nodes, index, depth + 1);
    }
    None
}

fn check_operands(
    node: &CheckedSemanticNodeV2,
    entry: &OperationCatalogEntry,
    arguments: &[Value],
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
    catalog: &super::operation_catalog::OperationCatalog,
) -> Option<ValidationFailure> {
    let required = entry.operands.len();
    let ineligible = || {
        Some(refuse_at(
            CheckedPackageRefusalCode::IllTyped,
            OPERATION_ARGUMENTS_PATH,
            CheckedPackageRefusalCause::OperatorIneligible,
            node,
        ))
    };
    if entry.rest.is_none() {
        if arguments.len() != required {
            return ineligible();
        }
    } else if arguments.len() < required {
        return ineligible();
    }
    for (position, expected) in entry.operands.iter().enumerate() {
        if let Some(actual) = argument_family(&arguments[position], nodes, index) {
            if !catalog.family_fits(actual, expected) {
                return ineligible();
            }
        }
    }
    if let Some(rest_family) = &entry.rest {
        for argument in &arguments[required..] {
            if let Some(actual) = argument_family(argument, nodes, index) {
                if !catalog.family_fits(actual, rest_family) {
                    return ineligible();
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
        match constraint.kind.as_ref() {
            "same_family" => {
                let families: Option<Vec<&str>> = indices
                    .iter()
                    .map(|position| argument_family(&arguments[*position], nodes, index))
                    .collect();
                if let Some(families) = families {
                    if families.windows(2).any(|pair| pair[0] != pair[1]) {
                        return ineligible();
                    }
                }
            }
            "same_type" => {
                let types: Option<Vec<CheckedNodeId>> = indices
                    .iter()
                    .map(|position| argument_type_id(&arguments[*position]))
                    .collect();
                if let Some(types) = types {
                    if types.windows(2).any(|pair| pair[0] != pair[1]) {
                        return ineligible();
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// The one `operation.member` shape an upstream mutation exercises in full: a
/// `field` member's `name` must actually be declared on the record type its
/// `declaration` names (that type's own `body` is an `aggregate` of
/// `binding` members, one per field, exactly the shape the record-type
/// fixture nodes carry).
fn check_field_member(
    node: &CheckedSemanticNodeV2,
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
        entry.get("term").and_then(Value::as_str) == Some("binding")
            && entry.get("name").and_then(Value::as_str) == Some(name)
    });
    if declared {
        None
    } else {
        Some(refuse_at(
            CheckedPackageRefusalCode::IllTyped,
            OPERATION_MEMBER_PATH,
            CheckedPackageRefusalCause::OperatorIneligible,
            node,
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
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<CheckedNodeId> {
    let target = argument_type_id(argument)?;
    let target_node = &nodes[*index.get(&target)?];
    let tag = CheckedNodeTag::from_wire(&target_node.node_tag)?;
    let type_shaped = matches!(
        tag,
        CheckedNodeTag::ScalarType
            | CheckedNodeTag::CompositeType
            | CheckedNodeTag::BoundedDomain
            | CheckedNodeTag::Relation
            | CheckedNodeTag::Function
    ) || (tag == CheckedNodeTag::Expression
        && target_node.semantic_form.as_ref() == "reference");
    if type_shaped {
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
        entry.get("term").and_then(Value::as_str) == Some("binding")
            && entry.get("name").and_then(Value::as_str) == Some(field)
    })?;
    let value = binding.get("value")?;
    if value.get("term").and_then(Value::as_str) != Some("reference") {
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
        entry.get("term").and_then(Value::as_str) == Some("binding")
            && entry.get("name").and_then(Value::as_str) == Some(kind)
    })?;
    binding.get("value")?.get("value")?.as_str().map(Box::from)
}

fn check_mode_type(
    node: &CheckedSemanticNodeV2,
    entry: &OperationCatalogEntry,
    operation: &OperationWire,
    arguments: &[Value],
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
    catalog: &super::operation_catalog::OperationCatalog,
) -> Option<ValidationFailure> {
    let mode = operation.mode.as_ref()?;
    if !catalog.is_type_pinned_mode(&mode.kind) {
        return None;
    }
    for (position, expected) in entry.operands.iter().enumerate() {
        let argument = arguments.get(position)?;
        let Some(type_id) = operand_type_node(argument, nodes, index) else {
            continue;
        };
        let Some(actual_family) = resolve_family(&type_id, nodes, index, 0) else {
            continue;
        };
        if !catalog.family_fits(actual_family, expected) {
            continue;
        }
        if let Some(pinned) = type_pin(&type_id, &mode.kind, nodes, index) {
            if pinned.as_ref() != mode.value.as_ref() {
                return Some(refuse_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    OPERATION_MODE_PATH,
                    CheckedPackageRefusalCause::OperationModeTypeMismatch,
                    node,
                ));
            }
        }
    }
    None
}

/// The one leaf-path shape an upstream mutation exercises: `["field:<name>"]`
/// against the first operand's record type. Any other path is not resolved.
fn check_leaves(
    node: &CheckedSemanticNodeV2,
    operation: &OperationWire,
    arguments: &[Value],
    nodes: &[CheckedSemanticNodeV2],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<ValidationFailure> {
    for leaf in &operation.leaves {
        let Some(mode) = &leaf.mode else { continue };
        let [segment] = leaf.path.as_slice() else {
            continue;
        };
        let Some(field) = segment.strip_prefix("field:") else {
            continue;
        };
        let Some(record_type) = arguments
            .first()
            .and_then(|argument| operand_type_node(argument, nodes, index))
        else {
            continue;
        };
        let Some(field_type) = record_field_type(&record_type, field, nodes, index) else {
            continue;
        };
        if let Some(pinned) = type_pin(&field_type, &mode.kind, nodes, index) {
            if pinned.as_ref() != mode.value.as_ref() {
                return Some(refuse_at(
                    CheckedPackageRefusalCode::InvalidPackage,
                    OPERATION_MODE_PATH,
                    CheckedPackageRefusalCause::OperationModeTypeMismatch,
                    node,
                ));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        operation_catalog, operation_defect, CheckedNodeId, CheckedPackageLockV2,
        CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedSemanticNodeV2,
        ValidationFailure, WorkMeter, OPERATION_PATH,
    };
    use crate::checked_package::common::NODE_DOMAIN;
    use crate::checked_package::shared::{CheckedArtifactRef, CheckedRevision, CheckedSelection};
    use serde_json::json;
    use std::collections::BTreeMap;

    /// A catalogued identity used correctly throughout this module's tests:
    /// zero laws, no mode, no member, two plain-literal operands (so the
    /// unexercised operand-family/type machinery never needs a second graph
    /// node to resolve against).
    const CATALOGUED_IDENTITY: &str = "quire.op.integer.add";
    /// Obviously synthetic; must never collide with a real catalogued
    /// identity.
    const UNCATALOGUED_IDENTITY: &str = "quire.op.test-only.not-a-real-operation";

    fn dummy_digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    /// A minimal single-node `application` term whose `operation.identity`
    /// is `identity` and whose `operator` is `operator`. Arguments are plain
    /// `literal` terms (not `reference`/`binding`), so `argument_family`
    /// resolves them to `None` and `check_operands` skips the family checks
    /// that would otherwise need a second, referenced graph node — the only
    /// thing this fixture needs to reach is the catalog lookup inside
    /// `operation_defect`.
    fn application_node(identity: &str, operator: &str) -> CheckedSemanticNodeV2 {
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
                "arguments": [
                    { "term": "literal", "value": 1 },
                    { "term": "literal", "value": 2 },
                ],
                "operation": {
                    "identity": identity,
                    "laws": [],
                    "mode": null,
                    "member": null,
                    "leaves": [],
                },
            },
        });
        serde_json::from_value(value).expect("test fixture node is well-formed")
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
                role: Box::from("edition"),
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
        operation_defect(node, nodes, &index, &lock, &mut meter)
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
            Ok(Some(ValidationFailure::RefusedAt(
                CheckedPackageRefusalCode::InvalidPackage,
                OPERATION_PATH,
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
        assert_eq!(entry.operator.as_ref(), "binary");
        assert_eq!(entry.operands.len(), 2);

        let node = application_node(CATALOGUED_IDENTITY, "binary");

        let result = defect_for(&node);

        assert_eq!(
            result,
            Ok(None),
            "a catalogued identity used correctly must not be refused, got {result:?}"
        );
    }
}
