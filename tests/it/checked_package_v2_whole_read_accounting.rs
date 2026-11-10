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
#[path = "support/qspec_tuple_inputs.rs"]
mod qspec_tuple_inputs;
use crate::support::checked_package::{
    canonical, evidence_for, index_fixture_node_ids, positive_operation_identities, settle,
    v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use proof::{Counter, Expected};
use quire_contract_model::{
    initialize_operation_catalog, CheckedPackageEvidence, CheckedPackageRefusalCode as Code,
    CheckedPackageV2, CheckedPackageV2BudgetedReadResult as Budgeted, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

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

/// The same wire and evidence preparation used by the original wide-bound
/// native control. No additional normalization or replacement recipe.
fn wide_bounded_quantity_case() -> Case {
    let (wire, _, _, _) = crate::checked_package_v2_map_type::nested_bounded_quantity_wire();
    Case {
        label: "native/map_type/nested_bounded_quantity_wire".into(),
        bytes: canonical(&wire),
        evidence: evidence_for(&wire),
        expected: Expected::Admitted,
    }
}

#[trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
#[test]
fn original_wide_bounded_quantity_provider_is_accounted_within_the_bound_at_core_bounds() {
    let input = wide_bounded_quantity_case();
    proof::assert_read::<IntegrationCounter>(
        &input.label,
        &input.bytes,
        &input.evidence,
        input.expected,
    );
    eprintln!("Original wide bounded-quantity builder: admitted; exact core accounting; no all-family FR-038-AC-516 or source-site census claim");
}

/// The already registered all-families input contains one protocol body
/// with thirteen direct children. It is not a chain of graph references.
/// This covers AC-516's term-walk growth leg; its numeric families remain separate.
#[cfg(feature = "test-support")]
#[trace("TC-472", "FR-038-AC-511", "FR-038-AC-512", "FR-038-AC-516")]
#[test]
fn original_protocol_term_tree_prepays_each_reserved_walk_growth() {
    use quire_contract_model::{
        CheckedNodeId, CheckedPackageReadLimits, CheckedPackageV2BudgetedReadResult as Budgeted,
        CheckedTerm, NativeBudgetResource, NativeJoinLocusSource, PackageReadStage,
    };
    let wire = v2_all_families();
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("original graph");
    let (protocol, node) = nodes
        .iter()
        .enumerate()
        .find(|(_, node)| node["semantic_form"] == "protocol_clause")
        .expect("the original protocol clause");
    assert_eq!(
        node["body"]["arguments"]
            .as_array()
            .expect("original arguments")
            .len(),
        13
    );
    let id: CheckedNodeId = serde_json::from_value(node["node_id"].clone())
        .expect("the original declared protocol identity");
    let bytes = canonical(&wire);
    let evidence = evidence_for(&wire);
    let limits = CheckedPackageReadLimits::bounded();
    let plain = crate::read_and_account(&bytes, limits, &evidence);
    let (reference, events) = crate::qspec_read_proof::observe_native::<IntegrationCounter>(
        &bytes,
        &evidence,
        limits,
        limits.work,
        u64::MAX,
    );
    let (CheckedPackageV2ReadResult::Admitted(plain), Budgeted::Admitted(checked)) =
        (&plain, &reference.result)
    else {
        panic!("the original provider must admit");
    };
    assert_eq!(
        plain, checked,
        "observer and budget preserve the complete original package"
    );
    let Some(CheckedTerm::Application { arguments, .. }) = checked.term(&id) else {
        panic!("the retained protocol body is the original application");
    };
    assert_eq!(
        arguments.len(),
        13,
        "the actual admitted tree has thirteen direct children"
    );
    assert!(
        matches!(arguments.first(), Some(CheckedTerm::Binding { .. })),
        "the first child has its own value child"
    );

    let growths = events
        .iter()
        .enumerate()
        .filter(|(_, event)| {
            event.resource == NativeBudgetResource::ReadStorage
                && event.admitted
                && event.amount > 0
                && event.allocation_class == Some("walker_growth")
                && event.locus.is_some_and(|locus| {
                    locus.form == "walk"
                        && locus.source
                            == NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, protocol)
                })
        })
        .collect::<Vec<_>>();
    // Instance/list IDs are not part of NativeBudgetEvent. Attribution is
    // source-bound: state/temporal placement enqueue every child, with no
    // SkipChildren, and quire-walk extends per child into one pending list.
    // This admitted thirteen-child enter therefore requires capacities
    // 4/8/16 in that list; the task stack has its separate full-capacity costs.
    assert!(
        growths.len() >= 6,
        "one wide term walk needs both growing lists"
    );
    // At this enter the SDK's task list grows from four elements to the
    // root frame plus thirteen children (fourteen). The first binding then
    // needs fifteen elements, so that list doubles to twenty-eight.
    // These are source-bound capacity patterns, not fabricated instance IDs.
    let wide_patterns = growths
        .windows(6)
        .filter(|sequence| {
            sequence[1].1.amount.checked_mul(2) == Some(sequence[2].1.amount)
                && sequence[2].1.amount.checked_mul(2) == Some(sequence[3].1.amount)
                && sequence[0].1.amount.checked_mul(14) == sequence[4].1.amount.checked_mul(4)
                && sequence[4].1.amount.checked_mul(2) == Some(sequence[5].1.amount)
        })
        .count();
    assert!(
        wide_patterns > 0,
        "actual full-capacity requests must match pending 4/8/16 and task 4/14/28 growth"
    );
    eprintln!("Source-bound wide-enter capacity patterns: {wide_patterns}; no list/invocation IDs are present in the events");
    eprintln!("One existing all-families provider; protocol node {protocol}; thirteen typed children; {} actual growth requests. Per-walk attribution is source-bound, not invocation-tagged.", growths.len());
    for (index, event) in &growths {
        let (index, event) = (*index, **event);
        eprintln!("Actual walker growth event {index}: {event:?}");
        let exact_bound = event
            .consumed_before
            .checked_add(u64::try_from(event.amount).expect("bounded growth request"))
            .expect("bounded cumulative read storage");
        let (exact, exact_events) = crate::qspec_read_proof::observe_native::<IntegrationCounter>(
            &bytes,
            &evidence,
            limits,
            limits.work,
            exact_bound,
        );
        assert_eq!(
            exact_events.get(..=index),
            events.get(..=index),
            "the exact allowance admits the same complete event prefix"
        );
        let (short, short_events) = crate::qspec_read_proof::observe_native::<IntegrationCounter>(
            &bytes,
            &evidence,
            limits,
            limits.work,
            exact_bound - 1,
        );
        assert_eq!(short_events.get(..index), events.get(..index));
        let mut denied = event;
        denied.admitted = false;
        assert_eq!(
            short_events.get(index),
            Some(&denied),
            "one below denies the actual full-size callback request"
        );
        assert_eq!(short.budget.first_unpaid_read_storage(), Some(event.amount));
        assert_eq!(short.budget.first_unpaid_locus(), event.locus);
        assert!(
            matches!(
                &short.result,
                Budgeted::BudgetIncomplete(_) | Budgeted::Exhausted(_)
            ),
            "the denied growth returns no partial admitted package"
        );
        assert!(
            !short_events[index + 1..].iter().any(|later| {
                later.resource == NativeBudgetResource::ReadStorage
                    && later.allocation_class == Some("walker_growth")
                    && later.locus.is_some_and(|locus| locus.form == "walk")
            }),
            "no later walker allocation runs after the unpaid growth"
        );
        let (repeat, repeat_events) = crate::qspec_read_proof::observe_native::<IntegrationCounter>(
            &bytes,
            &evidence,
            limits,
            limits.work,
            exact_bound - 1,
        );
        assert_eq!(short.result, repeat.result);
        assert_eq!(
            short.budget, repeat.budget,
            "the entire fresh caller state repeats"
        );
        assert_eq!(short.requested, repeat.requested);
        assert_eq!(short_events, repeat_events);
        for observation in [exact, short, repeat] {
            proof::drop_without_request::<IntegrationCounter>(observation);
        }
    }
    proof::drop_without_request::<IntegrationCounter>(reference);
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
    // These are the original native builders used by the corresponding
    // semantic tests, including their settled graph and model selections.
    // Construct outside the counter, then run the same public read oracle.
    let (wire, _, _, _) = crate::checked_package_v2_protocol_body::decision_fact_fixture();
    cases.push(positive("native/protocol_body/decision_fact_fixture", wire));
    let (wire, _, _) = crate::checked_package_v2_protocol_body::package_with_send();
    cases.push(positive("native/protocol_body/package_with_send", wire));
    let (wire, _, _) =
        crate::checked_package_v2_protocol_body::package_with_admitted_compensation();
    cases.push(positive(
        "native/protocol_body/package_with_admitted_compensation",
        wire,
    ));
    let (wire, _, _) = crate::checked_package_v2_protocol_body::package_with_present_commit();
    cases.push(positive(
        "native/protocol_body/package_with_present_commit",
        wire,
    ));
    let (wire, _) = crate::checked_package_v2_frame_entries::state_operand_fixture();
    cases.push(positive("native/frame_entries/state_operand_fixture", wire));
    let (wire, _) = crate::checked_package_v2_operand_families::quantity_sum_control();
    cases.push(positive(
        "native/operand_families/quantity_sum_control",
        wire,
    ));
    let (wire, _) = crate::checked_package_v2_operand_families::carried_sum_control();
    cases.push(positive(
        "native/operand_families/carried_sum_control",
        wire,
    ));
    cases.push(positive(
        "native/checked_package/admitted_protocol_families",
        crate::support::checked_package::admitted_protocol_families(),
    ));
    let (wire, document, _) = crate::support::checked_package::protocol_ports::role_port_fixture();
    let evidence =
        crate::support::checked_package::evidence_with_document(&wire, "test/families", &document);
    cases.push(case(
        "native/protocol_ports/role_port_fixture",
        wire,
        evidence,
        Expected::Admitted,
    ));
    cases.push(wide_bounded_quantity_case());
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

struct DependencyCallCase {
    input: Case,
    mutation: Option<Value>,
}

/// Reuses the dependency-call driver's original patch and unsupplied-evidence
/// helpers. The raw importer base is never resolved before its wire patches.
fn dependency_call_cases() -> (Vec<DependencyCallCase>, BTreeSet<String>) {
    let root = std::path::PathBuf::from(
        std::env::var_os("QUIRE_SPECIFICATION_DIR")
            .expect("QUIRE_SPECIFICATION_DIR is required: it names the selected QSpec checkout"),
    );
    let proposal = root.join("proposals/checked-package-v2");
    let fixtures = proposal.join("fixtures");
    let document = proposal.join(qspec_inputs::DOMAIN_DOCUMENT);
    let vectors = qspec_inputs::read_json(&proposal.join("dependency-call-vectors.json"))
        .expect("published dependency-call vectors");
    assert_eq!(vectors["base"], "fixtures/positive-dependency-call.json");
    assert_eq!(
        vectors["library"]["package"],
        "fixtures/positive-dependency-library.json"
    );
    let base = qspec_inputs::read_json_wire(&fixtures.join("positive-dependency-call.json"))
        .expect("original raw importer base");
    let mut cases = Vec::new();
    let mut providers = BTreeSet::new();
    for name in [
        "positive-dependency-call.json",
        "positive-dependency-library.json",
    ] {
        let wire = if name == "positive-dependency-call.json" {
            base.clone()
        } else {
            qspec_inputs::read_json(&fixtures.join(name)).expect("original library input")
        };
        let evidence = qspec_inputs::evidence_for_published(
            &wire,
            "checked package",
            &document,
            true,
            None,
            &fixtures,
        )
        .expect("original read_package evidence");
        cases.push(DependencyCallCase {
            input: Case {
                label: format!("QSpec/dependency-call/{name}"),
                bytes: canonical(&wire),
                evidence,
                expected: Expected::Admitted,
            },
            mutation: None,
        });
        providers.insert(format!(
            "conformance_qspec/dependency_call.rs: fixtures/{name}"
        ));
    }
    let mutations = vectors["mutations"]
        .as_array()
        .expect("published dependency mutations");
    assert_eq!(
        mutations.len(),
        11,
        "original complete dependency-call family"
    );
    for row in mutations {
        let id = row["id"].as_str().expect("mutation id");
        let mut wire = base.clone();
        for patch in row["patch"].as_array().expect("original patch list") {
            qspec_inputs::apply_dependency_call_patch(&mut wire, patch);
        }
        let evidence = if row["supplied"] == false {
            qspec_inputs::dependency_call_evidence_without_dependency(&wire)
        } else {
            qspec_inputs::evidence_for_published(
                &wire,
                "checked package",
                &document,
                true,
                None,
                &fixtures,
            )
            .expect("original supplied dependency evidence")
        };
        let outcome = row["outcome"].as_str().expect("published outcome");
        let code = outcome
            .strip_prefix("refused:")
            .expect("published refusal")
            .split('/')
            .next()
            .expect("refusal code");
        cases.push(DependencyCallCase {
            input: Case {
                label: format!("QSpec/dependency-call/{id}"),
                bytes: canonical(&wire),
                evidence,
                expected: Expected::Refused(
                    serde_json::from_value(json!(code)).expect("published refusal code"),
                ),
            },
            mutation: Some(row.clone()),
        });
        providers.insert(format!(
            "conformance_qspec/dependency_call.rs: dependency-call-vectors.json#/mutations#{id}"
        ));
    }
    (cases, providers)
}

/// The existing driver's complete code/cause/primary-locus assertions are
/// checked before these inputs may be credited to the allocation registry.
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn dependency_call_providers_preserve_original_outcomes_and_loci() {
    initialize_operation_catalog().expect("catalog gate");
    let (cases, _) = dependency_call_cases();
    let (mut admitted, mut refused) = (0, 0);
    for case in cases {
        let actual = CheckedPackageV2::read(
            &case.input.bytes,
            quire_contract_model::CheckedPackageReadLimits::bounded(),
            &case.input.evidence,
        );
        if let Some(row) = case.mutation {
            let CheckedPackageV2ReadResult::Refused(refusal) = actual else {
                panic!(
                    "{}: original mutation must refuse: {actual:?}",
                    case.input.label
                );
            };
            let code = serde_json::to_value(refusal.code).expect("code wire word");
            let expected = match refusal.cause {
                Some(cause) => format!(
                    "refused:{}/{}",
                    code.as_str().expect("code"),
                    serde_json::to_value(cause)
                        .expect("cause wire word")
                        .as_str()
                        .expect("cause")
                ),
                None => format!("refused:{}", code.as_str().expect("code")),
            };
            assert_eq!(
                expected, row["outcome"],
                "{}: code and cause",
                case.input.label
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().map(ToString::to_string),
                row["locus"].as_str().map(str::to_owned),
                "{}: primary locus",
                case.input.label
            );
            refused += 1;
        } else {
            assert!(
                matches!(actual, CheckedPackageV2ReadResult::Admitted(_)),
                "{}: original direct input must admit: {actual:?}",
                case.input.label
            );
            admitted += 1;
        }
    }
    eprintln!("Dependency-call family: direct admitted {admitted}; published mutations refused {refused}; unresolved 0; original-source-site census join pending");
}

/// The same files and construction functions the existing QSpec drivers use.
/// Files are read by reference; no copy of a published vector is stored here.
fn qspec_cases() -> (Vec<Case>, BTreeSet<String>) {
    let root = std::path::PathBuf::from(
        std::env::var_os("QUIRE_SPECIFICATION_DIR")
            .expect("QUIRE_SPECIFICATION_DIR is required: it names the selected QSpec checkout"),
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
            let id = row["id"].as_str().expect("published case id");
            assert_eq!(row["outcome"], "admitted", "published population control");
            cases.push(Case {
                label: format!("QSpec/population/{id}"),
                bytes: canonical(&package),
                evidence: crate::support::checked_package::evidence_with_document(
                    &package,
                    "acme/orders",
                    &document,
                ),
                expected: Expected::Admitted,
            });
            providers.insert(format!("it/checked_package_v2_owners.rs: fixtures/adverse.json#/population_reference_cases#{id}"));
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
    let (base, field_queries) = qspec_option_inputs::field_query_inputs(&vectors);
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
    // The original driver applies every query to this same admitted base;
    // source sharing establishes input provenance without crediting extra reads.
    for query in field_queries {
        let id = query["name"].as_str().expect("option field query name");
        providers.insert(format!("conformance_qspec/model_option_types.rs: model-option-type-vectors.json#/field_type_cases#{id}"));
    }
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
    let vectors = qspec_inputs::read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vectors");
    assert_eq!(vectors["base"], "fixtures/positive-model-tuple-union.json");
    let base = qspec_inputs::read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published tuple/union base");
    let selected = qspec_inputs::read_json(&proposal.join(qspec_inputs::positive_document(
        "positive-model-tuple-union.json",
        &selections,
    )))
    .expect("selected tuple/union document");
    for row in vectors["wire_cases"]
        .as_array()
        .expect("tuple/union wire cases")
    {
        let id = row["id"].as_str().expect("tuple/union case id");
        let mut wire = base.clone();
        for patch in row["patch"].as_array().expect("tuple/union patches") {
            qspec_tuple_inputs::apply_wire_patch(&mut wire, patch);
        }
        let expected = match row["outcome"].as_str().expect("tuple/union outcome") {
            "admitted" => Expected::Admitted,
            outcome => Expected::Refused(
                serde_json::from_value(Value::String(
                    outcome
                        .strip_prefix("refused:")
                        .expect("published tuple/union refusal")
                        .into(),
                ))
                .expect("published tuple/union code"),
            ),
        };
        let evidence = qspec_tuple_inputs::selected_evidence(&selected);
        providers.insert(format!("conformance_qspec/model_tuple_union.rs: model-tuple-union-vectors.json#/wire_cases#{id}"));
        cases.push(Case {
            label: format!("QSpec/tuple-union/wire/{id}"),
            bytes: canonical(&wire),
            evidence,
            expected,
        });
    }
    let (dependency_inputs, dependency_providers) = dependency_call_cases();
    cases.extend(dependency_inputs.into_iter().map(|case| case.input));
    providers.extend(dependency_providers);
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
    let mut native_providers = Vec::new();
    let (mut admitted, mut refused, mut stage_contracts) = (0, 0, 0);
    let mut positives = positive_cases();
    let (published, _) = qspec_cases();
    positives.extend(published);
    let refusals = refusal_cases();
    for input in positives.iter().chain(&refusals) {
        let outcome =
            CheckedPackageV2::read(&input.bytes, proof::generous_limits(), &input.evidence);
        if input.label.starts_with("native/") {
            let actual = match &outcome {
                CheckedPackageV2ReadResult::Admitted(_) => "admitted",
                CheckedPackageV2ReadResult::Refused(_) => "refused",
                CheckedPackageV2ReadResult::Incomplete(_) => "incomplete",
                CheckedPackageV2ReadResult::CatalogUnavailable(_) => "catalog_unavailable",
            };
            native_providers.push(json!({
                "provider": input.label,
                "expected": "admitted",
                "actual": actual,
                "reconciled": matches!(outcome, CheckedPackageV2ReadResult::Admitted(_)),
            }));
        }
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
    let report = std::env::temp_dir().join(format!(
        "ir1404-native-provider-outcomes-{}.json",
        std::process::id()
    ));
    let native_admitted = native_providers
        .iter()
        .filter(|row| row["actual"] == "admitted")
        .count();
    let native_refused = native_providers
        .iter()
        .filter(|row| row["actual"] == "refused")
        .count();
    let native_unresolved = native_providers
        .iter()
        .filter(|row| row["reconciled"] != true)
        .count();
    std::fs::write(
        &report,
        serde_json::to_vec_pretty(&json!({
            "scope": "registered original native builders; not an original-read-site census",
            "providers": native_providers,
            "admitted": native_admitted,
            "refused": native_refused,
            "unresolved": native_unresolved,
        }))
        .expect("native provider outcomes"),
    )
    .expect("write native provider outcome report");
    eprintln!("Original native builders: admitted {native_admitted}; refused {native_refused}; unresolved {native_unresolved}; {}", report.display());
    assert!(
        problems.is_empty() && missing.is_empty(),
        "Fixture defects:\n{}\nPublic refusal codes with no genuine input provider: {missing:?}",
        problems.join("\n")
    );
}

fn registered_cases() -> Vec<Case> {
    let (published, _) = qspec_cases();
    positive_cases()
        .into_iter()
        .chain(published)
        .chain(refusal_cases())
        .collect()
}

/// A fast core tier over the very same real registry. Full boundary discovery
/// remains a separate test; this tier does not claim normative sweep coverage.
#[trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
#[test]
fn every_registered_input_is_accounted_within_the_bound_at_core_bounds() {
    initialize_operation_catalog().expect("catalog gate");
    let cases = registered_cases();
    assert!(!cases.is_empty(), "the real registered corpus is nonempty");
    eprintln!("Registry core tier: {} actual inputs; ample/exact/last-short/repeat; full-boundary proof=false", cases.len());
    for input in cases {
        eprintln!("Registry core actual input: {}", input.label);
        proof::assert_read::<IntegrationCounter>(
            &input.label,
            &input.bytes,
            &input.evidence,
            input.expected,
        );
    }
}

/// Keep the refusal-diagnostic accounting claim tied to the named QSpec
/// cases that motivated it. These diagnostics are bounded independently of
/// unrelated package bytes; they do not justify a charge at the failing site.
#[cfg(feature = "test-support")]
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn named_qspec_refusal_diagnostics_stay_within_the_separate_bound() {
    initialize_operation_catalog().expect("catalog gate");
    let named = [
        "OP-21",
        "OP-40",
        "TW-25",
        "population-as-a-relationship-declaration",
        "population-as-a-field-declaration",
        "population-as-an-operation-declaration",
        "population-as-a-reaches-population-link",
        "population-as-a-collection-operand",
    ];
    let selected = qspec_cases()
        .0
        .into_iter()
        .filter(|input| named.iter().any(|case_id| input.label.contains(case_id)))
        .collect::<Vec<_>>();
    assert_eq!(
        selected.len(),
        named.len(),
        "each named QSpec refusal is registered exactly once"
    );
    let maxima = std::thread::scope(|scope| {
        let handles = selected
            .into_iter()
            .map(|input| {
                scope.spawn(move || {
                    let label = &input.label;
                    let Expected::Refused(expected_code) = input.expected else {
                        panic!("{label}: targeted case must be a refusal");
                    };
                    let observation = proof::measure_read::<IntegrationCounter>(
                        &input.bytes,
                        &input.evidence,
                        proof::generous_limits(),
                        u64::MAX,
                    );
                    let Budgeted::Refused(refusal) = &observation.result else {
                        panic!(
                            "{label}: selected QSpec input must refuse: {:?}",
                            observation.result
                        )
                    };
                    assert_eq!(refusal.code, expected_code, "{label}: refusal code");
                    proof::accounted_refusal_diagnostic(label, &observation);
                    let boundary_maximum =
                        proof::every_request::<IntegrationCounter>(&input.bytes, &input.evidence);
                    eprintln!(
                        "Named QSpec refusal diagnostic boundaries: {label}; input_bytes={}; ample_unpaid_bytes={}; maximum_boundary_unpaid_bytes={boundary_maximum}",
                        input.bytes.len(),
                        proof::unpaid(&observation),
                    );
                    proof::drop_without_request::<IntegrationCounter>(observation);
                    (input.label, boundary_maximum)
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .expect("named refusal boundary proof succeeds")
            })
            .collect::<Vec<_>>()
    });
    let maximum_diagnostic_unpaid = maxima
        .iter()
        .map(|(_, maximum)| *maximum)
        .max()
        .expect("named QSpec refusal set is nonempty");
    eprintln!(
        "Maximum named QSpec refusal-diagnostic unpaid bytes across all boundaries: {maximum_diagnostic_unpaid}; independent bound: {}",
        proof::REFUSAL_DIAGNOSTIC_UNPAID_BOUND,
    );
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn the_actual_tc048_catalogue_and_refusal_corpus_is_accounted_within_the_bound() {
    initialize_operation_catalog().expect("catalog gate");
    // Every registered input still gets its ample, exact, repeated, and
    // whole-read one-below checks. Exhaustive per-request refusal boundaries
    // are covered by the focused QSpec diagnostic proof above.
    for input in registered_cases() {
        eprintln!("Registry full actual input: {}", input.label);
        proof::assert_read::<IntegrationCounter>(
            &input.label,
            &input.bytes,
            &input.evidence,
            input.expected,
        );
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

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn repeated_reads_preserve_the_callers_storage_delta_with_the_integration_counter() {
    let value = v2_all_families();
    proof::shared_budget_reads::<IntegrationCounter>(&canonical(&value), &evidence_for(&value));
}

/// Instrumentation returns the original structural and finite-work outcomes;
/// it never substitutes generous limits before the driver's oracle sees them.
#[trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
#[test]
fn original_read_boundary_preserves_structural_and_work_limits() {
    let wire = v2_all_families();
    let bytes = canonical(&wire);
    let evidence = evidence_for(&wire);
    let mut limits = quire_contract_model::CheckedPackageReadLimits::bounded();
    limits.bytes = 0;
    let CheckedPackageV2ReadResult::Incomplete(bytes_stop) =
        crate::read_and_account(&bytes, limits, &evidence)
    else {
        panic!("the original byte cap must remain observable");
    };
    assert_eq!(
        bytes_stop.limit_kind,
        quire_contract_model::CheckedPackageLimit::Bytes
    );
    assert_eq!(bytes_stop.limit, 0);
    assert_eq!(
        bytes_stop.consumed,
        u64::try_from(bytes.len()).expect("fixture byte count")
    );

    limits = quire_contract_model::CheckedPackageReadLimits::bounded();
    limits.work = 0;
    let CheckedPackageV2ReadResult::Incomplete(work_stop) =
        crate::read_and_account(&bytes, limits, &evidence)
    else {
        panic!("the original zero work cap must remain observable");
    };
    assert_eq!(
        work_stop.limit_kind,
        quire_contract_model::CheckedPackageLimit::Work
    );
    assert_eq!(work_stop.limit, 0);
    assert!(
        work_stop.consumed > 0,
        "the first attempted work charge is retained"
    );

    let CheckedPackageV2ReadResult::Refused(malformed) = crate::read_and_account(
        b"not json",
        quire_contract_model::CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("a malformed original byte stream must refuse");
    };
    assert_eq!(malformed.code, Code::MalformedWire);
}

/// The proof's extra reads and logging must not change an original driver's
/// enclosing allocation oracle: it still observes precisely one public read.
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn original_enclosing_allocator_observes_only_its_original_read() {
    let wire = v2_all_families();
    let bytes = canonical(&wire);
    let evidence = evidence_for(&wire);
    let limits = quire_contract_model::CheckedPackageReadLimits::bounded();
    let mut wrapped = None;
    let accounted = allocation_counter::measure(|| {
        wrapped = Some(crate::read_and_account(&bytes, limits, &evidence));
    });
    let mut original = None;
    let ordinary = allocation_counter::measure(|| {
        original = Some(CheckedPackageV2::read(&bytes, limits, &evidence));
    });
    assert_eq!(
        wrapped, original,
        "the original semantic oracle sees the same result"
    );
    assert_eq!(
        accounted.count_total, ordinary.count_total,
        "no extra caller allocations"
    );
    assert_eq!(
        accounted.bytes_total, ordinary.bytes_total,
        "no extra caller requested bytes"
    );
    assert_eq!(
        accounted.count_current, ordinary.count_current,
        "no cross-thread proof drops charged to caller"
    );
    assert_eq!(
        accounted.bytes_current, ordinary.bytes_current,
        "only original returned storage remains"
    );
    assert_eq!(
        accounted.bytes_max, ordinary.bytes_max,
        "the original peak oracle is preserved"
    );
    let wrapped_drop = allocation_counter::measure(|| {
        drop(crate::read_and_account(&bytes, limits, &evidence));
    });
    let ordinary_drop = allocation_counter::measure(|| {
        drop(CheckedPackageV2::read(&bytes, limits, &evidence));
    });
    assert_eq!(
        wrapped_drop.bytes_total, ordinary_drop.bytes_total,
        "result drop included: original requested bytes preserved"
    );
    assert_eq!(
        wrapped_drop.count_total, ordinary_drop.count_total,
        "result drop included: original allocation count preserved"
    );
    assert_eq!(
        wrapped_drop.bytes_current, ordinary_drop.bytes_current,
        "result drop included: original retained bytes preserved"
    );
    assert_eq!(
        wrapped_drop.count_current, ordinary_drop.count_current,
        "result drop included: original retained allocation count preserved"
    );
}

/// A denied charge in the abstraction relation's arrays allocates nothing of
/// its own: each array is charged at its exact length before serde sizes it.
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn an_abstraction_relation_denial_allocates_no_unpaid_array() {
    use crate::checked_package_v2_abstraction_relation as relation;
    initialize_operation_catalog().expect("catalog gate");
    let package = relation::relations(&[relation::rich()]);
    let evidence = crate::support::checked_package::evidence_with_document(
        &package,
        relation::IDENTITY,
        &relation::domain_document(),
    );
    proof::every_request::<IntegrationCounter>(&canonical(&package), &evidence);
}
