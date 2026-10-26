//! FR-038 structural key authentication using QSL FR-092/FR-094's shared
//! preimage authority. Aggregate-bodied union values use the application
//! preimage instead. Retained labels select all same-label wire members.
//! Every structural key is checked before any group-label digest is checked;
//! no membership is inferred from a names cycle.

use super::encode::GroupBody;
use super::model_members::{
    structural_key, CollectionKind, MemberType, StructuralPreimage, StructuralRecursion,
    StructuralType,
};
use super::structural::{
    aggregate_members, binding, is_integer_bound, is_non_negative_integer, reference_target,
};
use super::{
    BoundedDomainForm, CheckedNodeKind, CheckedTerm, CheckedTermView, CompositeTypeForm,
    LiteralKind, RawSemanticNodeV2 as CheckedSemanticNodeV2, ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure, NODE_DOMAIN};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use quire_canonical::{Encode, Error, Sink, Writer};
use serde_json::Value;
use std::collections::BTreeMap;

/// One of the ten shapes whose key the reader derives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Shape {
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
    pub(super) fn of(kind: CheckedNodeKind) -> Option<Self> {
        match kind {
            CheckedNodeKind::ScalarType(form) => match form {
                ScalarTypeForm::Boolean => Some(Self::Boolean),
                ScalarTypeForm::Integer => Some(Self::Integer),
                ScalarTypeForm::Rational
                | ScalarTypeForm::Decimal
                | ScalarTypeForm::Float32
                | ScalarTypeForm::Float64
                | ScalarTypeForm::Text
                | ScalarTypeForm::String
                | ScalarTypeForm::Uuid
                | ScalarTypeForm::Timestamp
                | ScalarTypeForm::Dimension
                | ScalarTypeForm::DimensionEquivalence
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
            CheckedNodeKind::TimedModel(_)
            | CheckedNodeKind::Hybrid(_)
            | CheckedNodeKind::Value(_)
            | CheckedNodeKind::Extension(_)
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

/// Validates the closed body without trusting a supplied node key.
pub(super) fn closed_shape(
    node: &CheckedSemanticNodeV2,
    shape: Shape,
    integer: &str,
    reference: fn(&Value) -> Option<CheckedNodeId>,
) -> bool {
    if shape.is_self_typed() && node.semantic_type != node.node_id {
        return false;
    }
    let Some(members) = aggregate_members(&node.body) else {
        return false;
    };
    match (shape, members) {
        (Shape::Boolean | Shape::Integer, []) => true,
        (Shape::Reference | Shape::Option | Shape::Collection(_), [member]) => {
            reference(member).is_some()
        }
        (Shape::IntegerRange | Shape::CollectionBounds, [lower, upper]) => {
            let over = if shape == Shape::IntegerRange {
                BoundsOver::Integer
            } else {
                BoundsOver::Collection
            };
            if matches!(over, BoundsOver::Integer)
                && (!super::is_node_domain(&node.semantic_type.domain)
                    || node.semantic_type.digest.as_ref() != integer)
            {
                return false;
            }
            let typed = |id: &CheckedNodeId| {
                super::is_node_domain(&id.domain) && id.digest.as_ref() == integer
            };
            bound(lower, "min", &typed, over).is_some()
                && bound(upper, "max", &typed, over).is_some()
        }
        (
            Shape::Boolean
            | Shape::Integer
            | Shape::Reference
            | Shape::Option
            | Shape::Collection(_)
            | Shape::IntegerRange
            | Shape::CollectionBounds,
            _,
        ) => false,
    }
}

/// The reader's complete early structural-key shape rule, shared with the
/// pre-read candidate API. The caller resolves `semantic_type_kind` from its
/// own complete graph without altering reader lookup or charge order.
pub(super) fn valid_derived_shape(
    node: &CheckedSemanticNodeV2,
    kind: CheckedNodeKind,
    integer: &str,
    semantic_type_kind: Option<CheckedNodeKind>,
    reference: fn(&Value) -> Option<CheckedNodeId>,
) -> bool {
    if let Some(shape) = Shape::of(kind) {
        let bounded_type_matches = shape != Shape::CollectionBounds
            || semantic_type_kind.is_some_and(|kind| {
                matches!(
                    kind,
                    CheckedNodeKind::CompositeType(
                        CompositeTypeForm::Set
                            | CompositeTypeForm::Bag
                            | CompositeTypeForm::Sequence
                            | CompositeTypeForm::OrderedSet
                    )
                )
            });
        if !bounded_type_matches || !closed_shape(node, shape, integer, reference) {
            return false;
        }
    }
    // FR-094's fixed model-owned shape applies only without a source
    // declaration. Declared nodes retain the ordinary structural preimage.
    let model_owned = matches!(
        kind,
        CheckedNodeKind::Model(super::ModelForm::ObjectType | super::ModelForm::SystemsInterface)
            | CheckedNodeKind::Relation(
                super::RelationForm::Relationship | super::RelationForm::Population
            )
    ) || (matches!(
        kind,
        CheckedNodeKind::Model(super::ModelForm::RecordValueType | super::ModelForm::VariantType)
    ) && matches!(
        node.owner.as_ref(),
        Some(super::CheckedNodeOwner::Model { .. })
    ));
    !(node.declaration.is_none()
        && model_owned
        && (node.semantic_type != node.node_id
            || node.recursion_group.is_some()
            || (matches!(
                kind,
                CheckedNodeKind::Relation(super::RelationForm::Population)
            ) && !node.dependencies.is_empty())
            || aggregate_members(&node.body).is_none_or(|members| !members.is_empty())))
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

/// The decimal string of the `binding` named `name` when it is an `integer`
/// literal typed at the `Integer` key with a value in the grammar of `over`.
fn bound<'a>(
    term: &'a Value,
    name: &str,
    typed_at_integer: &dyn Fn(&CheckedNodeId) -> bool,
    over: BoundsOver,
) -> Option<&'a str> {
    let literal = binding(term, name)?;
    let CheckedTermView::Literal {
        type_id,
        value_kind: LiteralKind::Integer,
        value,
    } = CheckedTerm::view(literal)?
    else {
        return None;
    };
    let ty: CheckedNodeId = serde_json::from_value(type_id.clone()).ok()?;
    typed_at_integer(&ty).then_some(())?;
    value.as_str().filter(|value| over.is_bound(value))
}

/// A labelled group's members and ordinals, retained in graph array order.
pub(super) struct Group<'a> {
    pub(super) members: Vec<usize>,
    pub(super) ordinals: BTreeMap<&'a CheckedNodeId, usize>,
    pub(super) least: usize,
}

pub(super) fn node_preimage<'a>(
    node: &'a CheckedSemanticNodeV2,
    group: Option<&'a Group<'a>>,
    label: Option<&'a str>,
) -> StructuralPreimage<'a, GroupBody<'a>> {
    // The empty map is shared for all ungrouped bodies without allocating.
    static EMPTY: BTreeMap<&CheckedNodeId, usize> = BTreeMap::new();
    let ordinals = group.map_or(&EMPTY, |group| &group.ordinals);
    let semantic_type = if node.semantic_type == node.node_id {
        None
    } else {
        Some(ordinals.get(&node.semantic_type).copied().map_or_else(
            || StructuralType::node(&node.semantic_type),
            StructuralType::group,
        ))
    };
    StructuralPreimage {
        node_tag: &node.node_tag,
        semantic_form: &node.semantic_form,
        owner: node.owner.as_ref(),
        declaration: node.declaration.as_ref(),
        semantic_type,
        recursion: group.and_then(|group| {
            group
                .ordinals
                .get(&node.node_id)
                .copied()
                .map(|ordinal| StructuralRecursion {
                    size: group.members.len(),
                    ordinal,
                    group: label,
                })
        }),
        body: GroupBody {
            body: &node.body,
            ordinals,
        },
    }
}

