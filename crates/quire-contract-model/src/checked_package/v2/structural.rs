//! Graph-shape rules for node bodies that carry no nominal preimage: the
//! two structural forms QSL emits beyond QSpec FR-322's published form
//! list (`value`/`parameter` and `scalar_type`/`compound_unit`), and FR-322's
//! dependency join for every node whose body holds an `application`.
//!
//! - **Parameter** (QSL FR-092 "Parameter nodes"): a value an enclosing
//!   binder binds. Its body is exactly `aggregate{[binding "name" = a
//!   non-empty text literal typed at a `scalar_type`/`text` node, binding
//!   "level" = a canonical non-negative integer literal typed at a
//!   `scalar_type`/`integer` node]}`; its `semantic_type` is the binder's
//!   type node (a `scalar_type`, `composite_type` or `bounded_domain` node,
//!   never itself); its body names no node, so `dependencies` is empty.
//! - **Compound unit** (QSL FR-094 "Compound unit"): the anonymous type of
//!   a quantity whose unit is a product, quotient or power. It is its own
//!   semantic type; its body is an `aggregate` of terms, each exactly
//!   `aggregate{[binding "unit" = reference(a root `scalar_type`/`unit`
//!   node), binding "exponent" = a canonical nonzero integer literal typed
//!   at a `scalar_type`/`integer` node]}`, strictly ascending by unit key
//!   (the `quire.value.compound-unit/v1` term order); `dependencies` is
//!   exactly those unit keys in that order. The empty body is the
//!   dimensionless unit.
//! - **Application-node dependency join** (FR-322
//!   `application_node_preimage`): a node whose body contains an
//!   `application` term anywhere has `dependencies` exactly the unique,
//!   digest-ascending reference targets and operation member declarations
//!   of its body. `result_type` and literal `type` annotations are not
//!   dependencies.
//!
//! Neither structural form's node key is re-derived here: QSL keys both by
//! its proposed `quire.structural-node/v1` preimage, which QSpec does not
//! publish, and this reader re-derives only the published nominal and
//! application preimages.
//!
//! Every violation refuses as `invalid_semantic_graph`, located at the
//! offending node, at the first such node in ascending node-id digest order.
//! These are graph-shape refusals, so they run before the stale-key stage.
//! The stage charges no work of its own: every body term it walks was
//! parsed, shape-validated and charged once by the per-node body loop.

use super::{
    BoundedDomainForm, CheckedNodeKind, CheckedNodeTag, CheckedSemanticNodeV2, ClaimForm,
    CompositeTypeForm, CorrespondenceForm, ExpressionForm, FunctionForm, ModelForm,
    NominalIdentityPreimage, ProtocolForm, RelationForm, ScalarTypeForm, StateForm, TemporalForm,
    ValueForm,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{CheckedNodeId, CheckedPackageRefusalCode};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

const BODY_PATH: &str = "semantic_graph.nodes.body";
const DEPENDENCIES_PATH: &str = "semantic_graph.nodes.dependencies";
const SEMANTIC_TYPE_PATH: &str = "semantic_graph.nodes.semantic_type";

/// The graph one stage reads: each node with its decoded kind, by key.
struct Graph<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
}

impl Graph<'_> {
    fn node(&self, id: &CheckedNodeId) -> Option<(&CheckedSemanticNodeV2, CheckedNodeKind)> {
        let position = *self.index.get(id)?;
        Some((self.nodes.get(position)?, *self.kinds.get(position)?))
    }

    fn kind(&self, id: &CheckedNodeId) -> Option<CheckedNodeKind> {
        self.node(id).map(|(_, kind)| kind)
    }
}

/// Which body rule a node's kind selects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StructuralForm {
    Parameter,
    CompoundUnit,
}

