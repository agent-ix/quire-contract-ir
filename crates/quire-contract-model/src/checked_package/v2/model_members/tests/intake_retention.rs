//! Actual selected-document intake, including ownership after document release.

use super::*;

fn source(path: &str) -> Value {
    json!({"source": {"sourceIdentity": WIDGET, "path": path,
        "startLine": 9007199254740992_u64, "startColumn": 7, "endColumn": 3}})
}

fn expected_source(path: &str) -> IntakeDeclarationOrigin {
    IntakeDeclarationOrigin::Source {
        source_identity: WIDGET.into(),
        path: path.into(),
        start_line: 9007199254740992,
        start_column: 7,
        end_line: None,
        end_column: Some(3),
    }
}

fn selected(document: Value, work: u64) -> (Result<DomainModel, SelectionFailure>, u64) {
    let bytes = serde_json::to_vec(&document).expect("authored bytes");
    let digest = quire_canonical::sha256(&document, quire_canonical::Limits::new(BYTES))
        .expect("authored document has exact numbers")
        .to_string();
    let selection = CheckedDomainPackageRef {
        identity: "acme/orders".into(),
        digest_domain: DOMAIN_PACKAGE_DIGEST.into(),
        digest: digest.clone().into(),
    };
    let mut evidence = CheckedPackageEvidence::new();
    evidence.insert_domain_package_document(digest, bytes);
    let mut meter = WorkMeter::new(work);
    let result = admit_selection(
        &selection,
        &evidence,
        &mut Budget::new(&mut meter, 3, BYTES),
    );
    // Both this input and intake's own parsed document are gone before assertions.
    quire_canonical::drop_value(document);
    (result, meter.consumed())
}

fn refusal(document: Value) -> SelectionRefusal {
    match selected(document, u64::MAX).0 {
        Err(SelectionFailure::Refused(refusal)) => refusal,
        other => panic!("expected actual intake refusal, got {other:?}"),
    }
}

fn bad_type(origin: Value) -> Value {
    let mut node = object_type("ix://acme/orders/bad-id", &[], vec![]);
    node["origin"] = origin;
    node
}

/// Trace: FR-038-AC-174, FR-038-AC-175, FR-038-AC-186
#[test]
fn tc_048_intake_retains_source_values_after_authentic_document_release() {
    let value = refusal(document(vec![bad_type(source("types/source.md"))]));
    assert_eq!(value.refusal, ModelRefusal::malformed());
    assert_eq!(value.member, None);
    assert_eq!(
        value.declaration_identity.as_deref(),
        Some("ix://acme/orders/bad-id")
    );
    assert_eq!(
        value.declaration_origin,
        Some(expected_source("types/source.md"))
    );
    // Independent optional presence: no end-pair/span-order rule may be invented.
    let mut origin = source("types/end-line.md");
    let members = origin["source"].as_object_mut().expect("source");
    members.remove("endColumn");
    members.insert("endLine".into(), json!(1));
    let retained = refusal(document(vec![bad_type(origin)]));
    assert!(matches!(
        retained.declaration_origin,
        Some(IntakeDeclarationOrigin::Source {
            start_line: 9007199254740992,
            end_line: Some(1),
            end_column: None,
            ..
        })
    ));
}

/// Trace: FR-038-AC-187
#[test]
fn tc_048_intake_retains_generated_version_and_ordered_repeated_inputs() {
    let origin = json!({"generated": {"generatorIdentity": GADGET,
        "generatorVersion": "1.2.3-alpha.7+build.09",
        "inputIdentities": [WIDGET, GADGET, WIDGET]}});
    let retained = refusal(document(vec![bad_type(origin)]));
    assert_eq!(
        retained.declaration_origin,
        Some(IntakeDeclarationOrigin::Generated {
            generator_identity: GADGET.into(),
            generator_version: "1.2.3-alpha.7+build.09".into(),
            input_identities: vec![WIDGET.into(), GADGET.into(), WIDGET.into()],
        })
    );
}

