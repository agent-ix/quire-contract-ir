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
//! relation node in ascending node-id digest order. For one node the checks
//! run in this order and the first failing check is its defect:
//!
//! 0. **Shape.** The body root is not an application (`ill_typed`/
//!    `operator-ineligible` at the node, from the first step that reaches it)
//!    and the body is the closed shape; else `invalid_semantic_graph` at the
//!    offending array element, or at the body.
//! 1. **Identity.** `node_id` is the digest of `{version, body}` through
//!    `quire-canonical`, `semantic_type` is the node itself and `dependencies`
//!    are exactly the body's targets; else `invalid_semantic_graph` at the
//!    node.
//! 2. **Order.** The canonical order of every array; else
//!    `invalid_semantic_graph` at the array.
//! 3. **Targets,** in body order: the join to a declared dependency of the
//!    admitted kind, then, for a `type` or `context`, the owner recovery of
//!    FR-322 "Model-owned members" step 2.
//! 4. **Members,** in body order: field names, frame operations and their
//!    parameters, and the Rust spellings.
//!
//! Once every node's own checks hold, key uniqueness is checked over all
//! nodes, in ascending node-id digest order and body order.
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

use super::encode::AbstractionNodePreimage;
use super::frame::{join, StepGraph};
use super::identity::is_identifier;
use super::model_members::{MemberKind, ModelOwners, ModelRefusal};
use super::rust_spelling::{is_rust_field, is_rust_identifier, is_rust_path, is_rust_receiver};
use super::state::resolve_operation;
use super::{
    BodyTerm, CheckedNodeKind, CheckedSemanticNodeV2, CorrespondenceForm, ModelForm, RelationForm,
    WorkMeter,
};
use crate::checked_package::common::{
    body_term, exact_members, is_digest, node_pointer, validate_term, ReferenceVisitor,
    TermGrammar, Trail, ValidationFailure, NODE_DOMAIN,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// One `{name, rust_*}` binding: a `fields` entry (`rust_field`) or a
/// `parameters` entry (`rust_parameter`).
#[derive(Clone, Debug, Eq, PartialEq)]
struct Binding {
    name: Box<str>,
    rust: Box<str>,
}

/// An `objects` entry.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ObjectEntry {
    type_id: CheckedNodeId,
    rust_type: Vec<Box<str>>,
    fields: Vec<Binding>,
}

/// A `populations` entry.
#[derive(Clone, Debug, Eq, PartialEq)]
struct PopulationEntry {
    population: CheckedNodeId,
    collection: Vec<Box<str>>,
}

/// A `frames` entry.
#[derive(Clone, Debug, Eq, PartialEq)]
struct FrameEntry {
    context: CheckedNodeId,
    operation: Box<str>,
    function: Vec<Box<str>>,
    receiver: Box<str>,
    parameters: Vec<Binding>,
}

/// An abstraction relation body as the wire carries it, entries in wire
/// order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct AbstractionBody {
    objects: Vec<ObjectEntry>,
    populations: Vec<PopulationEntry>,
    frames: Vec<FrameEntry>,
}

impl AbstractionBody {
    /// Every `type`, `population` and `context` target: the unique,
    /// digest-ascending set a node's `dependencies` must equal.
    fn targets(&self) -> Vec<CheckedNodeId> {
        let objects = self.objects.iter().map(|entry| &entry.type_id);
        let populations = self.populations.iter().map(|entry| &entry.population);
        let contexts = self.frames.iter().map(|entry| &entry.context);
        objects
            .chain(populations)
            .chain(contexts)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .cloned()
            .collect()
    }
}

/// Strict wire validation of an abstraction relation node's body, run by the
/// per-node body loop: an application standing as a member value refuses, a
/// `case` application as `ill_typed`/`operator-ineligible` at its `operator`
/// and every other as `malformed_wire` at the application (FR-038 "The flat
/// wire"). An application at the body root is a term of the closed grammar and
/// is validated as one: the step that places it refuses it. Returns the work
/// done, one unit per value walked.
///
/// Implements: FR-346.
pub(super) fn scan_body(
    body: &Value,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    if body_term(body) == Some(BodyTerm::Application) {
        return validate_term(body, TermGrammar::V2, true, at, visit);
    }
    scan_members(body, at, visit)
}