impl StructuralForm {
    /// Every form is listed, so a new form is a compile error here until
    /// its body rule is decided.
    fn of(kind: CheckedNodeKind) -> Option<Self> {
        use CheckedNodeKind as K;
        match kind {
            K::Value(ValueForm::Parameter) => Some(Self::Parameter),
            K::ScalarType(ScalarTypeForm::CompoundUnit) => Some(Self::CompoundUnit),
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
                | ScalarTypeForm::Enum,
            ) => None,
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
                | ValueForm::OptionValue,
            ) => None,
            K::Expression(
                ExpressionForm::Reference
                | ExpressionForm::Call
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
            K::Function(
                FunctionForm::PureFunction
                | FunctionForm::Predicate
                | FunctionForm::RecursiveFunction,
            ) => None,
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
            ) => None,
            K::Relation(
                RelationForm::Relationship
                | RelationForm::Population
                | RelationForm::Membership
                | RelationForm::CausalRelation,
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
}

/// Validates every parameter and compound-unit node's body, type and
/// dependencies, and every application node's dependency join, visiting
/// nodes in ascending node-id digest order.
pub(super) fn validate_structural_nodes(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Result<(), ValidationFailure> {
    let graph = Graph {
        nodes,
        kinds,
        index,
    };
    for (&node_id, &position) in index {
        let (Some(node), Some(&kind)) = (nodes.get(position), kinds.get(position)) else {
            continue;
        };
        let refuse = |path| {
            ValidationFailure::RefusedAt(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                path,
                None,
                node_id.clone(),
            )
        };
        let defect = match StructuralForm::of(kind) {
            Some(StructuralForm::Parameter) => parameter_defect(node, &graph),
            Some(StructuralForm::CompoundUnit) => compound_unit_defect(node, &graph),
            None => None,
        };
        if let Some(path) = defect {
            return Err(refuse(path));
        }
        if let Some(expected) = application_dependencies(&node.body) {
            if !node.dependencies.iter().eq(expected.iter()) {
                return Err(refuse(DEPENDENCIES_PATH));
            }
        }
    }
    Ok(())
}

/// The first rule a `value`/`parameter` node breaks, as the path it breaks
/// it at.
fn parameter_defect(node: &CheckedSemanticNodeV2, graph: &Graph<'_>) -> Option<&'static str> {
    let is_binder_type = node.semantic_type != node.node_id
        && graph.kind(&node.semantic_type).is_some_and(|kind| {
            matches!(
                kind.tag(),
                CheckedNodeTag::ScalarType
                    | CheckedNodeTag::CompositeType
                    | CheckedNodeTag::BoundedDomain
            )
        });
    if !is_binder_type {
        return Some(SEMANTIC_TYPE_PATH);
    }
    let Some([name, level]) = aggregate_members(&node.body) else {
        return Some(BODY_PATH);
    };
    let name_ok = binding(name, "name")
        .and_then(|value| literal(value, "text", ScalarTypeForm::Text, graph))
        .and_then(Value::as_str)
        .is_some_and(|text| !text.is_empty());
    let level_ok = binding(level, "level")
        .and_then(|value| literal(value, "integer", ScalarTypeForm::Integer, graph))
        .and_then(Value::as_str)
        .is_some_and(is_non_negative_integer);
    if !(name_ok && level_ok) {
        return Some(BODY_PATH);
    }
    if !node.dependencies.is_empty() {
        return Some(DEPENDENCIES_PATH);
    }
    None
}

/// The first rule a `scalar_type`/`compound_unit` node breaks, as the path
/// it breaks it at.
fn compound_unit_defect(node: &CheckedSemanticNodeV2, graph: &Graph<'_>) -> Option<&'static str> {
    if node.semantic_type != node.node_id {
        return Some(SEMANTIC_TYPE_PATH);
    }
    let Some(terms) = aggregate_members(&node.body) else {
        return Some(BODY_PATH);
    };
    let mut units: Vec<CheckedNodeId> = Vec::with_capacity(terms.len());
    for term in terms {
        let Some(unit) = compound_unit_term(term, graph) else {
            return Some(BODY_PATH);
        };
        // Strictly ascending: canonical order and no repeated unit.
        if units.last().is_some_and(|previous| *previous >= unit) {
            return Some(BODY_PATH);
        }
        units.push(unit);
    }
    if node.dependencies != units {
        return Some(DEPENDENCIES_PATH);
    }
    None
}

/// One compound-unit term's root unit key, when the term has exactly the
/// closed term shape.
fn compound_unit_term(term: &Value, graph: &Graph<'_>) -> Option<CheckedNodeId> {
    let Some([unit, exponent]) = aggregate_members(term) else {
        return None;
    };
    let unit = binding(unit, "unit").and_then(reference_target)?;
    // QSL requires each term to name its dimension's canonical root unit.
    // A unit whose preimage has no target unit is that root: the nominal
    // stage admits exactly one per dimension.
    let is_root_unit = graph.node(&unit).is_some_and(|(node, kind)| {
        kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
            && matches!(
                &node.nominal_identity_preimage,
                Some(NominalIdentityPreimage::Unit(preimage))
                    if preimage.target_unit_node_id.is_none()
            )
    });
    let exponent_ok = binding(exponent, "exponent")
        .and_then(|value| literal(value, "integer", ScalarTypeForm::Integer, graph))
        .and_then(Value::as_str)
        .is_some_and(is_nonzero_integer);
    (is_root_unit && exponent_ok).then_some(unit)
}

/// FR-322's dependency join for a body that contains an `application` term
/// at any depth: `None` for a body holding none, else the unique
/// digest-ascending reference targets and member declarations. A member
/// `declaration` that is not a node key is skipped here: the operation stage
/// owns that refusal and its path.
fn application_dependencies(body: &Value) -> Option<BTreeSet<CheckedNodeId>> {
    let mut contains_application = false;
    let mut join = BTreeSet::new();
    let mut pending = vec![body];
    while let Some(term) = pending.pop() {
        let Some(object) = term.as_object() else {
            continue;
        };
        match object.get("term").and_then(Value::as_str) {
            Some("reference") => {
                if let Some(target) = reference_target(term) {
                    join.insert(target);
                }
            }
            Some("application") => {
                contains_application = true;
                let declaration = object
                    .get("operation")
                    .and_then(|operation| operation.get("member"))
                    .and_then(|member| member.get("declaration"))
                    .and_then(|declaration| {
                        serde_json::from_value::<CheckedNodeId>(declaration.clone()).ok()
                    });
                join.extend(declaration);
                pending.extend(terms(object, "arguments"));
            }
            Some("aggregate") => pending.extend(terms(object, "members")),
            Some("binding") => pending.extend(object.get("value")),
            // A literal names only its type annotation, which is not a
            // dependency; the body grammar has no other term.
            _ => {}
        }
    }
    contains_application.then_some(join)
}

fn terms<'a>(object: &'a Map<String, Value>, key: &str) -> impl Iterator<Item = &'a Value> {
    object
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// An `aggregate` term's members.
fn aggregate_members(term: &Value) -> Option<&[Value]> {
    let object = term.as_object()?;
    (object.get("term").and_then(Value::as_str) == Some("aggregate"))
        .then(|| object.get("members").and_then(Value::as_array))
        .flatten()
        .map(Vec::as_slice)
}

/// A `binding` term's value, when the binding is named `name`.
fn binding<'a>(term: &'a Value, name: &str) -> Option<&'a Value> {
    let object = term.as_object()?;
    (object.get("term").and_then(Value::as_str) == Some("binding")
        && object.get("name").and_then(Value::as_str) == Some(name))
    .then(|| object.get("value"))
    .flatten()
}

/// A `reference` term's target.
fn reference_target(term: &Value) -> Option<CheckedNodeId> {
    let object = term.as_object()?;
    if object.get("term").and_then(Value::as_str) != Some("reference") {
        return None;
    }
    serde_json::from_value(object.get("target")?.clone()).ok()
}

/// A `literal` term's value, when its `value_kind` is `value_kind` and its
/// `type` names a `scalar_type` node of form `type_form`.
fn literal<'a>(
    term: &'a Value,
    value_kind: &str,
    type_form: ScalarTypeForm,
    graph: &Graph<'_>,
) -> Option<&'a Value> {
    let object = term.as_object()?;
    if object.get("term").and_then(Value::as_str) != Some("literal")
        || object.get("value_kind").and_then(Value::as_str) != Some(value_kind)
    {
        return None;
    }
    let ty: CheckedNodeId = serde_json::from_value(object.get("type")?.clone()).ok()?;
    (graph.kind(&ty)? == CheckedNodeKind::ScalarType(type_form))
        .then(|| object.get("value"))
        .flatten()
}

