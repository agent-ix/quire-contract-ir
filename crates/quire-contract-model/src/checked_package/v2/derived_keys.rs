//! FR-038 "Anonymous structural node bodies and keys" (FR-038-AC-123 through
//! FR-038-AC-130, IR-627): the reader re-derives the `node_id` of every
//! undeclared node of the ten shapes [`MemberType::node_key`] already derives
//! from the node's own `semantic_type` and body, and refuses a node whose
//! stored key differs, wherever the node is reached from.
//!
//! The shapes are `scalar_type` `boolean` and `integer`; `composite_type`
//! `reference`, `option`, `set`, `bag`, `sequence` and `ordered_set`; and
//! `bounded_domain` `integer_range` and `collection_bounds`. A node whose
//! body is not exactly the closed body of its form, whose `semantic_type` is
//! not its own key (a scalar or composite), whose bounds are outside their
//! form's grammar, or whose literal `type` is not the `Integer` key, has no
//! derivable key and is refused like a stale one:
//! `invalid_package`/`stale-node-key` at the node's own `node_id`.
//!
//! **A node of a real recursion group is skipped**, neither verified nor
//! refused: QSL keys the `Option` or collection of a recursive record, and its
//! `collection_bounds`, under a group digest this reader cannot compute, since
//! it needs the declared member's `SourceOwner` (IR-627-Q4). A node is skipped
//! if and only if its shape is one of those six, it carries a
//! `recursion_group`, it lies on a cycle of the names graph (body references,
//! `semantic_type` unless self, literal and result types; `dependencies` are
//! not edges) and every member of its component in that graph carries the same
//! label. Any other labelled node is verified as an ungrouped
//! node, and `boolean`, `integer`, `integer_range` and `reference` nodes are
//! verified whatever they carry. A skipped node is still charged one work unit.
//!
//! Stated soundness limit: the key, and so the body, of a skipped node is NOT
//! verified, and a tamperer who forges a names cycle through a collection and a
//! `collection_bounds` node and labels both is skipped too. A `collection_bounds`
//! count or an element type read through such a node rests on the package
//! producer; a range read from an `integer_range` node is always verified.
//! A skipped node still passes every other check, including the cycle rule
//! that refuses a cycle outside a declared `recursion_group`.
//!
//! The key is built by [`anonymous`], the one function
//! [`MemberType::node_key`] calls, so no second derivation exists. Bounds
//! enter the preimage as their decimal strings, never parsed to a
//! fixed-width integer or a float.
//!
//! This is its own stage of `validate_graph`: after the graph-shape and
//! application key stages, before the nominal key stage and every
//! declaration, frame, state, temporal, abstraction and operation step, in
//! ascending node-id digest order, one work unit per node it derives for.
//! The forms with no preimage this reader derives, every node carrying a
//! `declaration` and every node carrying a `recursion_group` are gated
//! (IR-627-Q1 to Q4) and not visited.

use super::model_members::{anonymous, CollectionKind, MemberType, StructuralBody};
use super::structural::{
    aggregate_members, binding, is_integer_bound, is_non_negative_integer, reference_target,
};
use super::{
    BoundedDomainForm, CheckedNodeKind, CheckedSemanticNodeV2, CompositeTypeForm, LiteralKind,
    ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::{literal_kind, node_pointer, ValidationFailure, NODE_DOMAIN};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
};
use serde_json::Value;
use std::collections::BTreeMap;

/// One of the ten shapes whose key the reader derives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Shape {
    Boolean,
    Integer,
    Reference,
    Option,
    Collection(CollectionKind),
    IntegerRange,
    CollectionBounds,
}

