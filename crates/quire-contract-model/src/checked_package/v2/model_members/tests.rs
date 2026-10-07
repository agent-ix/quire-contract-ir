use super::super::model_fields::build_field_tables;
use super::*;
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use crate::checked_package::v2::DOMAIN_PACKAGE_DIGEST;
use ix_trace_rs::trace;

/// Compare every QSpec model-declaration golden with the exact key function
/// used when the production reader admits selected domain declarations.
/// Ordinary workspace tests have no QSpec checkout; the explicit conformance
/// target supplies one and runs this test alongside the public reader test.
///
/// Trace: TC-048, FR-038-AC-176
#[trace("TC-048", "FR-038-AC-176")]
#[test]
fn tc_048_qspec_model_declaration_keys_use_production_derivation() {
    let Ok(root) = std::env::var("QUIRE_SPECIFICATION_DIR") else {
        return;
    };
    let path = std::path::Path::new(&root)
        .join("proposals/checked-package-v2/model-member-type-vectors.json");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("{} cannot be read: {error}", path.display()));
    let vectors: serde_json::Value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("{} is not JSON: {error}", path.display()));
    let rows = vectors["model_declaration_nodes"]
        .as_array()
        .expect("QSpec model_declaration_nodes must be an array");
    assert_eq!(
        rows.len(),
        12,
        "QSpec must publish all twelve declaration vectors"
    );
    for row in rows {
        let name = row["name"].as_str().expect("vector name");
        let preimage = &row["preimage"];
        assert_eq!(preimage["version"], "quire.structural-node/v1", "{name}");
        assert_eq!(preimage["owner"]["kind"], "model", "{name}");
        assert_eq!(preimage["body"]["term"], "aggregate", "{name}");
        assert_eq!(preimage["body"]["members"], serde_json::json!([]), "{name}");
        assert!(preimage["declaration"].is_null(), "{name}");
        assert!(preimage["recursion"].is_null(), "{name}");
        assert!(preimage["semantic_type"].is_null(), "{name}");
        let form = match (
            preimage["node_tag"].as_str(),
            preimage["semantic_form"].as_str(),
        ) {
            (Some("model"), Some("object_type")) => DeclarationForm::ObjectType,
            (Some("model"), Some("systems_interface")) => DeclarationForm::SystemsInterface,
            (Some("relation"), Some("relationship")) => DeclarationForm::Relationship,
            other => panic!("{name}: unknown declaration form {other:?}"),
        };
        let identity = preimage["owner"]["identity"]
            .as_str()
            .expect("model identity");
        let node = preimage["owner"]["node"].as_str().expect("model node");
        let expected = row["sha256"].as_str().expect("published SHA-256");
        assert_eq!(
            declaration_key(identity, form, node, BYTES).expect("production declaration key"),
            expected,
            "{name}: production key must equal QSpec's recorded digest"
        );
    }
}

/// The byte limit these tests read under: far above any document here.
pub(in crate::checked_package::v2) const BYTES: u64 = 1 << 20;

