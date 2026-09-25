//! QSpec's `model-member-type-vectors.json` (TC-280, TC-281) through this
//! reader: owner recovery, member resolution, member types, selection
//! admission, reference equality and model field projection, each decided by
//! the same functions a `read` runs.
//!
//! The vectors and the operation catalog they are decided against are read
//! at run time from the quire-specification checkout `QSPEC_DIR` names;
//! nothing of QSpec is copied into this repository. Each test skips (and
//! passes) when `QSPEC_DIR` is unset; `make qspec-vectors` requires it and
//! fails when a test printed no `conformance:` line.
//!
//! The vectors' `domain_package` is a projection of the declaration records
//! FR-154 intake reads, not a Semantic IR document. The projection gives an
//! operation a `redefines` member (vectors MT-08 and MT-10 rely on it); a
//! Semantic IR 2.0.0 operation carries none (FCD FR-141-AC-3 publishes only
//! `field.redefines`), so [`super::super::model_members::read_semantic_ir`]
//! reads none and those two cases are decided here from the projection alone.
//! [`read_projection`]
//! reads it into the same [`DomainModel`] [`super::super::model_members::read_semantic_ir`]
//! builds from a real document.

use super::super::model_members::{
    admit_document, declaration_key, member_name, Budget, CollectionKind, DeclarationForm,
    DomainModel, FieldDecl, IntegerBounds, MemberKind, MemberType, ModelOwners, Multiplicity,
    ObjectTypeDecl, OperationDecl, SelectionFailure, TypedSlot,
};
use super::super::operation_catalog::{parse_catalog, OperationCatalog};
use super::super::{CheckedDomainPackageRef, CheckedPackageLockV2, CheckedSemanticNodeV2};
use super::{operation_defect, Application, Graph};
use crate::checked_package::common::{digest_json, NODE_DOMAIN};
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedNodeId, CheckedRevision, CheckedSelection,
};
use crate::checked_package::v2::{
    CheckedNodeKind, CheckedNodeTag, CheckedSelectionRole, WorkMeter,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const VECTORS: &str = "proposals/checked-package-v2/model-member-type-vectors.json";
const CATALOG: &str = "proposals/checked-package-v2/operation-catalog.json";

/// A file of the QSpec checkout `QSPEC_DIR` names, or `None` when unset.
fn qspec(relative: &str) -> Option<String> {
    let root = std::env::var_os("QSPEC_DIR")?;
    let path = std::path::Path::new(&root).join(relative);
    Some(
        std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
}

/// The parsed vectors and QSpec's catalog, or `None` (after saying so) when
/// `QSPEC_DIR` is unset.
fn vectors() -> Option<(Value, OperationCatalog)> {
    let (Some(vectors), Some(catalog)) = (qspec(VECTORS), qspec(CATALOG)) else {
        println!("skipped: QSPEC_DIR not set");
        return None;
    };
    Some((
        serde_json::from_str(&vectors).expect("the vectors are JSON"),
        parse_catalog(&catalog),
    ))
}

fn text<'v>(value: &'v Value, member: &str) -> &'v str {
    value[member]
        .as_str()
        .unwrap_or_else(|| panic!("{member} is a string in {value}"))
}

fn array<'v>(value: &'v Value, member: &str) -> &'v [Value] {
    value[member]
        .as_array()
        .unwrap_or_else(|| panic!("{member} is an array in {value}"))
}

fn projection_multiplicity(value: &Value) -> Multiplicity {
    let number = |member: &str| text(value, member).parse::<u64>().expect("a bound");
    Multiplicity {
        lower: number("lower"),
        upper: (text(value, "upper") != "unbounded").then(|| number("upper")),
        ordered: value["ordered"].as_bool().expect("ordered"),
        unique: value["unique"].as_bool().expect("unique"),
    }
}

fn projection_slot(value: &Value) -> TypedSlot {
    TypedSlot {
        type_ref: text(value, "typeRef").into(),
        multiplicity: projection_multiplicity(&value["multiplicity"]),
    }
}

fn projection_redefines(value: &Value) -> Option<Box<str>> {
    value["redefines"].as_str().map(Box::from)
}

