//! FR-346: the abstraction relation body of QSpec FR-451, and the abstraction
//! step of the reader order.
//!
//! A `correspondence`/`abstraction_relation` node's `body` is the flat shape
//! `{term: "abstraction_relation", objects, populations, frames}`: node keys,
//! identifiers and Rust strings, no application, aggregate, binding or
//! literal term. The per-node body loop does not read it as a semantic term.
//! [`scan_body`] is that loop's strict wire validation of it: an application
//! standing as a member value refuses there, a `case` application at its
//! `operator` and any other at the application. The typed read itself is the
//! first check of the abstraction step.
//!
//! [`validate_abstraction`] is the abstraction step: after the frame, state
//! and temporal steps and before the operation step, over every abstraction
//! relation node in ascending structural node-id order. For one node the checks
//! run in this order and the first failing check is its defect:
//!
//! 0. **Shape.** The body root is not an application (`ill_typed`/
//!    `operator-ineligible` at the node, from the first step that reaches it)
//!    and the body is the closed shape; else `invalid_semantic_graph` at the
//!    offending array element, or at the body.
//! 1. **Identity.** `semantic_type` is the node itself and `dependencies`
//!    are exactly the body's targets; else `invalid_semantic_graph` at the node.
//! 2. **Order.** The canonical order of every array; else
//!    `invalid_semantic_graph` at the array.
//! 3. **Targets,** in body order: the join to a declared dependency of the
//!    admitted kind, then, for a `type` or `context`, the owner recovery of
//!    FR-322 "Model-owned members" step 2.
//! 4. **Members,** in body order: field names, frame operations and their
//!    parameters, and the Rust spellings.
//!
//! Once every node's own checks hold, key uniqueness is checked over all
//! nodes, in ascending structural node-id order and body order.
//!
//! Every refusal of steps 3 and 4 and of the uniqueness check carries the
//! entry's path and, as its locus, the node key of the entry's `type`,
//! `population` or `context` target, as a frame or anchor refusal carries the
//! declaration it is about. A refusal carries one path and one locus, so a
//! duplicate does not carry the first entry it repeats.
//!
//! Names resolve through the selected domain package only when the target is
//! a model declaration node, the scope FR-038's "Model-owned members" gives
//! the resolution; a target that carries its own `declaration` is declared by
//! this package's source and is joined by kind alone.

use super::flat_wire::scan_members;
use super::frame::{join, StepGraph};
use super::graph_storage::GraphIndex;
use super::identity::is_identifier;
use super::model_members::{MemberKind, ModelOwners, ModelRefusal};
use super::rust_spelling::{is_rust_field, is_rust_identifier, is_rust_path, is_rust_receiver};
use super::state::resolve_operation;
use super::{
    BodyTerm, CheckedAbstractionFrame, CheckedAbstractionObject, CheckedAbstractionPopulation,
    CheckedNodeKind, CheckedTerm, CorrespondenceForm, ModelForm,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, RelationForm, WorkMeter,
};
use crate::checked_package::common::{
    exact_members, node_pointer, validate_term, Ids, ReferenceVisitor, TermGrammar, Trail,
    ValidationFailure,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use crate::checked_package::terms::TermSource;
use serde_json::{Map, Value};

/// One `{name, rust_*}` binding: a `fields` entry (`rust_field`) or a
/// `parameters` entry (`rust_parameter`).
/// An abstraction relation body as the wire carries it, entries in wire
/// order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AbstractionBody {
    objects: Vec<CheckedAbstractionObject>,
    populations: Vec<CheckedAbstractionPopulation>,
    frames: Vec<CheckedAbstractionFrame>,
}

impl AbstractionBody {
    /// Every `type`, `population` and `context` target: the unique,
    /// structural-ID-ascending set a node's `dependencies` must equal.
    fn targets(&self, meter: &mut WorkMeter) -> Result<Vec<CheckedNodeId>, ValidationFailure> {
        let count = self.objects.len() + self.populations.len() + self.frames.len();
        let mut targets = meter.vec(count)?;
        for id in self
            .objects
            .iter()
            .map(|entry| &entry.type_id)
            .chain(self.populations.iter().map(|entry| &entry.population))
            .chain(self.frames.iter().map(|entry| &entry.context))
        {
            if let Err(position) = targets.binary_search(id) {
                let id = meter.clone_id(id)?;
                targets.insert(position, id);
            }
        }
        Ok(targets)
    }
}