/// The lowercase SHA-256 of `text`, the digest of expected canonical bytes
/// written out in a test.
fn sha256_of(text: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

/// A structural node key is the SHA-256 of the canonical bytes of its
/// preimage, written out here from the preimage's JSON text with members in
/// RFC 8785 order, not produced by the code under test.
///
/// Trace: FR-038-AC-89
#[trace("TC-048", "FR-038-AC-89")]
#[test]
fn tc_048_structural_node_keys_hash_the_expected_canonical_bytes() {
    let anonymous = concat!(
        r#"{"body":{"members":[],"term":"aggregate"},"declaration":null,"#,
        r#""node_tag":"scalar_type","recursion":null,"semantic_form":"boolean","#,
        r#""semantic_type":null,"version":"quire.structural-node/v1"}"#,
    );
    assert_eq!(
        MemberType::Boolean.node_key(BYTES).as_deref().ok(),
        Some(sha256_of(anonymous).as_str())
    );
    let declaration = concat!(
        r#"{"body":{"members":[],"term":"aggregate"},"declaration":null,"#,
        r#""node_tag":"model","owner":{"identity":"acme/orders","kind":"model","#,
        r#""node":"ix://acme/orders/Order"},"recursion":null,"#,
        r#""semantic_form":"object_type","semantic_type":null,"#,
        r#""version":"quire.structural-node/v1"}"#,
    );
    assert_eq!(
        declaration_key(
            "acme/orders",
            DeclarationForm::ObjectType,
            "ix://acme/orders/Order",
            BYTES
        )
        .as_deref()
        .ok(),
        Some(sha256_of(declaration).as_str())
    );
    // The byte ceiling is the encoder's: one byte under the preimage refuses.
    let length = u64::try_from(anonymous.len()).expect("length");
    assert!(MemberType::Boolean.node_key(length).is_ok());
    assert!(MemberType::Boolean.node_key(length - 1).is_err());
}

/// A declaration key the encoder refuses (its preimage is over the byte limit)
/// refuses the read as `invalid_semantic_graph` at the declaring selection's
/// row, and is not skipped as though the declaration had no owner. This backs
/// the refusal-location rule of FR-038 "Every identity digest is computed
/// through quire-canonical", which no numbered criterion states, so it carries
/// no criterion tag.
#[test]
fn tc_048_a_declaration_key_over_the_byte_limit_refuses_at_its_selection_row() {
    let mut model = read(&document(vec![object_type(WIDGET, &[], vec![])])).expect("reads");
    let mut charged = |_: usize| Ok(());
    assert!(ModelOwners::new(std::slice::from_ref(&model), &mut charged).is_ok());
    model.bytes = 10;
    match ModelOwners::new(std::slice::from_ref(&model), &mut charged) {
        Err(ValidationFailure::Refused(refusal)) => {
            assert_eq!(refusal.code, Code::InvalidSemanticGraph);
            assert_eq!(
                refusal.path.map(|path| path.to_string()).as_deref(),
                Some("/lock/model_selections/0")
            );
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// [`read_semantic_ir`] under a work limit nothing reaches, with the field
/// tables the lock stage builds.
pub(in crate::checked_package::v2) fn read(document: &Value) -> Result<DomainModel, ModelRefusal> {
    let mut meter = WorkMeter::new(u64::MAX);
    match read_semantic_ir(document, &mut Budget::new(&mut meter, 0, BYTES)) {
        Ok(model) => {
            let mut models = [model];
            build_field_tables(&mut models, &mut meter).expect("no limit is reached");
            let [model] = models;
            Ok(model)
        }
        Err(ModelFailure::Refused(refusal)) => Err(refusal),
        Err(ModelFailure::Limit(_)) => panic!("no limit is reached"),
    }
}

/// [`DomainModel::resolve`] under a work limit nothing reaches.
fn resolve<'m>(
    model: &'m DomainModel,
    node: &str,
    kind: MemberKind,
    name: &str,
) -> Result<Resolved<'m>, ModelRefusal> {
    let mut meter = WorkMeter::new(u64::MAX);
    match model.resolve(node, kind, name, &mut Budget::new(&mut meter, 0, BYTES)) {
        Ok(resolved) => Ok(resolved),
        Err(ModelFailure::Refused(refusal)) => Err(refusal),
        Err(ModelFailure::Limit(_)) => panic!("no limit is reached"),
    }
}

const DIGIT: IntegerBounds = IntegerBounds { lower: 0, upper: 9 };

pub(in crate::checked_package::v2) fn multiplicity(lower: u64, upper: Option<u64>) -> Value {
    let mut value = json!({"lower": lower, "ordered": false, "unique": true});
    if let Some(upper) = upper {
        value["upper"] = json!(upper);
    }
    value
}

pub(in crate::checked_package::v2) fn field(owner: &str, name: &str, type_ref: &str) -> Value {
    json!({
        "identity": format!("{owner}/{name}"), "name": name, "typeRef": type_ref,
        "presence": "required", "nullable": false, "defaultKind": "none",
        "multiplicity": multiplicity(1, Some(1)),
    })
}

pub(in crate::checked_package::v2) fn object_type(
    node: &str,
    supertypes: &[&str],
    fields: Vec<Value>,
) -> Value {
    json!({
        "identity": node, "displayName": node, "kind": {"module": "acme/orders", "name": "entity"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes, "fields": fields, "operations": [],
    })
}

/// A Semantic IR 2.0.0 document of `acme/orders` `1.0.0` declaring `types`.
pub(in crate::checked_package::v2) fn document(types: Vec<Value>) -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": "acme/orders", "version": "1.0.0"},
        "constructs": [
            {"kind": {"module": "acme/orders", "name": "entity"},
             "construct": {"meaning": "quire.meaning.model.object-type/v1"}},
            {"kind": {"module": "acme/orders", "name": "digit"},
             "construct": {"meaning": "quire.meaning.model.value-type/v1"}},
        ],
        "types": types,
    })
}

const WIDGET: &str = "ix://acme/orders/Widget";
const GADGET: &str = "ix://acme/orders/Gadget";
const LINK: &str = "ix://acme/orders/relationship/Widget-links-Gadget";

pub(in crate::checked_package::v2) fn relationship() -> Value {
    json!({
        "identity": LINK,
        "category": "structural",
        "composite": false,
        "direction": "bidirectional",
        "sourceEnd": {"role": "links", "type": WIDGET, "multiplicity": multiplicity(0, Some(1))},
        "targetEnd": {"role": "linkedBy", "type": GADGET, "multiplicity": multiplicity(1, Some(1))},
        "origin": {"source": {"sourceIdentity": WIDGET, "path": "models/Widget.md", "startLine": 1, "startColumn": 1}},
    })
}

pub(in crate::checked_package::v2) fn relationship_document(relationships: Vec<Value>) -> Value {
    let mut widget = object_type(WIDGET, &[], vec![]);
    widget["relationships"] = json!(relationships);
    document(vec![widget, object_type(GADGET, &[], vec![])])
}

/// Trace: FR-038-AC-165, FR-038-AC-166, FR-038-AC-173
#[trace("TC-048", "FR-038-AC-165", "FR-038-AC-166", "FR-038-AC-173")]
#[test]
fn tc_048_relationship_roles_and_metadata_are_read_from_the_authored_declaration() {
    let mut declaration = relationship();
    let model = read(&relationship_document(vec![declaration.clone()])).expect("admitted");
    let selected = model
        .relationships
        .get(LINK)
        .expect("selected relationship");
    assert_eq!(selected.source.role.as_deref(), Some("links"));
    assert_eq!(selected.target.role.as_deref(), Some("linkedBy"));
    assert_eq!(selected.target.type_ref.as_ref(), GADGET);
    declaration["targetEnd"]
        .as_object_mut()
        .expect("end")
        .remove("role");
    let model = read(&relationship_document(vec![declaration.clone()])).expect("no inverse role");
    assert_eq!(
        model
            .relationships
            .get(LINK)
            .expect("relationship")
            .target
            .role,
        None
    );
    declaration["origin"] = json!({"generated": {
        "generatorIdentity": WIDGET, "generatorVersion": "1.0.0", "inputIdentities": [GADGET]
    }});
    assert!(read(&relationship_document(vec![declaration])).is_ok());
}

