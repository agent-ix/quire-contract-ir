// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Timed-model wire evidence read from the owning QSpec checkout.
//! Clause and aggregate-order controls exercise the public V2 reader.
use super::*;
use quire_contract_model::{
    encode_checked_package_v2, CheckedCollectionKind, CheckedMemberType,
    CheckedPackageIdentityPreimageRefV2, CheckedTerm,
};

fn published() -> Value {
    read_json(&fixtures_dir().join("positive-timed-model-nodes.json"))
        .expect("published timed model must be available")
}

fn apply_published_timed_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("published patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    match (
        patch["op"].as_str().expect("published patch operation"),
        target,
    ) {
        ("replace", Value::Object(map)) => {
            *map.get_mut(member).expect("published replacement") = patch["value"].clone();
        }
        ("replace", Value::Array(items)) => {
            let index = member.parse::<usize>().expect("published array index");
            *items.get_mut(index).expect("published replacement") = patch["value"].clone();
        }
        ("add", Value::Array(items)) if member == "-" => items.push(patch["value"].clone()),
        ("add", Value::Array(items)) => {
            let index = member.parse::<usize>().expect("published insertion index");
            items.insert(index, patch["value"].clone());
        }
        ("remove", Value::Array(items)) => {
            let index = member.parse::<usize>().expect("published removal index");
            items.remove(index);
        }
        (operation, _) => panic!("unsupported published patch {operation} at {path}"),
    }
}

/// Trace: FR-038-AC-3, FR-038-AC-359, TC-048.
#[test]
fn timed_equal_region_rational_order_charges_before_big_integer_work() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("timed-clause-node-vectors.json"))
        .expect("published clause vectors");
    let case = vectors["cases"]
        .as_array()
        .expect("published cases")
        .iter()
        .find(|case| case["id"] == "CL-47")
        .expect("equal-region rational control");
    let mut package = read_json(&fixtures.join("positive-timed-clause-nodes.json"))
        .expect("published clause fixture");
    for patch in case["patch"].as_array().expect("published patches") {
        apply_published_timed_patch(&mut package, patch);
    }
    let numerator = format!("3{}", "0".repeat(1_000));
    package["semantic_graph"]["nodes"][77]["body"]["members"][5]["value"]["value"] =
        json!(format!("{numerator}/1"));
    fixture_support::mint_ungrouped_structural_keys(&mut package);
    sort_timed_graph_dependencies(&mut package);
    fixture_support::refresh_identity(&mut package);
    let document = proposal.join("domain-package-acme-timed-clauses.json");
    let read_at = |work| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        read_package_with_document_and_extensions_at_limit(
            &package,
            "large equal-region rational",
            &document,
            true,
            limits,
        )
        .expect("selected package read")
    };
    let (mut low, mut high) = (0, 10_000_000);
    let at_high = read_at(high);
    assert!(
        matches!(at_high, CheckedPackageV2ReadResult::Admitted(_)),
        "large ratio with work {high}: {at_high:?}"
    );
    while low < high {
        let middle = low + (high - low) / 2;
        match read_at(middle) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
            other => panic!("expected admission or work exhaustion: {other:?}"),
        }
    }
    assert!(matches!(
        read_at(low),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let CheckedPackageV2ReadResult::Incomplete(incomplete) = read_at(low - 1) else {
        panic!("one below exact work budget must be incomplete")
    };
    assert_eq!(
        incomplete.limit_kind,
        quire_contract_model::CheckedPackageLimit::Work
    );
    assert_eq!(incomplete.limit, low - 1);
    assert_eq!(incomplete.consumed, low);
    assert!(
        low >= 1_000_000,
        "rational parse and compare must be charged"
    );
}

/// Trace: FR-038-AC-357, FR-038-AC-358, FR-038-AC-359, FR-038-AC-361.
/// Provenance: IR-1109; CL-25 remains unbacked on STD-558.
#[test]
fn timed_clause_published_vectors_except_pending_cl25_match_reader() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let dense = read_json(&fixtures.join("positive-timed-model-nodes.json"))
        .expect("published dense model fixture");
    let dense_result = read_package(&dense).expect("dense model reader result");
    assert!(
        matches!(dense_result, CheckedPackageV2ReadResult::Admitted(_)),
        "dense model fixture: {dense_result:?}"
    );
    let slot = read_json(&fixtures.join("positive-timed-slot-nodes.json"))
        .expect("published slot fixture");
    let slot_document = proposal.join("domain-package-acme-timed-slots.json");
    let slot_result = read_package_with_document(&slot, "dense slot fixture", &slot_document)
        .expect("slot fixture reader result");
    assert!(
        matches!(slot_result, CheckedPackageV2ReadResult::Admitted(_)),
        "slot fixture: {slot_result:?}"
    );
    let vectors = read_json(&proposal.join("timed-clause-node-vectors.json"))
        .expect("published clause vectors");
    let cases = vectors["cases"].as_array().expect("published clause cases");
    assert_eq!(cases.len(), 53);
    let fixture = read_json(&fixtures.join("positive-timed-clause-nodes.json"))
        .expect("published clause fixture");
    let document = proposal.join("domain-package-acme-timed-clauses.json");
    for (ordinal, case) in cases.iter().enumerate() {
        let id = case["id"].as_str().expect("clause case id");
        assert_eq!(id, format!("CL-{:02}", ordinal + 1));
        let mut candidate = fixture.clone();
        if let Some(patches) = case["patch"].as_array() {
            for patch in patches {
                apply_published_timed_patch(&mut candidate, patch);
            }
        }
        let actual = read_package_with_document(&candidate, id, &document)
            .expect("selected clause package read");
        if id == "CL-25" {
            // STD-558: the vector's foreign-model anchor also violates the
            // earlier state-stage frame join. Keep this exception observable
            // until QSpec repairs the vector; AC-360/362 are not claimed.
            assert_eq!(case["expected_code"], "invalid_package");
            assert_eq!(case["expected_cause"], "timed-node-reference");
            let CheckedPackageV2ReadResult::Refused(refusal) = actual else {
                panic!("STD-558 pending CL-25 state-stage conflict disappeared");
            };
            assert_eq!(
                (refusal.code, refusal.cause),
                (Code::InvalidModelBinding, Some(Cause::MalformedDeclaration))
            );
            assert_eq!(
                refusal.path.unwrap().to_string(),
                "/semantic_graph/nodes/70"
            );
            continue;
        }
        match (case["expected"].as_str(), actual) {
            (Some("admitted"), CheckedPackageV2ReadResult::Admitted(_)) => {}
            (
                Some("refused") | Some("refused-by-schema"),
                CheckedPackageV2ReadResult::Refused(refusal),
            ) => {
                if case["expected"] == "refused" {
                    assert_eq!(
                        Some(code_word(refusal.code)),
                        case["expected_code"].as_str(),
                        "{id}: {refusal:?}"
                    );
                    assert_eq!(
                        refusal.cause.map(cause_word),
                        case["expected_cause"].as_str(),
                        "{id}"
                    );
                    assert_eq!(
                        refusal.path.as_ref().map(ToString::to_string).as_deref(),
                        case["locus"].as_str(),
                        "{id}"
                    );
                } else {
                    assert_eq!(refusal.code, Code::MalformedWire, "{id}: schema refusal");
                    assert_eq!(
                        refusal.path.as_ref().map(ToString::to_string).as_deref(),
                        case["locus"].as_str(),
                        "{id}: schema refusal locus"
                    );
                }
            }
            (expected, actual) => panic!("{id}: expected {expected:?}, got {actual:?}"),
        }
    }
}

