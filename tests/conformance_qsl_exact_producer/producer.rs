// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! External typed consumer of QSL's selected-model identity producer.

use qsl_foundation::SourceIdentity;
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::{check::PackageDeclarations, model::intake};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, ExactTypeProjectionErrorCode as Code, SelectedExactDeclarationForm,
    SelectedExactDeclarationRow, SelectedExactFieldRow, SelectedExactIdentities,
    SelectedExactIdentity,
};
use quire_exact::{Presence, ValueType};
use quire_semantic_value::checking::CheckingLimits;
use serde_json::{json, Value};

const MODEL: &str = "test/ir1003-producer";

fn identity(name: &str) -> String {
    format!("ix://{MODEL}/{name}")
}

fn origin(name: &str) -> Value {
    json!({"generated":{"generatorIdentity":identity(name),
        "generatorVersion":"1.0.0", "inputIdentities":[identity(name)]}})
}

fn field(owner: &str, name: &str, native: &str, optional: bool) -> Value {
    json!({"identity":identity(&format!("{owner}/{name}")), "name":name,
        "typeRef":format!("ix://quire/native/{native}"),
        "presence":if optional {"optional"} else {"required"},
        "nullable":false, "defaultKind":"none",
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true},
        "origin":origin(&format!("{owner}/{name}"))})
}

fn declaration(name: &str, kind: &str, parents: &[&str], fields: Vec<Value>) -> Value {
    json!({"identity":identity(name), "displayName":name,
        "kind":{"module":MODEL,"name":kind}, "roles":[], "origin":origin(name),
        "constraints":[], "extensions":[], "unknownPolicy":"reject",
        "supertypes":parents.iter().map(|name| identity(name)).collect::<Vec<_>>(),
        "fields":fields, "operations":[], "relationships":[]})
}

fn selected_document() -> Value {
    let mut ledger = declaration(
        "Ledger",
        "entity",
        &[],
        vec![field("Ledger", "id", "Integer", false)],
    );
    ledger["identityFields"] = json!([identity("Ledger/id")]);
    json!({"contractVersion":"2.0.0",
        "canonicalization":{"algorithm":"rfc8785-v1"},
        "source":{"identity":identity("spec"), "version":"1.0.0", "dialect":"spec-bundle"},
        "occurrences":[], "extensions":[], "package":{"identity":MODEL,"version":"1", "mappingVersions":[], "profileVersions":[]},
    "constructs":[
        {"kind":{"module":MODEL,"name":"record"},"moduleVersion":"1.0.0",
            "construct":{"identity":"none","shape":"record",
                "members":{"identityFields":"optional"},
                "meaning":"quire.meaning.model.record-value-type/v1"}},
        {"kind":{"module":MODEL,"name":"entity"},"moduleVersion":"1.0.0",
            "construct":{"identity":"none","shape":"record",
                "members":{"identityFields":"required"},
                "meaning":"quire.meaning.model.object-type/v1"}}
    ],
    "types":[
        declaration("BaseEvent", "record", &[], vec![field("BaseEvent", "serial", "Integer", false)]),
        declaration("PostedEvent", "record", &["BaseEvent"], vec![
            field("PostedEvent", "amount", "Integer", false),
            field("PostedEvent", "confirmed", "Boolean", true)]),
        ledger
    ]})
}

fn join_error(
    admitted: &CheckedPackageV2,
    declarations: Vec<SelectedExactDeclarationRow>,
    fields: Vec<SelectedExactFieldRow>,
    node: &CheckedNodeId,
    code: Code,
) {
    let error = SelectedExactIdentities::from_stage_rows(admitted, declarations, fields)
        .expect_err("invalid producer rows must not yield a projection witness");
    assert_eq!(error.code, code, "{error:?}");
    assert_eq!(error.requested.as_ref(), node);
    assert_eq!(error.at.as_deref(), Some(node));
}