/// Charges each actual term visit before inspecting or scheduling its children.
fn charge_terms(
    body: &CheckedTerm,
    position: usize,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut pending = vec![(body, node_pointer(position).key("body"))];
    while let Some((term, at)) = pending.pop() {
        meter.charge(1, || at.clone())?;
        let children = match term {
            CheckedTerm::Aggregate { members } => Some(("members", members.as_slice())),
            CheckedTerm::Application { arguments, .. } => Some(("arguments", arguments.as_slice())),
            CheckedTerm::ExtensionForm { clauses, .. } => Some(("clauses", clauses.as_slice())),
            CheckedTerm::Binding { value, .. } => {
                pending.push((value.as_ref(), at.clone().key("value")));
                None
            }
            CheckedTerm::Literal { .. }
            | CheckedTerm::Reference { .. }
            | CheckedTerm::DependencyReference { .. }
            | CheckedTerm::Frame { .. }
            | CheckedTerm::AbstractionRelation { .. } => None,
            CheckedTerm::Refinement { .. } => None,
        };
        if let Some((name, children)) = children {
            for (ordinal, child) in children.iter().enumerate().rev() {
                pending.push((child, at.clone().key(name).index(ordinal)));
            }
        }
    }
    Ok(())
}

fn encoder_failure(at: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::InvalidSemanticGraph, at)
}