/// Trace: FR-038-AC-358, TC-048. AC-362 awaits STD-558.
#[test]
fn timed_clause_order_unlisted_join_and_model_wide_stages_are_distinct() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("timed-clause-node-vectors.json"))
        .expect("published clause vectors");
    let fixture = read_json(&fixtures.join("positive-timed-clause-nodes.json"))
        .expect("published clause fixture");
    let document = proposal.join("domain-package-acme-timed-clauses.json");
    let published_case = |id: &str| {
        let case = vectors["cases"]
            .as_array()
            .expect("clause cases")
            .iter()
            .find(|case| case["id"] == id)
            .expect("published case");
        let mut candidate = fixture.clone();
        for patch in case["patch"].as_array().expect("published patch") {
            apply_published_timed_patch(&mut candidate, patch);
        }
        candidate
    };

    let mut order_before_unlisted = published_case("CL-05");
    order_before_unlisted["semantic_graph"]["nodes"][52]["body"]["members"][5]["value"]["members"]
        .as_array_mut()
        .expect("root guards")
        .swap(0, 1);
    fixture_support::mint_ungrouped_structural_keys(&mut order_before_unlisted);
    sort_timed_graph_dependencies(&mut order_before_unlisted);
    fixture_support::refresh_identity(&mut order_before_unlisted);
    let CheckedPackageV2ReadResult::Refused(order) =
        read_package_with_document(&order_before_unlisted, "order before unlisted", &document)
            .expect("selected package read")
    else {
        panic!("order defect must refuse before an unlisted clause")
    };
    assert_eq!(
        (order.code, order.cause),
        (Code::InvalidSemanticGraph, None)
    );
    assert_eq!(
        order.path.unwrap().to_string(),
        "/semantic_graph/nodes/52/body/members/5/value/members/1"
    );

    let mut unlisted_before_join = published_case("CL-05");
    let frame = unlisted_before_join["semantic_graph"]["nodes"][71]["node_id"].clone();
    let guard = &mut unlisted_before_join["semantic_graph"]["nodes"][86];
    let old = guard["body"]["members"][0]["value"]["target"].clone();
    guard["body"]["members"][0]["value"]["target"] = frame.clone();
    replace_dependency(guard, &old, &frame);
    fixture_support::mint_ungrouped_structural_keys(&mut unlisted_before_join);
    sort_timed_graph_dependencies(&mut unlisted_before_join);
    fixture_support::refresh_identity(&mut unlisted_before_join);
    let CheckedPackageV2ReadResult::Refused(unlisted) =
        read_package_with_document(&unlisted_before_join, "unlisted before join", &document)
            .expect("selected package read")
    else {
        panic!("unlisted clause must refuse before a wrong-kind anchor")
    };
    assert_eq!(
        (unlisted.code, unlisted.cause),
        (Code::InvalidPackage, Some(Cause::TimedNodeReference))
    );
    assert_eq!(
        unlisted.path.unwrap().to_string(),
        "/semantic_graph/nodes/76/body"
    );

    let mut join_before_model_wide = published_case("CL-06");
    let frame = join_before_model_wide["semantic_graph"]["nodes"][71]["node_id"].clone();
    let guard = &mut join_before_model_wide["semantic_graph"]["nodes"][86];
    let old = guard["body"]["members"][0]["value"]["target"].clone();
    guard["body"]["members"][0]["value"]["target"] = frame.clone();
    replace_dependency(guard, &old, &frame);
    fixture_support::mint_ungrouped_structural_keys(&mut join_before_model_wide);
    sort_timed_graph_dependencies(&mut join_before_model_wide);
    fixture_support::refresh_identity(&mut join_before_model_wide);
    let CheckedPackageV2ReadResult::Refused(join) =
        read_package_with_document(&join_before_model_wide, "join before model-wide", &document)
            .expect("selected package read")
    else {
        panic!("wrong-kind anchor must refuse before unused constraint")
    };
    assert_eq!(
        (join.code, join.cause),
        (Code::InvalidPackage, Some(Cause::TimedNodeReference))
    );
    assert_eq!(
        join.path.unwrap().to_string(),
        "/semantic_graph/nodes/86/body/members/0/value/target"
    );
}

fn sort_timed_graph_dependencies(package: &mut Value) {
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
    {
        node["dependencies"]
            .as_array_mut()
            .expect("node dependencies")
            .sort_by(|left, right| {
                (left["domain"].as_str(), left["digest"].as_str())
                    .cmp(&(right["domain"].as_str(), right["digest"].as_str()))
            });
    }
}

/// Public-reader evidence over QSpec's selected package and exact AO patches.
#[trace("TC-048", "FR-038-AC-336", "FR-038-AC-342")]
#[test]
fn timed_aggregate_order_published_vectors_read_at_exact_loci() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("timed-slot-selector-vectors.json"))
        .expect("published timed slot vectors");
    let cases = vectors["aggregate_order"]["cases"]
        .as_array()
        .expect("published AO cases");
    assert_eq!(cases.len(), 10);
    let fixture = read_json(&fixtures.join("positive-timed-slot-nodes.json"))
        .expect("published timed slot package");
    let document = proposal.join("domain-package-acme-timed-slots.json");
    for case in cases {
        let id = case["id"].as_str().expect("AO id");
        let mut candidate = fixture.clone();
        if let Some(patches) = case["patch"].as_array() {
            for patch in patches {
                apply_published_timed_patch(&mut candidate, patch);
            }
        }
        let actual = read_package_with_document(&candidate, id, &document)
            .expect("published selected package read");
        match (case["expected"].as_str(), actual) {
            (Some("admitted"), CheckedPackageV2ReadResult::Admitted(_)) => {}
            (Some("refused"), CheckedPackageV2ReadResult::Refused(refusal)) => {
                assert_eq!(
                    Some(code_word(refusal.code)),
                    case["expected_code"].as_str(),
                    "{id}"
                );
                assert_eq!(refusal.cause, None, "{id}");
                assert_eq!(
                    refusal.path.as_ref().map(ToString::to_string).as_deref(),
                    case["locus"].as_str(),
                    "{id}"
                );
            }
            (expected, actual) => panic!("{id}: expected {expected:?}, got {actual:?}"),
        }
    }
    for id in ["GD-02", "GD-03"] {
        let case = vectors["guards"]
            .as_array()
            .expect("published guard cases")
            .iter()
            .find(|case| case["id"] == id)
            .expect("published guard control");
        let mut candidate = fixture.clone();
        for patch in case["patch"].as_array().expect("published guard patch") {
            apply_published_timed_patch(&mut candidate, patch);
        }
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package_with_document(&candidate, id, &document)
                .expect("selected guard control read")
        else {
            panic!("{id}: guard control must refuse");
        };
        if id == "GD-02" {
            // STD-559: its distinct same-path place mutates a resolved row
            // and therefore fails the earlier selected-declaration join.
            assert_eq!(case["expected_cause"], "duplicate-clock-place");
            assert_eq!(
                (refusal.code, refusal.cause),
                (Code::InvalidPackage, Some(Cause::MalformedClockPlace))
            );
            continue;
        }
        assert_eq!(
            Some(code_word(refusal.code)),
            case["expected_code"].as_str(),
            "{id}"
        );
        assert_eq!(
            refusal.cause.map(cause_word),
            case["expected_cause"].as_str(),
            "{id}"
        );
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            case["locus"].as_str(),
            "{id}"
        );
    }
}