/// Strict wire validation of an abstraction relation node's body, run by the
/// per-node body loop: an application standing as a member value refuses, a
/// `case` application as `ill_typed`/`operator-ineligible` at its `operator`
/// and every other as `malformed_wire` at the application (FR-038 "The flat
/// wire"; the same refusal [`super::flat_wire`] makes ahead of every identity
/// check). An application at the body root is a term of the closed grammar and
/// is validated as one: the step that places it refuses it. Returns the work
/// done, one unit per value walked.
///
/// Implements: FR-346.
pub(super) fn scan_body<'a, V: TermSource<'a>>(
    body: V,
    at: &Trail<'a>,
    visit: &mut ReferenceVisitor<'_>,
    ids: Ids<'_, '_>,
) -> Result<u64, ValidationFailure> {
    if body.kind() == Some(BodyTerm::Application) {
        return validate_term(body, TermGrammar::V2, true, at, visit, ids);
    }
    let Ids { sink, site, .. } = ids;
    let cell = super::read_charge::SinkCell::new(sink);
    scan_members(body, at, &mut cell.reservation(site))
}

/// Shared shape helpers used by other flat correspondence readers.
pub(super) struct Defect {
    pub(super) code: CheckedPackageRefusalCode,
    pub(super) path: JsonPointer,
}

impl Defect {
    pub(super) fn invalid(path: &JsonPointer) -> Self {
        Self {
            code: CheckedPackageRefusalCode::InvalidSemanticGraph,
            path: path.clone(),
        }
    }
}

#[ix_trace_rs::string_edge]
pub(super) fn node_key(
    object: &Map<String, Value>,
    key: &str,
    at: &JsonPointer,
) -> Result<CheckedNodeId, Defect> {
    let Some(value @ Value::Object(_)) = object.get(key) else {
        return Err(Defect::invalid(at));
    };
    match CheckedNodeId::from_value(value) {
        Some(id) if id.is_well_formed() => Ok(id),
        _ => Err(Defect {
            code: CheckedPackageRefusalCode::MalformedWire,
            path: at.clone().key(key),
        }),
    }
}

pub(super) fn text(object: &Map<String, Value>, key: &str) -> Option<Box<str>> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(Box::from)
}

pub(super) fn identifier(object: &Map<String, Value>, key: &str) -> Option<Box<str>> {
    text(object, key).filter(|name| is_identifier(name))
}

pub(super) fn path(object: &Map<String, Value>, key: &str) -> Option<Vec<Box<str>>> {
    let segments = object
        .get(key)?
        .as_array()?
        .iter()
        .map(|segment| {
            segment
                .as_str()
                .filter(|text| !text.is_empty())
                .map(Box::from)
        })
        .collect::<Option<Vec<_>>>()?;
    (!segments.is_empty()).then_some(segments)
}

pub(super) fn closed<'v>(
    value: &'v Value,
    members: &[&str],
    at: &JsonPointer,
) -> Result<&'v Map<String, Value>, Defect> {
    match value {
        Value::Object(object) if exact_members(object, members) => Ok(object),
        _ => Err(Defect::invalid(at)),
    }
}

pub(super) fn array<'v>(
    object: &'v Map<String, Value>,
    key: &str,
    at: &JsonPointer,
) -> Result<&'v [Value], Defect> {
    match object.get(key) {
        Some(Value::Array(items)) => Ok(items.as_slice()),
        _ => Err(Defect::invalid(at)),
    }
}

pub(super) fn entries<'v, T>(
    object: &'v Map<String, Value>,
    key: &str,
    at: &JsonPointer,
    read: impl Fn(&'v Value, &JsonPointer) -> Result<T, Defect>,
) -> Result<Vec<T>, Defect> {
    let items = array(object, key, at)?;
    let at = at.clone().key(key);
    items
        .iter()
        .enumerate()
        .map(|(i, item)| read(item, &at.clone().index(i)))
        .collect()
}

/// A model refusal at `path`, located at the key `locus`.
pub(super) fn model_refusal(
    refusal: ModelRefusal,
    path: JsonPointer,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    ValidationFailure::refused_at(refusal.code, path, Some(refusal.cause), locus.clone())
}

/// `invalid_model_binding`/`conflicting-binding` at `path`.
pub(super) fn conflict(path: JsonPointer, locus: &CheckedNodeId) -> ValidationFailure {
    ValidationFailure::refused_at(
        CheckedPackageRefusalCode::InvalidModelBinding,
        path,
        Some(CheckedPackageRefusalCause::ConflictingBinding),
        locus.clone(),
    )
}