/// Walks the values below `value` in document pre-order, refusing the first
/// application.
fn scan_members(
    value: &Value,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    let mut work = 1_u64;
    match value {
        Value::Object(members) => {
            for (key, member) in members {
                work = work.saturating_add(scan_member(member, &at.key(key), visit)?);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                work = work.saturating_add(scan_member(item, &at.index(index), visit)?);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
    Ok(work)
}

/// One member value: an application refuses, anything else is walked.
fn scan_member(
    value: &Value,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    if body_term(value) != Some(BodyTerm::Application) {
        return scan_members(value, at, visit);
    }
    // A malformed application, or a `case` one, is refused by the term walk
    // with its own code; any other nested application is outside the body.
    validate_term(value, TermGrammar::V2, false, at, visit)?;
    Err(ValidationFailure::refused(
        CheckedPackageRefusalCode::MalformedWire,
        at.pointer(),
    ))
}

/// A node key: in the node domain, with a lowercase digest. Any other value
/// is a body of the wrong shape, whatever the wire said about the domain.
fn node_key(value: &Value) -> Option<CheckedNodeId> {
    let id = CheckedNodeId::deserialize(value).ok()?;
    (id.domain.as_ref() == NODE_DOMAIN && is_digest(&id.digest)).then_some(id)
}

/// A non-empty string member.
fn text(object: &Map<String, Value>, key: &str) -> Option<Box<str>> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(Box::from)
}

/// An `Identifier` member.
fn identifier(object: &Map<String, Value>, key: &str) -> Option<Box<str>> {
    text(object, key).filter(|name| is_identifier(name))
}

/// A `RustPath` member: a non-empty array of non-empty strings. Whether each
/// is a Rust identifier is the members check's.
fn path(object: &Map<String, Value>, key: &str) -> Option<Vec<Box<str>>> {
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

/// An object holding exactly `members`, or the path of the value at fault.
fn closed<'v>(
    value: &'v Value,
    members: &[&str],
    at: &JsonPointer,
) -> Result<&'v Map<String, Value>, JsonPointer> {
    match value {
        Value::Object(object) if exact_members(object, members) => Ok(object),
        _ => Err(at.clone()),
    }
}

/// The array held in `object[key]`, or `at`.
fn array<'v>(
    object: &'v Map<String, Value>,
    key: &str,
    at: &JsonPointer,
) -> Result<&'v [Value], JsonPointer> {
    match object.get(key) {
        Some(Value::Array(items)) => Ok(items.as_slice()),
        _ => Err(at.clone()),
    }
}

/// The entries of `object[key]`, each read at its own element of `at`/`key`.
fn entries<'v, T>(
    object: &'v Map<String, Value>,
    key: &str,
    at: &JsonPointer,
    read: impl Fn(&'v Value, &JsonPointer) -> Result<T, JsonPointer>,
) -> Result<Vec<T>, JsonPointer> {
    let items = array(object, key, at)?;
    let at = at.clone().key(key);
    items
        .iter()
        .enumerate()
        .map(|(index, item)| read(item, &at.clone().index(index)))
        .collect()
}

/// One `{name, <rust>}` entry of `fields` or `parameters`.
fn read_binding(value: &Value, rust: &str, at: &JsonPointer) -> Result<Binding, JsonPointer> {
    let object = closed(value, &["name", rust], at)?;
    match (identifier(object, "name"), text(object, rust)) {
        (Some(name), Some(rust)) => Ok(Binding { name, rust }),
        _ => Err(at.clone()),
    }
}

fn read_object(value: &Value, at: &JsonPointer) -> Result<ObjectEntry, JsonPointer> {
    let object = closed(value, &["type", "rust_type", "fields"], at)?;
    let fail = || at.clone();
    Ok(ObjectEntry {
        type_id: object.get("type").and_then(node_key).ok_or_else(fail)?,
        rust_type: path(object, "rust_type").ok_or_else(fail)?,
        fields: entries(object, "fields", at, |field, field_at| {
            read_binding(field, "rust_field", field_at)
        })?,
    })
}

fn read_population(value: &Value, at: &JsonPointer) -> Result<PopulationEntry, JsonPointer> {
    let object = closed(value, &["population", "collection"], at)?;
    let fail = || at.clone();
    Ok(PopulationEntry {
        population: object
            .get("population")
            .and_then(node_key)
            .ok_or_else(fail)?,
        collection: path(object, "collection").ok_or_else(fail)?,
    })
}

fn read_frame(value: &Value, at: &JsonPointer) -> Result<FrameEntry, JsonPointer> {
    let object = closed(
        value,
        &["context", "operation", "function", "receiver", "parameters"],
        at,
    )?;
    let fail = || at.clone();
    Ok(FrameEntry {
        context: object.get("context").and_then(node_key).ok_or_else(fail)?,
        operation: identifier(object, "operation").ok_or_else(fail)?,
        function: path(object, "function").ok_or_else(fail)?,
        receiver: text(object, "receiver").ok_or_else(fail)?,
        parameters: entries(object, "parameters", at, |parameter, parameter_at| {
            read_binding(parameter, "rust_parameter", parameter_at)
        })?,
    })
}