/// Each published order defect remains present while an earlier reader stage
/// refuses a second, independent defect.
#[trace("TC-048", "FR-038-AC-342")]
#[test]
fn timed_aggregate_order_preserves_earlier_refusal_stages() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("timed-slot-selector-vectors.json"))
        .expect("published timed slot vectors");
    let original = read_json(&fixtures.join("positive-timed-slot-nodes.json"))
        .expect("published selected package");
    let document = proposal.join("domain-package-acme-timed-slots.json");
    let root_position = original["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model"
        })
        .expect("timed root");
    let original_root_id = original["semantic_graph"]["nodes"][root_position]["node_id"].clone();
    let root_at = format!("/semantic_graph/nodes/{root_position}");

    let cases = vectors["aggregate_order"]["cases"]
        .as_array()
        .expect("published AO cases");
    for id in ["AO-02", "AO-03", "AO-04", "AO-06", "AO-07"] {
        let case = cases
            .iter()
            .find(|case| case["id"] == id)
            .expect("published adverse AO case");
        assert_eq!(case["expected"], "refused", "{id}");
        let mut ordered_defect = original.clone();
        for patch in case["patch"].as_array().expect("AO patch") {
            apply_published_timed_patch(&mut ordered_defect, patch);
        }
        let CheckedPackageV2ReadResult::Refused(order_refusal) =
            read_package_with_document(&ordered_defect, id, &document)
                .expect("published AO package read")
        else {
            panic!("{id}: published AO patch must refuse");
        };
        assert_eq!(order_refusal.code, Code::InvalidSemanticGraph, "{id}");
        assert_eq!(order_refusal.cause, None, "{id}");
        assert_eq!(
            order_refusal
                .path
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            case["locus"].as_str(),
            "{id}"
        );

        let mut malformed = ordered_defect.clone();
        malformed["semantic_graph"]["nodes"][root_position]["occurrences"] = json!("not-an-array");
        fixture_support::refresh_identity(&mut malformed);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package_with_document(&malformed, id, &document)
                .expect("malformed AO package read")
        else {
            panic!("{id}: malformed occurrences must refuse");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::MalformedWire, None),
            "{id}"
        );
        assert_eq!(
            refusal.path.unwrap().to_string(),
            format!("{root_at}/occurrences"),
            "{id}"
        );

        let mut missing_dependency = ordered_defect.clone();
        let first_place = missing_dependency["semantic_graph"]["nodes"][root_position]["body"]
            ["members"][3]["value"]["members"][0]["target"]
            .clone();
        let dependencies = missing_dependency["semantic_graph"]["nodes"][root_position]
            ["dependencies"]
            .as_array_mut()
            .expect("root dependencies");
        let dependency_position = dependencies
            .iter()
            .position(|dependency| *dependency == first_place)
            .expect("referenced place dependency");
        dependencies.remove(dependency_position);
        fixture_support::refresh_identity(&mut missing_dependency);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package_with_document(&missing_dependency, id, &document)
                .expect("missing-dependency AO package read")
        else {
            panic!("{id}: missing dependency must refuse");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidSemanticGraph, None),
            "{id}"
        );
        assert_eq!(
            refusal.path.unwrap().to_string(),
            format!("{root_at}/dependencies"),
            "{id}"
        );

        let mut stale_key = ordered_defect;
        let root = &mut stale_key["semantic_graph"]["nodes"][root_position];
        if matches!(id, "AO-02" | "AO-03" | "AO-04") {
            let changed_root_id = root["node_id"].clone();
            root["node_id"] = original_root_id.clone();
            root["semantic_type"] = original_root_id.clone();
            let source = stale_key["source_map"]
                .as_array_mut()
                .expect("source map")
                .iter_mut()
                .find(|entry| entry["node_id"] == changed_root_id)
                .expect("changed root source");
            source["node_id"] = original_root_id.clone();
        } else {
            let last_place = root["body"]["members"][3]["value"]["members"]
                .as_array()
                .expect("root clock places")
                .last()
                .expect("last place")
                .clone();
            root["body"]["members"][3]["value"]["members"]
                .as_array_mut()
                .expect("root clock places")
                .push(last_place);
        }
        fixture_support::refresh_identity(&mut stale_key);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package_with_document(&stale_key, id, &document)
                .expect("stale-key AO package read")
        else {
            panic!("{id}: stale root key must refuse");
        };
        assert_eq!(refusal.code, Code::InvalidPackage, "{id}");
        assert_eq!(
            refusal.cause.map(cause_word),
            Some("stale-node-key"),
            "{id}"
        );
        assert_eq!(
            refusal.path.unwrap().to_string(),
            format!("{root_at}/node_id"),
            "{id}"
        );
    }
}

/// Distinct place paths remain ordered by their structural key when their
/// minimum source regions coincide.
#[trace("TC-048", "FR-038-AC-342")]
#[test]
fn timed_aggregate_equal_regions_use_place_path_tie_break() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let fixture = read_json(&fixtures.join("positive-timed-slot-nodes.json"))
        .expect("published timed slot package");
    let document = proposal.join("domain-package-acme-timed-slots.json");
    let root_position = fixture["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model"
        })
        .expect("timed root");

    let mut root_equal = fixture.clone();
    let places = &root_equal["semantic_graph"]["nodes"][root_position]["body"]["members"][3]
        ["value"]["members"];
    let first = places[0]["target"].clone();
    let second = places[1]["target"].clone();
    assert_ne!(first, second);
    let entries = root_equal["source_map"].as_array().expect("source map");
    let first_region = entries
        .iter()
        .find(|entry| entry["node_id"] == first)
        .expect("first place source")["regions"][0]
        .clone();
    let second_entry = root_equal["source_map"]
        .as_array_mut()
        .expect("source map")
        .iter_mut()
        .find(|entry| entry["node_id"] == second)
        .expect("second place source");
    second_entry["regions"][0] = first_region;
    let admitted = read_package_with_document(&root_equal, "equal region paths", &document)
        .expect("equal region package read");
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "distinct paths order a shared region: {admitted:?}"
    );
}