/// The one node a step's checks are about, with its position and the
/// pointers its refusals use.
pub(super) struct Site<'a> {
    pub(super) node: &'a CheckedSemanticNodeV2,
    pub(super) position: usize,
}

impl Site<'_> {
    pub(super) fn node_at(&self) -> JsonPointer {
        node_pointer(self.position)
    }

    pub(super) fn body_at(&self) -> JsonPointer {
        self.node_at().key("body")
    }

    /// `invalid_semantic_graph`, no cause, at `path`, located at the node.
    pub(super) fn invalid(&self, path: JsonPointer) -> ValidationFailure {
        ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            path,
            None,
            self.node.node_id.clone(),
        )
    }
}

/// The abstraction step. See the module documentation for the order.
///
/// Implements: FR-346.
pub(super) fn validate_abstraction(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = StepGraph {
        nodes,
        kinds,
        index,
        owners,
    };
    let count = kinds
        .iter()
        .filter(|kind| {
            **kind == CheckedNodeKind::Correspondence(CorrespondenceForm::AbstractionRelation)
        })
        .count();
    let mut admitted = meter.vec(count)?;
    for (position, node) in graph.each(CheckedNodeKind::Correspondence(
        CorrespondenceForm::AbstractionRelation,
    )) {
        let site = Site { node, position };
        let body = check_node(&site, &graph, meter)?;
        meter.push(&mut admitted, (position, body))?;
    }
    check_keys(&admitted)
}

/// One node's own checks, shape through members, returning its body.
fn check_node(
    site: &Site<'_>,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<AbstractionBody, ValidationFailure> {
    let node = site.node;
    // An application at the body root is the closed term grammar's, not this
    // body's: the temporal and state steps refused it already where they place
    // it, and the shape check opening this step refuses the rest.
    if super::TermNode::of(&node.body).kind() == Some(BodyTerm::Application) {
        return Err(ValidationFailure::refused_at(
            CheckedPackageRefusalCode::IllTyped,
            site.node_at(),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            node.node_id.clone(),
        ));
    }
    let decoded = meter.decode_body(site.position)?;
    let decoded = decoded.map_err(|_| {
        ValidationFailure::refused_at(
            CheckedPackageRefusalCode::MalformedWire,
            site.body_at(),
            None,
            node.node_id.clone(),
        )
    })?;
    let CheckedTerm::AbstractionRelation {
        objects,
        populations,
        frames,
    } = decoded
    else {
        return Err(site.invalid(site.body_at()));
    };
    let body = AbstractionBody {
        objects,
        populations,
        frames,
    };
    check_identity(site, &body, meter)?;
    check_order(site, &body)?;
    check_targets(site, &body, graph)?;
    check_members(site, &body, graph, meter)?;
    Ok(body)
}

/// Step 1: the node is its own semantic type and its dependencies are the
/// body's targets.
fn check_identity(
    site: &Site<'_>,
    body: &AbstractionBody,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    if node.semantic_type == node.node_id && node.dependencies == body.targets(meter)? {
        Ok(())
    } else {
        Err(site.invalid(site.node_at()))
    }
}

/// Whether `keys` ascend, equal keys in order.
pub(super) fn ascending<K: Ord>(keys: impl Iterator<Item = K>) -> bool {
    let mut previous = None;
    for key in keys {
        if previous.as_ref().is_some_and(|before| before > &key) {
            return false;
        }
        previous = Some(key);
    }
    true
}

/// Step 2: every array in its canonical order. A non-strict order: equal keys
/// are in order, and the member and uniqueness checks refuse them.
fn check_order(site: &Site<'_>, body: &AbstractionBody) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let out_of_order = |array_at: JsonPointer| Err(site.invalid(array_at));
    let objects_at = body_at.clone().key("objects");
    if !ascending(body.objects.iter().map(|entry| &entry.type_id)) {
        return out_of_order(objects_at);
    }
    for (index, entry) in body.objects.iter().enumerate() {
        if !ascending(entry.fields.iter().map(|field| field.name.as_ref())) {
            return out_of_order(objects_at.clone().index(index).key("fields"));
        }
    }
    if !ascending(body.populations.iter().map(|entry| &entry.population)) {
        return out_of_order(body_at.key("populations"));
    }
    let frames_at = body_at.key("frames");
    fn frame_key(entry: &CheckedAbstractionFrame) -> (&CheckedNodeId, &str) {
        (&entry.context, entry.operation.as_ref())
    }
    if !ascending(body.frames.iter().map(frame_key)) {
        return out_of_order(frames_at);
    }
    for (index, entry) in body.frames.iter().enumerate() {
        if !ascending(
            entry
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_ref()),
        ) {
            return out_of_order(frames_at.clone().index(index).key("parameters"));
        }
    }
    Ok(())
}