fn stale(at: JsonPointer, node: &CheckedNodeId, derived: Option<String>) -> ValidationFailure {
    match derived {
        Some(digest) => ValidationFailure::refused_stale_node_key(
            at,
            node.clone(),
            CheckedNodeId {
                domain: NODE_DOMAIN.into(),
                digest: digest.into(),
            },
        ),
        None => ValidationFailure::refused_at(
            CheckedPackageRefusalCode::InvalidPackage,
            at,
            Some(CheckedPackageRefusalCause::StaleNodeKey),
            node.clone(),
        ),
    }
}

/// Authenticate every structural key, then independently authenticate each label.
/// The existing entrypoint preserves owner/graph/application stage precedence.
pub(super) fn validate_derived_keys(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<super::CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    _references: &[Vec<super::BodyReference>],
    meter: &mut WorkMeter,
    bytes: u64,
) -> Result<(), ValidationFailure> {
    let mut groups: BTreeMap<&str, Group<'_>> = BTreeMap::new();
    for (position, node) in nodes.iter().enumerate() {
        if let Some(label) = node.recursion_group.as_deref() {
            meter.charge(1, || node_pointer(position).key("recursion_group"))?;
            let group = groups.entry(label).or_insert_with(|| Group {
                members: Vec::new(),
                ordinals: BTreeMap::new(),
                least: position,
            });
            let ordinal = group.members.len();
            group.members.push(position);
            group.ordinals.insert(&node.node_id, ordinal);
            if nodes
                .get(group.least)
                .is_some_and(|least| node.node_id < least.node_id)
            {
                group.least = position;
            }
        }
    }
    for (&node_id, &position) in index {
        let (Some(node), Some(&kind)) = (nodes.get(position), kinds.get(position)) else {
            continue;
        };
        if kind == CheckedNodeKind::Relation(super::RelationForm::Refinement) {
            continue;
        }
        if node.nominal_identity_preimage.is_some()
            || super::operations::has_application_key_admitted(
                terms.get(position).and_then(Option::as_ref),
                kind,
            )
            || kind
                == CheckedNodeKind::Correspondence(super::CorrespondenceForm::AbstractionRelation)
        {
            continue;
        }
        let at = || node_pointer(position).key("node_id");
        meter.charge(1, at)?;
        let shape = Shape::of(kind);
        let integer = if matches!(shape, Some(Shape::IntegerRange | Shape::CollectionBounds)) {
            // A bounds literal's type is authenticated against the shared Integer authority.
            meter.charge(1, at)?;
            MemberType::Integer
                .node_key(bytes)
                .map_err(|_| encoder_failure(at()))?
        } else {
            String::new()
        };
        let semantic_type_kind = index
            .get(&node.semantic_type)
            .and_then(|&target| kinds.get(target))
            .copied();
        if !valid_derived_shape(node, kind, &integer, semantic_type_kind, reference_target) {
            return Err(stale(at(), node_id, None));
        }
        let group = node
            .recursion_group
            .as_deref()
            .and_then(|label| groups.get(label));
        let Some(term) = terms.get(position).and_then(Option::as_ref) else {
            return Err(encoder_failure(node_pointer(position).key("body")));
        };
        charge_terms(term, position, meter)?;
        meter.charge(1, at)?;
        let derived = structural_key(
            &node_preimage(node, group, node.recursion_group.as_deref()),
            bytes,
        )
        .map_err(|_| encoder_failure(at()))?;
        if derived != node_id.digest.as_ref() {
            return Err(stale(at(), node_id, Some(derived)));
        }
    }
    // Sorting by retained member identity is independent of retained labels.
    let mut ordered: Vec<_> = groups.iter().collect();
    ordered.sort_by_key(|(_, group)| nodes.get(group.least).map(|node| &node.node_id));
    for (&label, group) in ordered {
        let mut local = Vec::with_capacity(group.members.len());
        for &position in &group.members {
            let Some(node) = nodes.get(position) else {
                continue;
            };
            let Some(term) = terms.get(position).and_then(Option::as_ref) else {
                return Err(encoder_failure(node_pointer(position).key("body")));
            };
            charge_terms(term, position, meter)?;
            meter.charge(1, || node_pointer(position).key("node_id"))?;
            let digest = if kinds.get(position).is_some_and(|&kind| {
                super::operations::has_application_key_admitted(
                    terms.get(position).and_then(Option::as_ref),
                    kind,
                )
            }) {
                let members: Vec<_> = group
                    .members
                    .iter()
                    .filter_map(|&member| nodes.get(member).map(|node| &node.node_id))
                    .collect();
                super::operations::application_local_digest(node, term, position, &members, bytes)?
            } else {
                structural_key(&node_preimage(node, Some(group), None), bytes)
                    .map_err(|_| encoder_failure(node_pointer(position).key("node_id")))?
            };
            local.push(digest);
        }
        let at = || node_pointer(group.least).key("node_id");
        meter.charge(1, at)?;
        let digest =
            quire_canonical::sha256(&LocalDigests(&local), quire_canonical::Limits::new(bytes))
                .map_err(|_| encoder_failure(at()))?
                .to_string();
        if digest != label {
            if let Some(node) = nodes.get(group.least) {
                return Err(stale(at(), &node.node_id, None));
            }
        }
    }
    Ok(())
}