/// The vectors' `domain_package` projection read into a [`DomainModel`].
fn read_projection(package: &Value) -> DomainModel {
    let object_types = array(package, "object_types")
        .iter()
        .map(|object| {
            let node = text(object, "node");
            let member = |member: &Value| format!("{node}/{}", text(member, "name")).into();
            let declared = ObjectTypeDecl {
                interface: !object["interfaceFeatures"].is_null(),
                supertypes: array(object, "supertypes")
                    .iter()
                    .map(|supertype| supertype.as_str().expect("a supertype").into())
                    .collect(),
                fields: array(object, "fields")
                    .iter()
                    .map(|field| FieldDecl {
                        identity: member(field),
                        slot: projection_slot(field),
                        optional: text(field, "presence") == "optional",
                        redefines: projection_redefines(field),
                    })
                    .collect(),
                operations: array(object, "operations")
                    .iter()
                    .map(|operation| OperationDecl {
                        identity: member(operation),
                        parameters: array(operation, "parameters")
                            .iter()
                            .map(projection_slot)
                            .collect(),
                        result: (!operation["result"].is_null())
                            .then(|| projection_slot(&operation["result"])),
                        redefines: projection_redefines(operation),
                    })
                    .collect(),
            };
            (node.into(), declared)
        })
        .collect();
    let value_types = array(package, "value_types")
        .iter()
        .map(|value_type| {
            let bound = |member: &str| text(value_type, member).parse::<i128>().ok();
            let bounds = (text(value_type, "native") == "ix://quire/native/Integer")
                .then(|| {
                    Some(IntegerBounds {
                        lower: bound("lower")?,
                        upper: bound("upper")?,
                    })
                })
                .flatten()
                .filter(|bounds| bounds.lower <= bounds.upper);
            (text(value_type, "node").into(), bounds)
        })
        .collect();
    DomainModel {
        identity: text(package, "identity").into(),
        version: text(package, "version").into(),
        object_types,
        value_types,
        relationships: array(package, "relationships")
            .iter()
            .map(|relationship| text(relationship, "node").into())
            .collect(),
        other_types: Default::default(),
    }
}

/// The projection's own top-level `identity` and `version`.
fn projection_identity(document: &Value) -> Option<(&str, &str)> {
    Some((
        document.get("identity")?.as_str()?,
        document.get("version")?.as_str()?,
    ))
}

/// A vector's `{type: ...}` member type.
fn member_type(value: &Value) -> MemberType {
    let bound = |member: &str| text(value, member).parse::<u64>().expect("a bound");
    let collection = |kind| MemberType::Collection {
        kind,
        element: Box::new(member_type(&value["element"])),
        bounds: value.get("min").map(|_| (bound("min"), bound("max"))),
    };
    match text(value, "type") {
        "Boolean" => MemberType::Boolean,
        "Integer" => MemberType::Integer,
        "Int" => MemberType::IntRange(IntegerBounds {
            lower: text(value, "min").parse().expect("min"),
            upper: text(value, "max").parse().expect("max"),
        }),
        "Reference" => MemberType::Reference(text(value, "target").into()),
        "Option" => MemberType::Option(Box::new(member_type(&value["inner"]))),
        "Set" => collection(CollectionKind::Set),
        "Bag" => collection(CollectionKind::Bag),
        "Sequence" => collection(CollectionKind::Sequence),
        "OrderedSet" => collection(CollectionKind::OrderedSet),
        other => panic!("unexpected member type {other}"),
    }
}

fn node_ref(digest: &str) -> Value {
    json!({"domain": NODE_DOMAIN, "digest": digest})
}

fn reference(digest: &str) -> Value {
    json!({"term": "reference", "target": node_ref(digest)})
}

/// A graph assembled node by node for one case.
#[derive(Default)]
struct CaseGraph {
    nodes: Vec<Value>,
}

impl CaseGraph {
    /// Adds a node and returns its digest.
    fn add(
        &mut self,
        digest: &str,
        tag: &str,
        form: &str,
        semantic_type: &str,
        body: Value,
    ) -> String {
        if !self
            .nodes
            .iter()
            .any(|node| node["node_id"]["digest"] == digest)
        {
            self.nodes.push(json!({
                "node_id": node_ref(digest),
                "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": tag,
                "semantic_form": form,
                "semantic_type": node_ref(semantic_type),
                "dependencies": [],
                "occurrences": [],
                "body": body,
            }));
        }
        digest.to_owned()
    }

    /// Adds a node keyed by the digest of its own content.
    fn add_keyed(&mut self, tag: &str, form: &str, semantic_type: &str, body: Value) -> String {
        let digest = digest_json(&json!([tag, form, semantic_type, body])).expect("digest");
        self.add(&digest, tag, form, semantic_type, body)
    }