/// Trace: FR-038-AC-342, TC-048.
#[test]
fn timed_aggregate_repeated_reference_refuses_as_order_defect() {
    let fixtures = fixtures_dir();
    let mut package = read_json(&fixtures.join("positive-timed-model-nodes.json"))
        .expect("published dense timed package");
    let root = 52;
    let places = package["semantic_graph"]["nodes"][root]["body"]["members"][3]["value"]["members"]
        .as_array_mut()
        .expect("root places");
    let first = places[0].clone();
    places.push(first);
    fixture_support::mint_ungrouped_structural_keys(&mut package);
    sort_timed_graph_dependencies(&mut package);
    fixture_support::refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("repeated reference package read")
    else {
        panic!("repeated reference must refuse before duplicate-path validation");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidSemanticGraph, None)
    );
    assert_eq!(
        refusal.path.unwrap().to_string(),
        format!("/semantic_graph/nodes/{root}/body/members/3/value/members/1")
    );
}

#[trace("TC-048", "FR-038-AC-342")]
#[test]
fn timed_aggregate_source_identity_precedes_region_start() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let mut package = read_json(&fixtures.join("positive-timed-slot-nodes.json"))
        .expect("published timed slot package");
    let document = proposal.join("domain-package-acme-timed-slots.json");
    let root_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model"
        })
        .expect("timed root");
    let first = package["semantic_graph"]["nodes"][root_position]["body"]["members"][3]["value"]
        ["members"][0]["target"]
        .clone();
    let first_source = package["source_map"]
        .as_array_mut()
        .expect("source map")
        .iter_mut()
        .find(|entry| entry["node_id"] == first)
        .expect("first place source");
    first_source["regions"][0]["source"]["identity"] = json!("A-example");
    first_source["regions"][0]["start"] = json!(5000);
    first_source["regions"][0]["end"] = json!(5001);
    let extra_source = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| entry["node_id"] == first)
        .expect("first place source")["regions"][0]["source"]
        .clone();
    package["lock"]["sources"]
        .as_array_mut()
        .expect("locked sources")
        .push(extra_source);
    let admitted = read_package_with_document(&package, "source identity first", &document)
        .expect("source identity control read");
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "earlier identity precedes later start: {admitted:?}"
    );
    let first_source = package["source_map"]
        .as_array_mut()
        .expect("source map")
        .iter_mut()
        .find(|entry| entry["node_id"] == first)
        .expect("first place source");
    first_source["regions"][0]["source"]["identity"] = json!("z-example");
    first_source["regions"][0]["start"] = json!(0);
    first_source["regions"][0]["end"] = json!(1);
    package["lock"]["sources"][1]["identity"] = json!("z-example");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package_with_document(&package, "source identity after", &document)
            .expect("source identity refusal read")
    else {
        panic!("later source identity must refuse before numeric start");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidSemanticGraph, None)
    );
    assert_eq!(
        refusal.path.unwrap().to_string(),
        format!("/semantic_graph/nodes/{root_position}/body/members/3/value/members/1")
    );
}

/// Each remaining timed-root aggregate has an admitted two-member control
/// and an identity-current swapped reference that the public reader rejects.
#[trace("TC-048", "FR-038-AC-342")]
#[test]
fn timed_aggregate_each_root_branch_enforces_source_order() {
    let fixtures = fixtures_dir();
    let fixture = read_json(&fixtures.join("positive-timed-clause-nodes.json"))
        .expect("published dense clause package");
    let document = proposal_dir_of(&fixtures).join("domain-package-acme-timed-clauses.json");
    for binding in [4, 5, 6, 8] {
        let mut package = fixture.clone();
        let root_position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("graph nodes")
            .iter()
            .position(|node| {
                node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model"
            })
            .expect("timed root");
        let original =
            read_package_with_document(&package, "ordered root clause aggregate", &document)
                .expect("selected package read");
        assert!(
            matches!(original, CheckedPackageV2ReadResult::Admitted(_)),
            "root binding {binding}: {original:?}"
        );
        let members = package["semantic_graph"]["nodes"][root_position]["body"]["members"][binding]
            ["value"]["members"]
            .as_array_mut()
            .expect("root clause aggregate");
        assert!(
            members.len() >= 2,
            "root binding {binding} needs two distinct nodes"
        );
        assert_ne!(members[0]["target"], members[1]["target"]);
        members.swap(0, 1);
        fixture_support::mint_ungrouped_structural_keys(&mut package);
        sort_timed_graph_dependencies(&mut package);
        fixture_support::refresh_identity(&mut package);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package_with_document(&package, "reversed root clause aggregate", &document)
                .expect("selected reversed package read")
        else {
            panic!("root binding {binding}: reverse must refuse");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidSemanticGraph, None),
            "root binding {binding}"
        );
        assert_eq!(
            refusal.path.unwrap().to_string(),
            format!("/semantic_graph/nodes/{root_position}/body/members/{binding}/value/members/1"),
            "root binding {binding}"
        );
    }
}

/// Trace: FR-038-AC-342, FR-038-AC-360, TC-048.
#[test]
fn timed_clause_equal_regions_use_structural_order_before_refusing_repetition() {
    let fixtures = fixtures_dir();
    let mut package = read_json(&fixtures.join("positive-timed-clause-nodes.json"))
        .expect("published dense clause package");
    let document = proposal_dir_of(&fixtures).join("domain-package-acme-timed-clauses.json");
    let root_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| node["semantic_form"] == "timed_model")
        .expect("timed root");
    let members = &package["semantic_graph"]["nodes"][root_position]["body"]["members"][8]["value"]
        ["members"];
    let first = members[0]["target"].clone();
    let second = members[1]["target"].clone();
    assert_ne!(first, second);
    let first_region = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| entry["node_id"] == first)
        .expect("first constraint source")["regions"][0]
        .clone();
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .iter_mut()
        .find(|entry| entry["node_id"] == second)
        .expect("second constraint source")["regions"][0] = first_region;
    let admitted = read_package_with_document(&package, "equal region distinct clauses", &document)
        .expect("selected package read");
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    let members = package["semantic_graph"]["nodes"][root_position]["body"]["members"][8]["value"]
        ["members"]
        .as_array_mut()
        .expect("constraints");
    members.swap(0, 1);
    fixture_support::mint_ungrouped_structural_keys(&mut package);
    sort_timed_graph_dependencies(&mut package);
    fixture_support::refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package_with_document(&package, "reversed equal region clauses", &document)
            .expect("selected package read")
    else {
        panic!("reversed equal-region clauses must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidSemanticGraph, None)
    );
    assert_eq!(
        refusal.path.unwrap().to_string(),
        format!("/semantic_graph/nodes/{root_position}/body/members/8/value/members/1")
    );
}

