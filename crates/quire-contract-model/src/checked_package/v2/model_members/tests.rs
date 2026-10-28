use super::super::model_fields::{build_field_tables, RecordBinderTypes};
use super::*;
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use crate::checked_package::v2::DOMAIN_PACKAGE_DIGEST;
use ix_trace_rs::trace;

/// Preserve the actual native converter's closed outputs and typed refusal.
#[trace("TC-460")]
#[test]
fn selected_model_intake_native_names_preserve_conversion_and_refusal() {
    for (name, expected) in [
        ("Boolean", NativeTypeRef::Boolean),
        ("Integer", NativeTypeRef::Integer),
        ("Rational", NativeTypeRef::Rational),
        ("Decimal", NativeTypeRef::Decimal),
        ("Float32", NativeTypeRef::Float32),
        ("Float64", NativeTypeRef::Float64),
        ("Text", NativeTypeRef::Text),
        ("String", NativeTypeRef::String),
        ("UUID", NativeTypeRef::Uuid),
        ("Timestamp", NativeTypeRef::Timestamp),
    ] {
        assert_eq!(NativeTypeRef::from_name(name), Some(expected), "{name}");
        assert_eq!(
            SelectedTypeRef::from_admitted_name(&format!("{NATIVE_PREFIX}{name}")),
            Some(SelectedTypeRef::Native(expected)),
            "{name}: selected reference retains the typed native variant"
        );
    }
    for unknown in ["", "Unknown", "integer", "Uuid"] {
        assert_eq!(NativeTypeRef::from_name(unknown), None, "{unknown:?}");
        assert_eq!(
            SelectedTypeRef::from_admitted_name(&format!("{NATIVE_PREFIX}{unknown}")),
            None,
            "{unknown:?}: no unknown native fallback"
        );
    }
}

/// Open unit and declaration identities remain owned strings in their variants.
#[trace("TC-460")]
#[test]
fn selected_model_intake_reference_classes_preserve_owned_values_and_empty_refusal() {
    let unit = "ix://quire/unit/metre";
    assert_eq!(
        SelectedTypeRef::from_admitted_name(unit),
        Some(SelectedTypeRef::Unit(unit.into()))
    );
    let declared = "ix://example/orders/Order";
    assert_eq!(
        SelectedTypeRef::from_admitted_name(declared),
        Some(SelectedTypeRef::Declared(declared.into()))
    );
    assert_eq!(SelectedTypeRef::from_admitted_name(""), None);
    let mut external = String::from("ix://example/orders/Mutable");
    let converted = SelectedTypeRef::from_admitted_name(&external);
    external.clear();
    assert_eq!(
        converted,
        Some(SelectedTypeRef::Declared(
            "ix://example/orders/Mutable".into()
        )),
        "conversion owns the caller's selected identity"
    );
}

