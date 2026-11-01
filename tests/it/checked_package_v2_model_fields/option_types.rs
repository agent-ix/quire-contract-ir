// SPDX-License-Identifier: AGPL-3.0-or-later
//! Named option definitions admitted through the public reader.

use super::*;

fn options(inner: Value, presence: bool) -> Value {
    document(vec![
        json!({"identity":orders("Opt"),"kind":"option","inner":inner}),
        object(
            "Node",
            &[],
            &[
                if presence {
                    field("named", &orders("Opt")).optional()
                } else {
                    field("named", &orders("Opt"))
                },
                field("optional", "ix://quire/native/Integer").optional(),
            ],
        ),
    ])
}

fn refusal(doc: &Value) -> quire_contract_model::CheckedPackageRefusal {
    let (wire, evidence, _) = package(doc, &[("Node", OBJECT)], &[]);
    match read_with(&wire, &evidence, CheckedPackageReadLimits::bounded()) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("selected option must refuse: {other:?}"),
    }
}

#[trace("TC-465", "FR-356-AC-1", "FR-356-AC-2")]
#[test]
fn named_and_presence_options_share_the_public_reader_result_node() {
    let doc = options(json!("ix://quire/native/Integer"), false);
    let admitted = admit(
        &doc,
        &[("Node", OBJECT)],
        &[
            Read {
                owner: "Node",
                field: "named",
                ty: Ty::Option(Box::new(Ty::Integer)),
            },
            Read {
                owner: "Node",
                field: "optional",
                ty: Ty::Option(Box::new(Ty::Integer)),
            },
        ],
    );
    let fields = admitted
        .model_object_fields(&declaration_id("Node"))
        .unwrap();
    assert_eq!(
        fields.field("named").unwrap().member_type(),
        Some(&CheckedMemberType::Option(Box::new(
            CheckedMemberType::Integer
        )))
    );
    assert_eq!(
        fields.field("named").unwrap().member_type(),
        fields.field("optional").unwrap().member_type()
    );
    let result_ids = admitted
        .graph()
        .nodes
        .iter()
        .filter_map(|node| {
            if node.semantic_form.as_ref() != "query" {
                return None;
            }
            match admitted.term(&node.node_id) {
                Some(quire_contract_model::CheckedTerm::Application { result_type, .. }) => {
                    Some(result_type)
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(result_ids.len(), 2);
    assert_eq!(result_ids[0], result_ids[1]);
    let (wrong, evidence, _) = package(
        &doc,
        &[("Node", OBJECT)],
        &[Read {
            owner: "Node",
            field: "named",
            ty: Ty::Option(Box::new(Ty::Boolean)),
        }],
    );
    assert!(
        matches!(
            read_with(&wrong, &evidence, CheckedPackageReadLimits::bounded()),
            CheckedPackageV2ReadResult::Refused(_)
        ),
        "a different option result type must not pass the owning member join"
    );
}

#[trace("TC-465", "FR-356-AC-3", "FR-356-AC-4")]
#[test]
fn invalid_option_inners_and_single_presence_retain_document_loci() {
    for inner in [
        Value::Null,
        json!(""),
        json!(true),
        json!(["ix://quire/native/Integer"]),
        json!({}),
        json!("Integer"),
        json!("ix://quire/native/Clock"),
        json!(orders("Opt")),
    ] {
        let refused = refusal(&options(inner.clone(), false));
        assert_eq!(
            refused.code,
            CheckedPackageRefusalCode::InvalidModelBinding,
            "{inner}"
        );
        assert_eq!(
            refused.cause,
            Some(CheckedPackageRefusalCause::MalformedDeclaration)
        );
        assert_eq!(
            refused
                .document_pointer
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("/types/0/inner"),
            "{inner}"
        );
    }
    let refused = refusal(&options(json!(orders("Missing")), false));
    assert_eq!(refused.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refused.cause, Some(CheckedPackageRefusalCause::MissingName));
    let refused = refusal(&options(json!("ix://quire/native/Integer"), true));
    assert_eq!(
        refused
            .document_pointer
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/types/1/fields/0/presence")
    );
    let mut malformed = options(json!("ix://quire/native/Integer"), true);
    malformed["types"][1]["fields"][0]["typeRef"] = json!("ix://quire/native/Option<Integer>");
    let refused = refusal(&malformed);
    assert_eq!(
        refused
            .document_pointer
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/types/1/fields/0/typeRef")
    );
}

#[trace("TC-465", "FR-356-AC-4")]
#[test]
fn option_shape_precedes_constraints_and_inner_kind() {
    let mut doc = options(json!("ix://quire/native/Clock"), false);
    doc["types"][0]["fields"] = json!([]);
    doc["types"][0]["constraints"] = json!([{"keyword":"bad"}]);
    let refused = refusal(&doc);
    assert_eq!(
        refused
            .document_pointer
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/types/0/fields")
    );
    doc["types"][0].as_object_mut().unwrap().remove("fields");
    let refused = refusal(&doc);
    assert_eq!(
        refused
            .document_pointer
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/types/0/constraints")
    );
}

#[trace("TC-465", "FR-356-AC-1", "FR-356-AC-2", "FR-356-AC-3")]
#[test]
fn aliases_and_collection_wrapping_preserve_single_option_layers() {
    let mut doc = options(json!("ix://quire/native/Integer"), false);
    doc["types"]
        .as_array_mut()
        .unwrap()
        .push(json!({"identity":orders("AO"),"kind":"alias","target":orders("Opt")}));
    doc["types"].as_array_mut().unwrap().push(json!({
        "identity":orders("AI"),"kind":"alias","target":"ix://quire/native/Integer"
    }));
    doc["types"][1]["fields"][1]["typeRef"] = json!(orders("AI"));
    doc["types"][1]["fields"][0]["typeRef"] = json!(orders("AO"));
    doc["types"][1]["fields"][0]["multiplicity"] =
        json!({"lower":0,"upper":3,"ordered":true,"unique":false});
    let admitted = admit(&doc, &[("Node", OBJECT)], &[]);
    let fields = fields_of(&admitted, "Node");
    assert_eq!(
        fields.field("optional").unwrap().member_type(),
        Some(&CheckedMemberType::Option(Box::new(
            CheckedMemberType::Integer
        )))
    );
    assert_eq!(
        fields.field("named").unwrap().member_type(),
        Some(&CheckedMemberType::Collection {
            kind: CheckedCollectionKind::Sequence,
            element: Box::new(CheckedMemberType::Option(Box::new(
                CheckedMemberType::Integer
            ))),
            bounds: Some((0, 3)),
        })
    );
    doc["types"][0]["inner"] = json!(orders("AO"));
    let refused = refusal(&doc);
    assert_eq!(
        refused
            .document_pointer
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("/types/0/inner")
    );
}