/// Trace: FR-038-AC-165
#[trace("TC-048", "FR-038-AC-165")]
#[test]
fn tc_048_relationship_slot_does_not_change_nested_field_operation_or_parameter_ids() {
    let mut document = relationship_document(vec![relationship()]);
    document["types"][0]["fields"] = json!([field(WIDGET, "count", "ix://quire/native/Integer")]);
    document["types"][0]["operations"] = json!([{
        "identity": format!("{WIDGET}/inspect"),
        "params": [{"identity": format!("{WIDGET}/inspect/item"),
            "typeRef": "ix://quire/native/Integer",
            "multiplicity": multiplicity(1, Some(1))}],
    }]);
    assert!(read(&document).is_ok());
    let malformed = ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration);
    for path in [
        "/types/0/fields/0/identity",
        "/types/0/operations/0/identity",
        "/types/0/operations/0/params/0/identity",
    ] {
        let mut moved = document.clone();
        let target = moved.pointer_mut(path).expect("member identity");
        *target = json!("ix://acme/orders/relationship/Widget-count-Gadget");
        assert_eq!(read(&moved).map(|_| ()), Err(malformed), "{path}");
    }
}

/// Trace: FR-038-AC-165, FR-038-AC-166, FR-038-AC-167, FR-038-AC-168, FR-038-AC-173
#[trace(
    "TC-048",
    "FR-038-AC-165",
    "FR-038-AC-166",
    "FR-038-AC-167",
    "FR-038-AC-168",
    "FR-038-AC-173"
)]
#[test]
fn tc_048_relationship_admission_refuses_bad_identity_shape_and_end_meaning() {
    let malformed = ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration);
    let missing = ModelRefusal::new(Code::MissingDeclaration, Cause::MissingName);
    let reversed = ModelRefusal::new(Code::InvalidModelBinding, Cause::UnpreservedModelMeaning);
    let mut cases = Vec::new();
    for identity in [
        "ix://acme/orders/Widget/links",
        "ix://acme/billing/relationship/Widget-links-Gadget",
        "ix://acme/orders/clause/Widget-links-Gadget",
    ] {
        let mut relation = relationship();
        relation["identity"] = json!(identity);
        cases.push((relation, malformed));
    }
    for member in ["direction", "category", "composite", "origin", "sourceEnd"] {
        let mut relation = relationship();
        relation
            .as_object_mut()
            .expect("relationship")
            .remove(member);
        cases.push((relation, malformed));
    }
    for member in ["direction", "category", "composite", "origin"] {
        for replacement in [Value::Null, json!(42)] {
            let mut relation = relationship();
            relation[member] = replacement;
            cases.push((relation, malformed));
        }
    }
    for (member, replacement) in [
        ("direction", json!("diagonal")),
        ("category", json!("unknown")),
        ("origin", json!({})),
        ("origin", json!({"source": {}, "generated": {}})),
        (
            "origin",
            json!({"source": {"sourceIdentity": WIDGET, "path": "x"}}),
        ),
        (
            "origin",
            json!({"generated": {"generatorIdentity": WIDGET,
            "generatorVersion": "bad", "inputIdentities": [GADGET]}}),
        ),
    ] {
        let mut relation = relationship();
        relation[member] = replacement;
        cases.push((relation, malformed));
    }
    for end in ["sourceEnd", "targetEnd"] {
        for role in [Value::Null, json!(""), json!(7)] {
            let mut relation = relationship();
            relation[end]["role"] = role;
            cases.push((relation, malformed));
        }
    }
    let mut relation = relationship();
    relation["targetEnd"]["type"] = json!("ix://acme/orders/Ghost");
    cases.push((relation, missing));
    let mut relation = relationship();
    relation["targetEnd"]["type"] = json!(LINK);
    cases.push((relation, malformed));
    let mut relation = relationship();
    relation["sourceEnd"]["type"] = json!(GADGET);
    cases.push((relation, malformed));
    let mut relation = relationship();
    relation["targetEnd"]["multiplicity"]["lower"] = json!(2);
    cases.push((relation, reversed));
    for (relation, expected) in cases {
        assert_eq!(
            read(&relationship_document(vec![relation.clone()])).map(|_| ()),
            Err(expected),
            "{relation}"
        );
    }
    let mut mixed = relationship();
    mixed["targetEnd"]["type"] = json!("ix://acme/orders/Ghost");
    mixed["targetEnd"]["multiplicity"]["lower"] = json!(2);
    assert_eq!(
        read(&relationship_document(vec![mixed])).map(|_| ()),
        Err(missing)
    );
    let mut mixed = relationship();
    mixed["targetEnd"]["role"] = json!("");
    mixed["targetEnd"]["type"] = json!("ix://acme/orders/Ghost");
    assert_eq!(
        read(&relationship_document(vec![mixed])).map(|_| ()),
        Err(malformed)
    );
    let conflicting = ModelRefusal::new(Code::InvalidModelBinding, Cause::ConflictingBinding);
    assert_eq!(
        read(&relationship_document(vec![relationship(), relationship()])).map(|_| ()),
        Err(conflicting)
    );
}