/// The slot fixture selects the complete FCD document, including its named
/// sequence and map definitions, rather than an isolated node projection.
#[test]
fn timed_slot_selected_package_admits_nested_and_optional_clock_places() {
    let fixtures = fixtures_dir();
    let wire = read_json(&fixtures.join("positive-timed-slot-nodes.json"))
        .expect("published timed slot package");
    let document = proposal_dir_of(&fixtures).join("domain-package-acme-timed-slots.json");
    let result = read_package_with_document(&wire, "timed slot package", &document)
        .expect("selected timed slot evidence");
    let CheckedPackageV2ReadResult::Admitted(package) = result else {
        panic!("selected timed slot package must admit: {result:?}");
    };
    assert_eq!(
        package.graph().nodes.len(),
        wire["semantic_graph"]["nodes"].as_array().unwrap().len()
    );
    assert_eq!(
        Some(package.package_id().digest.as_ref()),
        wire["package_id"]["digest"].as_str()
    );
    let order = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "model"
                && node.semantic_form.as_ref() == "object_type"
                && node.owner.as_ref().is_some_and(|owner| {
                    matches!(owner, quire_contract_model::CheckedNodeOwner::Model { node, .. }
                        if node.as_ref() == "ix://acme/orders/Order")
                })
        })
        .expect("selected Order declaration");
    let fields = package
        .model_object_fields(&order.node_id)
        .expect("selected Order fields");
    let g = fields
        .fields()
        .iter()
        .find(|field| field.name() == "g")
        .expect("nested clock field");
    let Some(CheckedMemberType::Collection {
        kind: CheckedCollectionKind::Sequence,
        element: outer,
        bounds: None,
    }) = g.member_type()
    else {
        panic!("g must resolve as an unbounded sequence");
    };
    assert!(matches!(
        outer.as_ref(),
        CheckedMemberType::Collection {
            kind: CheckedCollectionKind::Sequence,
            element,
            bounds: None,
        } if element.as_ref() == &CheckedMemberType::Clock
    ));
    let q = fields
        .fields()
        .iter()
        .find(|field| field.name() == "q")
        .expect("optional clock sequence field");
    assert!(matches!(
        q.member_type(),
        Some(CheckedMemberType::Collection {
            kind: CheckedCollectionKind::Sequence,
            element,
            bounds: Some((0, 2)),
        }) if element.as_ref() == &CheckedMemberType::Option(Box::new(CheckedMemberType::Clock))
    ));
}

fn encode_package(package: &CheckedPackageV2) -> Vec<u8> {
    let selected = package.identity_preimage();
    let preimage = CheckedPackageIdentityPreimageRefV2 {
        version: &selected.version,
        edition: &selected.edition,
        profile_selections: &selected.profile_selections,
        definition_selections: &selected.definition_selections,
        model_selections: &selected.model_selections,
        required_features: &selected.required_features,
        dependency_selections: &selected.dependency_selections,
        identity_projection: &package.graph().nodes,
    };
    encode_checked_package_v2(
        &preimage,
        package.package_id(),
        package.lock(),
        package.graph(),
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: package.source_map(),
            refinement_step_rows: package.refinement_step_rows(),
        },
        package.capability_report(),
        package.diagnostics(),
        Limits::new(1 << 24),
    )
    .expect("typed writer encodes admitted package")
}

#[trace("TC-048", "FR-038-AC-318", "FR-038-AC-322")]
#[test]
fn timed_model_published_tick_retains_typed_operation_anchor() {
    let wire = read_json(&fixtures_dir().join("positive-tick-timed-model-nodes.json"))
        .expect("published tick model");
    let CheckedPackageV2ReadResult::Admitted(package) = read_package(&wire).expect("tick read")
    else {
        panic!("healthy published tick model must admit");
    };
    let root = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("tick nodes")
        .iter()
        .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model")
        .expect("tick root");
    for member in &root["body"]["members"].as_array().expect("root members")[3..] {
        assert_eq!(
            member["value"]["members"],
            json!([]),
            "GD-01 empty aggregate"
        );
    }
    let anchor = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "state" && node.semantic_form.as_ref() == "operation_anchor"
        })
        .expect("published tick operation anchor");
    let CheckedTerm::OperationAnchorBody(body) =
        package.term(&anchor.node_id).expect("retained anchor")
    else {
        panic!("tick anchor must retain its typed body");
    };
    assert_eq!(body.members.len(), 3);
    assert!(body.execution_point.is_none());
    let bytes = encode_package(&package);
    let encoded: Value = serde_json::from_slice(&bytes).expect("writer JSON");
    assert_eq!(
        encoded, wire,
        "typed writer preserves the published tick package"
    );
    assert!(encoded["identity_preimage"]
        .get("identity_projection")
        .is_none());
    let CheckedPackageV2ReadResult::Admitted(again) = read_package(&encoded).expect("writer read")
    else {
        panic!("typed writer's tick package must read back");
    };
    assert_eq!(again.graph(), package.graph());
    assert_eq!(again.package_id(), package.package_id());
    assert_eq!(again.term(&anchor.node_id), package.term(&anchor.node_id));
    assert_eq!(
        encode_package(&again),
        bytes,
        "tick read/write/read is byte-identical"
    );
}

#[trace("TC-048")]
#[test]
fn timed_model_published_fixture_round_trips_complete_nodes() {
    let wire = published();
    assert_eq!(
        wire["semantic_graph"]["nodes"].as_array().unwrap().len(),
        60
    );
    let CheckedPackageV2ReadResult::Admitted(package) = read_package(&wire).unwrap() else {
        panic!(
            "published timed model must admit: {:?}",
            read_package(&wire)
        );
    };
    let timed: Vec<_> = package
        .graph()
        .nodes
        .iter()
        .filter(|n| n.node_tag.as_ref() == "timed_model")
        .collect();
    assert_eq!(timed.len(), 9);
    let forms: std::collections::BTreeSet<_> =
        timed.iter().map(|n| n.semantic_form.as_ref()).collect();
    assert_eq!(forms.len(), 8);
    for node in timed {
        assert_eq!(node.node_id, node.semantic_type);
        assert_eq!(
            serde_json::to_value(package.term(&node.node_id).expect("retained complete body"))
                .unwrap(),
            serde_json::to_value(&node.body).unwrap()
        );
    }
    let bytes = encode_package(&package);
    let encoded: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(encoded, wire);
    assert!(encoded["identity_preimage"]
        .get("identity_projection")
        .is_none());
    let CheckedPackageV2ReadResult::Admitted(again) = read_package(&encoded).unwrap() else {
        panic!("writer output must read back");
    };
    assert_eq!(again.graph(), package.graph());
    assert_eq!(again.package_id(), package.package_id());
}

#[trace("TC-048")]
#[test]
fn timed_model_published_schema_vectors_precede_stale_identity() {
    let vectors = read_json(&fixtures_dir().join("../timed-model-node-vectors.json")).unwrap();
    let rows = vectors["schema_refusals"]
        .as_array()
        .expect("schema vectors");
    assert_eq!(rows.len(), 5);
    for row in rows {
        let mut wire = published();
        let index = vectors["node_indexes"][row["node"].as_str().unwrap()]
            .as_u64()
            .unwrap() as usize;
        let body = &mut wire["semantic_graph"]["nodes"][index]["body"]["members"];
        match row["mutate"].as_str().unwrap() {
            "remove-unit" => {
                body.as_array_mut().unwrap().remove(1);
            }
            "extra-binding" => {
                let extra = body[0].clone();
                body.as_array_mut().unwrap().push(extra);
            }
            "invalid-path-step-kind" => {
                body[1]["value"]["members"][0]["members"][0]["value"]["value"] = json!("slot")
            }
            "swap-model-source" => body.as_array_mut().unwrap().swap(0, 1),
            "empty-path" => body[1]["value"]["members"] = json!([]),
            other => panic!("unhandled published vector {other}"),
        }
        let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
            panic!("{} must refuse", row["id"]);
        };
        assert_eq!(
            refusal.code,
            Code::MalformedWire,
            "{}: {refusal:?}",
            row["id"]
        );
        assert!(refusal
            .path
            .unwrap()
            .to_string()
            .starts_with(&format!("/semantic_graph/nodes/{index}/body")));
    }
}