/// Trace: FR-038-AC-174, FR-038-AC-188, FR-038-AC-189
#[test]
fn tc_048_intake_retains_actual_nested_declaration_not_owner_or_missing_target() {
    let missing = "ix://acme/orders/Absent";
    let mut field_node = field(WIDGET, "field", missing);
    field_node["origin"] = source("nested/field.md");
    let mut owner = object_type(WIDGET, &[], vec![field_node]);
    owner["origin"] = source("owner.md");
    let retained = refusal(document(vec![owner]));
    assert_eq!(retained.refusal, ModelRefusal::missing_name());
    assert_eq!(
        retained.declaration_identity.as_deref(),
        Some("ix://acme/orders/Widget/field")
    );
    assert_eq!(
        retained.declaration_origin,
        Some(expected_source("nested/field.md"))
    );

    let operation = json!({"identity": "ix://acme/orders/Widget/perform", "name": "perform",
        "origin": source("nested/operation.md"), "params": [],
        "returns": {"typeRef": missing, "multiplicity": multiplicity(1, Some(1)),
            "origin": source("wrong-return-slot.md")}});
    let mut owner = object_type(WIDGET, &[], vec![]);
    owner["origin"] = source("owner.md");
    owner["operations"] = json!([operation.clone()]);
    let retained = refusal(document(vec![owner]));
    assert_eq!(
        retained.declaration_identity.as_deref(),
        Some("ix://acme/orders/Widget/perform")
    );
    assert_eq!(
        retained.declaration_origin,
        Some(expected_source("nested/operation.md"))
    );

    let mut owner = object_type(WIDGET, &[], vec![]);
    let mut operation = operation;
    operation
        .as_object_mut()
        .expect("operation")
        .remove("returns");
    operation["params"] = json!([{"identity": "ix://acme/orders/Widget/perform/argument",
        "origin": source("nested/parameter.md"), "typeRef": missing,
        "multiplicity": multiplicity(1, Some(1))}]);
    owner["operations"] = json!([operation.clone()]);
    let retained = refusal(document(vec![owner.clone()]));
    assert_eq!(
        retained.declaration_identity.as_deref(),
        Some("ix://acme/orders/Widget/perform/argument")
    );
    assert_eq!(
        retained.declaration_origin,
        Some(expected_source("nested/parameter.md"))
    );
    let params = owner["operations"][0]["params"][0]
        .as_object_mut()
        .expect("parameter");
    params.remove("identity");
    params.remove("origin");
    let retained = refusal(document(vec![owner]));
    assert_eq!(retained.declaration_identity, None);
    assert_eq!(retained.declaration_origin, None);

    let mut link = relationship();
    link["origin"] = source("nested/relationship.md");
    link["targetEnd"]["type"] = json!(missing);
    let retained = refusal(relationship_document(vec![link]));
    assert_eq!(retained.declaration_identity.as_deref(), Some(LINK));
    assert_eq!(
        retained.declaration_origin,
        Some(expected_source("nested/relationship.md"))
    );

    for identity in [Value::Null, json!(42)] {
        let mut node = bad_type(source("no-identity.md"));
        node["identity"] = identity;
        let retained = refusal(document(vec![node]));
        assert_eq!(retained.declaration_identity, None);
        assert_eq!(
            retained.declaration_origin,
            Some(expected_source("no-identity.md"))
        );
    }
}