    /// A published model declaration node vector as its wire node.
    fn model_node(&mut self, vector: &Value) -> String {
        let digest = text(vector, "sha256");
        let preimage = &vector["preimage"];
        self.add(
            digest,
            text(preimage, "node_tag"),
            text(preimage, "semantic_form"),
            digest,
            json!({"term": "aggregate", "members": []}),
        )
    }

    /// The `composite_type`/`reference` node over `target`.
    fn reference_type(&mut self, target: &str) -> String {
        let key = MemberType::Reference(target.into()).node_key();
        self.add(
            &key,
            "composite_type",
            "reference",
            &key,
            json!({"term": "aggregate", "members": [reference(target)]}),
        )
    }

    /// A parameter value of type `type_digest`.
    fn value_of(&mut self, type_digest: &str) -> String {
        let label = format!("v{}", self.nodes.len());
        self.add_keyed(
            "value",
            "parameter",
            type_digest,
            json!({"term": "aggregate", "members": [
                {"term": "binding", "name": "name", "value": {"term": "literal", "value": label}},
            ]}),
        )
    }

    /// An application node.
    fn application(
        &mut self,
        form: &str,
        operator: &str,
        identity: &str,
        member: Value,
        arguments: &[String],
        result_type: &str,
    ) -> String {
        self.add_keyed(
            "expression",
            form,
            result_type,
            json!({
                "term": "application",
                "operator": operator,
                "operation": {
                    "identity": identity,
                    "laws": [], "mode": null, "member": member, "leaves": [],
                },
                "arguments": arguments.iter().map(|target| reference(target)).collect::<Vec<_>>(),
                "result_type": node_ref(result_type),
            }),
        )
    }

    /// `operation_defect` over the application `digest`: `None` when
    /// admitted, else the refusal's `{code, cause}` as the vectors spell it.
    fn decide(
        &self,
        digest: &str,
        owners: &ModelOwners<'_>,
        catalog: &OperationCatalog,
    ) -> Option<Value> {
        let nodes: Vec<CheckedSemanticNodeV2> = self
            .nodes
            .iter()
            .map(|node| serde_json::from_value(node.clone()).expect("a case node"))
            .collect();
        let kinds: Vec<CheckedNodeKind> = nodes
            .iter()
            .map(|node| {
                let tag = CheckedNodeTag::from_wire(&node.node_tag).expect("tag");
                CheckedNodeKind::decode(tag, &node.semantic_form).expect("form")
            })
            .collect();
        let index: BTreeMap<&CheckedNodeId, usize> = nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect();
        let position = nodes
            .iter()
            .position(|node| node.node_id.digest.as_ref() == digest)
            .expect("the application is in the graph");
        let graph = Graph {
            nodes: &nodes,
            kinds: &kinds,
            index: &index,
        };
        let mut meter = WorkMeter::new(u64::MAX);
        let failure = operation_defect(
            Application {
                node: &nodes[position],
                position,
            },
            &graph,
            &empty_lock(),
            owners,
            catalog,
            &mut meter,
        )
        .expect("no limit is reached")?;
        let crate::checked_package::common::ValidationFailure::Refused(refusal) = failure else {
            panic!("a refusal");
        };
        Some(json!({"refused": {
            "code": code_wire(refusal.code),
            "cause": cause_wire(refusal.cause.expect("a cause")),
        }}))
    }
}

fn code_wire(code: crate::checked_package::shared::CheckedPackageRefusalCode) -> &'static str {
    use crate::checked_package::shared::CheckedPackageRefusalCode as Code;
    match code {
        Code::IllTyped => "ill_typed",
        Code::MissingDeclaration => "missing_declaration",
        Code::AmbiguousDeclaration => "ambiguous_declaration",
        Code::InvalidPackage => "invalid_package",
        Code::StaleDependency => "stale_dependency",
        Code::MissingImport => "missing_import",
        Code::InvalidModelBinding => "invalid_model_binding",
        other => panic!("no vector names {other:?}"),
    }
}

fn cause_wire(cause: crate::checked_package::shared::CheckedPackageRefusalCause) -> &'static str {
    use crate::checked_package::shared::CheckedPackageRefusalCause as Cause;
    match cause {
        Cause::OperatorIneligible => "operator-ineligible",
        Cause::MissingSelection => "missing-selection",
        Cause::AmbiguousName => "ambiguous-name",
        Cause::StaleNodeKey => "stale-node-key",
        Cause::DigestDomainMismatch => "digest-domain-mismatch",
        Cause::ByteDigestMismatch => "byte-digest-mismatch",
        Cause::WrongModelSelection => "wrong-model-selection",
        other => panic!("no vector names {other:?}"),
    }
}