/// A `type` or `context` target: a `model`/`object_type` node.
pub(super) fn is_object_type(kind: CheckedNodeKind) -> bool {
    kind == CheckedNodeKind::Model(ModelForm::ObjectType)
}

/// A `population` target: a `relation`/`population` node.
fn is_population(kind: CheckedNodeKind) -> bool {
    kind == CheckedNodeKind::Relation(RelationForm::Population)
}

/// Step 3: every target joins a declared dependency of its admitted kind and,
/// for an object type that is a model declaration node, recovers its owner.
fn check_targets(
    site: &Site<'_>,
    body: &AbstractionBody,
    graph: &StepGraph<'_, '_>,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let target = |entry_at: JsonPointer,
                  id: &CheckedNodeId,
                  admits: fn(CheckedNodeKind) -> bool,
                  owned: bool| {
        let (declaring, kind) = join(site.node, id, admits, graph)
            .map_err(|refusal| model_refusal(refusal, entry_at.clone(), id))?;
        if owned
            && graph
                .owners
                .is_model_declaration_node(declaring, kind.tag())
        {
            graph
                .owners
                .recover(declaring)
                .map_err(|refusal| model_refusal(refusal, entry_at, id))?;
        }
        Ok(())
    };
    for (index, entry) in body.objects.iter().enumerate() {
        let at = body_at.clone().key("objects").index(index);
        target(at, &entry.type_id, is_object_type, true)?;
    }
    for (index, entry) in body.populations.iter().enumerate() {
        let at = body_at.clone().key("populations").index(index);
        target(at, &entry.population, is_population, false)?;
    }
    for (index, entry) in body.frames.iter().enumerate() {
        let at = body_at.clone().key("frames").index(index);
        target(at, &entry.context, is_object_type, true)?;
    }
    Ok(())
}

/// Step 4: the members of every entry, entries in body order.
fn check_members(
    site: &Site<'_>,
    body: &AbstractionBody,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let malformed = |at: JsonPointer, locus: &CheckedNodeId| {
        model_refusal(ModelRefusal::malformed(), at, locus)
    };
    for (index, entry) in body.objects.iter().enumerate() {
        let at = body_at.clone().key("objects").index(index);
        check_object(entry, &at, graph, meter)?;
    }
    for (index, entry) in body.populations.iter().enumerate() {
        if !is_rust_path(&entry.collection) {
            let at = body_at.clone().key("populations").index(index);
            return Err(malformed(at, &entry.population));
        }
    }
    for (index, entry) in body.frames.iter().enumerate() {
        let at = body_at.clone().key("frames").index(index);
        check_frame(entry, &at, graph, meter)?;
    }
    Ok(())
}

/// An object entry: a repeated field name, a name the object type does not
/// expose, then the Rust spellings.
fn check_object(
    entry: &CheckedAbstractionObject,
    at: &JsonPointer,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let fields_at = at.clone().key("fields");
    if let Some(repeat) = entry.fields.iter().enumerate().find_map(|(index, field)| {
        entry.fields[..index]
            .iter()
            .any(|previous| previous.name == field.name)
            .then_some(index)
    }) {
        return Err(conflict(fields_at.index(repeat), &entry.type_id));
    }
    if let Some((declaring, kind)) = graph.node(&entry.type_id) {
        if graph
            .owners
            .is_model_declaration_node(declaring, kind.tag())
        {
            for (index, field) in entry.fields.iter().enumerate() {
                let resolved = graph.owners.resolve_member(
                    declaring,
                    MemberKind::Field,
                    &field.name,
                    meter,
                )?;
                if let Err(refusal) = resolved {
                    // A name no field of the type carries is a malformed
                    // binding here, where an anchor's operation is a missing
                    // one; two exposed fields of one name stay ambiguous.
                    let refusal = if refusal == ModelRefusal::ineligible() {
                        ModelRefusal::malformed()
                    } else {
                        refusal
                    };
                    return Err(model_refusal(
                        refusal,
                        fields_at.index(index),
                        &entry.type_id,
                    ));
                }
            }
        }
    }
    let malformed = |at: JsonPointer| model_refusal(ModelRefusal::malformed(), at, &entry.type_id);
    if !is_rust_path(&entry.type_path) {
        return Err(malformed(at.clone()));
    }
    match entry
        .fields
        .iter()
        .position(|field| !is_rust_field(&field.field_ident))
    {
        Some(index) => Err(malformed(fields_at.index(index))),
        None => Ok(()),
    }
}

