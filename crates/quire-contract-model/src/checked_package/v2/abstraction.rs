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
use super::native_join_budget::PackageReadStage;
use super::rust_spelling::{is_rust_field, is_rust_identifier, is_rust_path, is_rust_receiver};
use super::stage_storage::{self, Path, Refusal, Site as ChargeSite};
use super::state::resolve_operation;
use super::term_view::TermNode;
use super::{
    BodyTerm, CheckedNodeKind, CheckedSemanticNodeV2, CorrespondenceForm, ModelForm, RelationForm,
    WorkMeter,
};
use crate::checked_package::common::{
    validate_term, Ids, ReferenceVisitor, TermGrammar, Trail, ValidationFailure,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
};
use crate::checked_package::terms::TermSource;

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
    /// structural-ID-ascending set a node's `dependencies` must equal.
    fn targets(&self, meter: &mut WorkMeter) -> Result<Vec<&CheckedNodeId>, ValidationFailure> {
        let mut targets =
            meter.vec(self.objects.len() + self.populations.len() + self.frames.len())?;
        targets.extend(
            self.objects
                .iter()
                .map(|entry| &entry.type_id)
                .chain(self.populations.iter().map(|entry| &entry.population))
                .chain(self.frames.iter().map(|entry| &entry.context)),
        );
        targets.sort_unstable();
        targets.dedup();
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

/// The small marker propagated after the parser recorded its first fault.
pub(super) struct Defect;
/// A completed fault held on the stack, never boxed merely to propagate it.
pub(super) struct Fault {
    pub(super) code: CheckedPackageRefusalCode,
    pub(super) path: Path<'static>,
    pub(super) budget: Option<ValidationFailure>,
}
/// The original row and actual caller meter used by every owned projection.
pub(super) struct Parse<'a, 'b> {
    pub(super) meter: std::cell::RefCell<&'a mut WorkMeter<'b>>,
    pub(super) row: &'a [super::CheckedOperandEntry],
    pub(super) position: usize,
    denied: std::cell::RefCell<Option<ValidationFailure>>,
    shape: std::cell::RefCell<Option<(CheckedPackageRefusalCode, Path<'static>)>>,
}
impl<'a, 'b> Parse<'a, 'b> {
    pub(super) fn new(meter: &'a mut WorkMeter<'b>, row: &'a [super::CheckedOperandEntry], position: usize) -> Self {
        Self {
            meter: std::cell::RefCell::new(meter),
            row,
            position,
            denied: std::cell::RefCell::new(None),
            shape: std::cell::RefCell::new(None),
        }
    }
    pub(super) fn copy(&self, text: &str) -> Option<Box<str>> {
        if self.denied.borrow().is_some() {
            return None;
        }
        match self.meter.borrow_mut().boxed_str(text) {
            Ok(text) => Some(text),
            Err(stop) => {
                self.budget(stop);
                None
            }
        }
    }
    pub(super) fn invalid(&self, path: &Path<'static>) -> Defect {
        self.invalid_code(CheckedPackageRefusalCode::InvalidSemanticGraph, *path)
    }
    fn invalid_code(&self, code: CheckedPackageRefusalCode, path: Path<'static>) -> Defect {
        if self.shape.borrow().is_none() {
            *self.shape.borrow_mut() = Some((code, path));
        }
        Defect
    }
    fn budget(&self, stop: ValidationFailure) -> Defect {
        if self.denied.borrow().is_none() {
            *self.denied.borrow_mut() = Some(stop);
        }
        Defect
    }
    pub(super) fn finish(&self, _marker: Defect) -> Fault {
        if let Some(stop) = self.denied.borrow_mut().take() {
            Fault {
                code: CheckedPackageRefusalCode::InvalidSemanticGraph,
                path: Path::root(),
                budget: Some(stop),
            }
        } else {
            let (code, path) = self
                .shape
                .borrow_mut()
                .take()
                .expect("shape marker has its recorded fault");
            Fault {
                code,
                path,
                budget: None,
            }
        }
    }
    fn vec<T>(&self, count: usize) -> Result<Vec<T>, Defect> {
        self.meter
            .borrow_mut()
            .vec(count)
            .map_err(|stop| self.budget(stop))
    }
}

pub(super) fn node_key(
    object: TermNode<'_>,
    key: &'static str,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<CheckedNodeId, Defect> {
    let Some(value) = object.get(key).filter(|value| value.is_object()) else {
        return Err(ctx.invalid(at));
    };
    let mut meter = ctx.meter.borrow_mut();
    match value
        .node_id_in(
            &mut **meter,
            super::read_charge::Site::new(PackageReadStage::Graph, ctx.position, "correspondence"),
            ctx.row,
        )
        .map_err(|_| ctx.budget(super::read_charge::ReadStorage::denial(&**meter)))?
    {
        super::term_view::NodeIdRead::Id(id) => Ok(id),
        _ => Err(ctx.invalid_code(CheckedPackageRefusalCode::MalformedWire, at.key(key))),
    }
}
pub(super) fn text(object: TermNode<'_>, key: &str, ctx: &Parse<'_, '_>) -> Option<Box<str>> {
    ctx.copy(object.get(key)?.as_str().filter(|text| !text.is_empty())?)
}
pub(super) fn identifier(object: TermNode<'_>, key: &str, ctx: &Parse<'_, '_>) -> Option<Box<str>> {
    text(object, key, ctx).filter(|name| is_identifier(name))
}
pub(super) fn path(object: TermNode<'_>, key: &str, ctx: &Parse<'_, '_>) -> Option<Vec<Box<str>>> {
    let items = object.get(key)?.items()?;
    items.clone().next()?;
    let mut result = match ctx.vec(items.clone().count()) {
        Ok(v) => v,
        Err(_) => return None,
    };
    for item in items {
        result.push(ctx.copy(item.as_str().filter(|text| !text.is_empty())?)?);
    }
    Some(result)
}
pub(super) fn closed<'d>(
    value: TermNode<'d>,
    members: &[&str],
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<TermNode<'d>, Defect> {
    if value.exact_members(members) {
        Ok(value)
    } else {
        Err(ctx.invalid(at))
    }
}
pub(super) fn array<'d>(
    object: TermNode<'d>,
    key: &str,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<super::term_view::TermItems<'d>, Defect> {
    object
        .get(key)
        .and_then(TermNode::items)
        .ok_or_else(|| ctx.invalid(at))
}
pub(super) fn entries<'d, T>(
    object: TermNode<'d>,
    key: &'static str,
    at: &Path<'static>,
    read: impl Fn(TermNode<'d>, &Path<'static>, &Parse<'_, '_>) -> Result<T, Defect>,
    ctx: &Parse<'_, '_>,
) -> Result<Vec<T>, Defect> {
    let items = array(object, key, at, ctx)?;
    let mut result = ctx.vec(items.clone().count())?;
    for (index, item) in items.enumerate() {
        result.push(read(item, &at.key(key).index(index), ctx)?);
    }
    Ok(result)
}

