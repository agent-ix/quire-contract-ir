// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Whole-read allocation accounting over actual existing TC-048 providers.
//! The dedicated binary retains its own allocator; this binary already has
//! allocation-counter's global allocator. Both use exactly the same oracle.

#[path = "support/read_allocation_proof.rs"]
mod proof;
#[path = "support/qspec_fma_inputs.rs"]
mod qspec_fma_inputs;
#[path = "support/qspec_inputs.rs"]
mod qspec_inputs;
#[path = "support/qspec_option_inputs.rs"]
mod qspec_option_inputs;
#[path = "support/qspec_protocol_location_inputs.rs"]
mod qspec_protocol_location_inputs;
#[path = "support/qspec_text_inputs.rs"]
mod qspec_text_inputs;
use crate::support::checked_package::{
    canonical, evidence_for, index_fixture_node_ids, positive_operation_identities, settle,
    v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use proof::{Counter, Expected};
use quire_contract_model::{
    initialize_operation_catalog, CheckedPackageEvidence, CheckedPackageRefusalCode as Code,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use syn::visit::{self, Visit};

struct IntegrationCounter;
impl Counter for IntegrationCounter {
    fn measure(read: impl FnOnce()) -> u64 {
        allocation_counter::measure(read).bytes_total
    }
}
struct Case {
    label: String,
    bytes: Vec<u8>,
    evidence: CheckedPackageEvidence,
    expected: Expected,
}
fn case(
    label: impl Into<String>,
    mut wire: Value,
    evidence: CheckedPackageEvidence,
    expected: Expected,
) -> Case {
    index_fixture_node_ids(&mut wire);
    Case {
        label: label.into(),
        bytes: canonical(&wire),
        evidence,
        expected,
    }
}
fn positive(label: impl Into<String>, wire: Value) -> Case {
    let evidence = evidence_for(&wire);
    case(label, wire, evidence, Expected::Admitted)
}

/// Reuses the exact builder dispatch of the existing catalogue admission
/// test, including its model-document controls. No copied wire builders.
fn positive_cases() -> Vec<Case> {
    let mut cases = vec![
        positive("TC048/all_families", v2_all_families()),
        positive("TC048/nominal", v2_nominal()),
        positive(
            "TC048/operation_identities",
            positive_operation_identities(),
        ),
    ];
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("actual operation catalogue");
    for entry in catalog["operations"].as_array().expect("operations") {
        let identity = entry["identity"].as_str().expect("identity");
        let external = match identity {
            "quire.op.record.project" => {
                Some(crate::checked_package_v2_model_fields::record_project_control())
            }
            "quire.op.model.navigate" | "quire.op.model.dispatch_call" => {
                Some(crate::checked_package_v2_model_members::model_member_control(identity))
            }
            _ => None,
        };
        if let Some((wire, evidence)) = external {
            cases.push(case(
                format!("TC048/{identity}"),
                wire,
                evidence,
                Expected::Admitted,
            ));
        } else if let Some(wire) =
            crate::checked_package_v2_catalogue_differential::admitted_fixture(identity)
                .or_else(|| crate::checked_package_v2_catalogue_differential::simple_fixture(entry))
        {
            cases.push(positive(format!("TC048/{identity}"), wire));
        }
    }
    cases
}

fn normalized_reference(reference: &str) -> &str {
    reference
        .strip_prefix("proposals/checked-package-v2/")
        .unwrap_or(reference)
}

/// The same files and construction functions the existing QSpec drivers use.
/// Files are read by reference; no copy of a published vector is stored here.
fn qspec_cases() -> (Vec<Case>, BTreeSet<String>) {
    let root = std::path::PathBuf::from(
        std::env::var_os("QUIRE_SPECIFICATION_DIR")
            .expect("whole-corpus proof requires the selected QSpec checkout"),
    );
    let proposal = root.join("proposals/checked-package-v2");
    let mut providers = BTreeSet::new();
    let mut cases = Vec::new();
    let fixtures = proposal.join("fixtures");
    let selections = qspec_inputs::vector_selected_documents(&proposal)
        .expect("actual published fixture-to-document selections");
    for name in qspec_inputs::POSITIVE_ALL {
        let wire = qspec_inputs::read_json(&fixtures.join(name)).expect("actual positive fixture");
        let document = proposal.join(qspec_inputs::positive_document(name, &selections));
        let evidence =
            qspec_inputs::evidence_for_published(&wire, name, &document, true, None, &fixtures)
                .expect("same conformance evidence construction");
        cases.push(Case {
            label: format!("QSpec/POSITIVE_ALL/{name}"),
            bytes: canonical(&wire),
            evidence,
            expected: Expected::Admitted,
        });
        providers.insert(format!("conformance_qspec/main.rs: fixtures/{name}"));
    }
    let read = |_driver: &str, file: &str| -> Value {
        let bytes = std::fs::read(proposal.join(file))
            .unwrap_or_else(|error| panic!("QSpec {file}: {error}"));
        serde_json::from_slice(&bytes).expect("published QSpec JSON")
    };
    let package = read(
        "checked_package_v2_owners.rs",
        "fixtures/positive-operation-identities.json",
    );
    let document = read(
        "checked_package_v2_owners.rs",
        "domain-package-acme-orders.json",
    );
    let evidence =
        crate::support::checked_package::evidence_with_document(&package, "acme/orders", &document);
    cases.push(Case {
        label: "QSpec/positive-operation-identities".into(),
        bytes: canonical(&package),
        evidence,
        expected: Expected::Admitted,
    });
    providers.insert(
        "it/checked_package_v2_owners.rs: fixtures/positive-operation-identities.json".into(),
    );
    let population_vectors = read("checked_package_v2_owners.rs", "fixtures/adverse.json");
    let population_rows = population_vectors["population_reference_cases"]
        .as_array()
        .expect("published population cases");
    assert_eq!(
        population_rows.len(),
        15,
        "same complete published driver inventory"
    );
    for row in population_rows {
        if row["operation"] == "published" {
            providers.insert(format!("it/checked_package_v2_owners.rs: fixtures/adverse.json#/population_reference_cases#{}",row["id"].as_str().expect("published case id")));
            continue;
        }
        providers.insert(format!(
            "it/checked_package_v2_owners.rs: fixtures/adverse.json#/population_reference_cases#{}",
            row["id"].as_str().expect("published case id")
        ));
        let fixture = crate::checked_package_v2_owners::population_reference_fixture(
            row,
            &population_vectors,
            package.clone(),
            document.clone(),
        );
        let expected = if row["outcome"] == "admitted" {
            Expected::Admitted
        } else {
            Expected::Refused(
                serde_json::from_value(row["expected_code"].clone())
                    .expect("published refusal code"),
            )
        };
        let evidence = crate::support::checked_package::evidence_with_document(
            &fixture.package,
            "acme/orders",
            &fixture.document,
        );
        cases.push(Case {
            label: format!("QSpec/{}", row["id"]),
            bytes: canonical(&fixture.package),
            evidence,
            expected,
        });
    }
    let vectors = read(
        "checked_package_v2_protocol_body.rs",
        "temporal-clock-binding-vectors.json",
    );
    let declarations = vectors["compensation_declarations"]
        .as_array()
        .expect("published compensation declarations");
    for vector in vectors["compensation_cases"]
        .as_array()
        .expect("published compensation cases")
    {
        providers.insert(format!("it/checked_package_v2_protocol_body.rs: temporal-clock-binding-vectors.json#/compensation_cases#{}",vector["id"].as_str().expect("published case id")));
        let (wire, _) =
            crate::checked_package_v2_protocol_body::compensation_clock_case(vector, declarations);
        let expected = if vector["expected"] == "admitted" {
            Expected::Admitted
        } else {
            Expected::Refused(
                serde_json::from_value(vector["code"].clone()).expect("published refusal code"),
            )
        };
        cases.push(Case {
            label: format!("QSpec/{}", vector["id"]),
            bytes: canonical(&wire),
            evidence: evidence_for(&wire),
            expected,
        });
    }
    let vectors = read(
        "checked_package_v2_protocol_body.rs",
        "temporal-fairness-target-vectors.json",
    );
    for vector in vectors["reader_cases"]
        .as_array()
        .expect("published fairness reader cases")
    {
        providers.insert(format!("it/checked_package_v2_protocol_body.rs: temporal-fairness-target-vectors.json#/reader_cases#{}",vector["id"].as_str().expect("published case id")));
        let (wire, _) =
            crate::checked_package_v2_protocol_body::instantiate_fairness_reader_vector(vector);
        let expected = Expected::Refused(
            serde_json::from_value(vector["expected"]["code"].clone())
                .expect("published refusal code"),
        );
        cases.push(Case {
            label: format!("QSpec/{}", vector["id"]),
            bytes: canonical(&wire),
            evidence: evidence_for(&wire),
            expected,
        });
    }
    for (file, shapes) in [
        ("text-bounds-shape-vectors.json", true),
        ("text-literal-vectors.json", false),
    ] {
        let vectors =
            qspec_inputs::read_json(&proposal.join(file)).expect("published Text vectors");
        for row in vectors["cases"].as_array().expect("published Text cases") {
            let id = row["id"].as_str().expect("Text case id");
            let (wire, _) = if shapes {
                let literal = match id {
                    "TS-02" => "ab",
                    "TS-03" => "e\u{301}",
                    _ => "outside-bounds",
                };
                qspec_text_inputs::text_bounds_package(
                    row["members"].as_array().expect("shape members"),
                    literal,
                )
            } else {
                let ty = &vectors["types"][row["type"].as_str().expect("Text type key")];
                qspec_text_inputs::bounded_literal(
                    ty["min"].as_u64().expect("minimum"),
                    ty["max"].as_u64().expect("maximum"),
                    ty["text_profile"].as_str().expect("Text profile"),
                    row["value"].clone(),
                )
            };
            let expected = match row["expected"].as_str().expect("published Text outcome") {
                "admitted" => Expected::Admitted,
                "refused" if shapes => Expected::Refused(Code::InvalidPackage),
                "refused" => Expected::Refused(Code::IllTyped),
                other => panic!("{file}#{id}: unknown outcome {other}"),
            };
            providers.insert(format!(
                "conformance_qspec/text_literals.rs: {file}#/cases#{id}"
            ));
            cases.push(Case {
                label: format!("QSpec/{file}#{id}"),
                bytes: canonical(&wire),
                evidence: evidence_for(&wire),
                expected,
            });
        }
    }
    let vectors = qspec_inputs::read_json(&proposal.join("model-root-vectors.json"))
        .expect("published model-root vectors");
    assert_eq!(vectors["base"], "fixtures/positive-all-families.json");
    let base = qspec_inputs::read_json_wire(&fixtures.join("positive-all-families.json"))
        .expect("published model-root base");
    for row in vectors["vectors"].as_array().expect("model-root vectors") {
        let id = row["id"].as_str().expect("model-root case id");
        let mut wire = base.clone();
        for patch in row["patch"].as_array().expect("model-root patches") {
            qspec_inputs::apply_vector_patch(&mut wire, patch);
        }
        let expected = match row["outcome"].as_str().expect("model-root outcome") {
            "admitted" => Expected::Admitted,
            outcome => Expected::Refused(
                serde_json::from_value(Value::String(
                    outcome
                        .strip_prefix("refused:")
                        .expect("published refusal")
                        .into(),
                ))
                .expect("published model-root refusal code"),
            ),
        };
        providers.insert(format!(
            "conformance_qspec/model_root.rs: model-root-vectors.json#/vectors#{id}"
        ));
        cases.push(Case {
            label: format!("QSpec/model-root/{id}"),
            bytes: canonical(&wire),
            evidence: qspec_inputs::evidence_for_published(
                &wire,
                id,
                &proposal.join(qspec_inputs::DOMAIN_DOCUMENT),
                true,
                None,
                &fixtures,
            )
            .expect("model-root evidence"),
            expected,
        });
    }
    let vectors = qspec_inputs::read_json(&proposal.join("node-ordinal-vectors.json"))
        .expect("published ordinal vectors");
    let base = qspec_inputs::read_json(&fixtures.join("positive-all-families.json"))
        .expect("published ordinal base");
    for row in vectors["uniqueness_cases"]
        .as_array()
        .expect("ordinal uniqueness cases")
    {
        let id = row["id"].as_str().expect("ordinal case id");
        let (wire, _) = qspec_inputs::ordinal_case_package(&base, row);
        let expected = match row["expected"].as_str().expect("ordinal outcome") {
            "admitted" => Expected::Admitted,
            code => Expected::Refused(
                serde_json::from_value(Value::String(code.into()))
                    .expect("published ordinal refusal code"),
            ),
        };
        providers.insert(format!(
            "conformance_qspec/node_ordinal.rs: node-ordinal-vectors.json#/uniqueness_cases#{id}"
        ));
        cases.push(Case {
            label: format!("QSpec/node-ordinal/{id}"),
            bytes: canonical(&wire),
            evidence: qspec_inputs::evidence_for_published(
                &wire,
                id,
                &proposal.join(qspec_inputs::DOMAIN_DOCUMENT),
                true,
                None,
                &fixtures,
            )
            .expect("ordinal evidence"),
            expected,
        });
    }
    let vectors = qspec_inputs::read_json(&proposal.join("fma-application-vectors.json"))
        .expect("published FMA vectors");
    for row in vectors["cases"].as_array().expect("FMA cases") {
        let id = row["id"].as_str().expect("FMA case id");
        let (wire, _) = qspec_fma_inputs::package_for(row);
        let expected = match row["expected"].as_str().expect("FMA outcome") {
            "admitted" => Expected::Admitted,
            "refused" => Expected::Refused(
                serde_json::from_value(row["code"].clone()).expect("published FMA refusal code"),
            ),
            other => panic!("{id}: unknown FMA outcome {other}"),
        };
        providers.insert(format!(
            "conformance_qspec/fma_application.rs: fma-application-vectors.json#/cases#{id}"
        ));
        cases.push(Case {
            label: format!("QSpec/FMA/{id}"),
            bytes: canonical(&wire),
            evidence: evidence_for(&wire),
            expected,
        });
    }
    let vectors = qspec_inputs::read_json(&proposal.join("node-index-vectors.json"))
        .expect("published operand vectors");
    assert_eq!(vectors["base"], "fixtures/positive-dependency-call.json");
    let base = qspec_inputs::read_json_wire(&fixtures.join("positive-dependency-call.json"))
        .expect("published operand base");
    for row in vectors["cases"].as_array().expect("operand cases") {
        let id = row["id"].as_str().expect("operand case id");
        let expected = row["expected"].as_str().expect("operand outcome");
        let expected = if expected.starts_with("passes_") {
            Expected::OriginalOracle(qspec_inputs::operand_checks_pass)
        } else {
            Expected::Refused(
                serde_json::from_value(Value::String(expected.into()))
                    .expect("published operand refusal code"),
            )
        };
        let mut wire = base.clone();
        for patch in row["patch"].as_array().expect("operand patches") {
            qspec_inputs::apply_node_index_patch(&mut wire, patch);
        }
        providers.insert(format!(
            "conformance_qspec/node_index.rs: node-index-vectors.json#/cases#{id}"
        ));
        cases.push(Case {
            label: format!("QSpec/node-index/{id}"),
            bytes: canonical(&wire),
            evidence: qspec_inputs::evidence_for_published(
                &wire,
                id,
                &proposal.join(qspec_inputs::DOMAIN_DOCUMENT),
                true,
                None,
                &fixtures,
            )
            .expect("operand evidence"),
            expected,
        });
    }
    let vectors = qspec_inputs::read_json_wire(&proposal.join("checked-term-data-vectors.json"))
        .expect("published protocol-location vectors");
    let positives = vectors["positive"]["protocol_locations"]
        .as_array()
        .expect("positive locations");
    for selected in positives {
        let id = selected["id"].as_str().expect("location case id");
        let rows = positives
            .iter()
            .filter(|row| {
                row["application"] == selected["application"]
                    && row["occurrence"] == selected["occurrence"]
                    && row["term_ordinal"].as_u64() <= selected["term_ordinal"].as_u64()
            })
            .map(|row| {
                let mut row = row.clone();
                row.as_object_mut().expect("location row").remove("id");
                row
            })
            .collect();
        let (wire, _) = qspec_protocol_location_inputs::fixture(rows);
        providers.insert(format!("conformance_qspec/protocol_locations.rs: checked-term-data-vectors.json#/positive/protocol_locations#{id}"));
        cases.push(positive(format!("QSpec/protocol-location/{id}"), wire));
    }
    for category in ["schema_invalid", "reader_refusals"] {
        for row in vectors[category]
            .as_array()
            .expect("location refusal cases")
            .iter()
            .filter(|row| row["id"].as_str().is_some_and(|id| id.starts_with("PL-")))
        {
            let id = row["id"].as_str().expect("location refusal id");
            let wire = if id.starts_with("PL-R3-") {
                qspec_protocol_location_inputs::published_leaf_fixture(row)
            } else {
                let rows = row["value"]
                    .as_array()
                    .cloned()
                    .unwrap_or_else(|| vec![row["value"].clone()]);
                qspec_protocol_location_inputs::fixture(rows).0
            };
            let expected = Expected::Refused(
                serde_json::from_value(row["expected_code"].clone())
                    .expect("published location refusal code"),
            );
            providers.insert(format!("conformance_qspec/protocol_locations.rs: checked-term-data-vectors.json#/{category}#{id}"));
            cases.push(Case {
                label: format!("QSpec/protocol-location/{id}"),
                bytes: canonical(&wire),
                evidence: evidence_for(&wire),
                expected,
            });
        }
    }
    let base = qspec_inputs::read_json(&fixtures.join("positive-all-families.json"))
        .expect("published structural/body base");
    let adverse = qspec_inputs::read_json(&fixtures.join("adverse.json"))
        .expect("published structural/body mutations");
    for list in ["structural_mutations", "body_grammar_mutations"] {
        for row in adverse[list].as_array().expect("published mutation list") {
            let id = row["id"].as_str().expect("mutation id");
            let pointer = row["pointer"].as_str().expect("mutation pointer");
            let mut wire = base.clone();
            qspec_inputs::replace_at(&mut wire, pointer, &row["replacement"])
                .expect("published mutation target");
            let outcome = row["outcome"].as_str().expect("mutation outcome");
            let code = outcome
                .strip_prefix("refused:")
                .expect("published refusal")
                .split('/')
                .next()
                .expect("refusal code");
            let expected = Expected::Refused(
                serde_json::from_value(Value::String(code.into()))
                    .expect("published mutation code"),
            );
            providers.insert(format!(
                "conformance_qspec/main.rs: fixtures/adverse.json#/{list}#{id}"
            ));
            let evidence = qspec_inputs::evidence_for_published(
                &wire,
                id,
                &proposal.join(qspec_inputs::DOMAIN_DOCUMENT),
                true,
                None,
                &fixtures,
            )
            .expect("mutation evidence");
            cases.push(Case {
                label: format!("QSpec/{list}/{id}"),
                bytes: canonical(&wire),
                evidence,
                expected,
            });
            if list == "body_grammar_mutations" {
                let mut control = base.clone();
                qspec_inputs::replace_at(
                    &mut control,
                    pointer,
                    row.get("flattened").expect("flattened control"),
                )
                .expect("flattened target");
                let evidence = qspec_inputs::evidence_for_published(
                    &control,
                    id,
                    &proposal.join(qspec_inputs::DOMAIN_DOCUMENT),
                    true,
                    None,
                    &fixtures,
                )
                .expect("flattened control evidence");
                cases.push(Case {
                    label: format!("QSpec/{list}/{id}/flattened-control"),
                    bytes: canonical(&control),
                    evidence,
                    expected: Expected::OriginalOracle(qspec_inputs::flat_body_control_passes),
                });
            }
        }
    }
    let vectors = qspec_inputs::read_json(&proposal.join("model-option-type-vectors.json"))
        .expect("published option vectors");
    let base = qspec_option_inputs::document(&vectors["domain_package"]);
    let (wire, evidence) = qspec_option_inputs::prepare(&base);
    cases.push(Case {
        label: format!(
            "QSpec/option/base-fields/{}-projected-types",
            base.origins.len()
        ),
        bytes: canonical(&wire),
        evidence,
        expected: Expected::Admitted,
    });
    for row in vectors["intake_cases"]
        .as_array()
        .expect("option intake cases")
    {
        let id = row["name"].as_str().expect("option case name");
        let mut projection = vectors["domain_package"].clone();
        for patch in row["patch"].as_array().expect("option projection patch") {
            qspec_option_inputs::apply(&mut projection, patch);
        }
        let input = qspec_option_inputs::document(&projection);
        let (wire, evidence) = qspec_option_inputs::prepare(&input);
        let expected = if row["expected"]["admitted"] == true {
            Expected::Admitted
        } else {
            Expected::Refused(
                serde_json::from_value(row["expected"]["refused"]["code"].clone())
                    .expect("published option refusal code"),
            )
        };
        providers.insert(format!("conformance_qspec/model_option_types.rs: model-option-type-vectors.json#/intake_cases#{id}"));
        cases.push(Case {
            label: format!("QSpec/option/{id}"),
            bytes: canonical(&wire),
            evidence,
            expected,
        });
    }
    (cases, providers)
}

fn refusal_cases() -> Vec<Case> {
    let mut base = v2_all_families();
    index_fixture_node_ids(&mut base);
    let evidence = evidence_for(&base);
    let mut cases = vec![
        Case {
            label: "malformed byte stream".into(),
            bytes: b"not JSON".to_vec(),
            evidence: evidence.clone(),
            expected: Expected::Refused(Code::MalformedWire),
        },
        Case {
            label: "duplicate member".into(),
            bytes: br#"{"name":"first","name":"second"}"#.to_vec(),
            evidence: evidence.clone(),
            expected: Expected::Refused(Code::DuplicateMember),
        },
    ];
    let mut mutate = |label: &str, code, edit: &dyn Fn(&mut Value)| {
        let mut wire = base.clone();
        edit(&mut wire);
        cases.push(Case {
            label: label.into(),
            bytes: canonical(&wire),
            evidence: evidence.clone(),
            expected: Expected::Refused(code),
        });
    };
    mutate("unknown contract", Code::UnknownContractVersion, &|v| {
        v["contract_version"] = json!("unknown")
    });
    mutate("unknown member", Code::UnknownMember, &|v| {
        v["extra"] = json!(true)
    });
    mutate(
        "required capability",
        Code::UnknownRequiredCapability,
        &|v| v["capability_report"] = json!([]),
    );
    mutate("repeated node id", Code::InvalidSemanticGraph, &|v| {
        v["semantic_graph"]["nodes"][1]["node_id"] =
            v["semantic_graph"]["nodes"][0]["node_id"].clone()
    });
    mutate(
        "missing source correspondence",
        Code::InvalidSourceMap,
        &|v| v["source_map"] = json!([]),
    );
    mutate("unknown node tag", Code::UnsupportedNodeTag, &|v| {
        v["semantic_graph"]["nodes"][0]["node_tag"] = json!("unknown-tag")
    });
    let mut noncanonical = b" ".to_vec();
    noncanonical.extend(canonical(&base));
    cases.push(Case {
        label: "noncanonical whitespace".into(),
        bytes: noncanonical,
        evidence: evidence.clone(),
        expected: Expected::Refused(Code::NoncanonicalWire),
    });
    let mut operation = positive_operation_identities();
    operation["semantic_graph"]["nodes"][4]["body"]["operation"]["identity"] =
        json!("quire.op.unknown");
    settle(&mut operation);
    cases.push(case(
        "unknown operation",
        operation.clone(),
        evidence_for(&operation),
        Expected::Refused(Code::InvalidPackage),
    ));
    let mut ambiguous = positive_operation_identities();
    crate::checked_package_v2_reader::declare(&mut ambiguous, 3, &["Example", "Widget"], 1);
    cases.push(case(
        "duplicate declaration",
        ambiguous.clone(),
        evidence_for(&ambiguous),
        Expected::Refused(Code::AmbiguousDeclaration),
    ));
    let mut no_documents = CheckedPackageEvidence::new();
    no_documents.support_feature(crate::support::checked_package::COMPLETE_VALUE_FEATURE);
    for feature in base["lock"]["required_features"]
        .as_array()
        .expect("features")
    {
        if let Some(feature) = feature.as_str() {
            no_documents.support_feature(feature);
        }
    }
    cases.push(Case {
        label: "selected model missing".into(),
        bytes: canonical(&base),
        evidence: no_documents,
        expected: Expected::Refused(Code::MissingImport),
    });
    let (law, _) =
        crate::checked_package_v2_temporal::clause_law("quire.fixture.temporal-profile/v1", None);
    cases.push(case(
        "unknown temporal profile",
        law.clone(),
        evidence_for(&law),
        Expected::Refused(Code::UnknownProfile),
    ));
    use crate::checked_package_v2_frame_entries as frame;
    for (label, code, wrong) in [
        ("missing frame dependency", Code::MissingDeclaration, false),
        ("wrong frame declaration", Code::InvalidModelBinding, true),
    ] {
        let (wire, _) = frame::frame_mutation(|node| {
            node["body"] = json!({"term":"frame","modifies":[frame::relationship(if wrong{frame::object_key()}else{frame::relationship_key()})],"creates":[],"deletes":[],"deletes_scoped":[]});
            node["dependencies"] = if wrong {
                json!([crate::support::checked_package::node_id(frame::object_key())])
            } else {
                json!([])
            };
        });
        cases.push(case(
            label,
            wire.clone(),
            evidence_for(&wire),
            Expected::Refused(code),
        ));
    }
    let mut relation = crate::checked_package_v2_model_members::authored_relationship();
    relation["targetEnd"]["multiplicity"]
        .as_object_mut()
        .expect("multiplicity")
        .remove("upper");
    let (wire, evidence) = crate::checked_package_v2_model_members::reaches_over(
        relation,
        "links",
        "ix://acme/orders/Order",
        "ix://acme/orders/Order",
    );
    cases.push(case(
        "unbounded relationship traversal",
        wire,
        evidence,
        Expected::Refused(Code::UnsupportedConstruct),
    ));
    let (mut wrong, decision, _, _) =
        crate::checked_package_v2_protocol_body::decision_fact_fixture();
    wrong["semantic_graph"]["nodes"][decision]["body"]["arguments"][1]["value"]["members"] =
        json!([]);
    settle(&mut wrong);
    cases.push(case(
        "unlisted decision fact",
        wrong.clone(),
        evidence_for(&wrong),
        Expected::Refused(Code::WrongSnapshot),
    ));
    let mut tasks =
        crate::checked_package_v2_schedulability::package("fixed-priority", false, false, 1);
    let n = tasks["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    tasks["semantic_graph"]["nodes"][n]["body"]["members"][4]["value"]["members"][0]["members"]
        [2]["value"]["value"] = json!("-1/2");
    crate::support::checked_package::refresh_identity(&mut tasks);
    cases.push(case(
        "negative task period",
        tasks.clone(),
        evidence_for(&tasks),
        Expected::Refused(Code::InvalidTimedModel),
    ));
    let mut wrong_result = crate::checked_package_v2_catalogue_differential::admitted_fixture(
        "quire.op.collection.count",
    )
    .expect("actual catalogue control");
    let position = wrong_result["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|n| n["body"]["operation"]["identity"] == "quire.op.collection.count")
        .expect("count application");
    let rational = crate::support::checked_package::node_id("scalar_type/rational");
    wrong_result["semantic_graph"]["nodes"][position]["semantic_type"] = rational.clone();
    wrong_result["semantic_graph"]["nodes"][position]["body"]["result_type"] = rational;
    settle(&mut wrong_result);
    cases.push(case(
        "wrong catalogue result type",
        wrong_result.clone(),
        evidence_for(&wrong_result),
        Expected::Refused(Code::IllTyped),
    ));
    cases
}

fn required_codes() -> BTreeSet<String> {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/crates/quire-contract-model/src/checked_package/shared.rs"
    ))
    .expect("public refusal source");
    let syntax = syn::parse_file(&source).expect("public refusal enum syntax");
    let enumeration = syntax
        .items
        .iter()
        .find_map(|i| match i {
            syn::Item::Enum(e) if e.ident == "CheckedPackageRefusalCode" => Some(e),
            _ => None,
        })
        .expect("actual public code enum");
    enumeration
        .variants
        .iter()
        .map(|v| v.ident.to_string())
        .collect()
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn every_registered_positive_and_refusal_is_a_genuine_public_input() {
    initialize_operation_catalog().expect("catalog gate");
    let mut seen = BTreeSet::new();
    let mut problems = Vec::new();
    let (mut admitted, mut refused, mut stage_contracts) = (0, 0, 0);
    let mut positives = positive_cases();
    let (published, _) = qspec_cases();
    positives.extend(published);
    let refusals = refusal_cases();
    for input in positives.iter().chain(&refusals) {
        let outcome =
            CheckedPackageV2::read(&input.bytes, proof::generous_limits(), &input.evidence);
        match (&input.expected, &outcome) {
            (Expected::Admitted, CheckedPackageV2ReadResult::Admitted(_)) => {
                admitted += 1;
            }
            (Expected::Refused(expected), CheckedPackageV2ReadResult::Refused(actual))
                if *expected == actual.code =>
            {
                seen.insert(format!("{:?}", actual.code));
                refused += 1;
            }
            (Expected::OriginalOracle(oracle), actual) if oracle(actual) => {
                stage_contracts += 1;
            }
            _ => problems.push(format!(
                "{}: fixture expectation differs from {outcome:?}",
                input.label
            )),
        }
    }
    let missing = required_codes()
        .difference(&seen)
        .cloned()
        .collect::<Vec<_>>();
    eprintln!("Registered candidates {}; confirmed admitted {}; confirmed refused {}; confirmed stage contracts {}; confirmed refusal-code classes {}",positives.len()+refusals.len(),admitted,refused,stage_contracts,seen.len());
    assert!(
        problems.is_empty() && missing.is_empty(),
        "Fixture defects:\n{}\nPublic refusal codes with no genuine input provider: {missing:?}",
        problems.join("\n")
    );
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn the_actual_tc048_catalogue_and_refusal_corpus_is_exactly_accounted() {
    initialize_operation_catalog().expect("catalog gate");
    let (published, _) = qspec_cases();
    let cases = positive_cases()
        .into_iter()
        .chain(published)
        .chain(refusal_cases())
        .collect::<Vec<_>>();
    for input in cases {
        proof::assert_read::<IntegrationCounter>(
            &input.label,
            &input.bytes,
            &input.evidence,
            input.expected,
        );
        proof::every_request::<IntegrationCounter>(&input.bytes, &input.evidence);
        proof::work_denials::<IntegrationCounter>(&input.label, &input.bytes, &input.evidence);
    }
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn the_counter_observes_full_reallocation_requests_and_drop_separately() {
    let requested = IntegrationCounter::measure(|| {
        let mut values = Vec::<u8>::with_capacity(8);
        values.extend([0; 8]);
        values.reserve_exact(24);
        drop(values);
    });
    assert_eq!(
        requested, 40,
        "8-byte initial allocation plus full32-byte replacement, ignoring frees"
    );
}

#[derive(Default)]
struct QSpecReferences {
    json: BTreeSet<String>,
    qspec: bool,
}
impl<'ast> Visit<'ast> for QSpecReferences {
    fn visit_lit_str(&mut self, string: &'ast syn::LitStr) {
        let value = string.value();
        if value.starts_with("QUIRE_SPECIFICATION_DIR")
            || value.contains("proposals/checked-package-v2")
        {
            self.qspec = true;
        }
        if value.ends_with(".json") {
            self.json.insert(value);
        }
        visit::visit_lit_str(self, string);
    }
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn by_reference_qspec_inputs_need_registered_whole_package_providers() {
    // These are actual driver references, not a guessed list of QSpec files.
    // A component-vector driver needs its real transformation into a complete
    // package before it may receive whole-read accounting credit.
    fn sources(directory: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(directory).expect("test source directory") {
            let path = entry.expect("test source entry").path();
            if path.is_dir() {
                sources(&path, files);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut files = Vec::new();
    sources(&root, &mut files);
    let mut references = BTreeSet::new();
    for path in files {
        if path
            .file_name()
            .is_some_and(|name| name == "checked_package_v2_whole_read_accounting.rs")
        {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("test source");
        let syntax = syn::parse_file(&source).expect("driver syntax");
        let mut driver = QSpecReferences::default();
        driver.visit_file(&syntax);
        let relative = path.strip_prefix(&root).expect("source below test root");
        if driver.qspec || relative.starts_with("conformance_qspec") {
            for reference in driver.json {
                references.insert(format!(
                    "{}: {}",
                    relative.display(),
                    normalized_reference(&reference)
                ));
            }
        }
    }
    let (_, providers) = qspec_cases();
    let qspec = std::path::PathBuf::from(
        std::env::var_os("QUIRE_SPECIFICATION_DIR").expect("selected QSpec checkout"),
    );
    let proposal = qspec.join("proposals/checked-package-v2");
    fn json_files(directory: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(directory).expect("QSpec source directory") {
            let path = entry.expect("QSpec file").path();
            if path.is_dir() {
                json_files(&path, files);
            } else if path.extension().is_some_and(|ext| ext == "json") {
                files.push(path);
            }
        }
    }
    fn vector_cases(value: &Value, path: &str, out: &mut BTreeSet<String>) {
        if let Value::Object(fields) = value {
            for (name, child) in fields {
                let path = format!("{path}/{name}");
                if let Some(rows) = child.as_array() {
                    for (i, row) in rows.iter().enumerate() {
                        let id = row
                            .get("id")
                            .or_else(|| row.get("name"))
                            .and_then(Value::as_str);
                        if let Some(id) = id.filter(|_| {
                            row.get("expected").is_some()
                                || row.get("outcome").is_some()
                                || row.get("expected_code").is_some()
                        }) {
                            out.insert(format!("{path}#{id}"));
                        } else {
                            vector_cases(row, &format!("{path}/{i}"), out);
                        }
                    }
                } else {
                    vector_cases(child, &path, out);
                }
            }
        }
    }
    let mut available = Vec::new();
    json_files(&qspec.join("proposals"), &mut available);
    let mut unresolved_packages = BTreeSet::new();
    let mut unresolved_recipes = BTreeSet::new();
    let mut auxiliary = BTreeSet::new();
    for reference in references {
        let (driver, name) = reference.split_once(": ").expect("source reference");
        let matches = available
            .iter()
            .filter(|path| path.ends_with(name))
            .collect::<Vec<_>>();
        if matches.is_empty() {
            auxiliary.insert(format!(
                "{reference}: no existing JSON bytes (not credited as a package input)"
            ));
            continue;
        }
        for path in matches {
            let bytes = std::fs::read(path).expect("referenced QSpec JSON");
            let value: Value = serde_json::from_slice(&bytes).expect("referenced QSpec syntax");
            let canonical = path.canonicalize().expect("source path");
            let matching_providers = providers
                .iter()
                .filter_map(|provider| {
                    let (source, recipe) = provider.split_once(": ")?;
                    if source != driver {
                        return None;
                    }
                    let (file, dataset) = recipe
                        .split_once('#')
                        .map_or((recipe, None), |(f, c)| (f, Some(c)));
                    let provided = proposal.join(file).canonicalize().ok()?;
                    (provided == canonical).then_some(dataset)
                })
                .collect::<Vec<_>>();
            if value.get("$schema").is_some()
                || (value.get("type").and_then(Value::as_str) == Some("object")
                    && value.get("properties").is_some())
            {
                auxiliary.insert(format!(
                    "{reference}: auxiliary schema, not complete V2 read bytes"
                ));
            } else if value.get("contractVersion").is_some() && value.get("package").is_some() {
                auxiliary.insert(format!(
                    "{reference}: selected IR document, not a separate checked-package read"
                ));
            } else if value.get("selected_documents").is_some() {
                auxiliary.insert(format!("{reference}: fixture evidence-selection manifest"));
            } else if value
                .get("contract_version")
                .is_some_and(|v| v == "quire.checked-package/v2")
                && value.pointer("/semantic_graph/nodes").is_some()
            {
                if !matching_providers.contains(&None) {
                    unresolved_packages.insert(format!("{reference}: complete V2 bytes exist, but this driver's input/evidence recipe remains unresolved ({})",path.display()));
                }
            } else if value.get("contract_version").is_some() {
                auxiliary.insert(format!(
                    "{reference}: another wire version; not credited as V2"
                ));
            } else {
                let mut rows = BTreeSet::new();
                vector_cases(&value, "", &mut rows);
                if rows.is_empty() {
                    unresolved_recipes.insert(format!("{reference}: JSON component/consumer provenance unresolved; not asserted to be a complete V2 package"));
                }
                for row in rows {
                    if !matching_providers.contains(&Some(row.as_str())) {
                        unresolved_recipes.insert(format!("{reference}#{row}: actual driver/list/case transformation remains unresolved"));
                    }
                }
            }
        }
    }
    let report = std::env::temp_dir().join(format!(
        "ir1404-f2-corpus-provider-gaps-{}.json",
        std::process::id()
    ));
    std::fs::write(&report,serde_json::to_vec_pretty(&json!({"unresolved_complete_package_recipes":unresolved_packages,"unresolved_vector_driver_recipes":unresolved_recipes,"auxiliary_or_no_bytes_not_credited":auxiliary})).expect("structured provider report")).expect("write provider report");
    assert!(unresolved_packages.is_empty()&&unresolved_recipes.is_empty(),"Unresolved complete-package recipes ({}), sample:\n{}\nUnresolved vector/consumer provenance ({}), sample:\n{}\nAuxiliary/no-byte references excluded without package credit: {}\nComplete structured classification: {}",unresolved_packages.len(),unresolved_packages.iter().take(15).cloned().collect::<Vec<_>>().join("\n"),unresolved_recipes.len(),unresolved_recipes.iter().take(15).cloned().collect::<Vec<_>>().join("\n"),auxiliary.len(),report.display());
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn repeated_reads_preserve_the_callers_storage_delta_with_the_integration_counter() {
    let value = v2_all_families();
    proof::shared_budget_reads::<IntegrationCounter>(&canonical(&value), &evidence_for(&value));
}