/// `^(0|[1-9][0-9]*)$`.
fn is_non_negative_integer(value: &str) -> bool {
    value == "0" || is_positive_integer(value)
}

/// `^-?[1-9][0-9]*$`.
fn is_nonzero_integer(value: &str) -> bool {
    is_positive_integer(value.strip_prefix('-').unwrap_or(value))
}

/// `^[1-9][0-9]*$`.
fn is_positive_integer(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|first| (b'1'..=b'9').contains(&first))
        && bytes.all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Tracing: TC-048, FR-038-AC-22
    #[test]
    fn tc_048_structural_integer_grammars_are_exact() {
        assert!(is_non_negative_integer("0"));
        assert!(is_non_negative_integer("12"));
        assert!(!is_non_negative_integer("-1"));
        assert!(!is_non_negative_integer("01"));
        assert!(!is_non_negative_integer(""));
        assert!(is_nonzero_integer("-3"));
        assert!(is_nonzero_integer("2"));
        assert!(!is_nonzero_integer("0"));
        assert!(!is_nonzero_integer("-0"));
        assert!(!is_nonzero_integer("--1"));
    }

    fn id(fill: char) -> Value {
        json!({"domain": "quire.checked-semantic-node/v1", "digest": fill.to_string().repeat(64)})
    }

    /// Tracing: TC-048, FR-038-AC-23
    #[test]
    fn tc_048_application_join_collects_targets_and_member_declarations_only() {
        let body = json!({
            "term": "application",
            "operator": "query",
            "operation": {"identity": "x", "laws": [], "mode": null,
                "member": {"kind": "field", "declaration": id('c'), "name": "f"}, "leaves": []},
            "result_type": id('d'),
            "arguments": [
                {"term": "reference", "target": id('b')},
                {"term": "literal", "type": id('e'), "value_kind": "integer", "value": "1"},
                {"term": "aggregate", "members": [
                    {"term": "binding", "name": "x", "value": {"term": "reference", "target": id('a')}},
                    {"term": "reference", "target": id('b')},
                ]},
            ],
        });
        let join = application_dependencies(&body).expect("the body holds an application");
        let expected: BTreeSet<CheckedNodeId> = ['a', 'b', 'c']
            .into_iter()
            .map(|fill| serde_json::from_value(id(fill)).expect("node id"))
            .collect();
        assert_eq!(join, expected);
        assert_eq!(
            application_dependencies(&json!({"term": "reference", "target": id('a')})),
            None
        );
        // An unparseable member declaration is left to the operation stage.
        let mut malformed = body.clone();
        malformed["operation"]["member"]["declaration"] = json!("not a node key");
        let expected: BTreeSet<CheckedNodeId> = ['a', 'b']
            .into_iter()
            .map(|fill| serde_json::from_value(id(fill)).expect("node id"))
            .collect();
        assert_eq!(application_dependencies(&malformed), Some(expected));
    }
}