/// One `{name, <rust>}` entry of `fields` or `parameters`.
fn read_binding(
    value: TermNode<'_>,
    rust: &str,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<Binding, Defect> {
    let object = closed(value, &["name", rust], at, ctx)?;
    match (identifier(object, "name", ctx), text(object, rust, ctx)) {
        (Some(name), Some(rust)) => Ok(Binding { name, rust }),
        _ => Err(ctx.invalid(at)),
    }
}

fn read_object(
    value: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<ObjectEntry, Defect> {
    let object = closed(value, &["type", "rust_type", "fields"], at, ctx)?;
    let fail = || ctx.invalid(at);
    Ok(ObjectEntry {
        type_id: node_key(object, "type", at, ctx)?,
        rust_type: path(object, "rust_type", ctx).ok_or_else(fail)?,
        fields: entries(
            object,
            "fields",
            at,
            |field, field_at, ctx| read_binding(field, "rust_field", field_at, ctx),
            ctx,
        )?,
    })
}

fn read_population(
    value: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<PopulationEntry, Defect> {
    let object = closed(value, &["population", "collection"], at, ctx)?;
    let fail = || ctx.invalid(at);
    Ok(PopulationEntry {
        population: node_key(object, "population", at, ctx)?,
        collection: path(object, "collection", ctx).ok_or_else(fail)?,
    })
}

fn read_frame(
    value: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<FrameEntry, Defect> {
    let object = closed(
        value,
        &["context", "operation", "function", "receiver", "parameters"],
        at,
        ctx,
    )?;
    let fail = || ctx.invalid(at);
    Ok(FrameEntry {
        context: node_key(object, "context", at, ctx)?,
        operation: identifier(object, "operation", ctx).ok_or_else(fail)?,
        function: path(object, "function", ctx).ok_or_else(fail)?,
        receiver: text(object, "receiver", ctx).ok_or_else(fail)?,
        parameters: entries(
            object,
            "parameters",
            at,
            |parameter, parameter_at, ctx| {
                read_binding(parameter, "rust_parameter", parameter_at, ctx)
            },
            ctx,
        )?,
    })
}

/// Reads the closed body shape of FR-346 at `at`; an `Err` is the defect and
/// the path of the value at fault: the array element of `objects`, `fields`,
/// `populations`, `frames` or `parameters`, else the body, or a node key
/// outside the node-identity domain at the key.
fn read_body(
    body: TermNode<'_>,
    at: &Path<'static>,
    ctx: &Parse<'_, '_>,
) -> Result<AbstractionBody, Defect> {
    let object = closed(body, &["term", "objects", "populations", "frames"], at, ctx)?;
    if body.kind() != Some(BodyTerm::AbstractionRelation) {
        return Err(ctx.invalid(at));
    }
    Ok(AbstractionBody {
        objects: entries(object, "objects", at, read_object, ctx)?,
        populations: entries(object, "populations", at, read_population, ctx)?,
        frames: entries(object, "frames", at, read_frame, ctx)?,
    })
}

pub(super) fn failure(
    meter: &mut WorkMeter,
    code: CheckedPackageRefusalCode,
    path: Path<'static>,
    cause: Option<CheckedPackageRefusalCause>,
    locus: &CheckedNodeId,
) -> ValidationFailure {
    // Every strict caller builds this prefix from its graph holder, never
    // from document keys. A missing prefix is a programming-contract error.
    let position = path
        .graph_node_index()
        .expect("flat-reader diagnostic names its graph holder");
    let mut refusal = Refusal::new(code, path).at_node(locus);
    refusal.cause = cause;
    stage_storage::refuse(
        meter,
        &ChargeSite::new(PackageReadStage::Graph, position, "correspondence", path),
        refusal,
    )
}
pub(super) fn model_refusal(
    refusal: ModelRefusal,
    path: Path<'static>,
    locus: &CheckedNodeId,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    failure(meter, refusal.code, path, Some(refusal.cause), locus)
}
pub(super) fn conflict(
    path: Path<'static>,
    locus: &CheckedNodeId,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    failure(
        meter,
        CheckedPackageRefusalCode::InvalidModelBinding,
        path,
        Some(CheckedPackageRefusalCause::ConflictingBinding),
        locus,
    )
}
pub(super) struct Site<'a, Body> {
    pub(super) node: &'a CheckedSemanticNodeV2<Body>,
    pub(super) position: usize,
}
impl<Body> Site<'_, Body> {
    pub(super) fn node_at(&self) -> Path<'static> {
        Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(self.position)
    }
    pub(super) fn body_at(&self) -> Path<'static> {
        self.node_at().key("body")
    }
    pub(super) fn invalid(&self, path: Path<'static>, meter: &mut WorkMeter) -> ValidationFailure {
        meter.at_node(self.position, "correspondence");
        failure(
            meter,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            path,
            None,
            &self.node.node_id,
        )
    }
}
pub(super) fn defect_failure<Body>(
    defect: Fault,
    site: &Site<'_, Body>,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    if let Some(stop) = defect.budget {
        stop
    } else {
        meter.at_node(site.position, "correspondence");
        failure(meter, defect.code, defect.path, None, &site.node.node_id)
    }
}

