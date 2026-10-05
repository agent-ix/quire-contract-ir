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
//! form's grammar, or that sits in a recursion group, has no derivable key
//! and is refused like a stale one: `invalid_package`/`stale-node-key` at the
//! node's own `node_id`.
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
//! The forms with no preimage this reader derives, and every node carrying a
//! `declaration`, are gated (IR-627-Q1 to Q4) and not visited.

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
    if node.recursion_group.is_some()
        || (shape.is_self_typed() && node.semantic_type != node.node_id)
    {
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
        // An `integer_range` bound is a signed integer and the range is typed
        // at `Integer`.
        (Shape::IntegerRange, [lower, upper]) => {
            bounded_key(node, shape, [lower, upper], BoundsOver::Integer, bytes)
        }
        // A `collection_bounds` bound is a count and the domain is typed at
        // the collection it bounds, which the preimage records.
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

/// What a bounded domain bounds, which fixes the grammar of its bounds and
/// what its `semantic_type` must be.
#[derive(Clone, Copy)]
enum BoundsOver {
    /// `Int[min, max]`: signed bounds, typed at `Integer`.
    Integer,
    /// `K<E>[min, max]`: counts, typed at the collection.
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
    if matches!(over, BoundsOver::Integer) && !typed_at_integer(&node.semantic_type) {
        return Ok(None);
    }
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
pub(super) fn validate_derived_keys(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
    bytes: u64,
) -> Result<(), ValidationFailure> {
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
        assert!(validate_derived_keys(&graph, &kinds, &index, &mut meter, BYTES).is_ok());
        let mut tampered = graph.clone();
        tampered[1].body = over(&"ab".repeat(32));
        let index: BTreeMap<&CheckedNodeId, usize> = tampered
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        let mut meter = WorkMeter::new(u64::MAX);
        assert!(validate_derived_keys(&tampered, &kinds, &index, &mut meter, BYTES).is_err());
    }
}