/// FR-154 retains the exact failing field and member path privately, even
/// though the public lock refusal is necessarily located at the selection row.
#[trace("TC-463", "FR-068-AC-4")]
#[test]
fn record_wrong_kind_type_refs_retain_field_identity_and_type_ref_path() {
    const ADDRESS: &str = "ix://acme/orders/Address";
    const OTHER: &str = "ix://acme/orders/Other";
    const ACCOUNT: &str = "ix://acme/orders/Account";
    for (kind, meaning) in [
        ("event", "quire.meaning.model.event-type/v1"),
        ("process", "quire.meaning.model.process/v1"),
        ("machine", "quire.meaning.model.state-machine/v1"),
        (
            "persistence",
            "quire.meaning.model.persistence-interface/v1",
        ),
        ("namespace", "quire.meaning.model.namespace/v1"),
    ] {
        let mut record = object_type(ADDRESS, &[], vec![field(ADDRESS, "street", OTHER)]);
        record["kind"]["name"] = json!("record");
        let mut other = object_type(OTHER, &[], vec![]);
        other["kind"]["name"] = json!(kind);
        match kind {
            "event" => {
                other["fields"] = json!([field(OTHER, "at", "ix://quire/native/Timestamp")]);
                other["occurrenceField"] = json!(format!("{OTHER}/at"));
            }
            "process" => {
                other["fields"] = json!([field(OTHER, "run", "ix://quire/native/UUID")]);
                other["identityFields"] = json!([format!("{OTHER}/run")]);
                other["steps"] = json!([{"identity":format!("{OTHER}/advance"),
                    "name":"advance", "stepKind":"command", "consumes":[], "emits":[]}]);
            }
            "machine" => {
                other["operations"] = json!([{"identity":format!("{OTHER}/toggle"), "params":[]}]);
                other["states"] = json!([{"identity":format!("{OTHER}/open"), "name":"open"}]);
                other["transitions"] = json!([{"identity":format!("{OTHER}/transition"),
                    "from":format!("{OTHER}/open"), "to":format!("{OTHER}/open"),
                    "trigger":format!("{OTHER}/toggle"), "emits":[]}]);
            }
            "persistence" => {
                other["operations"] = json!([{"identity":format!("{OTHER}/save"), "params":[]}]);
                other["persists"] = json!([ACCOUNT]);
            }
            "namespace" => {
                other.as_object_mut().expect("namespace").remove("fields");
                other
                    .as_object_mut()
                    .expect("namespace")
                    .remove("operations");
                other["members"] = json!([ACCOUNT]);
                other["vocabulary"] = json!([{"term":"Account", "doc":"Account"}]);
            }
            _ => unreachable!(),
        }
        let mut account = object_type(
            ACCOUNT,
            &[],
            vec![field(ACCOUNT, "id", "ix://quire/native/UUID")],
        );
        account["identityFields"] = json!([format!("{ACCOUNT}/id")]);
        let mut selected = document(vec![record, other, account]);
        selected["constructs"]
            .as_array_mut()
            .expect("constructs")
            .extend([
                json!({"kind":{"module":"acme/orders","name":"record"},
                "construct":{"meaning":"quire.meaning.model.record-value-type/v1"}}),
                json!({"kind":{"module":"acme/orders","name":kind},
                "construct":{"meaning":meaning}}),
            ]);
        let mut meter = WorkMeter::new(u64::MAX);
        let Err(SelectionFailure::Refused(refusal)) = read_semantic_ir(
            &selected,
            &mut Budget::new(&mut meter, 0, BYTES),
            &Arc::new(BTreeMap::new()),
        ) else {
            panic!("{kind}: wrong record field type must refuse");
        };
        assert_eq!(refusal.refusal, ModelRefusal::malformed(), "{kind}");
        assert_eq!(
            refusal.declaration_identity.as_deref(),
            Some(format!("{ADDRESS}/street").as_str()),
            "{kind}"
        );
        assert_eq!(
            refusal.declaration_path,
            Some(vec![
                Segment::Name("fields"),
                Segment::Index(0),
                Segment::Name("typeRef")
            ]),
            "{kind}"
        );
    }
}