/// The abstraction step. See the module documentation for the order.
///
/// Implements: FR-346.
pub(super) fn validate_abstraction<Body: super::model_members::EmptyAggregateBody>(
    nodes: &[CheckedSemanticNodeV2<Body>],
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
    meter.at_node(0, "correspondence");
    let mut admitted = meter.vec(
        graph
            .each(CheckedNodeKind::Correspondence(
                CorrespondenceForm::AbstractionRelation,
            ))
            .count(),
    )?;
    for (position, node) in graph.each(CheckedNodeKind::Correspondence(
        CorrespondenceForm::AbstractionRelation,
    )) {
        meter.at_node(position, "correspondence");
        let site = Site { node, position };
        let body = check_node(&site, &graph, meter)?;
        admitted.push((position, body));
    }
    check_keys(&admitted, meter)
}

/// One node's own checks, shape through members, returning its body.
fn check_node<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<AbstractionBody, ValidationFailure> {
    let node = site.node;
    // An application at the body root is the closed term grammar's, not this
    // body's: the temporal and state steps refused it already where they place
    // it, and the shape check opening this step refuses the rest.
    let Some((document, row)) = meter.body(site.position) else {
        return Err(site.invalid(site.body_at(), meter));
    };
    if document.kind() == Some(BodyTerm::Application) {
        return Err(failure(
            meter,
            CheckedPackageRefusalCode::IllTyped,
            site.node_at(),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            &node.node_id,
        ));
    }
    let result = {
        let ctx = Parse::new(meter, row, site.position);
        read_body(document, &site.body_at(), &ctx).map_err(|defect| ctx.finish(defect))
    };
    let body = result.map_err(|defect| defect_failure(defect, site, meter))?;
    check_identity(site, &body, meter)?;
    check_order(site, &body, meter)?;
    check_targets(site, &body, graph, meter)?;
    check_members(site, &body, graph, meter)?;
    Ok(body)
}