#[trace("TC-048", "QSpec-TM-09", "QSpec-TM-21", "QSpec-TM-30")]
#[test]
fn timed_model_semantic_vectors_refuse_at_the_published_cause_and_locus() {
    let vectors = read_json(&fixtures_dir().join("../timed-model-node-vectors.json")).unwrap();
    let rows = vectors["semantic_refusals"].as_array().unwrap();
    assert_eq!(
        rows.iter().filter(|row| row["id"] == "TM-30").count(),
        1,
        "published TM-30 control must be present exactly once"
    );
    for row in rows {
        let mutation = row["mutation"].as_str().unwrap();
        if matches!(
            row["id"].as_str(),
            Some("TM-23" | "TM-24" | "TM-25" | "TM-26" | "TM-27" | "TM-28" | "TM-29")
        ) {
            continue;
        }
        let fixture = match row["fixture"].as_str().unwrap_or("dense") {
            "dense" => "positive-timed-model-nodes.json",
            "tick" => "positive-tick-timed-model-nodes.json",
            other => panic!("{}: unknown fixture {other}", row["id"]),
        };
        let mut wire = read_json(&fixtures_dir().join(fixture)).unwrap();
        match mutation {
            "remove-timed-root" => {
                let root_id = wire["semantic_graph"]["nodes"][52]["node_id"].clone();
                wire["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .remove(52);
                wire["source_map"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|entry| entry["node_id"] != root_id);
            }
            "dangling-tick-operation" => {
                let target =
                    json!({"domain":"quire.checked-semantic-node/v1", "digest":"0".repeat(64)});
                let old = wire["semantic_graph"]["nodes"][44]["body"]["members"][3]["value"]
                    ["target"]
                    .clone();
                wire["semantic_graph"]["nodes"][44]["body"]["members"][3]["value"]["target"] =
                    target.clone();
                replace_dependency(&mut wire["semantic_graph"]["nodes"][44], &old, &target);
            }
            "function-as-tick-operation" => {
                let old = wire["semantic_graph"]["nodes"][44]["body"]["members"][3]["value"]
                    ["target"]
                    .clone();
                let target = wire["semantic_graph"]["nodes"][5]["node_id"].clone();
                wire["semantic_graph"]["nodes"][44]["body"]["members"][3]["value"]["target"] =
                    target.clone();
                replace_dependency(&mut wire["semantic_graph"]["nodes"][44], &old, &target);
            }
            "duplicate-clock-place" => {
                let mut second = wire["semantic_graph"]["nodes"][45].clone();
                let rows = second["body"]["members"][2]["value"]["members"]
                    .as_array_mut()
                    .expect("resolved path rows");
                let last = rows.last_mut().expect("resolved path row");
                let identity = last["members"][2]["value"]["value"]
                    .as_str()
                    .expect("resolved path identity")
                    .to_owned();
                last["members"][2]["value"]["value"] = json!(format!("{identity}#again"));
                // Keep the clone distinct until the fixture helper mints its
                // content key; otherwise it would rename both place nodes.
                let provisional = json!({
                    "domain": "quire.checked-semantic-node/v1",
                    "digest": "f".repeat(64)
                });
                assert!(wire["semantic_graph"]["nodes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|node| node["node_id"] != provisional));
                second["node_id"] = provisional.clone();
                second["semantic_type"] = provisional.clone();
                let mut source = wire["source_map"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|entry| {
                        entry["node_id"] == wire["semantic_graph"]["nodes"][45]["node_id"]
                    })
                    .expect("first place source")
                    .clone();
                source["node_id"] = provisional.clone();
                source["regions"][0]["start"] = json!(9000);
                source["regions"][0]["end"] = json!(9001);
                wire["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(second);
                let root = &mut wire["semantic_graph"]["nodes"][52];
                root["body"]["members"][3]["value"]["members"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"term": "reference", "target": provisional}));
                root["dependencies"]
                    .as_array_mut()
                    .unwrap()
                    .push(provisional);
                wire["source_map"].as_array_mut().unwrap().push(source);
            }
            "repeated-clock-place-reference" => {
                let places = wire["semantic_graph"]["nodes"][52]["body"]["members"][3]["value"]
                    ["members"]
                    .as_array_mut()
                    .expect("root places");
                let first = places.first().expect("first clock place").clone();
                places.insert(1, first);
                assert_eq!(places[0], places[1], "TM-30 repeats one node reference");
            }
            "noncanonical-place-path" => {
                wire["semantic_graph"]["nodes"][45]["body"]["members"][1]["value"]["members"][0]
                    ["members"][1]["value"]["value"] = json!("not-clock")
            }
            "zero-tick-period" => {
                wire["semantic_graph"]["nodes"][44]["body"]["members"][4]["value"]["value"] =
                    json!("0")
            }
            "missing-clock-resolution-row" => {
                let steps = wire["semantic_graph"]["nodes"][45]["body"]["members"][1]["value"]
                    ["members"]
                    .as_array_mut()
                    .unwrap();
                let duplicate = steps[0].clone();
                steps.push(duplicate);
            }
            "mismatched-clock-path-step" => {
                wire["semantic_graph"]["nodes"][45]["body"]["members"][2]["value"]["members"][0]
                    ["members"][0]["value"]["value"] = json!(1)
            }
            "wrong-clock-collection-metadata" => {
                wire["semantic_graph"]["nodes"][45]["body"]["members"][2]["value"]["members"][0]
                    ["members"][4]["value"]["value"] = json!("sequence")
            }
            "undeclared-clock-member" => {
                wire["semantic_graph"]["nodes"][45]["body"]["members"][2]["value"]["members"][0]
                    ["members"][2]["value"]["value"] = json!("ix://acme/orders/Order/unknown")
            }
            "wrong-terminal-clock-type" => {
                wire["semantic_graph"]["nodes"][45]["body"]["members"][2]["value"]["members"][0]
                    ["members"][3]["value"]["value"] = json!("ix://quire/native/Integer")
            }
            "coherent-forged-selected-chain" => {
                let place = &mut wire["semantic_graph"]["nodes"][45]["body"]["members"];
                place[1]["value"]["members"][1]["members"][1]["value"]["value"] = json!("sequence");
                let rows = &mut place[2]["value"]["members"];
                rows[0]["members"][3]["value"]["value"] = json!("Sequence<Evil>");
                rows[0]["members"][4]["value"]["value"] = json!("sequence");
                rows[1]["members"][1]["value"]["value"] = json!("Evil");
                rows[1]["members"][2]["value"]["value"] = json!("Sequence<Evil>[slot]");
            }
            "tick-model-context-mismatch" => {
                let old = wire["semantic_graph"]["nodes"][44]["body"]["members"][2]["value"]
                    ["target"]
                    .clone();
                let target = wire["semantic_graph"]["nodes"][14]["node_id"].clone();
                wire["semantic_graph"]["nodes"][44]["body"]["members"][2]["value"]["target"] =
                    target.clone();
                replace_dependency(&mut wire["semantic_graph"]["nodes"][44], &old, &target);
            }
            "unresolved-tick-operation" => {
                wire["semantic_graph"]["nodes"][46]["body"]["members"][1]["value"]["value"] =
                    json!("missing")
            }
            other => panic!("{}: unknown semantic vector {other}", row["id"]),
        }
        fixture_support::mint_ungrouped_structural_keys(&mut wire);
        for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
            node["dependencies"]
                .as_array_mut()
                .unwrap()
                .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        }
        fixture_support::refresh_identity(&mut wire);
        if mutation == "duplicate-clock-place" {
            let places =
                &wire["semantic_graph"]["nodes"][52]["body"]["members"][3]["value"]["members"];
            assert_ne!(places[0]["target"], places[1]["target"]);
        }
        let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
            panic!("{} must refuse", row["id"]);
        };
        if row["id"] == "TM-12" {
            // STD-559 owns the invalid resolved-path row used to distinguish
            // the second same-path node. Preserve the earlier refusal until
            // the published control has a genuinely valid second place.
            assert_eq!(row["expected_cause"], "duplicate-clock-place");
            assert_eq!(
                (refusal.code, refusal.cause),
                (Code::InvalidPackage, Some(Cause::MalformedClockPlace))
            );
            assert_eq!(
                refusal.path.unwrap().to_string(),
                "/semantic_graph/nodes/60/body/members/2/value/members/1"
            );
            continue;
        }
        assert_eq!(
            code_word(refusal.code),
            row["expected_code"].as_str().unwrap(),
            "{}: {refusal:?}",
            row["id"]
        );
        assert_eq!(
            refusal.cause.map(cause_word),
            row["expected_cause"].as_str(),
            "{}: {refusal:?}",
            row["id"]
        );
        assert_eq!(
            refusal.path.unwrap().to_string(),
            row["locus"].as_str().unwrap(),
            "{}",
            row["id"]
        );
    }
}