/// Compare every QSpec model-declaration golden with the exact key function
/// used when the production reader admits selected domain declarations.
/// Ordinary workspace tests have no QSpec checkout; the explicit conformance
/// target supplies one and runs this test alongside the public reader test.
#[trace("TC-048", "TC-229", "FR-038-AC-176", "FR-038-AC-303", "FR-038-AC-307")]
#[test]
fn tc_048_qspec_model_declaration_keys_use_production_derivation() {
    let root = match std::env::var("QUIRE_SPECIFICATION_DIR") {
        Ok(root) if !root.is_empty() => root,
        _ if std::env::var_os("QSPEC_CONFORMANCE").is_none() => return,
        _ => panic!("QUIRE_SPECIFICATION_DIR is required for QSpec conformance"),
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
    assert!(!rows.is_empty(), "QSpec must publish declaration vectors");
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
            (Some("model"), Some("record_value_type")) => DeclarationForm::RecordValueType,
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
    let population_path = std::path::Path::new(&root)
        .join("proposals/checked-package-v2/model-owner-form-vectors.json");
    let population_bytes = std::fs::read(&population_path)
        .unwrap_or_else(|error| panic!("{} cannot be read: {error}", population_path.display()));
    let population_vectors: serde_json::Value = serde_json::from_slice(&population_bytes)
        .unwrap_or_else(|error| panic!("{} is not JSON: {error}", population_path.display()));
    let owner_rows = population_vectors["model_declaration_nodes"]
        .as_array()
        .expect("QSpec model declaration vectors");
    assert!(owner_rows.len() >= 8, "QSpec P1-P8 owner vectors");
    for row in owner_rows {
        let name = row["name"].as_str().expect("owner vector name");
        let preimage = &row["preimage"];
        assert_eq!(preimage["semantic_type"], serde_json::Value::Null, "{name}");
        assert_eq!(preimage["declaration"], serde_json::Value::Null, "{name}");
        assert_eq!(preimage["recursion"], serde_json::Value::Null, "{name}");
        assert_eq!(
            preimage["body"],
            serde_json::json!({"term": "aggregate", "members": []}),
            "{name}"
        );
        assert_eq!(
            declaration_key(
                preimage["owner"]["identity"].as_str().expect("identity"),
                match (
                    preimage["node_tag"].as_str(),
                    preimage["semantic_form"].as_str()
                ) {
                    (Some("model"), Some("object_type")) => DeclarationForm::ObjectType,
                    (Some("model"), Some("process")) => DeclarationForm::Process,
                    (Some("model"), Some("state_machine")) => DeclarationForm::StateMachine,
                    (Some("model"), Some("persistence_interface")) =>
                        DeclarationForm::PersistenceInterface,
                    (Some("model"), Some("systems_interface")) => DeclarationForm::SystemsInterface,
                    (Some("relation"), Some("population")) => DeclarationForm::Population,
                    other => panic!("{name}: unknown declaration form {other:?}"),
                },
                preimage["owner"]["node"].as_str().expect("population node"),
                BYTES,
            )
            .expect("production declaration key"),
            row["sha256"].as_str().expect("published SHA-256"),
            "{name}"
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
#[trace("TC-048")]
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

/// A record owner adds one selected-row indexing charge, and the next lower
/// limit refuses at that row before a frame could read the owner.
#[trace("TC-056", "FR-040-AC-16")]
#[test]
fn tc_056_record_owner_index_uses_selected_row_work_budget() {
    let mut record = object_type("ix://acme/orders/Address", &[], vec![]);
    record["kind"]["name"] = json!("record");
    let mut selected = document(vec![object_type(WIDGET, &[], vec![]), record]);
    selected["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": "acme/orders", "name": "record"},
            "construct": {"meaning": "quire.meaning.model.record-value-type/v1"},
        }));
    let model = read(&selected).expect("selected object and record");
    let row = || member_pointer(&["lock", "model_selections"]).index(0);
    let index = |limit| {
        let mut meter = WorkMeter::new(limit);
        ModelOwners::new(std::slice::from_ref(&model), |_| meter.charge(1, row))
    };
    assert!(index(2).is_ok(), "the two owner keys use exactly two units");
    match index(1) {
        Err(ValidationFailure::Incomplete(record)) => {
            assert_eq!(record.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(
                record.path.as_ref().map(|path| path.as_str()),
                Some("/lock/model_selections/0")
            );
        }
        other => panic!("one less owner-index unit must be incomplete: {other:?}"),
    }
}

/// [`read_semantic_ir`] under a work limit nothing reaches, with the field
/// tables the lock stage builds.
pub(in crate::checked_package::v2) fn read(document: &Value) -> Result<DomainModel, ModelRefusal> {
    let mut meter = WorkMeter::new(u64::MAX);
    match read_semantic_ir(
        document,
        &mut Budget::new(&mut meter, 0, BYTES),
        &Arc::new(BTreeMap::new()),
    ) {
        Ok(model) => {
            let mut models = [model];
            build_field_tables(&mut models, &mut meter).expect("no limit is reached");
            let [model] = models;
            Ok(model)
        }
        Err(SelectionFailure::Refused(refusal)) => Err(refusal.refusal),
        Err(SelectionFailure::Limit(_) | SelectionFailure::InexactNumber { .. }) => {
            panic!("no limit or inexact number is reached")
        }
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

/// Trace: FR-069-AC-1, FR-069-AC-4, FR-069-AC-6.
#[trace("TC-940")]
#[test]
fn selected_tuple_positions_and_variant_payloads_are_retained() {
    const PAIR: &str = "ix://acme/orders/Pair";
    const MAYBE: &str = "ix://acme/orders/Maybe";
    let tuple = json!({
        "identity": PAIR, "kind": {"module": "acme/orders", "name": "tuple"},
        "positions": [
            {"identity": format!("{PAIR}/p0"), "typeRef": "ix://quire/native/Boolean"},
            {"identity": format!("{PAIR}/p1"), "typeRef": "ix://quire/native/Integer"},
        ]
    });
    let variant = json!({
        "identity": MAYBE, "kind": {"module": "acme/orders", "name": "variant"},
        "variants": [
            {"identity": format!("{MAYBE}/None"), "name": "None"},
            {"identity": format!("{MAYBE}/Some"), "name": "Some", "payloadType": PAIR},
        ]
    });
    let mut selected = document(vec![tuple.clone(), variant.clone()]);
    selected["constructs"]
        .as_array_mut()
        .expect("constructs")
        .extend([
            json!({"kind":{"module":"acme/orders","name":"tuple"},
            "construct":{"meaning":"quire.meaning.model.tuple-value-type/v1"}}),
            json!({"kind":{"module":"acme/orders","name":"variant"},
            "construct":{"meaning":"quire.meaning.model.variant-type/v1"}}),
        ]);
    let model = read(&selected).expect("selected tuple and payload variant");
    assert_eq!(
        model.tuple_types.get(PAIR).map(Vec::as_slice),
        Some(
            &[
                SelectedTypeRef::Native(NativeTypeRef::Boolean),
                SelectedTypeRef::Native(NativeTypeRef::Integer),
            ][..]
        )
    );
    let cases = model.variant_types.get(MAYBE).expect("selected variant");
    assert_eq!(cases.get("None"), Some(&None));
    assert_eq!(
        cases.get("Some"),
        Some(&Some(SelectedTypeRef::Declared(PAIR.into())))
    );

    selected["types"][0]["positions"][1]["identity"] = json!(format!("{PAIR}/p0"));
    assert_eq!(
        read(&selected).map(|_| ()),
        Err(ModelRefusal::new(
            Code::InvalidModelBinding,
            Cause::ConflictingBinding
        ))
    );
    selected["types"][0] = tuple;
    selected["types"][1]["variants"][1]["payloadType"] = json!("ix://acme/orders/Missing");
    assert_eq!(
        read(&selected).map(|_| ()),
        Err(ModelRefusal::new(
            Code::UnsupportedConstruct,
            Cause::DeclarationForm
        ))
    );

    selected["types"][1] = variant;
    selected["types"][0]["fields"] = json!([]);
    assert_eq!(read(&selected).map(|_| ()), Err(ModelRefusal::malformed()));
    selected["types"][0]
        .as_object_mut()
        .expect("tuple")
        .remove("fields");
    selected["types"][0]["positions"][1]["identity"] = json!(format!("{PAIR}/p2"));
    selected["types"][1]["variants"][1]["payloadType"] = json!("ix://acme/orders/Missing");
    assert_eq!(read(&selected).map(|_| ()), Err(ModelRefusal::malformed()));
}

#[trace("TC-048", "FR-038-AC-103")]
#[test]
fn operation_redefinitions_select_the_most_derived_member() {
    const CHILD: &str = "ix://acme/orders/Child";
    let mut types = vec![
        object_type(WIDGET, &[], vec![]),
        object_type(GADGET, &[WIDGET], vec![]),
        object_type(CHILD, &[GADGET], vec![]),
    ];
    for (position, owner) in [WIDGET, GADGET, CHILD].into_iter().enumerate() {
        let mut operation = json!({"identity": format!("{owner}/act"), "params": []});
        if owner != WIDGET {
            operation["redefines"] = json!(format!("{WIDGET}/act"));
        }
        types[position]["operations"] = json!([operation]);
    }
    let model = read(&document(types)).expect("operation declarations read");
    let resolved = resolve(&model, CHILD, MemberKind::Operation, "act")
        .expect("the most-derived operation is unambiguous");
    assert_eq!(resolved.identity(), "ix://acme/orders/Child/act");
}

/// Provenance: IR-874 inherited attempt operation binder join.
#[test]
fn inherited_operation_binders_use_named_receiver_and_retained_result() {
    let mut parent = object_type(WIDGET, &[], vec![]);
    parent["operations"] = json!([{
        "identity": format!("{WIDGET}/act"),
        "params": [{
            "typeRef": "ix://quire/native/Integer",
            "multiplicity": multiplicity(1, Some(1))
        }],
        "returns": {
            "typeRef": "ix://quire/native/Integer",
            "multiplicity": multiplicity(1, Some(1))
        }
    }]);
    let child = object_type(GADGET, &[WIDGET], vec![]);
    let model = read(&document(vec![parent, child])).expect("selected model");
    let mut meter = WorkMeter::new(u64::MAX);
    let signature = model
        .effective_operation_binder_types(GADGET, "act", &mut Budget::new(&mut meter, 0, BYTES))
        .expect("bounded resolution")
        .expect("inherited operation");
    let receiver = MemberType::Reference(
        declaration_key(&model.identity, DeclarationForm::ObjectType, GADGET, BYTES)
            .expect("object key")
            .into(),
    );
    assert_eq!(signature.len(), 3);
    assert_eq!(
        signature[0].digest.as_ref(),
        receiver.node_key(BYTES).expect("receiver key")
    );
    let integer = MemberType::Integer.node_key(BYTES).expect("integer key");
    assert_eq!(signature[1].digest.as_ref(), integer);
    assert_eq!(signature[2].digest.as_ref(), integer);
}

/// Provenance: IR-874 effective record binder order.
#[trace("TC-048", "FR-038-AC-325")]
#[test]
fn record_binders_keep_field_declaration_order() {
    const RECORD: &str = "ix://acme/orders/Record";
    let mut record = object_type(
        RECORD,
        &[],
        vec![
            field(RECORD, "z", "ix://quire/native/Integer"),
            field(RECORD, "a", "ix://quire/native/Boolean"),
        ],
    );
    record["kind"]["name"] = json!("record");
    let mut source = document(vec![record]);
    source["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": "acme/orders", "name": "record"},
            "construct": {"meaning": "quire.meaning.model.record-value-type/v1"}
        }));
    let model = read(&source).expect("selected record");
    let RecordBinderTypes::Known(binders) =
        model.record_binder_types(RECORD).expect("record signature")
    else {
        panic!("unambiguous record has a binder signature");
    };
    let integer = MemberType::Integer.node_key(BYTES).expect("integer key");
    let boolean = MemberType::Boolean.node_key(BYTES).expect("boolean key");
    assert_eq!(binders.len(), 2);
    assert_eq!(binders[0].digest.as_ref(), integer);
    assert_eq!(binders[1].digest.as_ref(), boolean);
}

#[trace("TC-048", "FR-038-AC-28")]
#[test]
fn operation_redefinitions_refuse_malformed_and_missing_targets() {
    for (target, parameter, expected) in [
        (
            Value::Null,
            json!({"typeRef": "ix://acme/orders/Absent", "multiplicity": multiplicity(1, Some(1))}),
            ModelRefusal::malformed(),
        ),
        (
            json!("ix://acme/orders/Absent/act"),
            json!({"typeRef": "ix://quire/native/Integer", "multiplicity": multiplicity(3, Some(1))}),
            ModelRefusal::missing_name(),
        ),
    ] {
        let mut declared = object_type(WIDGET, &[], vec![]);
        declared["operations"] = json!([
            {"identity": format!("{WIDGET}/act"), "params": [parameter], "redefines": target},
        ]);
        assert_eq!(read(&document(vec![declared])).map(|_| ()), Err(expected));
    }
}

#[trace("TC-225", "FR-346-AC-7")]
#[test]
fn operation_parameter_names_are_validated_at_selection() {
    let operation = format!("{WIDGET}/act");
    let parameter = |name: Value, identity: &str| {
        json!({"identity": identity, "name": name,
            "typeRef": "ix://quire/native/Integer",
            "multiplicity": multiplicity(1, Some(1))})
    };
    for name in [
        Value::Null,
        json!(1),
        json!(""),
        json!("1n"),
        json!("n-1"),
        json!("other"),
    ] {
        let mut object = object_type(WIDGET, &[], vec![]);
        object["operations"] = json!([{"identity": operation, "params": [parameter(name, &format!("{operation}/n"))]}]);
        assert_eq!(
            read(&document(vec![object])).map(|_| ()),
            Err(ModelRefusal::malformed())
        );
    }
    let mut object = object_type(WIDGET, &[], vec![]);
    let mut missing = parameter(json!("n"), &format!("{operation}/n"));
    missing.as_object_mut().expect("parameter").remove("name");
    object["operations"] = json!([{"identity": operation, "params": [missing]}]);
    assert_eq!(
        read(&document(vec![object])).map(|_| ()),
        Err(ModelRefusal::malformed())
    );
    let mut object = object_type(WIDGET, &[], vec![]);
    object["operations"] = json!([{"identity": operation, "params": [
        parameter(json!(""), &format!("{operation}/n")),
        parameter(json!("n"), &format!("{operation}/n"))]}]);
    let repeated = document(vec![object]);
    assert_eq!(
        read(&repeated).map(|_| ()),
        Err(ModelRefusal::new(
            Code::InvalidModelBinding,
            Cause::ConflictingBinding
        ))
    );
    let mut meter = WorkMeter::new(u64::MAX);
    let Err(SelectionFailure::Refused(refusal)) = read_semantic_ir(
        &repeated,
        &mut Budget::new(&mut meter, 0, BYTES),
        &Arc::new(BTreeMap::new()),
    ) else {
        panic!("expected repeated parameter identity refusal");
    };
    assert_eq!(
        refusal.declaration_identity.as_deref(),
        Some(format!("{operation}/n").as_str())
    );
    assert_eq!(
        refusal.declaration_path,
        Some(vec![
            Segment::Name("operations"),
            Segment::Index(0),
            Segment::Name("params"),
            Segment::Index(1),
        ])
    );
    assert_eq!(refusal.declaration_origin, None);
    let mut object = object_type(WIDGET, &[], vec![]);
    object["operations"] = json!([{"identity": operation, "params": [
        parameter(json!("n"), &format!("{operation}/n")),
        parameter(json!("N"), &format!("{operation}/N"))]}]);
    assert!(read(&document(vec![object])).is_ok());
}

#[trace("TC-225", "FR-346-AC-7")]
#[test]
fn operation_parameter_name_refusal_retains_the_operation_origin() {
    let operation = format!("{WIDGET}/act");
    let mut object = object_type(WIDGET, &[], vec![]);
    object["operations"] = json!([{
        "identity": operation,
        "origin": {"source": {"sourceIdentity": WIDGET, "path": "Widget.md",
            "startLine": 3, "startColumn": 7}},
        "params": [{"identity": format!("{operation}/n"), "name": "other",
            "typeRef": "ix://quire/native/Integer",
            "multiplicity": multiplicity(1, Some(1))}]
    }]);
    let mut meter = WorkMeter::new(u64::MAX);
    let selection = read_semantic_ir(
        &document(vec![object]),
        &mut Budget::new(&mut meter, 0, BYTES),
        &Arc::new(BTreeMap::new()),
    );
    let Err(SelectionFailure::Refused(refusal)) = selection else {
        panic!("expected a selected-document refusal");
    };
    assert_eq!(refusal.refusal, ModelRefusal::malformed());
    assert_eq!(
        refusal.declaration_path,
        Some(vec![
            Segment::Name("operations"),
            Segment::Index(0),
            Segment::Name("params"),
            Segment::Index(0),
            Segment::Name("name"),
        ])
    );
    assert_eq!(
        refusal.declaration_identity.as_deref(),
        Some(operation.as_str())
    );
    assert_eq!(
        refusal.declaration_origin,
        Some(IntakeDeclarationOrigin::Source {
            source_identity: WIDGET.into(),
            path: "Widget.md".into(),
            start_line: 3,
            start_column: 7,
            end_line: None,
            end_column: None,
        })
    );
}

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

#[trace("TC-048", "FR-038-AC-165")]
#[test]
fn tc_048_relationship_slot_does_not_change_nested_field_operation_or_parameter_ids() {
    let mut document = relationship_document(vec![relationship()]);
    document["types"][0]["fields"] = json!([field(WIDGET, "count", "ix://quire/native/Integer")]);
    document["types"][0]["operations"] = json!([{
        "identity": format!("{WIDGET}/inspect"),
        "params": [{"identity": format!("{WIDGET}/inspect/item"), "name": "item",
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

#[trace("TC-048", "FR-038-AC-168")]
#[test]
fn tc_048_relationship_document_read_uses_the_exact_selected_row_work_budget() {
    let document = relationship_document(vec![relationship()]);
    let read_at = |work| {
        let mut meter = WorkMeter::new(work);
        read_semantic_ir(
            &document,
            &mut Budget::new(&mut meter, 0, BYTES),
            &Arc::new(BTreeMap::new()),
        )
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
        Err(SelectionFailure::Limit(ValidationFailure::Incomplete(incomplete))) => {
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
#[trace("TC-048", "FR-038-AC-29")]
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

/// Native type references become closed selected metadata during the domain
/// document read; member typing does not reinterpret their wire spellings.
#[trace("TC-048", "FR-038-AC-29")]
#[test]
fn tc_048_selected_native_type_refs_are_retained_for_member_typing() {
    let model = read(&document(vec![object_type(
        WIDGET,
        &[],
        vec![
            field(WIDGET, "flag", "ix://quire/native/Boolean"),
            field(WIDGET, "count", "ix://quire/native/Integer"),
            field(WIDGET, "label", "ix://quire/native/String"),
            field(WIDGET, "key", "ix://quire/native/UUID"),
            field(WIDGET, "when", "ix://quire/native/Timestamp"),
            field(WIDGET, "ratio", "ix://quire/native/Rational"),
        ],
    )]))
    .expect("all declared native references admit");
    for (name, native, expected) in [
        ("flag", NativeTypeRef::Boolean, Some(MemberType::Boolean)),
        ("count", NativeTypeRef::Integer, Some(MemberType::Integer)),
        ("label", NativeTypeRef::String, Some(MemberType::String)),
        ("key", NativeTypeRef::Uuid, Some(MemberType::Uuid)),
        (
            "when",
            NativeTypeRef::Timestamp,
            Some(MemberType::Timestamp),
        ),
        ("ratio", NativeTypeRef::Rational, None),
    ] {
        let Resolved::Field(field) =
            resolve(&model, WIDGET, MemberKind::Field, name).expect("field resolves")
        else {
            panic!("expected a field");
        };
        assert_eq!(
            field.slot.type_ref,
            Some(SelectedTypeRef::Native(native)),
            "{name}"
        );
        assert_eq!(model.field_type(field), expected, "{name}");
    }
}

/// An FCD value type `scalar: integer` with `min` and `max` constraints is
/// the element type `Int[lo, hi]`.
#[trace("TC-048", "FR-038-AC-29")]
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
#[trace("TC-048", "FR-038-AC-28")]
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
#[trace("TC-048", "FR-038-AC-27")]
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
    evidence
        .insert_domain_package_document(digest.clone(), bytes.clone())
        .expect("first domain evidence insertion succeeds");
    let admit = |selection: &CheckedDomainPackageRef, evidence: &CheckedPackageEvidence| {
        let mut meter = WorkMeter::new(u64::MAX);
        admit_selection(
            selection,
            evidence,
            &mut Budget::new(&mut meter, 0, BYTES),
            &Arc::new(BTreeMap::new()),
        )
    };
    let admitted = admit(&selection("acme/orders", &digest), &evidence).expect("admitted");
    assert!(admitted.object_types.contains_key(WIDGET));
    let refusal =
        |selection: &CheckedDomainPackageRef, evidence: &CheckedPackageEvidence| match admit(
            selection, evidence,
        ) {
            Err(SelectionFailure::Refused(refusal)) => *refusal,
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
    forged
        .insert_domain_package_document(digest.clone(), b"{}".to_vec())
        .expect("first domain evidence insertion succeeds");
    assert_eq!(
        refusal(&selection("acme/orders", &digest), &forged),
        SelectionRefusal::mismatch(
            Cause::ContentMismatch,
            CheckedSemanticId {
                domain: DOMAIN_PACKAGE_DIGEST.into(),
                algorithm: "sha256".into(),
                digest: digest.into(),
            },
            CheckedSemanticId {
                domain: DOMAIN_PACKAGE_DIGEST.into(),
                algorithm: "sha256".into(),
                digest: quire_canonical::sha256(
                    &serde_json::json!({}),
                    quire_canonical::Limits::new(BYTES)
                )
                .expect("empty document digest")
                .to_string()
                .into(),
            },
        )
    );
}

const ZETA: &str = "ix://acme/orders/Zeta";
const BAD_ID: &str = "ix://acme/orders/bad-id";

/// QSpec FR-154: a reference to a node refused for its own object id is not
/// reported again as `missing-name`; the refused node's own refusal stands,
/// whether the referencing node is read before or after it.
#[trace("TC-048", "FR-038-AC-28")]
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
#[trace("TC-048", "FR-038-AC-28")]
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
#[trace("TC-048", "FR-038-AC-28")]
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
#[trace("TC-048", "FR-038-AC-28")]
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
#[trace("TC-048", "FR-038-AC-30")]
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
    read_semantic_ir(
        &document,
        &mut Budget::new(&mut unlimited, 2, BYTES),
        &Arc::new(BTreeMap::new()),
    )
    .expect("reads");
    let used = unlimited.consumed();
    assert!(used > 0, "a read is charged");
    let mut exact = WorkMeter::new(used);
    read_semantic_ir(
        &document,
        &mut Budget::new(&mut exact, 2, BYTES),
        &Arc::new(BTreeMap::new()),
    )
    .expect("exact work admits");
    let mut tight = WorkMeter::new(used - 1);
    let Err(SelectionFailure::Limit(failure)) = read_semantic_ir(
        &document,
        &mut Budget::new(&mut tight, 2, BYTES),
        &Arc::new(BTreeMap::new()),
    ) else {
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
#[trace("TC-227", "FR-038-AC-144")]
#[test]
fn tc_227_resolve_answers_match_the_values_recorded_before_the_field_tables() {
    let actual: Vec<String> = differential_documents()
        .iter()
        .flat_map(|(label, types)| resolutions(label, types))
        .collect();
    assert_eq!(actual, RECORDED);
}

mod intake_retention;