fn empty_lock() -> CheckedPackageLockV2 {
    let placeholder = CheckedArtifactRef {
        authority: "test".into(),
        identity: "test".into(),
        revision: CheckedRevision {
            namespace: "test".into(),
            value: "test".into(),
        },
        digest_domain: "sha256-jcs".into(),
        digest: "a".repeat(64).into(),
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

/// The published node vector named `name`.
fn node_vector<'v>(vectors: &'v Value, name: &str) -> &'v Value {
    array(vectors, "model_declaration_nodes")
        .iter()
        .find(|vector| vector["name"] == name)
        .unwrap_or_else(|| panic!("{name} is a node vector"))
}

fn owners_of(model: &DomainModel) -> ModelOwners<'_> {
    ModelOwners::new(std::slice::from_ref(model), |_| Ok::<(), ()>(())).expect("no charge fails")
}

/// TC-280 (FR-322-AC-29): every published model declaration node vector
/// recomputes to its digest by this reader's `declaration_key`, and the
/// selected package recovers exactly the vectors of its own version.
#[test]
fn tc_280_model_declaration_node_keys_recompute_and_recover_only_selected_owners() {
    let Some((vectors, _)) = vectors() else {
        return;
    };
    let model = read_projection(&vectors["domain_package"]);
    let owners = owners_of(&model);
    let nodes = array(&vectors, "model_declaration_nodes");
    for vector in nodes {
        let preimage = &vector["preimage"];
        let owner = &preimage["owner"];
        let form = match text(preimage, "semantic_form") {
            "object_type" => DeclarationForm::ObjectType,
            "systems_interface" => DeclarationForm::SystemsInterface,
            "relationship" => DeclarationForm::Relationship,
            other => panic!("unexpected form {other}"),
        };
        assert_eq!(
            declaration_key(
                text(owner, "identity"),
                text(owner, "version"),
                form,
                text(owner, "node")
            ),
            text(vector, "sha256"),
            "{}",
            text(vector, "name")
        );
        assert_eq!(
            digest_json(preimage).expect("digest"),
            text(vector, "sha256"),
            "{} preimage",
            text(vector, "name")
        );
        let mut graph = CaseGraph::default();
        graph.model_node(vector);
        let node: CheckedSemanticNodeV2 =
            serde_json::from_value(graph.nodes[0].clone()).expect("node");
        let selected = owner["version"] == vectors["domain_package"]["version"];
        assert_eq!(
            owners.recover(&node).is_ok(),
            selected,
            "{} recovers exactly when its owner is selected",
            text(vector, "name")
        );
    }
    println!(
        "conformance: {} model declaration node vectors",
        nodes.len()
    );
}

/// TC-280 (FR-322-AC-29): FR-322 step 2 over each published wire node.
#[test]
fn tc_280_wire_nodes_recover_their_owner_or_refuse() {
    let Some((vectors, _)) = vectors() else {
        return;
    };
    let model = read_projection(&vectors["domain_package"]);
    let owners = owners_of(&model);
    let cases = array(&vectors, "wire_cases");
    for case in cases {
        let mut node = case["node"].clone();
        node["schema_version"] = json!("quire.checked-semantic-graph/v2");
        node["dependencies"] = json!([]);
        node["occurrences"] = json!([]);
        let node: CheckedSemanticNodeV2 = serde_json::from_value(node).expect("a wire node");
        let decided = match owners.recover(&node) {
            Ok(owner) => json!({"recovers": owner.node}),
            Err(refusal) => json!({"refused": {
                "code": code_wire(refusal.code),
                "cause": cause_wire(refusal.cause),
            }}),
        };
        assert_eq!(decided, case["expected"], "{}", text(case, "name"));
    }
    println!("conformance: {} model wire cases", cases.len());
}