/// QSL's group digest array consists of lowercase member-local digests.
pub(super) struct LocalDigests<'a>(pub(super) &'a [String]);
impl Encode for LocalDigests<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_array()?;
        for digest in self.0 {
            writer.string(digest)?;
        }
        writer.end_array()
    }
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

    fn decoded_terms(nodes: &[CheckedSemanticNodeV2]) -> Vec<Option<super::super::CheckedTerm>> {
        nodes
            .iter()
            .map(|node| super::super::CheckedTerm::decode_admitted(&node.body).ok())
            .collect()
    }

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
            MemberType::String => wire(&key, "scalar_type", "string", &key, empty()),
            MemberType::Uuid => wire(&key, "scalar_type", "uuid", &key, empty()),
            MemberType::Timestamp => wire(&key, "scalar_type", "timestamp", &key, empty()),
            MemberType::Clock => wire(&key, "scalar_type", "clock", &key, empty()),
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
            MemberType::Record(_) | MemberType::Variant(_) | MemberType::Quantity(_) => {
                panic!("a nominal type is a selected declaration, not a derived node")
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
    #[trace("TC-226", "FR-038-AC-130")]
    #[test]
    fn tc_226_the_stage_derives_the_key_member_type_node_key_derives() {
        let mut shapes = BTreeSet::new();
        for member in each_shape() {
            let (node, kind) = node_of(&member);
            let shape = Shape::of(kind).expect("a derived shape");
            assert!(closed_shape(
                &node,
                shape,
                &MemberType::Integer.node_key(BYTES).expect("integer"),
                reference_target,
            ));
            shapes.insert(format!("{}/{}", node.node_tag, node.semantic_form));
            assert_eq!(
                Some(structural_key(&node_preimage(&node, None, None), BYTES).expect("derives")),
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
        let terms = decoded_terms(&graph);
        assert!(validate_derived_keys(
            &graph,
            &kinds,
            &terms,
            &index,
            &no_references,
            &mut meter,
            BYTES
        )
        .is_ok());
        let mut tampered = graph.clone();
        tampered[1].body = over(&"ab".repeat(32));
        let index: BTreeMap<&CheckedNodeId, usize> = tampered
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        let mut meter = WorkMeter::new(u64::MAX);
        let terms = decoded_terms(&tampered);
        assert!(validate_derived_keys(
            &tampered,
            &kinds,
            &terms,
            &index,
            &no_references,
            &mut meter,
            BYTES
        )
        .is_err());
    }

    #[trace("FR-038-AC-145", "FR-038-AC-148")]
    #[test]
    fn genuine_self_option_group_authenticates_both_passes() {
        // Independently derive G1's local body and digest array, with no retained key in it.
        let local = json!({
            "version": "quire.structural-node/v1", "node_tag": "composite_type",
            "semantic_form": "option", "semantic_type": null, "declaration": null,
            "recursion": {"size": 1, "ordinal": 0},
            "body": {"term": "aggregate", "members": [{"term": "group_reference", "ordinal": 0}]},
        });
        let local_digest = quire_canonical::sha256(&local, quire_canonical::Limits::new(BYTES))
            .expect("local")
            .to_string();
        let label =
            quire_canonical::sha256(&json!([local_digest]), quire_canonical::Limits::new(BYTES))
                .expect("group")
                .to_string();
        let mut full = local;
        full["recursion"]["group"] = json!(label);
        let key = quire_canonical::sha256(&full, quire_canonical::Limits::new(BYTES))
            .expect("key")
            .to_string();
        let (mut node, kind) = wire(&key, "composite_type", "option", &key, over(&key));
        node.recursion_group = Some(label.into());
        let graph = [node];
        let index = BTreeMap::from([(&graph[0].node_id, 0)]);
        let terms = decoded_terms(&graph);
        let mut meter = WorkMeter::new(9);
        assert!(validate_derived_keys(
            &graph,
            &[kind],
            &terms,
            &index,
            &[vec![]],
            &mut meter,
            BYTES
        )
        .is_ok());
        assert_eq!(
            meter.consumed(),
            9,
            "membership + visit + two terms/two digests + group digest"
        );
        let mut unpaid = WorkMeter::new(8);
        assert!(matches!(
            validate_derived_keys(
                &graph,
                &[kind],
                &terms,
                &index,
                &[vec![]],
                &mut unpaid,
                BYTES
            ),
            Err(ValidationFailure::Incomplete(_))
        ));
    }

    #[trace("FR-038-AC-149", "TC-226")]
    #[test]
    fn tree_group_stops_at_the_first_unpaid_local_preimage_charge() {
        let integer = MemberType::Integer.node_key(BYTES).expect("Integer key");
        let owner = json!({"kind":"source", "authority":"example", "identity":"source"});
        let declaration = json!({"qualified_name":["Example","Tree"]});
        let bound_body = bounds(&integer, "0", "3");
        let local = [
            json!({
                "version":"quire.structural-node/v1", "node_tag":"composite_type",
                "semantic_form":"record", "semantic_type":null,
                "declaration":declaration, "owner":owner,
                "recursion":{"size":3,"ordinal":0},
                "body":{"term":"aggregate","members":[{"term":"binding","name":"kids",
                    "value":{"term":"group_reference","ordinal":2}}]}
            }),
            json!({
                "version":"quire.structural-node/v1", "node_tag":"composite_type",
                "semantic_form":"sequence", "semantic_type":null, "declaration":null,
                "recursion":{"size":3,"ordinal":1},
                "body":{"term":"aggregate","members":[{"term":"group_reference","ordinal":0}]}
            }),
            json!({
                "version":"quire.structural-node/v1", "node_tag":"bounded_domain",
                "semantic_form":"collection_bounds",
                "semantic_type":{"term":"group_reference","ordinal":1},
                "declaration":null, "recursion":{"size":3,"ordinal":2},
                "body":bound_body
            }),
        ];
        let digest = |value: &Value| {
            quire_canonical::sha256(value, quire_canonical::Limits::new(BYTES))
                .expect("canonical digest")
                .to_string()
        };
        let local_digests = local.iter().map(digest).collect::<Vec<_>>();
        let label = digest(&json!(local_digests));
        let keys = local
            .iter()
            .map(|preimage| {
                let mut preimage = preimage.clone();
                preimage["recursion"]["group"] = json!(label);
                digest(&preimage)
            })
            .collect::<Vec<_>>();
        let (mut tree, tree_kind) = wire(
            &keys[0],
            "composite_type",
            "record",
            &keys[0],
            json!({"term":"aggregate","members":[{"term":"binding","name":"kids",
                "value":{"term":"reference","target":id(&keys[2])}}]}),
        );
        tree.owner = Some(serde_json::from_value(owner).expect("source owner"));
        tree.declaration = Some(serde_json::from_value(declaration).expect("declaration"));
        let (mut sequence, sequence_kind) = wire(
            &keys[1],
            "composite_type",
            "sequence",
            &keys[1],
            over(&keys[0]),
        );
        let (mut bound, bound_kind) = wire(
            &keys[2],
            "bounded_domain",
            "collection_bounds",
            &keys[1],
            bounds(&integer, "0", "3"),
        );
        for member in [&mut tree, &mut sequence, &mut bound] {
            member.recursion_group = Some(label.clone().into());
        }
        let (integer_node, integer_kind) =
            wire(&integer, "scalar_type", "integer", &integer, empty());
        let graph = [tree, sequence, bound, integer_node];
        let kinds = [tree_kind, sequence_kind, bound_kind, integer_kind];
        let index = graph
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect::<BTreeMap<_, _>>();
        let references = vec![Vec::new(); graph.len()];
        let terms = decoded_terms(&graph);

        // Three memberships; first-pass Tree/Sequence/bounds/Integer charges
        // are 5+4+8+3; second-pass local charges are 4+3+6 and one group digest.
        const TREE_WORK: u64 = 3 + (5 + 4 + 8 + 3) + (4 + 3 + 6 + 1);
        let mut exact = WorkMeter::new(TREE_WORK);
        assert!(validate_derived_keys(
            &graph,
            &kinds,
            &terms,
            &index,
            &references,
            &mut exact,
            BYTES
        )
        .is_ok());
        assert_eq!(exact.consumed(), TREE_WORK);

        // The first local body term is unpaid. Sequence, bounds and the group
        // digest still await charges; continuing after this error increments
        // the meter and fails the exact consumed/pointer assertion below.
        const BEFORE_LOCAL: u64 = 3 + (5 + 4 + 8 + 3);
        let mut denied = WorkMeter::new(BEFORE_LOCAL);
        let failure = validate_derived_keys(
            &graph,
            &kinds,
            &terms,
            &index,
            &references,
            &mut denied,
            BYTES,
        )
        .expect_err("the first local term is unpaid");
        let ValidationFailure::Incomplete(report) = failure else {
            panic!("expected work-incomplete, got {failure:?}");
        };
        assert_eq!(report.limit, BEFORE_LOCAL);
        assert_eq!(report.consumed, BEFORE_LOCAL + 1);
        assert_eq!(
            report.path.as_ref().map(|path| path.as_str()),
            Some("/semantic_graph/nodes/0/body")
        );
        assert_eq!(denied.consumed(), BEFORE_LOCAL + 1);
    }
}