/// Trace: FR-038-AC-168
#[trace("TC-048", "FR-038-AC-168")]
#[test]
fn tc_048_relationship_duplicates_follow_declaration_and_failure_row_order() {
    let malformed = ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration);
    let missing = ModelRefusal::new(Code::MissingDeclaration, Cause::MissingName);
    let conflicting = ModelRefusal::new(Code::InvalidModelBinding, Cause::ConflictingBinding);
    let mut first = relationship();
    first["sourceEnd"]["type"] = json!(GADGET);
    let second = relationship();
    let with_owners = |first: Value, second: Value| {
        let mut gadget = object_type(GADGET, &[], vec![]);
        gadget["relationships"] = json!([first]);
        let mut widget = object_type(WIDGET, &[], vec![]);
        widget["relationships"] = json!([second]);
        document(vec![widget, gadget])
    };
    let mut bad_role = first.clone();
    bad_role["sourceEnd"]["role"] = json!("");
    assert_eq!(
        read(&with_owners(bad_role, second.clone())).map(|_| ()),
        Err(malformed)
    );
    let mut missing_end = first.clone();
    missing_end["targetEnd"]["type"] = json!("ix://acme/orders/Ghost");
    assert_eq!(
        read(&with_owners(missing_end, second.clone())).map(|_| ()),
        Err(missing)
    );
    assert_eq!(
        read(&with_owners(first.clone(), second.clone())).map(|_| ()),
        Err(conflicting)
    );
    let mut reversed_later = second.clone();
    reversed_later["targetEnd"]["multiplicity"]["lower"] = json!(2);
    assert_eq!(
        read(&with_owners(first, reversed_later)).map(|_| ()),
        Err(conflicting)
    );
    let mut malformed_first = relationship();
    malformed_first["sourceEnd"]["role"] = json!("");
    assert_eq!(
        read(&relationship_document(vec![
            malformed_first,
            relationship()
        ]))
        .map(|_| ()),
        Err(malformed),
    );
}

/// Trace: FR-038-AC-170
#[trace("TC-048", "FR-038-AC-170")]
#[test]
fn tc_048_relationship_destination_bounds_derive_the_canonical_navigation_type() {
    let mut relation = relationship();
    let cases = [
        (0, Some(1), true, "option"),
        (1, Some(1), true, "reference"),
        (0, Some(3), true, "set"),
        (0, Some(3), false, "bag"),
    ];
    for (lower, upper, unique, expected) in cases {
        relation["targetEnd"]["multiplicity"] = json!({
            "lower": lower, "upper": upper, "ordered": false, "unique": unique,
        });
        let model = read(&relationship_document(vec![relation.clone()])).expect("admitted");
        let declaration = model.relationships.get(LINK).expect("relationship");
        let type_key = declaration_key("acme/orders", DeclarationForm::ObjectType, GADGET, BYTES)
            .expect("destination key");
        let result = declaration
            .navigation_type(true, type_key.into())
            .expect("navigation type");
        match (expected, result) {
            ("option", MemberType::Option(_))
            | ("reference", MemberType::Reference(_))
            | (
                "set",
                MemberType::Collection {
                    kind: CollectionKind::Set,
                    ..
                },
            )
            | (
                "bag",
                MemberType::Collection {
                    kind: CollectionKind::Bag,
                    ..
                },
            ) => {}
            (_, actual) => panic!("expected {expected}, got {actual:?}"),
        }
    }
    relation["targetEnd"]["multiplicity"] = json!({
        "lower": 0, "ordered": true, "unique": true,
    });
    let model = read(&relationship_document(vec![relation])).expect("admitted");
    let declaration = model.relationships.get(LINK).expect("relationship");
    assert_eq!(
        declaration.navigation_type(true, "destination-key".into()),
        Err(ModelRefusal::new(
            Code::UnsupportedConstruct,
            Cause::ExpressionForm
        ))
    );
}

/// Trace: FR-038-AC-168
#[trace("TC-048", "FR-038-AC-168")]
#[test]
fn tc_048_relationship_document_read_uses_the_exact_selected_row_work_budget() {
    let document = relationship_document(vec![relationship()]);
    let read_at = |work| {
        let mut meter = WorkMeter::new(work);
        read_semantic_ir(&document, &mut Budget::new(&mut meter, 0, BYTES))
    };
    let mut below = 0;
    let mut enough = 100;
    assert!(read_at(enough).is_ok());
    while below + 1 < enough {
        let middle = below + (enough - below) / 2;
        if read_at(middle).is_ok() {
            enough = middle;
        } else {
            below = middle;
        }
    }
    assert!(read_at(enough).is_ok());
    match read_at(enough - 1) {
        Err(ModelFailure::Limit(ValidationFailure::Incomplete(incomplete))) => {
            assert_eq!(
                incomplete.limit_kind,
                crate::checked_package::shared::CheckedPackageLimit::Work
            );
            assert_eq!(
                incomplete.path.as_ref().map(ToString::to_string).as_deref(),
                Some("/lock/model_selections/0")
            );
        }
        other => panic!("expected selected-row work exhaustion, got {other:?}"),
    }
}