/// Trace: FR-038-AC-189, FR-038-AC-190
#[test]
fn tc_048_intake_group_origin_requires_every_valid_equal_candidate() {
    let valid = source("same.md");
    let mut different_presence = valid.clone();
    different_presence["source"]
        .as_object_mut()
        .expect("source")
        .remove("endColumn");
    let malformed = json!({"source": {"path": "partial.md"}});
    for bad_kind in [false, true] {
        for (other_origin, expected) in [
            (Some(valid.clone()), Some(expected_source("same.md"))),
            (Some(source("different.md")), None),
            (Some(different_presence.clone()), None),
            (None, None),
            (Some(malformed.clone()), None),
        ] {
            let mut left = object_type(WIDGET, &[], vec![]);
            left["origin"] = valid.clone();
            let mut right = left.clone();
            match other_origin {
                Some(value) => right["origin"] = value,
                None => {
                    right.as_object_mut().expect("type").remove("origin");
                }
            }
            if bad_kind {
                right["kind"] = json!({"module": "acme/orders", "name": "unknown"});
            }
            for nodes in [
                vec![left.clone(), right.clone()],
                vec![right.clone(), left.clone()],
            ] {
                let retained = refusal(document(nodes));
                assert_eq!(retained.declaration_identity.as_deref(), Some(WIDGET));
                assert_eq!(retained.declaration_origin, expected);
                assert_eq!(
                    retained.refusal.cause,
                    if bad_kind {
                        Cause::MalformedDeclaration
                    } else {
                        Cause::ConflictingBinding
                    }
                );
            }
        }
    }
    let malformed_identity = bad_type(valid.clone());
    let retained = refusal(document(vec![
        malformed_identity.clone(),
        malformed_identity,
    ]));
    assert_eq!(retained.refusal.cause, Cause::MalformedDeclaration);
    assert_eq!(
        retained.declaration_origin,
        Some(expected_source("same.md"))
    );
    for origin in [
        Value::Null,
        json!({"source": {}, "generated": {}}),
        json!({"source": {"sourceIdentity": WIDGET, "path":"ok.md", "startLine":1, "startColumn":1, "extra":0}}),
        json!({"generated": {"generatorIdentity": WIDGET, "generatorVersion":"1.0.0", "inputIdentities":[]}}),
    ] {
        let retained = refusal(document(vec![bad_type(origin)]));
        assert_eq!(retained.declaration_origin, None);
    }
    let mut node = bad_type(valid);
    node.as_object_mut().expect("type").remove("origin");
    assert_eq!(
        refusal(document(vec![node.clone(), node])).declaration_origin,
        None
    );
}

/// Trace: FR-038-AC-196
#[test]
fn tc_048_intake_sequential_relationship_uses_later_actual_context() {
    let mut first = relationship();
    first["origin"] = source("earlier.md");
    let mut later = relationship();
    later["origin"] = source("later.md");
    let retained = refusal(relationship_document(vec![first.clone(), later.clone()]));
    assert_eq!(retained.refusal.cause, Cause::ConflictingBinding);
    assert_eq!(retained.declaration_identity.as_deref(), Some(LINK));
    assert_eq!(
        retained.declaration_origin,
        Some(expected_source("later.md"))
    );
    // Gadget precedes Widget in the existing node order. Both relationships
    // have a source matching their actual declaring owner.
    first["sourceEnd"]["type"] = json!(GADGET);
    let mut gadget = object_type(GADGET, &[], vec![]);
    gadget["relationships"] = json!([first]);
    let mut widget = object_type(WIDGET, &[], vec![]);
    widget["relationships"] = json!([later.clone()]);
    let retained = refusal(document(vec![widget.clone(), gadget.clone()]));
    assert_eq!(retained.refusal.cause, Cause::ConflictingBinding);
    assert_eq!(
        retained.declaration_origin,
        Some(expected_source("later.md"))
    );
    later
        .as_object_mut()
        .expect("relationship")
        .remove("origin");
    widget["relationships"] = json!([later]);
    let retained = refusal(document(vec![gadget, widget]));
    assert_eq!(retained.refusal.cause, Cause::MalformedDeclaration);
    assert_eq!(retained.declaration_identity.as_deref(), Some(LINK));
    assert_eq!(retained.declaration_origin, None);
}

/// Trace: FR-038-AC-191, FR-038-AC-192
#[test]
fn tc_048_intake_metadata_work_and_original_first_refusal_are_exact() {
    let mut admitted_node = object_type(WIDGET, &[], vec![]);
    admitted_node["origin"] = source("valid.md");
    let admitted_document = document(vec![admitted_node]);
    let (read, used) = selected(admitted_document.clone(), u64::MAX);
    assert!(read
        .expect("actual success")
        .object_types
        .contains_key(WIDGET));
    assert!(selected(admitted_document.clone(), used).0.is_ok());
    assert!(matches!(
        selected(admitted_document, used - 1).0,
        Err(SelectionFailure::Limit(_))
    ));
    let mut good = field(WIDGET, "good", "ix://quire/native/Integer");
    good["origin"] = source("wrong.md");
    let mut first = field(WIDGET, "first", "ix://acme/orders/Missing");
    first["origin"] = source("first.md");
    let mut later = field(WIDGET, "later", "ix://quire/native/Integer");
    later["presence"] = json!("invalid");
    later["origin"] = source("meaning.md");
    let doc = document(vec![object_type(WIDGET, &[], vec![good, first, later])]);
    // Meaning precedes Reference even though the reference field occurs first.
    let (result, used) = selected(doc.clone(), u64::MAX);
    let Err(SelectionFailure::Refused(retained)) = result else {
        panic!("refusal");
    };
    assert_eq!(retained.refusal.cause, Cause::MalformedDeclaration);
    assert_eq!(
        retained.declaration_origin,
        Some(expected_source("meaning.md"))
    );
    let Err(SelectionFailure::Refused(exact)) = selected(doc.clone(), used).0 else {
        panic!("exact work must preserve refusal");
    };
    assert_eq!(exact, retained);
    match selected(doc, used - 1).0 {
        Err(SelectionFailure::Limit(ValidationFailure::Incomplete(failure))) => {
            assert_eq!(
                failure.limit_kind,
                crate::checked_package::shared::CheckedPackageLimit::Work
            );
            assert_eq!(failure.limit, used - 1);
            assert_eq!(failure.consumed, used);
            assert_eq!(
                failure.path.as_ref().map(ToString::to_string).as_deref(),
                Some("/lock/model_selections/3")
            );
        }
        other => panic!("expected first unpayable charge, got {other:?}"),
    }
}

