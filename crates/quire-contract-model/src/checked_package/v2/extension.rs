//! FR-348 extension form body and declaration joins.

use super::identity::is_identifier;
use super::model_members::{Budget, DeclarationForm, ModelOwners};
use super::{
    BoundedDomainForm, CheckedNodeKind, CheckedTerm, ModelForm,
    RawCheckedPackageWireV2 as CheckedPackageWireV2, RawSemanticNodeV2 as CheckedSemanticNodeV2,
    ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedExtensionRefusalDetail, CheckedNodeId,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, JsonPointer,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExtensionBody {
    #[serde(rename = "term")]
    _term: ExtensionTerm,
    definition: CheckedArtifactRef,
    node_kind: Box<str>,
    clauses: Vec<ExtensionClause>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtensionClause {
    #[serde(rename = "term")]
    _term: BindingTerm,
    name: Box<str>,
    value: ExtensionReference,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtensionReference {
    #[serde(rename = "term")]
    _term: ReferenceTerm,
    target: CheckedNodeId,
}

#[derive(Deserialize)]
enum ExtensionTerm {
    #[serde(rename = "extension_form")]
    ExtensionForm,
}

#[derive(Deserialize)]
enum BindingTerm {
    #[serde(rename = "binding")]
    Binding,
}

#[derive(Deserialize)]
enum ReferenceTerm {
    #[serde(rename = "reference")]
    Reference,
}

/// The body has its own closed grammar, checked before key authentication.
#[ix_trace_rs::string_edge]
pub(super) fn read_body(body: &Value, at: JsonPointer) -> Result<ExtensionBody, ValidationFailure> {
    let parsed: ExtensionBody = serde_json::from_value(body.clone())
        .map_err(|_| ValidationFailure::refused(Code::MalformedWire, at.clone()))?;
    if parsed.definition.authority.is_empty()
        || parsed.definition.identity.is_empty()
        || !is_identifier(&parsed.node_kind)
        || parsed
            .clauses
            .iter()
            .any(|clause| !is_identifier(&clause.name) || !clause.value.target.is_well_formed())
    {
        return Err(ValidationFailure::refused(Code::MalformedWire, at));
    }
    Ok(parsed)
}

pub(super) fn targets(body: &ExtensionBody) -> impl Iterator<Item = &CheckedNodeId> {
    body.clauses.iter().map(|clause| &clause.value.target)
}

pub(super) fn body_work(body: &ExtensionBody) -> u64 {
    u64::try_from(body.clauses.len())
        .unwrap_or(u64::MAX)
        .saturating_mul(2)
        .saturating_add(1)
}

pub(super) fn validate_source_regions(
    wire: &CheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
) -> Result<(), ValidationFailure> {
    let extensions = wire
        .semantic_graph
        .nodes
        .iter()
        .zip(kinds)
        .filter(|(_, kind)| matches!(kind, CheckedNodeKind::Extension(_)))
        .map(|(node, _)| &node.node_id)
        .collect::<BTreeSet<_>>();
    for (entry_index, entry) in wire.source_map.iter().enumerate() {
        if extensions.contains(&entry.node_id) && entry.regions.len() != 1 {
            return Err(ValidationFailure::refused_at(
                Code::InvalidSourceMap,
                JsonPointer::root()
                    .key("source_map")
                    .index(entry_index)
                    .key("regions"),
                None,
                entry.node_id.clone(),
            ));
        }
    }
    Ok(())
}

fn refusal(
    node: &CheckedSemanticNodeV2,
    pointer: JsonPointer,
    code: Code,
    cause: Option<Cause>,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, pointer, cause, node.node_id.clone())
}

fn with_detail(
    mut failure: ValidationFailure,
    member: &str,
    expected: Option<&str>,
    actual: Option<String>,
    actual_semantic_type: Option<CheckedNodeId>,
) -> ValidationFailure {
    if let ValidationFailure::Refused(ref mut refusal) = failure {
        refusal.extension_detail = Some(Box::new(CheckedExtensionRefusalDetail {
            member: member.into(),
            expected_type: expected.map(Into::into),
            actual_type: actual.map(Into::into),
            actual_semantic_type,
        }));
    }
    failure
}

/// Joins selected and held definitions, node kinds, clauses and clause types.
pub(super) fn validate(
    wire: &CheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    evidence: &CheckedPackageEvidence,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let nodes = &wire.semantic_graph.nodes;
    for &position in index.values() {
        if !matches!(kinds.get(position), Some(CheckedNodeKind::Extension(_))) {
            continue;
        }
        let node = &nodes[position];
        let at = node_pointer(position).key("body");
        let Some(CheckedTerm::ExtensionForm {
            definition,
            node_kind,
            clauses,
        }) = terms.get(position).and_then(Option::as_ref)
        else {
            return Err(ValidationFailure::refused(Code::MalformedWire, at));
        };
        let clauses = clauses
            .iter()
            .map(|clause| match clause {
                CheckedTerm::Binding { name, value } => match value.as_ref() {
                    CheckedTerm::Reference { target } => Some((name.as_ref(), target)),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| ValidationFailure::refused(Code::MalformedWire, at.clone()))?;
        let def_at = at.clone().key("definition");
        let fail = |code, cause, pointer| refusal(node, pointer, code, cause);
        if !wire.lock.definition_selections.contains(definition) {
            return Err(fail(
                Code::MissingDeclaration,
                Some(Cause::MissingSelection),
                def_at,
            ));
        }
        let Some(definition) = evidence.extension_definition(definition) else {
            return Err(ValidationFailure::unknown_extension_definition(
                def_at,
                node.node_id.clone(),
            ));
        };
        let Some(schema) = definition
            .typed_node_schema
            .nodes
            .iter()
            .find(|entry| entry.node_kind.as_ref() == node_kind.as_ref())
        else {
            return Err(fail(
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                at.key("node_kind"),
            ));
        };
        for (ordinal, ((name, _), expected)) in clauses.iter().zip(&schema.clauses).enumerate() {
            if *name != expected.name.as_ref() {
                return Err(fail(
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    at.clone().key("clauses").index(ordinal),
                ));
            }
        }
        if clauses.len() > schema.clauses.len() {
            return Err(fail(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                at.clone().key("clauses").index(schema.clauses.len()),
            ));
        }
        if clauses.len() < schema.clauses.len() {
            return Err(with_detail(
                fail(
                    Code::InvalidPackage,
                    Some(Cause::MissingMember),
                    at.clone().key("clauses"),
                ),
                &schema.clauses[clauses.len()].name,
                None,
                None,
                None,
            ));
        }
        for (ordinal, ((name, target_id), expected)) in
            clauses.iter().zip(&schema.clauses).enumerate()
        {
            let pointer = at.clone().key("clauses").index(ordinal).key("value");
            let Some(&target_position) = index.get(target_id) else {
                return Err(fail(
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    pointer,
                ));
            };
            let target = &nodes[target_position];
            if !type_matches(
                &expected.type_ref,
                nodes,
                kinds,
                terms,
                target_position,
                index,
                owners,
                meter,
            )? {
                return Err(with_detail(
                    fail(Code::IllTyped, Some(Cause::TypeMismatch), pointer),
                    name,
                    Some(&expected.type_ref),
                    Some(actual_type(
                        target,
                        kinds[target_position],
                        nodes,
                        kinds,
                        terms,
                        target_position,
                        index,
                    )),
                    Some(target.semantic_type.clone()),
                ));
            }
        }
    }
    Ok(())
}

fn integer_range_bounds(term: &CheckedTerm) -> Option<(&str, &str)> {
    fn bound<'a>(term: &'a CheckedTerm, expected: &str) -> Option<&'a str> {
        let CheckedTerm::Binding { name, value } = term else {
            return None;
        };
        if name.as_ref() != expected {
            return None;
        }
        let CheckedTerm::Literal { value, .. } = value.as_ref() else {
            return None;
        };
        value.as_str()
    }
    let CheckedTerm::Aggregate { members } = term else {
        return None;
    };
    let [lo, hi] = members.as_slice() else {
        return None;
    };
    Some((bound(lo, "min")?, bound(hi, "max")?))
}

enum ExpectedType {
    IntegerRange(Box<str>),
    Scalar(ScalarTypeForm),
    Other,
}

impl ExpectedType {
    #[ix_trace_rs::string_edge]
    fn from_text(expected: &str) -> Self {
        if let Some(bounds) = expected
            .strip_prefix("Int[")
            .and_then(|tail| tail.strip_suffix(']'))
        {
            return Self::IntegerRange(bounds.into());
        }
        let scalar = match expected {
            "Boolean" => Some(ScalarTypeForm::Boolean),
            "Integer" | "Int" => Some(ScalarTypeForm::Integer),
            "Rational" => Some(ScalarTypeForm::Rational),
            "Decimal" => Some(ScalarTypeForm::Decimal),
            "Float32" => Some(ScalarTypeForm::Float32),
            "Float64" => Some(ScalarTypeForm::Float64),
            "Text" => Some(ScalarTypeForm::Text),
            "String" => Some(ScalarTypeForm::String),
            "UUID" => Some(ScalarTypeForm::Uuid),
            "Timestamp" => Some(ScalarTypeForm::Timestamp),
            _ => None,
        };
        scalar.map_or(Self::Other, Self::Scalar)
    }
}

fn type_matches(
    expected: &str,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    target_position: usize,
    index: &BTreeMap<&CheckedNodeId, usize>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let Some((target, &kind)) = nodes.get(target_position).zip(kinds.get(target_position)) else {
        return Ok(false);
    };
    if !matches!(
        kind,
        CheckedNodeKind::Value(_) | CheckedNodeKind::Expression(_) | CheckedNodeKind::Extension(_)
    ) {
        return Ok(false);
    }
    let expected_type = ExpectedType::from_text(expected);
    if let ExpectedType::IntegerRange(bound) = &expected_type {
        let Some((lo, hi)) = bound.split_once(',') else {
            return Ok(false);
        };
        let Some(&position) = index.get(&target.semantic_type) else {
            return Ok(false);
        };
        let bounds_match = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(integer_range_bounds)
            .is_some_and(|(lower, upper)| lower == lo.trim() && upper == hi.trim());
        return Ok(kinds.get(position)
            == Some(&CheckedNodeKind::BoundedDomain(
                BoundedDomainForm::IntegerRange,
            ))
            && bounds_match);
    }
    if let ExpectedType::Scalar(form) = expected_type {
        return Ok(index
            .get(&target.semantic_type)
            .and_then(|&position| nodes.get(position).zip(kinds.get(position)))
            .is_some_and(|(_, kind)| *kind == CheckedNodeKind::ScalarType(form)));
    }
    if let Some((type_node, type_kind)) = index
        .get(&target.semantic_type)
        .and_then(|&position| nodes.get(position).zip(kinds.get(position)))
    {
        if *type_kind == CheckedNodeKind::Model(ModelForm::RecordValueType) {
            if let Ok(actual) = owners.recover(type_node) {
                if actual.form == DeclarationForm::RecordValueType
                    && actual.package.record_types.contains_key(expected)
                {
                    let mut budget = Budget::new(meter, actual.package.root, actual.package.bytes);
                    return actual
                        .package
                        .conforms_to(actual.node, expected, &mut budget);
                }
            }
        }
    }
    Ok(matches!(kind, CheckedNodeKind::Extension(_))
        && matches!(terms.get(target_position).and_then(Option::as_ref),
            Some(CheckedTerm::ExtensionForm { node_kind, .. }) if node_kind.as_ref() == expected))
}

fn actual_type(
    target: &CheckedSemanticNodeV2,
    kind: CheckedNodeKind,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    target_position: usize,
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> String {
    if matches!(kind, CheckedNodeKind::Extension(_)) {
        return match terms.get(target_position).and_then(Option::as_ref) {
            Some(CheckedTerm::ExtensionForm { node_kind, .. }) => node_kind.to_string(),
            _ => "extension".into(),
        };
    }
    let structural_id = || {
        quire_canonical::to_vec(
            &target.semantic_type,
            quire_canonical::Limits::new(u64::MAX),
        )
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_else(|| "unknown structural type".into())
    };
    let Some(&type_position) = index.get(&target.semantic_type) else {
        return structural_id();
    };
    let Some((type_node, type_kind)) = nodes.get(type_position).zip(kinds.get(type_position))
    else {
        return structural_id();
    };
    if *type_kind == CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange) {
        if let Some((lo, hi)) = terms
            .get(type_position)
            .and_then(Option::as_ref)
            .and_then(integer_range_bounds)
        {
            return format!("Int[{lo}, {hi}]");
        }
    }
    match type_kind {
        CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean) => "Boolean".into(),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Integer) => "Integer".into(),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Rational) => "Rational".into(),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Decimal) => "Decimal".into(),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Float32) => "Float32".into(),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Float64) => "Float64".into(),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Text) => "Text".into(),
        CheckedNodeKind::ScalarType(ScalarTypeForm::String) => "String".into(),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Uuid) => "UUID".into(),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Timestamp) => "Timestamp".into(),
        _ => type_node
            .declaration
            .as_ref()
            .map(|declaration| declaration.qualified_name.join("."))
            .unwrap_or_else(|| type_node.semantic_form.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::evidence::CheckedExtensionDefinition;
    use crate::checked_package::v2::{
        test_support::node_id, BoundedDomainForm, ExtensionForm, ModelForm, ValueForm,
    };
    use ix_trace_rs::trace;
    use serde_json::json;

    /// This graph enters the actual extension join directly. Its metadata/kind
    /// mismatches are deliberately private states, not newly admitted packages.
    struct Join {
        wire: CheckedPackageWireV2,
        kinds: Vec<CheckedNodeKind>,
        terms: Vec<Option<CheckedTerm>>,
        evidence: CheckedPackageEvidence,
    }

    impl Join {
        fn new(expected: &str, kind: CheckedNodeKind) -> Self {
            let definition = json!({"authority": "test", "identity": "open-extension"});
            let edition = json!({"role": "edition", "definition": definition});
            let mut wire: CheckedPackageWireV2 = serde_json::from_value(json!({
                "contract_version": super::super::CHECKED_PACKAGE_V2,
                "identity_preimage": {
                    "version": "quire.checked-package-id/v2", "edition": edition,
                    "profile_selections": [], "definition_selections": [],
                    "required_features": [], "dependency_selections": []
                },
                "package_id": {"domain": super::super::PACKAGE_DOMAIN_V2,
                    "algorithm": "sha256", "digest": "a".repeat(64)},
                "lock": {"sources": [], "edition": edition, "profile_selections": [],
                    "definition_selections": [definition],
                    "required_features": [], "dependency_selections": []},
                "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": []},
                "source_map": [], "capability_report": [],
                "diagnostics": {"catalog": definition, "entries": []}
            }))
            .expect("private join wire");
            let node = |id: char, kind: CheckedNodeKind, ty: CheckedNodeId| {
                serde_json::from_value(json!({
                    "node_id": node_id(id), "schema_version": "quire.checked-semantic-graph/v2",
                    "node_tag": kind.tag().as_wire(), "semantic_form": kind.form_wire(),
                    "semantic_type": ty, "dependencies": [], "occurrences": [],
                    "body": {"term": "aggregate", "members": []}
                }))
                .expect("private graph node")
            };
            wire.semantic_graph.nodes = vec![
                node('0', kind, node_id('0')),
                node(
                    '1',
                    CheckedNodeKind::Value(ValueForm::Parameter),
                    node_id('0'),
                ),
                node(
                    '2',
                    CheckedNodeKind::Extension(ExtensionForm::ExtensionForm),
                    node_id('2'),
                ),
            ];
            let extension = CheckedTerm::ExtensionForm {
                definition: serde_json::from_value(definition).expect("definition ref"),
                node_kind: "open-kind".into(),
                clauses: vec![CheckedTerm::Binding {
                    name: "subject".into(),
                    value: Box::new(CheckedTerm::Reference {
                        target: node_id('1'),
                    }),
                }],
            };
            let mut evidence = CheckedPackageEvidence::new();
            let schema: CheckedExtensionDefinition = serde_json::from_value(json!({
                "grammar_schema": {"productions": [{"keyword": "open", "clauses": ["subject"]},
                    {"keyword": "future", "clauses": []}]},
                "typed_node_schema": {"nodes": [{"keyword": "open", "node_kind": "open-kind",
                    "clauses": [{"name": "subject", "type": expected}]},
                        {"keyword": "future", "node_kind": "future-kind", "clauses": []}]}
            }))
            .expect("held definition");
            evidence
                .insert_extension_definition(wire.lock.definition_selections[0].clone(), schema)
                .expect("definition evidence");
            Self {
                wire,
                kinds: vec![
                    kind,
                    CheckedNodeKind::Value(ValueForm::Parameter),
                    CheckedNodeKind::Extension(ExtensionForm::ExtensionForm),
                ],
                terms: vec![
                    Some(CheckedTerm::Aggregate { members: vec![] }),
                    None,
                    Some(extension),
                ],
                evidence,
            }
        }

        fn run(&self) -> (Result<(), ValidationFailure>, u64) {
            let owners = ModelOwners::new(&[], |_| Ok(())).expect("empty selected-model owners");
            self.run_with_owners(&owners)
        }

        fn run_with_owners(
            &self,
            owners: &ModelOwners<'_>,
        ) -> (Result<(), ValidationFailure>, u64) {
            let index = self
                .wire
                .semantic_graph
                .nodes
                .iter()
                .enumerate()
                .map(|(position, node)| (&node.node_id, position))
                .collect();
            let mut meter = WorkMeter::new(100_000);
            let result = validate(
                &self.wire,
                &self.kinds,
                &self.terms,
                &index,
                &self.evidence,
                owners,
                &mut meter,
            );
            (result, meter.consumed())
        }

        fn range(&mut self) {
            let bound = |name: &str, text: &str| CheckedTerm::Binding {
                name: name.into(),
                value: Box::new(CheckedTerm::Literal {
                    type_id: node_id('0'),
                    value_kind: super::super::LiteralKind::Integer,
                    value: json!(text),
                }),
            };
            self.terms[0] = Some(CheckedTerm::Aggregate {
                members: vec![bound("min", "1"), bound("max", "3650")],
            });
        }

        fn mismatch(&self, expected: &str, actual: &str) {
            let (result, work) = self.run();
            let Err(ValidationFailure::Refused(refusal)) = result else {
                panic!("expected extension mismatch, got {result:?}");
            };
            assert_eq!(
                (refusal.code, refusal.cause),
                (Code::IllTyped, Some(Cause::TypeMismatch))
            );
            assert_eq!(
                refusal.loci[0].path.clone().expect("path").to_string(),
                "/semantic_graph/nodes/2/body/clauses/0/value"
            );
            assert_eq!(refusal.locus, Some(node_id('2')));
            let detail = refusal.extension_detail.expect("detail");
            assert_eq!(detail.member.as_ref(), "subject");
            assert_eq!(detail.expected_type.as_deref(), Some(expected));
            assert_eq!(detail.actual_type.as_deref(), Some(actual));
            assert_eq!(
                detail.actual_semantic_type,
                Some(self.wire.semantic_graph.nodes[1].semantic_type.clone())
            );
            assert_eq!(
                work, 0,
                "native type checks charge no model-conformance work"
            );
        }
    }

    #[trace("TC-459")]
    #[test]
    fn extension_join_scalar_dispatch_uses_retained_type_kind() {
        for (name, form) in [
            ("Boolean", ScalarTypeForm::Boolean),
            ("Integer", ScalarTypeForm::Integer),
            ("Int", ScalarTypeForm::Integer),
            ("Rational", ScalarTypeForm::Rational),
            ("Decimal", ScalarTypeForm::Decimal),
            ("Float32", ScalarTypeForm::Float32),
            ("Float64", ScalarTypeForm::Float64),
            ("Text", ScalarTypeForm::Text),
            ("String", ScalarTypeForm::String),
            ("UUID", ScalarTypeForm::Uuid),
            ("Timestamp", ScalarTypeForm::Timestamp),
        ] {
            let mut join = Join::new(name, CheckedNodeKind::ScalarType(form));
            let (baseline, work) = join.run();
            assert!(baseline.is_ok(), "{name}: {baseline:?}");
            join.wire.semantic_graph.nodes[0].node_tag = "model".into();
            join.wire.semantic_graph.nodes[0].semantic_form = "record_value_type".into();
            let (damaged, damaged_work) = join.run();
            assert!(
                damaged.is_ok(),
                "retained {name} must match despite private raw damage: {damaged:?}"
            );
            assert_eq!(damaged_work, work);

            let mut inverse = Join::new(name, CheckedNodeKind::ScalarType(form));
            inverse.kinds[0] = CheckedNodeKind::Model(ModelForm::ObjectType);
            inverse.mismatch(name, form.as_wire());
        }
    }

    #[trace("TC-459")]
    #[test]
    fn extension_join_range_dispatch_and_diagnostic_use_retained_kind() {
        let range = CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange);
        let mut join = Join::new("Int[1, 3650]", range);
        join.range();
        assert!(join.run().0.is_ok());
        join.wire.semantic_graph.nodes[0].node_tag = "scalar_type".into();
        join.wire.semantic_graph.nodes[0].semantic_form = "text".into();
        assert!(
            join.run().0.is_ok(),
            "private raw damage cannot change retained range"
        );
        let mut inverse = Join::new("Int[1, 3650]", range);
        inverse.range();
        inverse.kinds[0] = CheckedNodeKind::ScalarType(ScalarTypeForm::Text);
        inverse.mismatch("Int[1, 3650]", "Text");
        let mut diagnostic = Join::new("Boolean", range);
        diagnostic.range();
        diagnostic.wire.semantic_graph.nodes[0].node_tag = "scalar_type".into();
        diagnostic.wire.semantic_graph.nodes[0].semantic_form = "text".into();
        diagnostic.mismatch("Boolean", "Int[1, 3650]");
    }

    #[trace("TC-459")]
    #[test]
    fn extension_join_scalar_diagnostic_uses_retained_kind() {
        let mut join = Join::new("Boolean", CheckedNodeKind::ScalarType(ScalarTypeForm::Text));
        join.wire.semantic_graph.nodes[0].semantic_form = "integer".into();
        join.mismatch("Boolean", "Text");
    }
    #[trace("TC-459")]
    #[test]
    fn extension_join_selected_record_dispatch_preserves_owner_conformance() {
        use crate::checked_package::v2::model_members::{declaration_key, tests as model_tests};
        let base = "ix://acme/orders/Base";
        let child = "ix://acme/orders/Child";
        let mut types = vec![
            model_tests::object_type(base, &[], vec![]),
            model_tests::object_type(child, &[base], vec![]),
        ];
        for ty in &mut types {
            ty["kind"]["name"] = json!("record");
        }
        let mut document = model_tests::document(types);
        document["constructs"]
            .as_array_mut()
            .expect("constructs")
            .push(json!({
                "kind": {"module": "acme/orders", "name": "record"},
                "construct": {"meaning": "quire.meaning.model.record-value-type/v1"}
            }));
        let model = model_tests::read(&document).expect("genuine selected record document");
        let owners = ModelOwners::new(std::slice::from_ref(&model), |_| Ok(())).expect("owners");
        let mut join = Join::new(base, CheckedNodeKind::Model(ModelForm::RecordValueType));
        let key = declaration_key(
            "acme/orders",
            DeclarationForm::RecordValueType,
            child,
            model_tests::BYTES,
        )
        .expect("child declaration key");
        let ty = &mut join.wire.semantic_graph.nodes[0];
        ty.node_id.digest = key.into();
        ty.semantic_type = ty.node_id.clone();
        ty.owner = Some(
            serde_json::from_value(json!({
                "kind": "model", "identity": "acme/orders", "node": child
            }))
            .expect("owner"),
        );
        join.wire.semantic_graph.nodes[1].semantic_type = ty.node_id.clone();
        let (baseline, baseline_work) = join.run_with_owners(&owners);
        assert!(
            baseline.is_ok(),
            "directional record conformance: {baseline:?}"
        );
        assert!(
            baseline_work > 0,
            "real owner conformance must charge actual work"
        );
        join.kinds[0] = CheckedNodeKind::Model(ModelForm::ObjectType);
        let (result, work) = join.run_with_owners(&owners);
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("nonrecord retained kind cannot enter record conformance: {result:?}");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::IllTyped, Some(Cause::TypeMismatch))
        );
        assert_eq!(
            refusal.loci[0].path.clone().expect("path").to_string(),
            "/semantic_graph/nodes/2/body/clauses/0/value"
        );
        assert_eq!(
            work, 0,
            "ineligible kind must not charge owner-conformance work"
        );
        println!("extension_record baseline_work={baseline_work} retained_nonrecord_work={work}");
    }

    #[trace("TC-459")]
    #[test]
    fn extension_join_preserves_open_kind_names_and_selection_precedence() {
        let kind = CheckedNodeKind::Extension(ExtensionForm::ExtensionForm);
        let mut join = Join::new(
            "future-kind",
            CheckedNodeKind::ScalarType(ScalarTypeForm::Text),
        );
        join.kinds[1] = kind;
        join.terms[1] = Some(CheckedTerm::ExtensionForm {
            definition: join.wire.lock.definition_selections[0].clone(),
            node_kind: "future-kind".into(),
            clauses: vec![],
        });
        assert!(join.run().0.is_ok(), "open extension name must remain data");
        join.wire.lock.definition_selections.clear();
        join.terms[1] = None;
        join.kinds[1] = CheckedNodeKind::Value(ValueForm::Parameter);
        let (result, work) = join.run();
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("missing selection must precede target typing: {result:?}");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::MissingDeclaration, Some(Cause::MissingSelection))
        );
        assert_eq!(
            refusal.loci[0].path.clone().expect("path").to_string(),
            "/semantic_graph/nodes/2/body/definition"
        );
        assert_eq!(work, 0);
    }
}