/// Reads the closed body shape of FR-346 at `at`; an `Err` is the path of the
/// value at fault: the array element of `objects`, `fields`, `populations`,
/// `frames` or `parameters`, else the body.
fn read_body(body: &Value, at: &JsonPointer) -> Result<AbstractionBody, JsonPointer> {
    let object = closed(body, &["term", "objects", "populations", "frames"], at)?;
    if body_term(body) != Some(BodyTerm::AbstractionRelation) {
        return Err(at.clone());
    }
    Ok(AbstractionBody {
        objects: entries(object, "objects", at, read_object)?,
        populations: entries(object, "populations", at, read_population)?,
        frames: entries(object, "frames", at, read_frame)?,
    })
}

/// A model refusal at `path`, located at the key `locus`.
fn model_refusal(
    refusal: ModelRefusal,
    path: JsonPointer,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    ValidationFailure::refused_at(refusal.code, path, Some(refusal.cause), locus.clone())
}

/// `invalid_model_binding`/`conflicting-binding` at `path`.
fn conflict(path: JsonPointer, locus: &CheckedNodeId) -> ValidationFailure {
    ValidationFailure::refused_at(
        CheckedPackageRefusalCode::InvalidModelBinding,
        path,
        Some(CheckedPackageRefusalCause::ConflictingBinding),
        locus.clone(),
    )
}

/// The one node a step's checks are about, with its position and the
/// pointers its refusals use.
struct Site<'a> {
    node: &'a CheckedSemanticNodeV2,
    position: usize,
}

impl Site<'_> {
    fn node_at(&self) -> JsonPointer {
        node_pointer(self.position)
    }

    fn body_at(&self) -> JsonPointer {
        self.node_at().key("body")
    }

    /// `invalid_semantic_graph`, no cause, at `path`, located at the node.
    fn invalid(&self, path: JsonPointer) -> ValidationFailure {
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
    index: &BTreeMap<&CheckedNodeId, usize>,
    owners: &ModelOwners<'_>,
    bytes: u64,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = StepGraph {
        nodes,
        kinds,
        index,
        owners,
    };
    let mut admitted = Vec::new();
    for (position, node) in graph.each(CheckedNodeKind::Correspondence(
        CorrespondenceForm::AbstractionRelation,
    )) {
        let site = Site { node, position };
        let body = check_node(&site, &graph, bytes, meter)?;
        admitted.push((position, body));
    }
    check_keys(&admitted)
}

/// One node's own checks, shape through members, returning its body.
fn check_node(
    site: &Site<'_>,
    graph: &StepGraph<'_, '_>,
    bytes: u64,
    meter: &mut WorkMeter,
) -> Result<AbstractionBody, ValidationFailure> {
    let node = site.node;
    // An application at the body root is the closed term grammar's, not this
    // body's: the temporal and state steps refused it already where they place
    // it, and the shape check opening this step refuses the rest.
    if body_term(&node.body) == Some(BodyTerm::Application) {
        return Err(ValidationFailure::refused_at(
            CheckedPackageRefusalCode::IllTyped,
            site.node_at(),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            node.node_id.clone(),
        ));
    }
    let body = read_body(&node.body, &site.body_at()).map_err(|path| site.invalid(path))?;
    check_identity(site, &body, bytes)?;
    check_order(site, &body)?;
    check_targets(site, &body, graph)?;
    check_members(site, &body, graph, meter)?;
    Ok(body)
}

/// Step 1: the node key is the digest of the preimage, the node is its own
/// semantic type, and its dependencies are the body's targets.
fn check_identity(
    site: &Site<'_>,
    body: &AbstractionBody,
    bytes: u64,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    let computed = quire_canonical::sha256(
        &AbstractionNodePreimage { body: &node.body },
        quire_canonical::Limits::new(bytes),
    )
    .map_err(|_| site.invalid(site.node_at()))?;
    if computed.to_string() == node.node_id.digest.as_ref()
        && node.semantic_type == node.node_id
        && node.dependencies == body.targets()
    {
        Ok(())
    } else {
        Err(site.invalid(site.node_at()))
    }
}

/// Whether `keys` ascend, equal keys in order.
fn ascending<K: Ord>(keys: impl Iterator<Item = K>) -> bool {
    let keys = keys.collect::<Vec<_>>();
    keys.windows(2).all(|pair| pair[0] <= pair[1])
}