/// TC-280 (FR-322-AC-29..31): each member case as the application a producer
/// emits, a `record.project` over a `deref` for a field and a
/// `model.dispatch_call` for an operation, decided by the reader; an
/// admitted case's resolved member and derived type are its expectation.
#[test]
fn tc_280_member_cases_resolve_and_type_through_the_reader() {
    let Some((vectors, catalog)) = vectors() else {
        return;
    };
    let model = read_projection(&vectors["domain_package"]);
    let owners = owners_of(&model);
    let cases = array(&vectors, "member_cases");
    for case in cases {
        let name = text(case, "name");
        let member = &case["member"];
        let declaring = node_vector(&vectors, text(member, "declaration"));
        let mut graph = CaseGraph::default();
        for vector in array(&vectors, "model_declaration_nodes") {
            graph.model_node(vector);
        }
        let declaring = text(declaring, "sha256");
        let expected = &case["expected"];
        let result_type = expected.get("member_type").map_or_else(
            || MemberType::Integer.node_key(),
            |ty| member_type(ty).node_key(),
        );
        let wire_member = json!({
            "kind": text(member, "kind"),
            "declaration": node_ref(declaring),
            "name": text(member, "name"),
        });
        let application = if text(member, "kind") == "field" {
            let reference_type = graph.reference_type(declaring);
            let value = graph.value_of(&reference_type);
            let deref = graph.application(
                "deref",
                "deref",
                "quire.op.model.deref",
                Value::Null,
                &[value],
                declaring,
            );
            graph.application(
                "query",
                "query",
                "quire.op.record.project",
                wire_member,
                &[deref],
                &result_type,
            )
        } else {
            let receiver = graph.reference_type(text(&member["receiver"], "target"));
            let mut arguments = vec![graph.value_of(&receiver)];
            for argument in array(member, "arguments") {
                let type_digest = member_type(argument).node_key();
                arguments.push(graph.value_of(&type_digest));
            }
            graph.application(
                "call",
                "call",
                "quire.op.model.dispatch_call",
                wire_member,
                &arguments,
                &result_type,
            )
        };
        let decided = graph.decide(&application, &owners, &catalog);
        match expected.get("resolves") {
            Some(resolves) => {
                assert_eq!(decided, None, "{name} is admitted");
                let kind = if text(member, "kind") == "field" {
                    MemberKind::Field
                } else {
                    MemberKind::Operation
                };
                let owner_node = text(
                    &node_vector(&vectors, text(member, "declaration"))["preimage"]["owner"],
                    "node",
                );
                let mut meter = WorkMeter::new(u64::MAX);
                let resolved = model
                    .resolve(
                        owner_node,
                        kind,
                        text(member, "name"),
                        &mut Budget::new(&mut meter, 0),
                    )
                    .expect("the member resolves");
                assert_eq!(
                    resolved.identity(),
                    resolves.as_str().expect("identity"),
                    "{name}"
                );
                assert_eq!(
                    member_name(resolved.identity()),
                    text(member, "name"),
                    "{name}"
                );
            }
            None => assert_eq!(decided.as_ref(), Some(expected), "{name}"),
        }
    }
    println!("conformance: {} model member cases", cases.len());
}

/// TC-280 (FR-322-AC-32): FR-322 step 1's admission checks over the
/// supplied documents of each selection case.
#[test]
fn tc_280_selection_cases_admit_the_selected_document() {
    let Some((vectors, _)) = vectors() else {
        return;
    };
    let package = &vectors["domain_package"];
    let document_digest = digest_json(package).expect("digest");
    let resolve = |value: &str| {
        if value == "$document" {
            document_digest.clone()
        } else {
            value.to_owned()
        }
    };
    let cases = array(&vectors, "selection_cases");
    for case in cases {
        let selection = &case["selection"];
        let row = CheckedDomainPackageRef {
            identity: text(selection, "identity").into(),
            version: text(selection, "version").into(),
            digest_domain: text(selection, "digest_domain").into(),
            digest: resolve(text(selection, "digest")).into(),
        };
        let supplied: BTreeMap<String, Vec<u8>> = array(case, "supplied")
            .iter()
            .map(|entry| {
                let mut document = package.clone();
                for patch in array(entry, "patch") {
                    *document
                        .pointer_mut(text(patch, "path"))
                        .expect("a patch names a member") = patch["value"].clone();
                }
                (
                    resolve(text(entry, "under")),
                    serde_json::to_vec(&document).expect("bytes"),
                )
            })
            .collect();
        let mut meter = WorkMeter::new(u64::MAX);
        let decided = match admit_document(
            &row,
            supplied.get(row.digest.as_ref()).map(Vec::as_slice),
            projection_identity,
            &mut Budget::new(&mut meter, 0),
        ) {
            Ok(_) => json!({"admission": "passed"}),
            Err(SelectionFailure::Refused(refused)) => json!({"refused": {
                "code": code_wire(refused.refusal.code),
                "cause": cause_wire(refused.refusal.cause),
            }}),
            Err(SelectionFailure::Limit(_)) => panic!("no limit is reached"),
        };
        assert_eq!(decided, case["expected"], "{}", text(case, "name"));
    }
    println!("conformance: {} model selection cases", cases.len());
}