/// A frame entry: the operation's resolution, its parameter list, the
/// Rust parameters against each other and the receiver, then the Rust
/// spellings.
fn check_frame(
    entry: &CheckedAbstractionFrame,
    at: &JsonPointer,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let parameters_at = at.clone().key("parameters");
    let malformed = |at: JsonPointer| model_refusal(ModelRefusal::malformed(), at, &entry.context);
    if let Some((context, kind)) = graph.node(&entry.context) {
        match resolve_operation(context, kind, &entry.operation, graph, meter)? {
            Err(refusal) => return Err(model_refusal(refusal, at.clone(), &entry.context)),
            Ok(Some((_, operation))) => {
                // Domain selection validated the declared names (FR-154-AC-14);
                // compare the frame's list with that admitted declaration.
                let exact = operation.parameters.len() == entry.parameters.len()
                    && entry.parameters.iter().all(|parameter| {
                        operation
                            .parameters
                            .iter()
                            .any(|slot| slot.name.as_deref() == Some(parameter.name.as_ref()))
                    });
                if !exact {
                    return Err(malformed(at.clone()));
                }
            }
            // A context that is not a model declaration node: its operation
            // is not resolved against any domain package.
            Ok(None) => {}
        }
    }
    if let Some(repeat) = entry
        .parameters
        .iter()
        .enumerate()
        .find_map(|(index, parameter)| {
            entry.parameters[..index]
                .iter()
                .any(|previous| previous.parameter_ident == parameter.parameter_ident)
                .then_some(index)
        })
    {
        return Err(malformed(parameters_at.index(repeat)));
    }
    if let Some(clash) = entry
        .parameters
        .iter()
        .position(|parameter| parameter.parameter_ident == entry.receiver)
    {
        return Err(malformed(parameters_at.index(clash)));
    }
    if !is_rust_path(&entry.function) || !is_rust_receiver(&entry.receiver) {
        return Err(malformed(at.clone()));
    }
    match entry
        .parameters
        .iter()
        .position(|parameter| !is_rust_identifier(&parameter.parameter_ident))
    {
        Some(index) => Err(malformed(parameters_at.index(index))),
        None => Ok(()),
    }
}

/// Package-wide key uniqueness over every admitted node, in ascending
/// structural node-id order and body order: the second entry to claim an object,
/// population or operation key refuses `invalid_model_binding`/
/// `conflicting-binding`.
fn check_keys(admitted: &[(usize, AbstractionBody)]) -> Result<(), ValidationFailure> {
    for (body_index, (position, body)) in admitted.iter().enumerate() {
        let body_at = node_pointer(*position).key("body");
        for (index, entry) in body.objects.iter().enumerate() {
            let repeated = admitted[..body_index].iter().any(|(_, previous)| {
                previous
                    .objects
                    .iter()
                    .any(|item| item.type_id == entry.type_id)
            }) || body.objects[..index]
                .iter()
                .any(|item| item.type_id == entry.type_id);
            if repeated {
                let at = body_at.clone().key("objects").index(index);
                return Err(conflict(at, &entry.type_id));
            }
        }
        for (index, entry) in body.populations.iter().enumerate() {
            let repeated = admitted[..body_index].iter().any(|(_, previous)| {
                previous
                    .populations
                    .iter()
                    .any(|item| item.population == entry.population)
            }) || body.populations[..index]
                .iter()
                .any(|item| item.population == entry.population);
            if repeated {
                let at = body_at.clone().key("populations").index(index);
                return Err(conflict(at, &entry.population));
            }
        }
        for (index, entry) in body.frames.iter().enumerate() {
            let repeated = admitted[..body_index].iter().any(|(_, previous)| {
                previous
                    .frames
                    .iter()
                    .any(|item| item.context == entry.context && item.operation == entry.operation)
            }) || body.frames[..index]
                .iter()
                .any(|item| item.context == entry.context && item.operation == entry.operation);
            if repeated {
                let at = body_at.clone().key("frames").index(index);
                return Err(conflict(at, &entry.context));
            }
        }
    }
    Ok(())
}