/// Step 1: the node is its own semantic type and its dependencies are the
/// body's targets.
fn check_identity<Body>(
    site: &Site<'_, Body>,
    body: &AbstractionBody,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    if node.semantic_type == node.node_id && node.dependencies.iter().eq(body.targets(meter)?) {
        Ok(())
    } else {
        Err(site.invalid(site.node_at(), meter))
    }
}

/// Whether `keys` ascend, equal keys in order.
pub(super) fn ascending<K: Ord>(keys: impl Iterator<Item = K>) -> bool {
    let mut keys = keys;
    let Some(mut previous) = keys.next() else {
        return true;
    };
    keys.all(|next| {
        let ascending = previous <= next;
        previous = next;
        ascending
    })
}

/// Step 2: every array in its canonical order. A non-strict order: equal keys
/// are in order, and the member and uniqueness checks refuse them.
fn check_order<Body>(
    site: &Site<'_, Body>,
    body: &AbstractionBody,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let out_of_order =
        |array_at: Path<'static>, meter: &mut WorkMeter| Err(site.invalid(array_at, meter));
    let objects_at = body_at.key("objects");
    if !ascending(body.objects.iter().map(|entry| &entry.type_id)) {
        return out_of_order(objects_at, meter);
    }
    for (index, entry) in body.objects.iter().enumerate() {
        if !ascending(entry.fields.iter().map(|field| field.name.as_ref())) {
            return out_of_order(objects_at.index(index).key("fields"), meter);
        }
    }
    if !ascending(body.populations.iter().map(|entry| &entry.population)) {
        return out_of_order(body_at.key("populations"), meter);
    }
    let frames_at = body_at.key("frames");
    fn frame_key(entry: &FrameEntry) -> (&CheckedNodeId, &str) {
        (&entry.context, entry.operation.as_ref())
    }
    if !ascending(body.frames.iter().map(frame_key)) {
        return out_of_order(frames_at, meter);
    }
    for (index, entry) in body.frames.iter().enumerate() {
        if !ascending(
            entry
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_ref()),
        ) {
            return out_of_order(frames_at.index(index).key("parameters"), meter);
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
fn check_targets<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    body: &AbstractionBody,
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let mut target = |entry_at: Path<'static>,
                      id: &CheckedNodeId,
                      admits: fn(CheckedNodeKind) -> bool,
                      owned: bool| {
        let (declaring, kind) = join(site.node, id, admits, graph)
            .map_err(|refusal| model_refusal(refusal, entry_at, id, meter))?;
        if owned
            && graph
                .owners
                .is_model_declaration_node(declaring, kind.tag())
        {
            graph
                .owners
                .recover(declaring)
                .map_err(|refusal| model_refusal(refusal, entry_at, id, meter))?;
        }
        Ok(())
    };
    for (index, entry) in body.objects.iter().enumerate() {
        let at = body_at.key("objects").index(index);
        target(at, &entry.type_id, is_object_type, true)?;
    }
    for (index, entry) in body.populations.iter().enumerate() {
        let at = body_at.key("populations").index(index);
        target(at, &entry.population, is_population, false)?;
    }
    for (index, entry) in body.frames.iter().enumerate() {
        let at = body_at.key("frames").index(index);
        target(at, &entry.context, is_object_type, true)?;
    }
    Ok(())
}

/// Step 4: the members of every entry, entries in body order.
fn check_members<Body: super::model_members::EmptyAggregateBody>(
    site: &Site<'_, Body>,
    body: &AbstractionBody,
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let body_at = site.body_at();
    let malformed = |at: Path<'static>, locus: &CheckedNodeId, meter: &mut WorkMeter| {
        model_refusal(ModelRefusal::malformed(), at, locus, meter)
    };
    for (index, entry) in body.objects.iter().enumerate() {
        let at = body_at.key("objects").index(index);
        check_object(entry, &at, graph, meter)?;
    }
    for (index, entry) in body.populations.iter().enumerate() {
        if !is_rust_path(&entry.collection) {
            let at = body_at.key("populations").index(index);
            return Err(malformed(at, &entry.population, meter));
        }
    }
    for (index, entry) in body.frames.iter().enumerate() {
        let at = body_at.key("frames").index(index);
        check_frame(entry, &at, graph, meter)?;
    }
    Ok(())
}