impl Shape {
    /// The shape of `kind`, or `None` for a form this stage does not derive.
    /// Every form of the three tags is listed, so a new form is a compile
    /// error here until its shape is decided.
    fn of(kind: CheckedNodeKind) -> Option<Self> {
        match kind {
            CheckedNodeKind::ScalarType(form) => match form {
                ScalarTypeForm::Boolean => Some(Self::Boolean),
                ScalarTypeForm::Integer => Some(Self::Integer),
                ScalarTypeForm::Rational
                | ScalarTypeForm::Decimal
                | ScalarTypeForm::Float32
                | ScalarTypeForm::Float64
                | ScalarTypeForm::Text
                | ScalarTypeForm::Dimension
                | ScalarTypeForm::Unit
                | ScalarTypeForm::Enum
                | ScalarTypeForm::CompoundUnit => None,
            },
            CheckedNodeKind::CompositeType(form) => match form {
                CompositeTypeForm::Reference => Some(Self::Reference),
                CompositeTypeForm::Option => Some(Self::Option),
                CompositeTypeForm::Set => Some(Self::Collection(CollectionKind::Set)),
                CompositeTypeForm::Bag => Some(Self::Collection(CollectionKind::Bag)),
                CompositeTypeForm::Sequence => Some(Self::Collection(CollectionKind::Sequence)),
                CompositeTypeForm::OrderedSet => Some(Self::Collection(CollectionKind::OrderedSet)),
                CompositeTypeForm::Record
                | CompositeTypeForm::Tuple
                | CompositeTypeForm::Union
                | CompositeTypeForm::Alias => None,
            },
            CheckedNodeKind::BoundedDomain(form) => match form {
                BoundedDomainForm::IntegerRange => Some(Self::IntegerRange),
                BoundedDomainForm::CollectionBounds => Some(Self::CollectionBounds),
                BoundedDomainForm::RationalRange
                | BoundedDomainForm::DecimalRange
                | BoundedDomainForm::FloatRounding
                | BoundedDomainForm::TextBounds
                | BoundedDomainForm::ModelPopulation => None,
            },
            CheckedNodeKind::Value(_)
            | CheckedNodeKind::Expression(_)
            | CheckedNodeKind::Function(_)
            | CheckedNodeKind::Model(_)
            | CheckedNodeKind::Relation(_)
            | CheckedNodeKind::State(_)
            | CheckedNodeKind::Temporal(_)
            | CheckedNodeKind::Protocol(_)
            | CheckedNodeKind::Claim(_)
            | CheckedNodeKind::Correspondence(_) => None,
        }
    }

    const fn tag(self) -> &'static str {
        match self {
            Self::Boolean | Self::Integer => "scalar_type",
            Self::Reference | Self::Option | Self::Collection(_) => "composite_type",
            Self::IntegerRange | Self::CollectionBounds => "bounded_domain",
        }
    }

    const fn form(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Reference => "reference",
            Self::Option => "option",
            Self::Collection(kind) => kind.form(),
            Self::IntegerRange => "integer_range",
            Self::CollectionBounds => "collection_bounds",
        }
    }

    /// Whether a node of this shape can legitimately be a member of a
    /// recursion group (QSL FR-092): the recursive field's `option` or
    /// collection, and the `collection_bounds` over it. A `boolean`, `integer`,
    /// `integer_range` or `reference` names only the `Integer` node or an
    /// empty-body model node, so no path leads back to it, and it is verified
    /// whatever label it carries.
    const fn can_sit_in_group(self) -> bool {
        match self {
            Self::Option | Self::Collection(_) | Self::CollectionBounds => true,
            Self::Boolean | Self::Integer | Self::Reference | Self::IntegerRange => false,
        }
    }

    /// Whether the preimage records `semantic_type` as `null` (a node typed
    /// by itself) rather than as a node key.
    const fn is_self_typed(self) -> bool {
        match self {
            Self::Boolean
            | Self::Integer
            | Self::Reference
            | Self::Option
            | Self::Collection(_) => true,
            Self::IntegerRange | Self::CollectionBounds => false,
        }
    }
}