/// FR-154 over a Semantic IR document: an inherited field resolves on the
/// subtype and a subtype conforms to its supertype, decided in either order.
///
/// Trace: FR-038-AC-29
#[test]
fn tc_048_a_semantic_ir_document_reads_inherited_members_and_conformance() {
    let model = read(&document(vec![
        object_type(
            WIDGET,
            &[],
            vec![field(WIDGET, "code", "ix://quire/native/Integer")],
        ),
        object_type(GADGET, &[WIDGET], vec![]),
    ]))
    .expect("the document reads");
    let Resolved::Field(code) =
        resolve(&model, GADGET, MemberKind::Field, "code").expect("code resolves on the subtype")
    else {
        panic!("a field");
    };
    assert_eq!(code.identity.as_ref(), "ix://acme/orders/Widget/code");
    assert_eq!(model.field_type(code), Some(MemberType::Integer));
    let conforms = |a, b| {
        let mut meter = WorkMeter::new(u64::MAX);
        model
            .conforms(a, b, &mut Budget::new(&mut meter, 0, BYTES))
            .expect("no limit is reached")
    };
    assert!(conforms(GADGET, WIDGET));
    assert!(
        conforms(WIDGET, GADGET),
        "conformance is decided in either order"
    );
    assert_eq!(
        resolve(&model, GADGET, MemberKind::Operation, "code").map(|_| ()),
        Err(ModelRefusal::ineligible())
    );
}

/// An FCD value type `scalar: integer` with `min` and `max` constraints is
/// the element type `Int[lo, hi]`.
///
/// Trace: FR-038-AC-29
#[test]
fn tc_048_a_semantic_ir_integer_value_type_is_an_integer_range() {
    let digit = json!({
        "identity": "ix://acme/orders/Digit", "displayName": "Digit",
        "kind": {"module": "acme/orders", "name": "digit"}, "roles": [], "extensions": [],
        "unknownPolicy": "reject", "scalar": "integer",
        "constraints": [
            {"keyword": "min", "operands": {"value": 0}},
            {"keyword": "max", "operands": {"value": "9"}},
        ],
    });
    let model = read(&document(vec![
        digit,
        object_type(
            WIDGET,
            &[],
            vec![field(WIDGET, "digit", "ix://acme/orders/Digit")],
        ),
    ]))
    .expect("the document reads");
    let Resolved::Field(digit) =
        resolve(&model, WIDGET, MemberKind::Field, "digit").expect("digit")
    else {
        panic!("a field");
    };
    assert_eq!(model.field_type(digit), Some(MemberType::IntRange(DIGIT)));
}

/// FR-154's declaration refusals the reader draws, each for one defect.
///
/// Trace: FR-038-AC-28
#[test]
fn tc_048_semantic_ir_declaration_defects_refuse_with_their_fr_154_cause() {
    let refused = |types| read(&document(types)).map(|_| ()).expect_err("refused");
    let widget = || object_type(WIDGET, &[], vec![]);
    assert_eq!(
        refused(vec![object_type(
            GADGET,
            &["ix://acme/orders/Nope"],
            vec![]
        )]),
        ModelRefusal::new(Code::MissingDeclaration, Cause::MissingName)
    );
    assert_eq!(
        refused(vec![object_type(
            WIDGET,
            &[],
            vec![field(WIDGET, "x", "ix://acme/orders/Nope")]
        )]),
        ModelRefusal::new(Code::MissingDeclaration, Cause::MissingName)
    );
    assert_eq!(
        refused(vec![object_type(
            WIDGET,
            &[],
            vec![field(WIDGET, "x", "ix://quire/native/Uuid")]
        )]),
        ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration)
    );
    assert_eq!(
        refused(vec![widget(), widget()]),
        ModelRefusal::new(Code::InvalidModelBinding, Cause::ConflictingBinding)
    );
    assert_eq!(
        refused(vec![object_type("ix://acme/orders/sys-pump", &[], vec![])]),
        ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration)
    );
    let mut backwards = field(WIDGET, "x", "ix://quire/native/Integer");
    backwards["multiplicity"] = multiplicity(3, Some(1));
    assert_eq!(
        refused(vec![object_type(WIDGET, &[], vec![backwards])]),
        ModelRefusal::new(Code::InvalidModelBinding, Cause::UnpreservedModelMeaning)
    );
    let mut unknown_kind = widget();
    unknown_kind["kind"] = json!({"module": "acme/orders", "name": "nope"});
    assert_eq!(
        refused(vec![unknown_kind]),
        ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration)
    );
}

/// FR-322 step 1 over a Semantic IR document: the digest is recomputed from
/// the bytes and the document's own `package` identity compared; no version
/// is read.
///
/// Trace: FR-038-AC-27
#[test]
fn tc_048_a_selection_admits_only_the_document_it_names() {
    let document = document(vec![object_type(WIDGET, &[], vec![])]);
    let bytes = serde_json::to_vec(&document).expect("bytes");
    let digest = quire_canonical::sha256(&document, quire_canonical::Limits::new(BYTES))
        .expect("digest")
        .to_string();
    let selection = |identity: &str, digest: &str| CheckedDomainPackageRef {
        identity: identity.into(),
        digest_domain: DOMAIN_PACKAGE_DIGEST.into(),
        digest: digest.into(),
    };
    let mut evidence = CheckedPackageEvidence::new();
    evidence.insert_domain_package_document(digest.clone(), bytes.clone());
    let admit = |selection: &CheckedDomainPackageRef, evidence: &CheckedPackageEvidence| {
        let mut meter = WorkMeter::new(u64::MAX);
        admit_selection(selection, evidence, &mut Budget::new(&mut meter, 0, BYTES))
    };
    let admitted = admit(&selection("acme/orders", &digest), &evidence).expect("admitted");
    assert!(admitted.object_types.contains_key(WIDGET));
    let refusal =
        |selection: &CheckedDomainPackageRef, evidence: &CheckedPackageEvidence| match admit(
            selection, evidence,
        ) {
            Err(SelectionFailure::Refused(refusal)) => refusal,
            other => panic!("expected a refusal, got {other:?}"),
        };
    assert_eq!(
        refusal(&selection("acme/other", &digest), &evidence),
        SelectionRefusal::at(
            Code::InvalidModelBinding,
            Cause::WrongModelSelection,
            Some("identity")
        )
    );
    assert_eq!(
        refusal(&selection("acme/orders", &"0".repeat(64)), &evidence),
        SelectionRefusal::at(Code::MissingImport, Cause::MissingSelection, Some("digest"))
    );
    let mut forged = CheckedPackageEvidence::new();
    forged.insert_domain_package_document(digest.clone(), b"{}".to_vec());
    assert_eq!(
        refusal(&selection("acme/orders", &digest), &forged),
        SelectionRefusal::at(
            Code::StaleDependency,
            Cause::ByteDigestMismatch,
            Some("digest")
        )
    );
}