/// An object entry: a repeated field name, a name the object type does not
/// expose, then the Rust spellings.
fn check_object<Body: super::model_members::EmptyAggregateBody>(
    entry: &ObjectEntry,
    at: &Path<'static>,
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let fields_at = at.key("fields");
    if let Some(repeat) = super::graph_storage::first_repeat(
        entry.fields.iter().map(|field| field.name.as_ref()),
        meter,
    )? {
        return Err(conflict(fields_at.index(repeat), &entry.type_id, meter));
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
                        meter,
                    ));
                }
            }
        }
    }
    let malformed = |at: Path<'static>, meter: &mut WorkMeter| {
        model_refusal(ModelRefusal::malformed(), at, &entry.type_id, meter)
    };
    if !is_rust_path(&entry.rust_type) {
        return Err(malformed(*at, meter));
    }
    match entry
        .fields
        .iter()
        .position(|field| !is_rust_field(&field.rust))
    {
        Some(index) => Err(malformed(fields_at.index(index), meter)),
        None => Ok(()),
    }
}

/// A frame entry: the operation's resolution, its parameter list, the
/// Rust parameters against each other and the receiver, then the Rust
/// spellings.
fn check_frame<Body: super::model_members::EmptyAggregateBody>(
    entry: &FrameEntry,
    at: &Path<'static>,
    graph: &StepGraph<'_, '_, Body>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let parameters_at = at.key("parameters");
    let malformed = |at: Path<'static>, meter: &mut WorkMeter| {
        model_refusal(ModelRefusal::malformed(), at, &entry.context, meter)
    };
    if let Some((context, kind)) = graph.node(&entry.context) {
        match resolve_operation(context, kind, &entry.operation, graph, meter)? {
            Err(refusal) => return Err(model_refusal(refusal, *at, &entry.context, meter)),
            Ok(Some((_, operation))) => {
                // Domain selection validated the declared names (FR-154-AC-14);
                // compare the frame's list with that admitted declaration.
                let mut declared = meter.vec(operation.parameters.len())?;
                let mut present = true;
                for slot in &operation.parameters {
                    if let Some(name) = slot.name.as_deref() {
                        declared.push(name);
                    } else {
                        present = false;
                    }
                }
                declared.sort_unstable();
                let exact = present
                    && declared.windows(2).all(|pair| pair[0] != pair[1])
                    && declared.iter().copied().eq(entry
                        .parameters
                        .iter()
                        .map(|parameter| parameter.name.as_ref()));
                if !exact {
                    return Err(malformed(*at, meter));
                }
            }
            // A context that is not a model declaration node: its operation
            // is not resolved against any domain package.
            Ok(None) => {}
        }
    }
    if let Some(repeat) = super::graph_storage::first_repeat(
        entry
            .parameters
            .iter()
            .map(|parameter| parameter.rust.as_ref()),
        meter,
    )? {
        return Err(malformed(parameters_at.index(repeat), meter));
    }
    if let Some(clash) = entry
        .parameters
        .iter()
        .position(|parameter| parameter.rust == entry.receiver)
    {
        return Err(malformed(parameters_at.index(clash), meter));
    }
    if !is_rust_path(&entry.function) || !is_rust_receiver(&entry.receiver) {
        return Err(malformed(*at, meter));
    }
    match entry
        .parameters
        .iter()
        .position(|parameter| !is_rust_identifier(&parameter.rust))
    {
        Some(index) => Err(malformed(parameters_at.index(index), meter)),
        None => Ok(()),
    }
}