/// Trace: FR-038-AC-175, FR-038-AC-191
#[test]
fn tc_048_intake_predeclaration_failures_have_no_invented_metadata() {
    let original_document = document(vec![bad_type(source("unused.md"))]);
    let bytes = serde_json::to_vec(&original_document).expect("bytes");
    let digest = quire_canonical::sha256(&original_document, quire_canonical::Limits::new(BYTES))
        .expect("digest")
        .to_string();
    for (identity, supplied, cause, member) in [
        ("acme/orders", None, Cause::MissingSelection, "digest"),
        (
            "acme/orders",
            Some(b"{}".as_slice()),
            Cause::ByteDigestMismatch,
            "digest",
        ),
        (
            "acme/other",
            Some(bytes.as_slice()),
            Cause::WrongModelSelection,
            "identity",
        ),
    ] {
        let selection = CheckedDomainPackageRef {
            identity: identity.into(),
            digest_domain: DOMAIN_PACKAGE_DIGEST.into(),
            digest: digest.clone().into(),
        };
        let mut evidence = CheckedPackageEvidence::new();
        if let Some(bytes) = supplied {
            evidence.insert_domain_package_document(digest.clone(), bytes.to_vec());
        }
        let mut meter = WorkMeter::new(u64::MAX);
        let Err(SelectionFailure::Refused(retained)) = admit_selection(
            &selection,
            &evidence,
            &mut Budget::new(&mut meter, 0, BYTES),
        ) else {
            panic!("refusal");
        };
        assert_eq!(retained.refusal.cause, cause);
        assert_eq!(retained.member, Some(member));
        assert_eq!(retained.declaration_identity, None);
        assert_eq!(retained.declaration_origin, None);
    }
    // Independently authored bad-number bytes, supplied through the actual
    // selection evidence path. Number admission precedes both byte identity
    // comparison and the declaration's bad object-id spelling.
    let inexact = document(vec![bad_type(json!({"source": {
        "sourceIdentity": WIDGET, "path": "inexact.md",
        "startLine": 9007199254740993_u64, "startColumn": 1
    }}))]);
    let bytes = serde_json::to_vec(&inexact).expect("authored exact number text");
    use sha2::{Digest, Sha256};
    let raw_digest = format!("{:x}", Sha256::digest(&bytes));
    for selected_digest in [raw_digest, digest] {
        let selection = CheckedDomainPackageRef {
            identity: "acme/orders".into(),
            digest_domain: DOMAIN_PACKAGE_DIGEST.into(),
            digest: selected_digest.clone().into(),
        };
        let mut evidence = CheckedPackageEvidence::new();
        evidence.insert_domain_package_document(selected_digest, bytes.clone());
        let mut meter = WorkMeter::new(u64::MAX);
        match admit_selection(&selection, &evidence, &mut Budget::new(&mut meter, 0, BYTES)) {
            Err(SelectionFailure::InexactNumber { document_pointer, cause }) => {
                assert_eq!(document_pointer.to_string(), "/types/0/origin/source/startLine");
                assert_eq!(cause, Cause::InexactInteger);
            }
            other => panic!("expected original number refusal before dependency/declaration checks, got {other:?}"),
        }
    }
}