const ZETA: &str = "ix://acme/orders/Zeta";
const BAD_ID: &str = "ix://acme/orders/bad-id";

/// QSpec FR-154: a reference to a node refused for its own object id is not
/// reported again as `missing-name`; the refused node's own refusal stands,
/// whether the referencing node is read before or after it.
///
/// Trace: FR-038-AC-28
#[test]
fn tc_048_a_reference_to_a_refused_node_leaves_that_nodes_own_refusal() {
    let malformed = ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration);
    let bad = || object_type(BAD_ID, &[], vec![]);
    // `bad-id` sorts after `Gadget` and `Widget`, so each reference below is
    // read before the node it names.
    for referencing in [
        object_type(GADGET, &[BAD_ID], vec![]),
        object_type(GADGET, &[], vec![field(GADGET, "peer", BAD_ID)]),
        object_type(WIDGET, &[BAD_ID], vec![]),
    ] {
        assert_eq!(
            read(&document(vec![referencing, bad()])).map(|_| ()),
            Err(malformed)
        );
    }
    // A node refused for a duplicated identity stands for its references too.
    assert_eq!(
        read(&document(vec![
            object_type(GADGET, &[WIDGET], vec![]),
            object_type(WIDGET, &[], vec![]),
            object_type(WIDGET, &[], vec![]),
        ]))
        .map(|_| ()),
        Err(ModelRefusal::new(
            Code::InvalidModelBinding,
            Cause::ConflictingBinding
        ))
    );
}

/// QSpec FR-154: within one node the failures report in the declaration-refusal
/// table's order, so a dangling `typeRef` (`missing-name`) reports before a
/// multiplicity with `lower > upper` (`unpreserved-model-meaning`), and a
/// malformed member before either, wherever each sits in the node.
///
/// Trace: FR-038-AC-28
#[test]
fn tc_048_one_nodes_failures_report_in_table_order() {
    let missing = ModelRefusal::new(Code::MissingDeclaration, Cause::MissingName);
    let unpreserved = ModelRefusal::new(Code::InvalidModelBinding, Cause::UnpreservedModelMeaning);
    let malformed = ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration);
    let mut backwards = field(WIDGET, "a", "ix://quire/native/Integer");
    backwards["multiplicity"] = multiplicity(3, Some(1));
    let dangling = field(WIDGET, "b", "ix://acme/orders/Nope");
    for fields in [
        vec![backwards.clone(), dangling.clone()],
        vec![dangling.clone(), backwards.clone()],
    ] {
        assert_eq!(
            read(&document(vec![object_type(WIDGET, &[], fields)])).map(|_| ()),
            Err(missing)
        );
    }
    // The same field failing both rows: the reference reports first.
    let mut both = field(WIDGET, "a", "ix://acme/orders/Nope");
    both["multiplicity"] = multiplicity(3, Some(1));
    assert_eq!(
        read(&document(vec![object_type(WIDGET, &[], vec![both])])).map(|_| ()),
        Err(missing)
    );
    // The multiplicity alone is the last row.
    assert_eq!(
        read(&document(vec![object_type(
            WIDGET,
            &[],
            vec![backwards.clone()]
        )]))
        .map(|_| ()),
        Err(unpreserved)
    );
    // A malformed member reports before a dangling reference.
    let mut no_presence = field(WIDGET, "c", "ix://quire/native/Integer");
    no_presence["presence"] = json!("sometimes");
    assert_eq!(
        read(&document(vec![object_type(
            WIDGET,
            &[],
            vec![dangling, no_presence]
        )]))
        .map(|_| ()),
        Err(malformed)
    );
}

/// FR-154 row 3: a `typeRef` naming a relationship names a node of the wrong
/// meaning, whether the object type declaring the relationship is read before
/// or after the one naming it.
///
/// Trace: FR-038-AC-28
#[test]
fn tc_048_a_type_ref_to_a_relationship_is_wrong_meaning_in_any_node_order() {
    let mut widget = object_type(WIDGET, &[], vec![]);
    widget["relationships"] = json!([relationship()]);
    let malformed = ModelRefusal::new(Code::InvalidModelBinding, Cause::MalformedDeclaration);
    for referencing in [GADGET, ZETA] {
        let holder = object_type(referencing, &[], vec![field(referencing, "owned", LINK)]);
        let mut types = vec![widget.clone(), holder];
        if referencing != GADGET {
            types.push(object_type(GADGET, &[], vec![]));
        }
        assert_eq!(
            read(&document(types)).map(|_| ()),
            Err(malformed),
            "{referencing} is read {} the relationship's owner",
            if referencing < WIDGET {
                "before"
            } else {
                "after"
            }
        );
    }
}