/// Package-wide key uniqueness over every admitted node, in ascending
/// structural node-id order and body order: the second entry to claim an object,
/// population or operation key refuses `invalid_model_binding`/
/// `conflicting-binding`.
fn check_keys(
    admitted: &[(usize, AbstractionBody)],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    #[derive(Eq, PartialEq, Ord, PartialOrd)]
    enum Key<'a> {
        Object(&'a CheckedNodeId),
        Population(&'a CheckedNodeId),
        Operation(&'a CheckedNodeId, &'a str),
    }
    let count = admitted
        .iter()
        .map(|(_, body)| body.objects.len() + body.populations.len() + body.frames.len())
        .sum();
    let mut rows = meter.vec(count)?;
    for (position, body) in admitted {
        let at = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(*position)
            .key("body");
        rows.extend(body.objects.iter().enumerate().map(|(i, entry)| {
            (
                Key::Object(&entry.type_id),
                at.key("objects").index(i),
                &entry.type_id,
                *position,
            )
        }));
        rows.extend(body.populations.iter().enumerate().map(|(i, entry)| {
            (
                Key::Population(&entry.population),
                at.key("populations").index(i),
                &entry.population,
                *position,
            )
        }));
        rows.extend(body.frames.iter().enumerate().map(|(i, entry)| {
            (
                Key::Operation(&entry.context, &entry.operation),
                at.key("frames").index(i),
                &entry.context,
                *position,
            )
        }));
    }
    if let Some(repeat) = super::graph_storage::first_repeat(rows.iter().map(|row| &row.0), meter)?
    {
        let (_, at, id, position) = &rows[repeat];
        meter.at_node(*position, "correspondence");
        return Err(conflict(*at, id, meter));
    }
    Ok(())
}

#[cfg(test)]
mod document_storage_tests {
    use super::super::{test_support::node_id, CheckedJson, CheckedNodeOwner, NativeJoinBudget};
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn node(body: serde_json::Value) -> CheckedSemanticNodeV2<CheckedJson> {
        serde_json::from_value(json!({"node_id":node_id('a'),"schema_version":"quire.checked-semantic-graph/v2","node_tag":"correspondence","semantic_form":"abstraction_relation","semantic_type":node_id('a'),"dependencies":[],"occurrences":[],"body":body})).unwrap()
    }

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn abstraction_document_projects_original_rows_with_exact_storage_and_target_fault() {
        for target in [false, true] {
            let body = if target {
                json!({"term":"abstraction_relation","objects":[{"type":{"package":"pkg","path":[{"operand":0}],"ordinal":0},"rust_type":["Thing"],"fields":[]}],"populations":[],"frames":[]})
            } else {
                json!({"term":"abstraction_relation","objects":[],"populations":[],"frames":[]})
            };
            let mut nodes = [node(body)];
            if target {
                nodes[0].dependencies = vec![CheckedNodeId {
                    package: "pkg".into(),
                    path: vec![crate::checked_package::shared::CheckedNodePathSegment::Index(42)],
                    ordinal: 0,
                }];
            }
            let kinds = [CheckedNodeKind::Correspondence(
                CorrespondenceForm::AbstractionRelation,
            )];
            let index = nodes
                .iter()
                .enumerate()
                .map(|(p, n)| (&n.node_id, p))
                .collect::<GraphIndex<'_>>();
            let handles = [nodes[0].body.clone()];
            let rows = [vec![42]];
            let owners = ModelOwners::default();
            let run = |limit| {
                let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let measured = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(&mut budget);
                    meter.attach_bodies(&handles, &rows);
                    result = Some(validate_abstraction(
                        &nodes, &kinds, &index, &owners, &mut meter,
                    ));
                });
                (result.unwrap(), measured.bytes_total, budget)
            };
            let (result, allocated, budget) = run(u64::MAX);
            if target {
                let Err(ValidationFailure::Refused(refusal)) = result else {
                    panic!("missing resolved target");
                };
                assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
                assert_eq!(
                    refusal.loci[0].path.as_ref().unwrap().as_str(),
                    "/semantic_graph/nodes/0/body/objects/0"
                );
            } else {
                result.unwrap();
            }
            let required = budget.read_storage_consumed();
            assert_eq!(allocated, required);
            assert_eq!(run(required).0.is_ok(), !target);
            assert!(matches!(
                run(required - 1).0,
                Err(ValidationFailure::Budget(_))
            ));
            assert_eq!(run(0).1, 0);
        }
    }

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn abstraction_model_resolution_drift_keeps_diagnostic_holder_and_one_short_locus() {
        use super::super::model_members::tests::{document, object_type, read, WIDGET};
        use super::super::native_join_budget::NativeJoinLocusSource;
        let models = [read(&document(vec![object_type(WIDGET, &[], vec![])])).unwrap()];
        let mut declaring = node(json!({"term":"aggregate","members":[]}));
        declaring.node_tag = "model".into();
        declaring.semantic_form = "object_type".into();
        declaring.node_id = super::super::model_members::declaration_id(
            "acme/orders",
            super::super::model_members::DeclarationForm::ObjectType,
            WIDGET,
        );
        declaring.semantic_type = declaring.node_id.clone();
        declaring.owner = Some(CheckedNodeOwner::Model {
            identity: "acme/orders".into(),
            node: WIDGET.into(),
        });
        let nodes = [declaring];
        let kinds = [CheckedNodeKind::Model(ModelForm::ObjectType)];
        let owners = ModelOwners::new(&models, &nodes, |_| Ok(())).unwrap();
        let index = nodes
            .iter()
            .enumerate()
            .map(|(p, n)| (&n.node_id, p))
            .collect::<GraphIndex<'_>>();
        let graph = StepGraph {
            nodes: &nodes,
            kinds: &kinds,
            index: &index,
            owners: &owners,
        };
        let entry = ObjectEntry {
            type_id: nodes[0].node_id.clone(),
            rust_type: vec!["Widget".into()],
            fields: vec![Binding {
                name: "absent".into(),
                rust: "field".into(),
            }],
        };
        let at = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(7)
            .key("body")
            .key("objects")
            .index(0);
        let mut probe = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        {
            let mut meter = WorkMeter::over(&mut probe);
            meter.at_node(7, "correspondence");
            assert!(owners
                .resolve_member(&nodes[0], MemberKind::Field, "absent", &mut meter)
                .unwrap()
                .is_err());
        }
        assert_eq!(
            probe.active_locus().unwrap().source,
            NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 0)
        );
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(7, "correspondence");
                result = Some(check_object(&entry, &at, &graph, &mut meter));
            });
            (result.unwrap(), measured.bytes_total, budget)
        };
        let (result, allocated, budget) = run(u64::MAX);
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("unexposed field");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
        assert_eq!(
            refusal.loci[0].path.as_ref().unwrap().as_str(),
            "/semantic_graph/nodes/7/body/objects/0/fields/0"
        );
        let required = budget.read_storage_consumed();
        assert_eq!(allocated, required);
        assert!(matches!(
            run(required).0,
            Err(ValidationFailure::Refused(_))
        ));
        let (short, _, short_budget) = run(required - 1);
        assert!(matches!(short, Err(ValidationFailure::Budget(_))));
        assert_eq!(
            short_budget.first_unpaid_locus().unwrap().source,
            NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 7)
        );
    }
}