/// The key `node` would carry if its body, `semantic_type` and recursion are
/// the closed form of `shape`, or `None` when they are not.
///
/// # Errors
///
/// The encoder's refusal when the preimage's canonical bytes exceed `bytes`.
fn derived_key(
    node: &CheckedSemanticNodeV2,
    shape: Shape,
    bytes: u64,
) -> Result<Option<String>, quire_canonical::Error> {
    if shape.is_self_typed() && node.semantic_type != node.node_id {
        return Ok(None);
    }
    let Some(members) = aggregate_members(&node.body) else {
        return Ok(None);
    };
    match (shape, members) {
        (Shape::Boolean | Shape::Integer, []) => anonymous(
            shape.tag(),
            shape.form(),
            None,
            StructuralBody::Empty,
            bytes,
        )
        .map(Some),
        (Shape::Reference | Shape::Option | Shape::Collection(_), [member]) => {
            let Some(target) = reference_target(member) else {
                return Ok(None);
            };
            anonymous(
                shape.tag(),
                shape.form(),
                None,
                StructuralBody::Over(&target.digest),
                bytes,
            )
            .map(Some)
        }
        // An `integer_range` bound is a signed integer.
        (Shape::IntegerRange, [lower, upper]) => {
            bounded_key(node, shape, [lower, upper], BoundsOver::Integer, bytes)
        }
        // A `collection_bounds` bound is a count. The `semantic_type` of a
        // bounded domain is in the preimage, so the key covers it.
        (Shape::CollectionBounds, [lower, upper]) => {
            bounded_key(node, shape, [lower, upper], BoundsOver::Collection, bytes)
        }
        // A body of any other arity is not the closed body of its form.
        (
            Shape::Boolean
            | Shape::Integer
            | Shape::Reference
            | Shape::Option
            | Shape::Collection(_)
            | Shape::IntegerRange
            | Shape::CollectionBounds,
            _,
        ) => Ok(None),
    }
}

/// What a bounded domain bounds, which fixes the grammar of its bounds.
#[derive(Clone, Copy)]
enum BoundsOver {
    /// `Int[min, max]`: signed bounds.
    Integer,
    /// `K<E>[min, max]`: counts.
    Collection,
}

impl BoundsOver {
    fn is_bound(self, value: &str) -> bool {
        match self {
            Self::Integer => is_integer_bound(value),
            Self::Collection => is_non_negative_integer(value),
        }
    }
}

/// The key of the bounded domain `node` of `shape` when its two members are
/// the `min` and `max` bindings of `over`'s grammar, each an `integer`
/// literal typed at the `Integer` key.
///
/// # Errors
///
/// The encoder's refusal when the preimage's canonical bytes exceed `bytes`.
fn bounded_key(
    node: &CheckedSemanticNodeV2,
    shape: Shape,
    [lower, upper]: [&Value; 2],
    over: BoundsOver,
    bytes: u64,
) -> Result<Option<String>, quire_canonical::Error> {
    let integer = MemberType::Integer.node_key(bytes)?;
    let typed_at_integer =
        |id: &CheckedNodeId| id.domain.as_ref() == NODE_DOMAIN && *id.digest == *integer;
    let (Some(lower), Some(upper)) = (
        bound(lower, "min", &typed_at_integer, over),
        bound(upper, "max", &typed_at_integer, over),
    ) else {
        return Ok(None);
    };
    anonymous(
        shape.tag(),
        shape.form(),
        Some(&node.semantic_type.digest),
        StructuralBody::Bounds {
            integer: &integer,
            lower,
            upper,
        },
        bytes,
    )
    .map(Some)
}

/// The decimal string of the `binding` named `name` when it is an `integer`
/// literal typed at the `Integer` key with a value in the grammar of `over`.
fn bound<'a>(
    term: &'a Value,
    name: &str,
    typed_at_integer: &dyn Fn(&CheckedNodeId) -> bool,
    over: BoundsOver,
) -> Option<&'a str> {
    let literal = binding(term, name)?;
    if literal_kind(literal) != Some(LiteralKind::Integer) {
        return None;
    }
    let ty: CheckedNodeId = serde_json::from_value(literal.get("type")?.clone()).ok()?;
    typed_at_integer(&ty).then_some(())?;
    literal
        .get("value")?
        .as_str()
        .filter(|value| over.is_bound(value))
}