#[trace("TC-367", "FR-038-AC-323")]
#[test]
fn timed_model_tm_23_to_tm_29_enforce_dense_time_and_base_time_identity() {
    let vectors = read_json(&fixtures_dir().join("../timed-model-node-vectors.json")).unwrap();
    for row in vectors["semantic_refusals"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        if !matches!(
            id,
            "TM-23" | "TM-24" | "TM-25" | "TM-26" | "TM-27" | "TM-28" | "TM-29"
        ) {
            continue;
        }
        let fixture = match row["fixture"].as_str().unwrap_or("dense") {
            "dense" => "positive-timed-model-nodes.json",
            "tick" => "positive-tick-timed-model-nodes.json",
            other => panic!("{id}: unknown fixture {other}"),
        };
        let mut wire = read_json(&fixtures_dir().join(fixture)).unwrap();
        match row["mutation"].as_str().unwrap() {
            "tick-source-with-clock-place" => {
                let dense = published();
                let place = dense["semantic_graph"]["nodes"][45].clone();
                let place_id = place["node_id"].clone();
                wire["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(place);
                let root = &mut wire["semantic_graph"]["nodes"][45];
                root["body"]["members"][3]["value"]["members"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"term":"reference", "target":place_id}));
                root["dependencies"].as_array_mut().unwrap().push(place_id);
                root["dependencies"]
                    .as_array_mut()
                    .unwrap()
                    .sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
            }
            "source-unit-length" | "source-unit-derived-non-time" | "source-unit-derived-time" => {
                let mutation = row["mutation"].as_str().unwrap();
                let name = match mutation {
                    "source-unit-length" => "Length",
                    "source-unit-derived-non-time" => "Area",
                    _ => "Duration",
                };
                let time_id = wire["semantic_graph"]["nodes"][42]["node_id"].clone();
                let mut dimension = wire["semantic_graph"]["nodes"][42].clone();
                dimension["nominal_identity_preimage"]["qualified_declaration"] =
                    json!(["Example", name]);
                dimension["declaration"]["qualified_name"] = json!(["Example", name]);
                if name == "Area" || name == "Duration" {
                    let base_id = if name == "Area" {
                        let mut length = dimension.clone();
                        length["nominal_identity_preimage"]["qualified_declaration"] =
                            json!(["Example", "Length"]);
                        length["declaration"]["qualified_name"] = json!(["Example", "Length"]);
                        let id = fixture_support::node_id(&fixture_support::sha256_hex(
                            &fixture_support::canonical(&length["nominal_identity_preimage"]),
                        ));
                        length["node_id"] = id.clone();
                        length["semantic_type"] = id.clone();
                        wire["semantic_graph"]["nodes"]
                            .as_array_mut()
                            .unwrap()
                            .push(length);
                        id
                    } else {
                        time_id
                    };
                    dimension["nominal_identity_preimage"]["terms"] = json!([{
                        "dimension_node_id": base_id,
                        "exponent": if name == "Area" { "2" } else { "1" }
                    }]);
                    dimension["dependencies"] = json!([base_id]);
                }
                let dimension_id = fixture_support::node_id(&fixture_support::sha256_hex(
                    &fixture_support::canonical(&dimension["nominal_identity_preimage"]),
                ));
                dimension["node_id"] = dimension_id.clone();
                dimension["semantic_type"] = dimension_id.clone();
                wire["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(dimension);
                let mut unit = wire["semantic_graph"]["nodes"][43].clone();
                unit["nominal_identity_preimage"]["qualified_declaration"] =
                    json!(["Example", "u"]);
                unit["nominal_identity_preimage"]["dimension_node_id"] = dimension_id.clone();
                unit["node_id"] = fixture_support::node_id(&fixture_support::sha256_hex(
                    &fixture_support::canonical(&unit["nominal_identity_preimage"]),
                ));
                unit["semantic_type"] = dimension_id;
                unit["dependencies"] = json!([unit["semantic_type"].clone()]);
                unit["declaration"]["qualified_name"] = json!(["Example", "u"]);
                let unit_id = unit["node_id"].clone();
                wire["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(unit);
                let source = &mut wire["semantic_graph"]["nodes"][44];
                let old = source["body"]["members"][1]["value"]["target"].clone();
                source["body"]["members"][1]["value"]["target"] = unit_id.clone();
                let deps = source["dependencies"].as_array_mut().unwrap();
                deps.retain(|dep| dep != &old);
                deps.push(unit_id);
                deps.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
            }
            other => panic!("{id}: unknown mutation {other}"),
        }
        fixture_support::mint_ungrouped_structural_keys(&mut wire);
        for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
            node["dependencies"]
                .as_array_mut()
                .unwrap()
                .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        }
        fixture_support::refresh_identity(&mut wire);
        let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
            panic!("{id} must refuse");
        };
        assert_eq!(
            code_word(refusal.code),
            row["expected_code"].as_str().unwrap(),
            "{id}: {refusal:?}"
        );
        assert_eq!(
            refusal.cause.map(cause_word),
            row["expected_cause"].as_str(),
            "{id}: {refusal:?}"
        );
        assert_eq!(
            refusal.path.unwrap().to_string(),
            row["locus"].as_str().unwrap(),
            "{id}"
        );
    }
}