fn main() {
    let document = serde_json::to_vec(&selected_document()).expect("independently authored model");
    let packages = intake::package_input([document.as_slice()]).expect("public model intake");
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n\
         model Domain = {MODEL:?};\n\
         function serial using v(e: Domain::PostedEvent): Integer pure {{ e.serial }}\n\
         function keep using v(a: Domain::Ledger): Domain::Ledger pure {{ a }}\n"
    );
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "test:ir1003-producer"),
        "identity-producer.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("public source parser");
    assert_eq!(parsed.diagnostics(), []);
    let unit = qsl_forms::build_unit(&parsed).expect("public forms builder");
    let models = intake::admit_unit(
        &unit.selections().models,
        &packages,
        qsl_semantics::model::accounting::ModelNormalizationLimits::default(),
    )
    .expect("public selected-model admission");
    let graph = PackageDeclarations::assemble(
        parsed.source().reference().clone(),
        "ir1003-producer".to_owned(),
        unit,
        models,
        Vec::new(),
    )
    .expect("public assembly")
    .check(CheckingLimits::default())
    .expect("public checking");
    assert!(!graph.selected_exact_declaration_rows().is_empty());
    assert!(!graph.selected_exact_field_rows().is_empty());
    let checked = CheckedPackage::link(graph);
    let emission = emit_checked(&checked).expect("public V2 emission");
    assert_eq!(emission.omitted(), []);
    let wire: Value = serde_json::from_slice(emission.package().bytes()).expect("V2 envelope");
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(MODEL, document.as_slice())
        .expect("same selected document");
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature name"));
    }
    let read = CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    let CheckedPackageV2ReadResult::Admitted(admitted) = read else {
        panic!("actual QSL emission must admit with its selected document: {read:?}");
    };
    // The consumer neither mints identities nor serializes witness rows across processes.
    let witnesses = checked
        .graph()
        .selected_exact_identities(&admitted)
        .expect("genuine QSL stage rows join public IR admission");
    let declarations = witnesses.declaration_rows().cloned().collect::<Vec<_>>();
    let fields = witnesses.field_rows().to_vec();
    let record = declarations
        .iter()
        .find(|row| row.original_identity.as_ref() == identity("PostedEvent"))
        .expect("producer child-record row");
    let object = declarations
        .iter()
        .find(|row| row.original_identity.as_ref() == identity("Ledger"))
        .expect("producer object row");
    let SelectedExactIdentity::Record(record_key) = &record.exact_identity else {
        panic!("record key");
    };
    let SelectedExactIdentity::Object(effective) = &object.exact_identity else {
        panic!("object id");
    };
    let projection = admitted
        .project_exact_type(&record.structural_node, &witnesses, 256)
        .expect("external consumer projects the genuine record");
    assert_eq!(
        projection.value_type(),
        &ValueType::Composite(record_key.clone())
    );
    let view = projection.record_view().expect("complete record view");
    assert_eq!(view.record_key(), record_key);
    let expected = [
        (
            "BaseEvent/serial",
            "serial",
            ValueType::Integer,
            Presence::Required,
        ),
        (
            "PostedEvent/amount",
            "amount",
            ValueType::Integer,
            Presence::Required,
        ),
        (
            "PostedEvent/confirmed",
            "confirmed",
            ValueType::Boolean,
            Presence::Optional,
        ),
    ];
    assert_eq!(
        view.fields().len(),
        expected.len(),
        "unread fields must survive"
    );
    for (field, (original, name, value_type, presence)) in view.fields().iter().zip(expected) {
        assert_eq!(field.ir_identity(), identity(original));
        assert_eq!(field.name(), name);
        assert_eq!(field.value_type(), &value_type);
        assert_eq!(field.presence(), presence);
        let producer = fields
            .iter()
            .find(|row| {
                row.record_node == record.structural_node
                    && row.field_original_identity.as_ref() == identity(original)
            })
            .expect("actual producer field");
        assert_eq!(field.member_id(), &producer.member_id);
    }
    let reference = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "composite_type" && node.semantic_form.as_ref() == "reference"
        })
        .expect("emitted object-reference type");
    assert_eq!(
        admitted
            .project_exact_type(&reference.node_id, &witnesses, 256)
            .expect("external reference consumer")
            .value_type(),
        &ValueType::Reference(effective.clone())
    );

    let absent_object_rows = declarations
        .iter()
        .filter(|row| row.structural_node != object.structural_node)
        .cloned()
        .collect::<Vec<_>>();
    let absent_object =
        SelectedExactIdentities::from_stage_rows(&admitted, absent_object_rows, fields.clone())
            .expect("record witnesses still join without the object row");
    let error = admitted
        .project_exact_type(&reference.node_id, &absent_object, 256)
        .expect_err("reference projection cannot default a missing object identity");
    assert_eq!(error.code, Code::UnavailableIdentityWitness);
    assert_eq!(error.requested.as_ref(), &reference.node_id);
    assert_eq!(error.at.as_deref(), Some(&object.structural_node));

    let absent = admitted
        .project_exact_type(
            &record.structural_node,
            &SelectedExactIdentities::empty(),
            256,
        )
        .expect_err("no authoritative declaration witness");
    assert_eq!(absent.code, Code::UnavailableIdentityWitness);
    assert_eq!(absent.at.as_deref(), Some(&record.structural_node));
    let missing_fields = fields
        .iter()
        .filter(|row| {
            !(row.record_node == record.structural_node
                && row.field_original_identity.as_ref() == identity("BaseEvent/serial"))
        })
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(missing_fields.len() + 1, fields.len());
    let missing =
        SelectedExactIdentities::from_stage_rows(&admitted, declarations.clone(), missing_fields)
            .expect("other genuine producer rows still join");
    let error = admitted
        .project_exact_type(&record.structural_node, &missing, 256)
        .expect_err("missing inherited field refuses whole projection");
    assert_eq!(error.code, Code::UnavailableIdentityWitness);
    assert_eq!(error.at.as_deref(), Some(&record.structural_node));

    for mutation in ["document", "form", "original", "alias", "namespace"] {
        let mut changed = declarations.clone();
        let row = changed
            .iter_mut()
            .find(|row| row.structural_node == record.structural_node)
            .unwrap();
        match mutation {
            "document" => row.selected_document = "test/foreign-document".into(),
            "form" => {
                row.form = SelectedExactDeclarationForm::ObjectType;
                row.exact_identity = object.exact_identity.clone();
            }
            "original" => row.original_identity = identity("BaseEvent").into(),
            "alias" => row.original_identity = "Domain::PostedEvent".into(),
            "namespace" => row.original_identity = identity("Other/PostedEvent").into(),
            _ => unreachable!(),
        }
        join_error(
            &admitted,
            changed,
            fields.clone(),
            &record.structural_node,
            Code::MismatchedIdentityWitness,
        );
    }
    let mut duplicate = declarations.clone();
    let mut competing = record.clone();
    competing.exact_identity = declarations
        .iter()
        .find(|row| row.original_identity.as_ref() == identity("BaseEvent"))
        .expect("independent real record identity")
        .exact_identity
        .clone();
    duplicate.push(competing);
    join_error(
        &admitted,
        duplicate,
        fields.clone(),
        &record.structural_node,
        Code::ConflictingIdentityWitness,
    );
    let mut duplicate = fields.clone();
    let child_field = fields
        .iter()
        .find(|row| row.record_node == record.structural_node)
        .unwrap()
        .clone();
    duplicate.push(child_field.clone());
    join_error(
        &admitted,
        declarations.clone(),
        duplicate,
        &record.structural_node,
        Code::ConflictingIdentityWitness,
    );
    for mutation in ["document", "record", "field"] {
        let mut changed = fields.clone();
        let row = changed
            .iter_mut()
            .find(|row| {
                row.record_node == child_field.record_node
                    && row.field_original_identity == child_field.field_original_identity
            })
            .unwrap();
        match mutation {
            "document" => row.selected_document = "test/foreign-document".into(),
            "record" => row.record_original_identity = identity("BaseEvent").into(),
            "field" => row.field_original_identity = identity("Ledger/id").into(),
            _ => unreachable!(),
        }
        join_error(
            &admitted,
            declarations.clone(),
            changed,
            &record.structural_node,
            Code::MismatchedIdentityWitness,
        );
    }
    let mut foreign = record.clone();
    foreign.structural_node.package = "test/foreign-namespace".into();
    let foreign_node = foreign.structural_node.clone();
    join_error(
        &admitted,
        vec![foreign],
        Vec::new(),
        &foreign_node,
        Code::ForeignNode,
    );
    println!("FR-352-AC-9 producer, record/reference projections and typed adverse joins passed");
}