/// Every undeclared node of the ten shapes re-derived from its own body, in
/// ascending node-id digest order, reporting the first stale one.
///
/// Each key is hashed through `quire-canonical` under `bytes`, the reader's
/// byte limit; a preimage whose canonical bytes exceed it refuses
/// `invalid_semantic_graph` at the node's `node_id`.
///
/// `references` are the body references of each node as the per-node walk read
/// them; with each node's `semantic_type` they are the names graph the group
/// skip reads.
pub(super) fn validate_derived_keys(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    references: &[Vec<super::BodyReference>],
    meter: &mut WorkMeter,
    bytes: u64,
) -> Result<(), ValidationFailure> {
    let grouped = GroupedNodes::new(nodes, kinds, index, references);
    for (&node_id, &position) in index {
        let (Some(node), Some(&kind)) = (nodes.get(position), kinds.get(position)) else {
            continue;
        };
        let Some(shape) = Shape::of(kind) else {
            continue;
        };
        // A declared node's preimage has a `declaration` member: gated.
        if node.declaration.is_some() {
            continue;
        }
        let at = || node_pointer(position).key("node_id");
        meter.charge(1, at)?;
        // A node of a real recursion group is keyed under a group digest the
        // reader cannot compute: skipped, and still charged (see the module
        // docs for the soundness limit).
        if grouped.skips(position, shape) {
            continue;
        }
        let derived = derived_key(node, shape, bytes).map_err(|_| {
            ValidationFailure::refused(CheckedPackageRefusalCode::InvalidSemanticGraph, at())
        })?;
        if derived.as_deref() != Some(node_id.digest.as_ref()) {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                at(),
                Some(CheckedPackageRefusalCause::StaleNodeKey),
                node_id.clone(),
            ));
        }
    }
    Ok(())
}

/// The nodes the stage skips: those that stand as members of a real recursion
/// group of QSL FR-092's names graph.
///
/// The names graph has an edge from a node to each node its body names (a
/// body `reference`, a literal `type`, a `result_type`) and to its
/// `semantic_type` unless it is its own; `dependencies` are not edges. A node
/// is skipped if and only if its shape can sit in a group, it carries a
/// `recursion_group`, it lies on a cycle of the names graph, and every member
/// of the strongly connected component holding it carries the same label. Any
/// other labelled node is verified as an ungrouped node.
struct GroupedNodes {
    /// Whether the node at each position is skipped.
    skipped: Vec<bool>,
}

impl GroupedNodes {
    fn new(
        nodes: &[CheckedSemanticNodeV2],
        kinds: &[CheckedNodeKind],
        index: &BTreeMap<&CheckedNodeId, usize>,
        references: &[Vec<super::BodyReference>],
    ) -> Self {
        let mut skipped = vec![false; nodes.len()];
        let candidate = |position: usize| {
            nodes
                .get(position)
                .zip(kinds.get(position))
                .is_some_and(|(node, kind)| {
                    node.recursion_group.is_some()
                        && Shape::of(*kind).is_some_and(Shape::can_sit_in_group)
                })
        };
        if !(0..nodes.len()).any(candidate) {
            return Self { skipped };
        }
        let adjacency: Vec<Vec<usize>> = nodes
            .iter()
            .zip(references)
            .enumerate()
            .map(|(position, (node, named))| {
                let own = (node.semantic_type != node.node_id).then_some(&node.semantic_type);
                named
                    .iter()
                    .map(|(target, _)| target)
                    .chain(own)
                    .filter_map(|target| index.get(target).copied())
                    .filter(|target| *target != position || named_by_body(named, &node.node_id))
                    .collect()
            })
            .collect();
        let component = components(&adjacency);
        let mut size = vec![0_usize; nodes.len()];
        for id in &component {
            if let Some(count) = size.get_mut(*id) {
                *count = count.saturating_add(1);
            }
        }
        for position in (0..nodes.len()).filter(|position| candidate(*position)) {
            let (Some(node), Some(&id)) = (nodes.get(position), component.get(position)) else {
                continue;
            };
            let members = size.get(id).copied().unwrap_or(0);
            let on_cycle = members > 1
                || adjacency
                    .get(position)
                    .is_some_and(|edges| edges.contains(&position));
            let label = node.recursion_group.as_deref();
            let component_is_the_group = on_cycle
                && component
                    .iter()
                    .zip(nodes)
                    .filter(|(other, _)| **other == id)
                    .all(|(_, member)| member.recursion_group.as_deref() == label);
            if let Some(slot) = skipped.get_mut(position) {
                *slot = component_is_the_group;
            }
        }
        Self { skipped }
    }