/// A TC-281 operand: `{reference: M}` is a value of `Reference<M>`, and
/// `{object: M}` a value of `M` itself.
fn operand(graph: &mut CaseGraph, vectors: &Value, operand: &Value) -> String {
    match (operand["reference"].as_str(), operand["object"].as_str()) {
        (Some(name), None) => {
            let target = text(node_vector(vectors, name), "sha256");
            let reference_type = graph.reference_type(target);
            graph.value_of(&reference_type)
        }
        (None, Some(name)) => {
            let target = text(node_vector(vectors, name), "sha256").to_owned();
            graph.value_of(&target)
        }
        _ => panic!("an operand names a reference or an object"),
    }
}

fn case_graph(vectors: &Value) -> CaseGraph {
    let mut graph = CaseGraph::default();
    for vector in array(vectors, "model_declaration_nodes") {
        graph.model_node(vector);
    }
    graph.add(
        &MemberType::Boolean.node_key(),
        "scalar_type",
        "boolean",
        &MemberType::Boolean.node_key(),
        json!({"term": "aggregate", "members": []}),
    );
    graph
}

/// TC-281 (FR-322-AC-33): `reference.eq`/`ne` over conforming references,
/// decided by the reader against QSpec's catalog.
#[test]
fn tc_281_reference_equality_admits_conforming_object_types() {
    let Some((vectors, catalog)) = vectors() else {
        return;
    };
    let model = read_projection(&vectors["domain_package"]);
    let owners = owners_of(&model);
    let cases = array(&vectors, "reference_equality_cases");
    for case in cases {
        let mut graph = case_graph(&vectors);
        let arguments: Vec<String> = array(case, "operands")
            .iter()
            .map(|value| operand(&mut graph, &vectors, value))
            .collect();
        let application = graph.application(
            "binary",
            "binary",
            text(case, "operation"),
            Value::Null,
            &arguments,
            &MemberType::Boolean.node_key(),
        );
        let decided = graph
            .decide(&application, &owners, &catalog)
            .unwrap_or_else(|| json!({"admitted": true}));
        assert_eq!(decided, case["expected"], "{}", text(case, "name"));
    }
    println!(
        "conformance: {} model reference equality cases",
        cases.len()
    );
}

/// TC-281 (FR-322-AC-34): `record.project` of a model field over a
/// `model.deref` result, the `deref` decided first and then the projection.
#[test]
fn tc_281_record_projection_reads_a_model_field_over_a_deref_result() {
    let Some((vectors, catalog)) = vectors() else {
        return;
    };
    let model = read_projection(&vectors["domain_package"]);
    let owners = owners_of(&model);
    let cases = array(&vectors, "projection_cases");
    for case in cases {
        let name = text(case, "name");
        let mut graph = case_graph(&vectors);
        let source = &case["operand"];
        let (operand_node, deref) = match source.get("deref") {
            Some(inner) => {
                let value = operand(&mut graph, &vectors, inner);
                let declared = text(node_vector(&vectors, text(source, "result_type")), "sha256");
                let deref = graph.application(
                    "deref",
                    "deref",
                    "quire.op.model.deref",
                    Value::Null,
                    &[value],
                    declared,
                );
                (deref.clone(), Some(deref))
            }
            None => (operand(&mut graph, &vectors, source), None),
        };
        let member = &case["member"];
        let projection = graph.application(
            "query",
            "query",
            "quire.op.record.project",
            json!({
                "kind": "field",
                "declaration": node_ref(text(node_vector(&vectors, text(member, "declaration")), "sha256")),
                "name": text(member, "name"),
            }),
            &[operand_node],
            &member_type(&case["result_type"]).node_key(),
        );
        let decided = deref
            .and_then(|deref| graph.decide(&deref, &owners, &catalog))
            .or_else(|| graph.decide(&projection, &owners, &catalog));
        let expected = &case["expected"];
        match expected.get("resolves") {
            Some(_) => assert_eq!(decided, None, "{name} is admitted"),
            None => assert_eq!(decided.as_ref(), Some(expected), "{name}"),
        }
    }
    println!("conformance: {} model projection cases", cases.len());
}