fn replace_dependency(node: &mut Value, old: &Value, new: &Value) {
    for dependency in node["dependencies"].as_array_mut().expect("dependencies") {
        if dependency == old {
            *dependency = new.clone();
        }
    }
    node["dependencies"]
        .as_array_mut()
        .unwrap()
        .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
}

#[trace("TC-048")]
#[test]
fn timed_model_missing_root_precedes_later_quantity_validation() {
    let mut wire = published();
    let root = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model")
        .expect("published root")["node_id"]
        .clone();
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .retain(|node| node["node_id"] != root);
    wire["source_map"]
        .as_array_mut()
        .unwrap()
        .retain(|entry| entry["node_id"] != root);
    fixture_support::refresh_identity(&mut wire);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
        panic!("a timed clause without a root must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingTimedModel), "{refusal:?}");
    assert!(
        refusal.locus.is_some(),
        "missing root must retain affected node"
    );
}

#[trace("TC-048")]
#[test]
fn timed_model_repeated_reference_precedes_later_quantity_validation() {
    let mut wire = published();
    let root = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model")
        .expect("published root");
    let places = root["body"]["members"][3]["value"]["members"]
        .as_array_mut()
        .unwrap();
    assert_eq!(places.len(), 1);
    places.push(places[0].clone());
    fixture_support::mint_ungrouped_structural_keys(&mut wire);
    fixture_support::refresh_identity(&mut wire);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
        panic!("repeated model place must refuse");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidSemanticGraph, None),
        "{refusal:?}"
    );
    assert!(refusal
        .path
        .unwrap()
        .to_string()
        .ends_with("/body/members/3/value/members/1"));
}

#[trace("TC-048")]
#[test]
fn timed_model_rebuilt_semantic_defects_keep_paired_causes() {
    for (change, form, member, code, cause) in [
        (
            "zero_period",
            "time_source",
            4,
            Code::IllTyped,
            Cause::InvalidTimeQuantity,
        ),
        (
            "unreduced_bound",
            "clock_constraint",
            5,
            Code::IllTyped,
            Cause::InvalidTimeQuantity,
        ),
        (
            "wrong_predicate",
            "clock_guard",
            1,
            Code::InvalidPackage,
            Cause::TimedNodeReference,
        ),
    ] {
        let mut wire = if change == "zero_period" {
            read_json(&fixtures_dir().join("positive-tick-timed-model-nodes.json")).unwrap()
        } else {
            published()
        };
        let scalar = wire["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
            .expect("published integer type")["node_id"]
            .clone();
        let node = wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == form)
            .expect("published timed form");
        match change {
            "zero_period" => node["body"]["members"][member]["value"]["value"] = json!("0/1"),
            "unreduced_bound" => node["body"]["members"][member]["value"]["value"] = json!("6/4"),
            "wrong_predicate" => {
                let predicate = &mut node["body"]["members"][member]["value"]["members"][0]
                    ["members"][0]["value"]["target"];
                let old = predicate.clone();
                *predicate = scalar.clone();
                replace_dependency(node, &old, &scalar);
            }
            _ => unreachable!("test case list is closed"),
        }
        fixture_support::mint_ungrouped_structural_keys(&mut wire);
        for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
            node["dependencies"]
                .as_array_mut()
                .unwrap()
                .sort_by(|left, right| {
                    (left["domain"].as_str(), left["digest"].as_str())
                        .cmp(&(right["domain"].as_str(), right["digest"].as_str()))
                });
        }
        fixture_support::refresh_identity(&mut wire);
        let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
            panic!("{change} must refuse");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (code, Some(cause)),
            "{change}: {refusal:?}"
        );
        assert!(
            refusal
                .path
                .unwrap()
                .to_string()
                .contains(&format!("/body/members/{member}/value")),
            "{change}"
        );
    }
}

/// A root that stops listing an existing guard still leaves a real guard node.
/// STD-540 owns GD-03's exact locus; this control checks its published code
/// and cause after authenticated package identities are rebuilt.
/// Provenance: IR-1083, QSpec STD-526 GD-03.
#[trace("TC-048")]
#[test]
fn timed_model_unlisted_guard_refuses_at_the_timed_join() {
    let vectors = read_json(&fixtures_dir().join("../timed-slot-selector-vectors.json"))
        .expect("published timed slot vectors");
    let gd03 = vectors["guards"]
        .as_array()
        .expect("published guard cases")
        .iter()
        .find(|row| row["id"] == "GD-03")
        .expect("GD-03 row");
    assert_eq!(gd03["expected_code"], "invalid_package");
    assert_eq!(gd03["expected_cause"], "timed-node-reference");
    let mut wire = published();
    let root = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model")
        .expect("timed root");
    let guard = root["body"]["members"][5]["value"]["members"]
        .as_array_mut()
        .expect("root guards")
        .pop()
        .expect("published guard")["target"]
        .clone();
    root["dependencies"]
        .as_array_mut()
        .expect("root dependencies")
        .retain(|dependency| *dependency != guard);
    fixture_support::mint_ungrouped_structural_keys(&mut wire);
    fixture_support::refresh_identity(&mut wire);
    let result = read_package(&wire).expect("read unlisted guard package");
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("GD-03 must refuse, got {result:?}");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::TimedNodeReference)),
        "{refusal:?}"
    );
}

/// Trace: FR-038-AC-336, TC-048.
#[test]
fn timed_root_without_guards_keeps_the_place_template() {
    let mut wire = published();
    let original = read_package(&wire).expect("selected dense package read");
    assert!(matches!(original, CheckedPackageV2ReadResult::Admitted(_)));
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes");
    let place = nodes
        .iter()
        .find(|node| node["semantic_form"] == "clock_place")
        .expect("clock place")["node_id"]
        .clone();
    let guard = nodes
        .iter()
        .find(|node| node["semantic_form"] == "clock_guard")
        .expect("clock guard")["node_id"]
        .clone();
    let root = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .iter_mut()
        .find(|node| node["semantic_form"] == "timed_model")
        .expect("timed root");
    let guards = root["body"]["members"][5]["value"]["members"]
        .as_array_mut()
        .expect("root guards");
    assert_eq!(guards.len(), 1);
    guards.clear();
    root["dependencies"]
        .as_array_mut()
        .expect("root dependencies")
        .retain(|dependency| dependency != &guard);
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .retain(|node| node["node_id"] != guard);
    wire["source_map"]
        .as_array_mut()
        .expect("source map")
        .retain(|entry| entry["node_id"] != guard);
    fixture_support::mint_ungrouped_structural_keys(&mut wire);
    sort_timed_graph_dependencies(&mut wire);
    fixture_support::refresh_identity(&mut wire);
    let retained_place = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .find(|node| node["semantic_form"] == "clock_place")
        .expect("clock place")["node_id"]
        .clone();
    assert_eq!(
        retained_place, place,
        "removing a clause cannot rekey the place template"
    );
    let result = read_package(&wire).expect("guard-free selected package read");
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
}