    fn skips(&self, position: usize, shape: Shape) -> bool {
        shape.can_sit_in_group() && self.skipped.get(position).copied().unwrap_or(false)
    }
}

/// Whether a body reference of the node names the node itself (a self-typed
/// node's own `semantic_type` is not an edge, a body reference to itself is).
fn named_by_body(named: &[super::BodyReference], own: &CheckedNodeId) -> bool {
    named.iter().any(|(target, _)| target == own)
}

/// The strongly connected component of each vertex of `adjacency`: iterative
/// Tarjan on a heap stack, so a deep graph cannot overflow the call stack.
fn components(adjacency: &[Vec<usize>]) -> Vec<usize> {
    const UNVISITED: usize = usize::MAX;
    let size = adjacency.len();
    let mut order = vec![UNVISITED; size];
    let mut low = vec![0_usize; size];
    let mut on_stack = vec![false; size];
    let mut component = vec![UNVISITED; size];
    let mut stack: Vec<usize> = Vec::new();
    let mut calls: Vec<(usize, usize)> = Vec::new();
    let (mut next, mut components) = (0_usize, 0_usize);
    for root in 0..size {
        if order.get(root) != Some(&UNVISITED) {
            continue;
        }
        calls.push((root, 0));
        while let Some((vertex, edge)) = calls.last().copied() {
            if edge == 0 && order.get(vertex) == Some(&UNVISITED) {
                if let (Some(o), Some(l), Some(s)) = (
                    order.get_mut(vertex),
                    low.get_mut(vertex),
                    on_stack.get_mut(vertex),
                ) {
                    *o = next;
                    *l = next;
                    *s = true;
                }
                next = next.saturating_add(1);
                stack.push(vertex);
            }
            let successor = adjacency
                .get(vertex)
                .and_then(|edges| edges.get(edge))
                .copied();
            if let Some(successor) = successor {
                if let Some(frame) = calls.last_mut() {
                    frame.1 = edge.saturating_add(1);
                }
                if order.get(successor) == Some(&UNVISITED) {
                    calls.push((successor, 0));
                } else if on_stack.get(successor) == Some(&true) {
                    let reached = order.get(successor).copied().unwrap_or(UNVISITED);
                    if let Some(current) = low.get_mut(vertex) {
                        *current = (*current).min(reached);
                    }
                }
                continue;
            }
            calls.pop();
            let vertex_low = low.get(vertex).copied().unwrap_or(0);
            if let Some((parent, _)) = calls.last() {
                if let Some(parent_low) = low.get_mut(*parent) {
                    *parent_low = (*parent_low).min(vertex_low);
                }
            }
            if order.get(vertex) == Some(&vertex_low) {
                while let Some(member) = stack.pop() {
                    if let Some(flag) = on_stack.get_mut(member) {
                        *flag = false;
                    }
                    if let Some(slot) = component.get_mut(member) {
                        *slot = components;
                    }
                    if member == vertex {
                        break;
                    }
                }
                components = components.saturating_add(1);
            }
        }
    }
    component
}

#[cfg(test)]
mod tests {
    use super::super::model_members::IntegerBounds;
    use super::*;
    use crate::checked_package::v2::CheckedNodeTag;
    use ix_trace_rs::trace;
    use serde_json::json;
    use std::collections::BTreeSet;

    /// The byte limit these tests derive under: far above any node here.
    const BYTES: u64 = 1 << 20;
    const I128_MIN: i128 = i128::MIN;
    const I128_MAX: i128 = i128::MAX;

    fn id(digest: &str) -> Value {
        json!({"domain": "quire.checked-semantic-node/v1", "digest": digest})
    }

    fn empty() -> Value {
        json!({"term": "aggregate", "members": []})
    }