/// Step 2: every array in its canonical order. A non-strict order: equal keys
/// are in order, and the member and uniqueness checks refuse them.
fn check_order(site: &Site<'_>, body: &AbstractionBody) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let out_of_order = |array_at: JsonPointer| Err(site.invalid(array_at));
    let objects_at = body_at.clone().key("objects");
    if !ascending(body.objects.iter().map(|entry| &entry.type_id.digest)) {
        return out_of_order(objects_at);
    }
    for (index, entry) in body.objects.iter().enumerate() {
        if !ascending(entry.fields.iter().map(|field| field.name.as_ref())) {
            return out_of_order(objects_at.clone().index(index).key("fields"));
        }
    }
    if !ascending(
        body.populations
            .iter()
            .map(|entry| &entry.population.digest),
    ) {
        return out_of_order(body_at.key("populations"));
    }
    let frames_at = body_at.key("frames");
    let frame_key = |entry: &FrameEntry| (entry.context.digest.clone(), entry.operation.clone());
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
fn is_object_type(kind: CheckedNodeKind) -> bool {
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
    entry: &ObjectEntry,
    at: &JsonPointer,
    graph: &StepGraph<'_, '_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let fields_at = at.clone().key("fields");
    let mut names = BTreeSet::new();
    if let Some(repeat) = entry
        .fields
        .iter()
        .position(|field| !names.insert(field.name.as_ref()))
    {
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
    if !is_rust_path(&entry.rust_type) {
        return Err(malformed(at.clone()));
    }
    match entry
        .fields
        .iter()
        .position(|field| !is_rust_field(&field.rust))
    {
        Some(index) => Err(malformed(fields_at.index(index))),
        None => Ok(()),
    }
}

/// A frame entry: the operation's resolution, its parameter list, the
/// Rust parameters against each other and the receiver, then the Rust
/// spellings.
fn check_frame(
    entry: &FrameEntry,
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
                let mut declared = operation
                    .parameters
                    .iter()
                    .map(|slot| slot.name.as_deref())
                    .collect::<Option<Vec<_>>>()
                    .unwrap_or_default();
                declared.sort_unstable();
                let every_name = operation.parameters.iter().all(|slot| slot.name.is_some());
                let bound = entry
                    .parameters
                    .iter()
                    .map(|parameter| parameter.name.as_ref());
                if !(every_name && declared.iter().copied().eq(bound)) {
                    return Err(malformed(at.clone()));
                }
            }
            // A context that is not a model declaration node: its operation
            // is not resolved against any domain package.
            Ok(None) => {}
        }
    }
    let mut seen = BTreeSet::new();
    if let Some(repeat) = entry
        .parameters
        .iter()
        .position(|parameter| !seen.insert(parameter.rust.as_ref()))
    {
        return Err(malformed(parameters_at.index(repeat)));
    }
    if let Some(clash) = entry
        .parameters
        .iter()
        .position(|parameter| parameter.rust == entry.receiver)
    {
        return Err(malformed(parameters_at.index(clash)));
    }
    if !is_rust_path(&entry.function) || !is_rust_receiver(&entry.receiver) {
        return Err(malformed(at.clone()));
    }
    match entry
        .parameters
        .iter()
        .position(|parameter| !is_rust_identifier(&parameter.rust))
    {
        Some(index) => Err(malformed(parameters_at.index(index))),
        None => Ok(()),
    }
}

/// Package-wide key uniqueness over every admitted node, in ascending
/// node-id digest order and body order: the second entry to claim an object,
/// population or operation key refuses `invalid_model_binding`/
/// `conflicting-binding`.
fn check_keys(admitted: &[(usize, AbstractionBody)]) -> Result<(), ValidationFailure> {
    let mut objects = BTreeSet::new();
    let mut populations = BTreeSet::new();
    let mut operations = BTreeSet::new();
    for (position, body) in admitted {
        let body_at = node_pointer(*position).key("body");
        for (index, entry) in body.objects.iter().enumerate() {
            if !objects.insert(&entry.type_id) {
                let at = body_at.clone().key("objects").index(index);
                return Err(conflict(at, &entry.type_id));
            }
        }
        for (index, entry) in body.populations.iter().enumerate() {
            if !populations.insert(&entry.population) {
                let at = body_at.clone().key("populations").index(index);
                return Err(conflict(at, &entry.population));
            }
        }
        for (index, entry) in body.frames.iter().enumerate() {
            if !operations.insert((&entry.context, entry.operation.as_ref())) {
                let at = body_at.clone().key("frames").index(index);
                return Err(conflict(at, &entry.context));
            }
        }
    }
    Ok(())
}