/// QSpec FR-154: two nodes that carry no identity share none; each is
/// malformed, not a `conflicting-binding`.
///
/// Trace: FR-038-AC-28
#[test]
fn tc_048_nodes_with_no_identity_are_malformed_not_conflicting() {
    let mut nameless = object_type(WIDGET, &[], vec![]);
    nameless
        .as_object_mut()
        .expect("an object")
        .remove("identity");
    assert_eq!(
        read(&document(vec![nameless.clone(), nameless])).map(|_| ()),
        Err(ModelRefusal::new(
            Code::InvalidModelBinding,
            Cause::MalformedDeclaration
        ))
    );
}

/// FR-322 step 1: reading a document and resolving a member are charged to
/// the reader's `work` limit at the document's `model_selections` row; a
/// limit one below what a read used is `incomplete` there.
///
/// Trace: FR-038-AC-30
#[test]
fn tc_048_reading_and_resolving_are_charged_to_the_work_limit() {
    let types = || {
        vec![
            object_type(
                WIDGET,
                &[],
                vec![field(WIDGET, "code", "ix://quire/native/Integer")],
            ),
            object_type(GADGET, &[WIDGET], vec![]),
        ]
    };
    let incomplete_at = |failure: ValidationFailure| match failure {
        ValidationFailure::Incomplete(record) => record.path.map(|path| path.to_string()),
        ValidationFailure::Refused(refusal) => panic!("a limit, not {refusal:?}"),
    };
    // The read: exact work admits, one less is incomplete at the row.
    let document = document(types());
    let mut unlimited = WorkMeter::new(u64::MAX);
    read_semantic_ir(&document, &mut Budget::new(&mut unlimited, 2, BYTES)).expect("reads");
    let used = unlimited.consumed();
    assert!(used > 0, "a read is charged");
    let mut exact = WorkMeter::new(used);
    read_semantic_ir(&document, &mut Budget::new(&mut exact, 2, BYTES)).expect("exact work admits");
    let mut tight = WorkMeter::new(used - 1);
    let Err(ModelFailure::Limit(failure)) =
        read_semantic_ir(&document, &mut Budget::new(&mut tight, 2, BYTES))
    else {
        panic!("one unit less than the read used is a limit");
    };
    assert_eq!(
        incomplete_at(failure).as_deref(),
        Some("/lock/model_selections/2")
    );
    // Resolution: the same.
    let model = read(&document).expect("reads");
    let mut meter = WorkMeter::new(u64::MAX);
    model
        .resolve(
            GADGET,
            MemberKind::Field,
            "code",
            &mut Budget::new(&mut meter, 1, BYTES),
        )
        .expect("resolves");
    let used = meter.consumed();
    assert!(used > 0, "a resolution is charged");
    let mut tight = WorkMeter::new(used - 1);
    let Err(ModelFailure::Limit(failure)) = model.resolve(
        GADGET,
        MemberKind::Field,
        "code",
        &mut Budget::new(&mut tight, 1, BYTES),
    ) else {
        panic!("one unit less than the resolution used is a limit");
    };
    assert_eq!(
        incomplete_at(failure).as_deref(),
        Some("/lock/model_selections/1")
    );
}