    /// The graph node a reader would hold for `key`.
    fn wire(
        key: &str,
        tag: &str,
        form: &str,
        semantic_type: &str,
        body: Value,
    ) -> (CheckedSemanticNodeV2, CheckedNodeKind) {
        let node: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id": id(key),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag,
            "semantic_form": form,
            "semantic_type": id(semantic_type),
            "dependencies": [],
            "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": body,
        }))
        .expect("a node");
        let kind = CheckedNodeKind::decode(CheckedNodeTag::from_wire(tag).expect("tag"), form)
            .expect("form");
        (node, kind)
    }

    fn over(target: &str) -> Value {
        json!({"term": "aggregate", "members": [{"term": "reference", "target": id(target)}]})
    }

    fn bounds(integer: &str, lower: &str, upper: &str) -> Value {
        let bound = |name: &str, value: &str| {
            json!({"term": "binding", "name": name, "value": {
                "term": "literal", "type": id(integer), "value_kind": "integer",
                "value": value}})
        };
        json!({"term": "aggregate", "members": [bound("min", lower), bound("max", upper)]})
    }

    /// The node that holds `member`'s closed body under `member.node_key`:
    /// built from the same preimage a second way, by hand, over the keys of
    /// the nodes it names.
    fn node_of(member: &MemberType) -> (CheckedSemanticNodeV2, CheckedNodeKind) {
        let key = member.node_key(BYTES).expect("key");
        let integer = MemberType::Integer.node_key(BYTES).expect("integer key");
        match member {
            MemberType::Boolean => wire(&key, "scalar_type", "boolean", &key, empty()),
            MemberType::Integer => wire(&key, "scalar_type", "integer", &key, empty()),
            MemberType::IntRange(range) => wire(
                &key,
                "bounded_domain",
                "integer_range",
                &integer,
                bounds(&integer, &range.lower.to_string(), &range.upper.to_string()),
            ),
            MemberType::Reference(target) => {
                wire(&key, "composite_type", "reference", &key, over(target))
            }
            MemberType::Option(inner) => {
                let inner = inner.node_key(BYTES).expect("inner key");
                wire(&key, "composite_type", "option", &key, over(&inner))
            }
            MemberType::Collection {
                kind,
                element,
                bounds: None,
            } => {
                let element = element.node_key(BYTES).expect("element key");
                wire(&key, "composite_type", kind.form(), &key, over(&element))
            }
            MemberType::Collection {
                kind,
                element,
                bounds: Some((lower, upper)),
            } => {
                let collection = MemberType::Collection {
                    kind: *kind,
                    element: element.clone(),
                    bounds: None,
                }
                .node_key(BYTES)
                .expect("collection key");
                wire(
                    &key,
                    "bounded_domain",
                    "collection_bounds",
                    &collection,
                    bounds(&integer, &lower.to_string(), &upper.to_string()),
                )
            }
        }
    }

    /// One member type per shape, over the bounds of FR-038-AC-128.
    fn each_shape() -> Vec<MemberType> {
        let wide = MemberType::IntRange(IntegerBounds {
            lower: I128_MIN,
            upper: I128_MAX,
        });
        let zero = MemberType::IntRange(IntegerBounds { lower: 0, upper: 0 });
        let collection = |kind, bounds| MemberType::Collection {
            kind,
            element: Box::new(wide.clone()),
            bounds,
        };
        vec![
            MemberType::Boolean,
            MemberType::Integer,
            zero,
            wide.clone(),
            MemberType::Reference("ab".repeat(32).into()),
            MemberType::Option(Box::new(wide.clone())),
            collection(CollectionKind::Set, None),
            collection(CollectionKind::Bag, None),
            collection(CollectionKind::Sequence, None),
            collection(CollectionKind::OrderedSet, None),
            collection(CollectionKind::Sequence, Some((0, u64::MAX))),
        ]
    }

    /// FR-038-AC-130: for each of the ten derived shapes the key the
    /// admission stage derives from a node's own body is the key
    /// `MemberType::node_key` derives from the matching member type, and the
    /// node built from that preimage admits.
    ///
    /// Tracing: TC-226, FR-038-AC-130
    #[trace("TC-226", "FR-038-AC-130")]
    #[test]
    fn tc_226_the_stage_derives_the_key_member_type_node_key_derives() {
        let mut shapes = BTreeSet::new();
        for member in each_shape() {
            let (node, kind) = node_of(&member);
            let shape = Shape::of(kind).expect("a derived shape");
            shapes.insert(format!("{}/{}", shape.tag(), shape.form()));
            assert_eq!(
                derived_key(&node, shape, BYTES).expect("derives"),
                Some(member.node_key(BYTES).expect("key")),
                "{member:?}"
            );
        }
        // Every one of the ten shapes is covered.
        assert_eq!(shapes.len(), 10, "{shapes:?}");

        // The stage admits each node under its key, and refuses a node whose
        // key is any other node's.
        let nodes: Vec<(CheckedSemanticNodeV2, CheckedNodeKind)> =
            each_shape().iter().map(node_of).collect();
        let (graph, kinds): (Vec<_>, Vec<_>) = nodes.into_iter().unzip();
        let index: BTreeMap<&CheckedNodeId, usize> = graph
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        let mut meter = WorkMeter::new(u64::MAX);
        let no_references = vec![Vec::new(); graph.len()];
        assert!(
            validate_derived_keys(&graph, &kinds, &index, &no_references, &mut meter, BYTES)
                .is_ok()
        );
        let mut tampered = graph.clone();
        tampered[1].body = over(&"ab".repeat(32));
        let index: BTreeMap<&CheckedNodeId, usize> = tampered
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        let mut meter = WorkMeter::new(u64::MAX);
        assert!(validate_derived_keys(
            &tampered,
            &kinds,
            &index,
            &no_references,
            &mut meter,
            BYTES
        )
        .is_err());
    }

    /// An `option` over itself in the group `g`: on a names cycle of one,
    /// carrying the label its whole component shares.
    fn lone_group_option() -> (Vec<CheckedSemanticNodeV2>, Vec<CheckedNodeKind>) {
        let key = "cd".repeat(32);
        let (mut node, kind) = wire(&key, "composite_type", "option", &key, over(&key));
        node.recursion_group = Some("g".into());
        (vec![node], vec![kind])
    }

    fn run(
        graph: &[CheckedSemanticNodeV2],
        kinds: &[CheckedNodeKind],
        named: Vec<Vec<super::super::BodyReference>>,
        limit: u64,
    ) -> Result<(), ValidationFailure> {
        let index: BTreeMap<&CheckedNodeId, usize> = graph
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        validate_derived_keys(
            graph,
            kinds,
            &index,
            &named,
            &mut WorkMeter::new(limit),
            BYTES,
        )
    }

    fn self_reference(node: &CheckedSemanticNodeV2) -> Vec<Vec<super::super::BodyReference>> {
        use crate::checked_package::common::{ReferenceMember, ReferenceSite};
        vec![vec![(
            node.node_id.clone(),
            ReferenceSite {
                member: ReferenceMember::Target,
                is_body_root: false,
            },
        )]]
    }

    /// A node of a real group is skipped, and a skipped node is still charged
    /// one work unit; the same node with no names cycle (its body reference
    /// not read) is verified, and its placeholder key refuses.
    ///
    /// Tracing: TC-226
    #[trace("TC-226")]
    #[test]
    fn tc_226_a_skipped_group_node_is_charged_one_unit_and_an_acyclic_one_is_verified() {
        let (graph, kinds) = lone_group_option();
        let named = || self_reference(&graph[0]);
        assert!(run(&graph, &kinds, named(), 1).is_ok());
        assert!(
            run(&graph, &kinds, named(), 0).is_err(),
            "a skipped node is charged"
        );
        // No names edge: the node is on no cycle, so it is verified as
        // ungrouped, and a placeholder key is stale.
        let refusal = run(&graph, &kinds, vec![Vec::new()], 1).expect_err("verified");
        assert!(
            format!("{refusal:?}").contains("StaleNodeKey"),
            "{refusal:?}"
        );
    }
}