/// One object type of a differential document: its short name, the short
/// names of its supertypes, and its fields as `(name, redefined member)`.
pub(in crate::checked_package::v2) type Declared<'a> =
    (&'a str, &'a [&'a str], &'a [(&'a str, Option<&'a str>)]);

pub(in crate::checked_package::v2) const ORDERS: &str = "ix://acme/orders/";

/// A document declaring `types`; a redefined member is written `Owner/name`.
pub(in crate::checked_package::v2) fn differential_document(types: &[Declared<'_>]) -> Value {
    document(
        types
            .iter()
            .map(|(short, supertypes, fields)| {
                let node = format!("{ORDERS}{short}");
                let supertypes: Vec<String> = supertypes
                    .iter()
                    .map(|supertype| format!("{ORDERS}{supertype}"))
                    .collect();
                let supertypes: Vec<&str> = supertypes.iter().map(String::as_str).collect();
                let fields = fields
                    .iter()
                    .map(|(name, redefines)| {
                        let mut declared = field(&node, name, "ix://quire/native/Integer");
                        if let Some(target) = redefines {
                            declared["redefines"] = json!(format!("{ORDERS}{target}"));
                        }
                        declared
                    })
                    .collect();
                object_type(&node, &supertypes, fields)
            })
            .collect(),
    )
}

/// For each type of `types`, the field names that resolve on it (and to
/// which member), or are ambiguous, in name order, as `resolve` answers them:
/// `Type: name=Owner/name name=ambiguous`.
fn resolutions(label: &str, types: &[Declared<'_>]) -> Vec<String> {
    let model = read(&differential_document(types)).expect("the document reads");
    let names: BTreeSet<&str> = types
        .iter()
        .flat_map(|(_, _, fields)| fields.iter().map(|(name, _)| *name))
        .collect();
    types
        .iter()
        .map(|(short, _, _)| {
            let node = format!("{ORDERS}{short}");
            let answers: Vec<String> = names
                .iter()
                .filter_map(
                    |name| match resolve(&model, &node, MemberKind::Field, name) {
                        Ok(resolved) => Some(format!(
                            "{name}={}",
                            resolved.identity().trim_start_matches(ORDERS)
                        )),
                        Err(refusal) if refusal == ModelRefusal::ambiguous() => {
                            Some(format!("{name}=ambiguous"))
                        }
                        Err(_) => None,
                    },
                )
                .collect();
            format!("{label}/{short}: {}", answers.join(" "))
        })
        .collect()
}

/// The documents whose resolutions are recorded below, each with cyclic or
/// redefining declarations.
pub(in crate::checked_package::v2) fn differential_documents(
) -> Vec<(&'static str, Vec<Declared<'static>>)> {
    vec![
        ("self_cycle", vec![("A", &["A"], &[("f", None)])]),
        (
            "two_cycle",
            vec![("A", &["B"], &[("a", None)]), ("B", &["A"], &[("b", None)])],
        ),
        (
            "self_cycle_redefinition",
            vec![("A", &["A"], &[("f", None), ("g", Some("A/f"))])],
        ),
        (
            "extends_cycle",
            vec![("A", &["A"], &[("f", None)]), ("C", &["A"], &[("c", None)])],
        ),
        (
            "cycle_over_base",
            vec![
                ("P", &[], &[("x", None)]),
                ("A", &["A", "P"], &[("f", Some("P/x")), ("h", None)]),
                ("C", &["A"], &[("c", None)]),
            ],
        ),
        (
            "two_cycle_shared_target",
            vec![
                ("P", &[], &[("x", None)]),
                ("A", &["B"], &[("a", Some("P/x"))]),
                ("B", &["A", "P"], &[("b", Some("P/x"))]),
                ("C", &["A"], &[("c", None)]),
            ],
        ),
        (
            "diamond",
            vec![
                ("Base", &[], &[("x", None), ("z", None)]),
                ("L", &["Base"], &[("x", Some("Base/x"))]),
                ("R", &["Base"], &[("x", Some("Base/x"))]),
                ("M", &["L"], &[("x", Some("L/x"))]),
                ("Bottom", &["M", "R"], &[("y", None)]),
                ("Both", &["L", "R"], &[]),
            ],
        ),
        (
            "ambiguity",
            vec![
                ("T1", &[], &[("x", None)]),
                ("T2", &[], &[("x", None)]),
                ("T3", &["T1", "T2"], &[("y", None)]),
            ],
        ),
        (
            "redefining_chain",
            vec![
                ("C0", &[], &[("f", None), ("g", None)]),
                ("C1", &["C0"], &[("f", Some("C0/f"))]),
                ("C2", &["C1"], &[("f", Some("C1/f"))]),
                ("C3", &["C2"], &[]),
            ],
        ),
        (
            "branch_dominance",
            vec![
                ("Base", &[], &[("x", None)]),
                ("L", &["Base"], &[("x", Some("Base/x"))]),
                ("M", &["L"], &[("x2", Some("Base/x"))]),
                ("X", &["M", "L"], &[]),
                ("Y", &["L", "M"], &[("y", None)]),
            ],
        ),
    ]
}

/// The values `DomainModel::resolve` returned for each document of
/// [`differential_documents`] before IR-628 changed it, recorded by running
/// that function and pasted here; the field tables of IR-628 are compared
/// with them, not with a reading of the code.
pub(in crate::checked_package::v2) const RECORDED: [&str; 31] = [
    "self_cycle/A: f=A/f",
    "two_cycle/A: a=A/a b=B/b",
    "two_cycle/B: a=A/a b=B/b",
    "self_cycle_redefinition/A: ",
    "extends_cycle/A: f=A/f",
    "extends_cycle/C: c=C/c f=A/f",
    "cycle_over_base/P: x=P/x",
    "cycle_over_base/A: h=A/h",
    "cycle_over_base/C: c=C/c h=A/h",
    "two_cycle_shared_target/P: x=P/x",
    "two_cycle_shared_target/A: ",
    "two_cycle_shared_target/B: ",
    "two_cycle_shared_target/C: c=C/c",
    "diamond/Base: x=Base/x z=Base/z",
    "diamond/L: x=L/x z=Base/z",
    "diamond/R: x=R/x z=Base/z",
    "diamond/M: x=M/x z=Base/z",
    "diamond/Bottom: x=ambiguous y=Bottom/y z=Base/z",
    "diamond/Both: x=ambiguous z=Base/z",
    "ambiguity/T1: x=T1/x",
    "ambiguity/T2: x=T2/x",
    "ambiguity/T3: x=ambiguous y=T3/y",
    "redefining_chain/C0: f=C0/f g=C0/g",
    "redefining_chain/C1: f=C1/f g=C0/g",
    "redefining_chain/C2: f=C2/f g=C0/g",
    "redefining_chain/C3: f=C2/f g=C0/g",
    "branch_dominance/Base: x=Base/x",
    "branch_dominance/L: x=L/x",
    "branch_dominance/M: x2=M/x2",
    "branch_dominance/X: x2=M/x2",
    "branch_dominance/Y: x2=M/x2 y=Y/y",
];

/// `resolve` over the field tables answers as the pre-change `resolve` did,
/// for supertype cycles, redefinitions, a diamond and an ambiguous name.
///
/// Trace: FR-038-AC-144
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_resolve_answers_match_the_values_recorded_before_the_field_tables() {
    let actual: Vec<String> = differential_documents()
        .iter()
        .flat_map(|(label, types)| resolutions(label, types))
        .collect();
    assert_eq!(actual, RECORDED);
}
