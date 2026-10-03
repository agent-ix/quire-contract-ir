// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038 "Artifact references" (FR-038-AC-46 through FR-038-AC-61): a
//! definition reference is exactly `{authority, identity}` and a source
//! reference exactly `{authority, identity, digest_domain, digest}`, each
//! refused as the closed shape it is. The operation catalog's own read
//! (FR-038-AC-58) and the value-role law join (FR-038-AC-56) are unit tests in
//! `crates/quire-contract-model/src/checked_package/v2/`.

use crate::support::checked_package::{
    canonical, evidence_for, refresh_identity, refusal, refusal_cause, rekey_application_node,
    v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

const DEFINITION_MEMBERS: [&str; 2] = ["authority", "identity"];
const SOURCE_MEMBERS: [&str; 4] = ["authority", "identity", "digest_domain", "digest"];

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn admitted(package: &Value) -> Box<CheckedPackageV2> {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(admitted) => admitted,
        other => panic!("expected admission, read {other:?}"),
    }
}

fn refused(package: &Value) -> CheckedPackageRefusal {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

/// The whole refusal `package` draws is `code` at `path`, with no cause.
fn expect(package: &Value, code: CheckedPackageRefusalCode, path: &str, name: &str) {
    assert_eq!(refused(package), refusal(code, path), "{name}");
}

fn at<'a>(package: &'a mut Value, path: &str) -> &'a mut Value {
    package
        .pointer_mut(path)
        .unwrap_or_else(|| panic!("no value at {path}"))
}

/// Adds a diagnostic whose one locus names the lock's first source.
fn with_locus(mut package: Value) -> Value {
    let source = package["lock"]["sources"][0].clone();
    package["diagnostics"]["entries"] = json!([{
        "stage": "lowering", "code": "unimplemented_capability",
        "cause_tag": "unsupported-feature", "details": [],
        "loci": [{"source": source, "start": 0, "end": 1}],
    }]);
    package
}

/// The position of the `temporal` application node, whose one law is the
/// lock's `temporal_profile` selection.
fn temporal_node(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "temporal" && node["body"]["term"] == "application")
        .expect("the fixture's temporal clause node")
}

fn law_definition(package: &Value) -> String {
    format!(
        "/semantic_graph/nodes/{}/body/operation/laws/0/definition",
        temporal_node(package)
    )
}

/// Every definition reference of the fixtures, outside an operation law, as
/// `(package, pointer)`: the lock sites with their `identity_preimage` mirrors
/// and the diagnostics catalog.
fn definition_places() -> Vec<(Value, String)> {
    let families = v2_all_families();
    let nominal = v2_nominal();
    let mut places = Vec::new();
    for place in [
        "/lock/edition/definition",
        "/lock/profile_selections/0/definition",
        "/identity_preimage/edition/definition",
        "/identity_preimage/profile_selections/0/definition",
        "/diagnostics/catalog",
    ] {
        places.push((families.clone(), place.to_owned()));
    }
    for place in [
        "/lock/definition_selections/0",
        "/identity_preimage/definition_selections/0",
    ] {
        places.push((nominal.clone(), place.to_owned()));
    }
    places
}

/// Every source reference of the fixtures as `(package, pointer)`: a lock row,
/// a `source_map` region `source` and a diagnostics locus `source`.
fn source_places() -> Vec<(Value, String)> {
    let package = with_locus(v2_nominal());
    [
        "/lock/sources/0",
        "/source_map/0/regions/0/source",
        "/diagnostics/entries/0/loci/0/source",
    ]
    .into_iter()
    .map(|place| (package.clone(), place.to_owned()))
    .collect()
}

fn members(value: &Value) -> Vec<&str> {
    let mut members: Vec<&str> = value
        .as_object()
        .expect("a reference is an object")
        .keys()
        .map(String::as_str)
        .collect();
    members.sort_unstable();
    members
}

fn sorted(members: &[&'static str]) -> Vec<&'static str> {
    let mut members = members.to_vec();
    members.sort_unstable();
    members
}

/// Tracing: TC-048, FR-038-AC-46
#[trace("TC-048", "FR-038-AC-46")]
#[test]
fn tc_048_definition_references_of_exactly_authority_and_identity_admit() {
    let families = v2_all_families();
    let mut places = definition_places();
    places.push((families.clone(), law_definition(&families)));
    for (package, place) in &places {
        assert_eq!(
            members(package.pointer(place).expect("place")),
            sorted(&DEFINITION_MEMBERS),
            "{place}"
        );
    }
    admitted(&families);
    admitted(&v2_nominal());
}

/// Tracing: TC-048, FR-038-AC-47
#[trace("TC-048", "FR-038-AC-47")]
#[test]
fn tc_048_source_references_of_exactly_four_members_admit() {
    for (package, place) in source_places() {
        assert_eq!(
            members(package.pointer(&place).expect("place")),
            sorted(&SOURCE_MEMBERS),
            "{place}"
        );
    }
    admitted(&with_locus(v2_nominal()));
}

/// Tracing: TC-048, FR-038-AC-48
#[trace("TC-048", "FR-038-AC-48")]
#[test]
fn tc_048_a_definition_reference_with_a_digest_member_is_an_unknown_member() {
    let extras = [
        (
            "revision",
            json!({"namespace": "quire-draft", "value": "1"}),
        ),
        ("digest_domain", json!("quire.definition.bytes/v1")),
        ("digest", json!("1".repeat(64))),
        ("export", json!("Status")),
    ];
    for (package, place) in definition_places() {
        for (member, value) in &extras {
            let mut mutated = package.clone();
            at(&mut mutated, &place)[member] = value.clone();
            expect(
                &mutated,
                CheckedPackageRefusalCode::UnknownMember,
                &format!("{place}/{member}"),
                &format!("{place} {member}"),
            );
        }
        // The earlier five-member shape refuses at its first extra member in
        // document order and is never read.
        let mut earlier = package.clone();
        let reference = at(&mut earlier, &place);
        let identity = reference["identity"].clone();
        let authority = reference["authority"].clone();
        *reference = json!({
            "authority": authority, "identity": identity,
            "revision": {"namespace": "quire-draft", "value": "1"},
            "digest_domain": "quire.definition.bytes/v1", "digest": "1".repeat(64),
        });
        expect(
            &earlier,
            CheckedPackageRefusalCode::UnknownMember,
            &format!("{place}/digest"),
            &format!("{place} earlier shape"),
        );
    }
}

/// Tracing: TC-048, FR-038-AC-49
#[trace("TC-048", "FR-038-AC-49")]
#[test]
fn tc_048_an_operation_law_definition_refuses_as_a_closed_graph_shape() {
    let base = v2_all_families();
    let node = temporal_node(&base);
    let definition = law_definition(&base);
    type Mutation = Box<dyn Fn(&mut Value)>;
    let cases: Vec<(&str, Mutation, String)> = vec![
        (
            "an extra member",
            Box::new({
                let definition = definition.clone();
                move |package| at(package, &definition)["revision"] = json!("1")
            }),
            format!("{definition}/revision"),
        ),
        (
            "an absent authority",
            Box::new({
                let definition = definition.clone();
                move |package| {
                    at(package, &definition)
                        .as_object_mut()
                        .expect("object")
                        .remove("authority");
                }
            }),
            definition.clone(),
        ),
        (
            "a non-string identity",
            Box::new({
                let definition = definition.clone();
                move |package| at(package, &definition)["identity"] = json!(7)
            }),
            format!("{definition}/identity"),
        ),
    ];
    for (name, mutate, path) in cases {
        let mut mutated = base.clone();
        mutate(&mut mutated);
        rekey_application_node(&mut mutated, node);
        refresh_identity(&mut mutated);
        expect(
            &mutated,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &path,
            name,
        );
    }
}

/// Tracing: TC-048, FR-038-AC-50
#[trace("TC-048", "FR-038-AC-50")]
#[test]
fn tc_048_a_source_reference_with_a_revision_or_export_is_an_unknown_member() {
    for (package, place) in source_places() {
        for (member, value) in [
            (
                "revision",
                json!({"namespace": "quire-draft", "value": "1"}),
            ),
            ("export", json!("Status")),
        ] {
            let mut mutated = package.clone();
            at(&mut mutated, &place)[member] = value;
            expect(
                &mutated,
                CheckedPackageRefusalCode::UnknownMember,
                &format!("{place}/{member}"),
                &format!("{place} {member}"),
            );
        }
    }
}

/// Tracing: TC-048, FR-038-AC-51
#[trace("TC-048", "FR-038-AC-51")]
#[test]
fn tc_048_a_definition_reference_member_that_is_absent_empty_or_not_a_string_is_malformed() {
    for (package, place) in definition_places() {
        for member in DEFINITION_MEMBERS {
            let mut absent = package.clone();
            at(&mut absent, &place)
                .as_object_mut()
                .expect("object")
                .remove(member);
            expect(
                &absent,
                CheckedPackageRefusalCode::MalformedWire,
                &place,
                &format!("{place} {member} absent"),
            );
            let mut wrong = package.clone();
            at(&mut wrong, &place)[member] = json!(7);
            expect(
                &wrong,
                CheckedPackageRefusalCode::MalformedWire,
                &format!("{place}/{member}"),
                &format!("{place} {member} not a string"),
            );
            // An empty member refuses at a lock or catalog site; the preimage
            // mirror is not a site of its own, it mirrors the lock.
            if !place.starts_with("/identity_preimage") {
                let mut empty = package.clone();
                at(&mut empty, &place)[member] = json!("");
                refresh_identity(&mut empty);
                expect(
                    &empty,
                    CheckedPackageRefusalCode::MalformedWire,
                    &format!("{place}/{member}"),
                    &format!("{place} {member} empty"),
                );
            }
        }
    }
}

/// Tracing: TC-048, FR-038-AC-52
#[trace("TC-048", "FR-038-AC-52")]
#[test]
fn tc_048_a_source_reference_member_that_is_absent_or_not_a_string_is_malformed() {
    for (package, place) in source_places() {
        for member in SOURCE_MEMBERS {
            let mut absent = package.clone();
            at(&mut absent, &place)
                .as_object_mut()
                .expect("object")
                .remove(member);
            expect(
                &absent,
                CheckedPackageRefusalCode::MalformedWire,
                &place,
                &format!("{place} {member} absent"),
            );
            let mut wrong = package.clone();
            at(&mut wrong, &place)[member] = json!(7);
            expect(
                &wrong,
                CheckedPackageRefusalCode::MalformedWire,
                &format!("{place}/{member}"),
                &format!("{place} {member} not a string"),
            );
        }
    }
}

/// Tracing: TC-048, FR-038-AC-53
#[trace("TC-048", "FR-038-AC-53")]
#[test]
fn tc_048_a_lock_source_of_another_domain_refuses_ahead_of_its_other_members() {
    let base = v2_nominal();
    let row = "/lock/sources/0";
    let domain = format!("{row}/digest_domain");
    for (name, edit) in [
        ("alone", json!({})),
        ("beside an empty authority", json!({"authority": ""})),
        ("beside an empty identity", json!({"identity": ""})),
        ("beside a non-hex digest", json!({"digest": "x".repeat(64)})),
        ("beside a short digest", json!({"digest": "1"})),
    ] {
        let mut mutated = base.clone();
        let source = at(&mut mutated, row);
        source["digest_domain"] = json!("quire.definition.bytes/v1");
        for (member, value) in edit.as_object().expect("object") {
            source[member] = value.clone();
        }
        expect(
            &mutated,
            CheckedPackageRefusalCode::DigestDomainMismatch,
            &domain,
            name,
        );
    }
}

/// Tracing: TC-048, FR-038-AC-54
#[trace("TC-048", "FR-038-AC-54")]
#[test]
fn tc_048_a_lock_source_of_the_right_domain_refuses_an_empty_member_or_a_bad_digest() {
    let base = v2_nominal();
    for (member, value) in [
        ("authority", json!("")),
        ("identity", json!("")),
        ("digest", json!("A".repeat(64))),
        ("digest", json!("1".repeat(63))),
        ("digest", json!("1".repeat(65))),
        ("digest", json!("g".repeat(64))),
        ("digest", json!("")),
    ] {
        let mut mutated = base.clone();
        at(&mut mutated, "/lock/sources/0")[member] = value.clone();
        expect(
            &mutated,
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/lock/sources/0/{member}"),
            &format!("{member} {value}"),
        );
    }
}

/// Tracing: TC-048, FR-038-AC-55
#[trace("TC-048", "FR-038-AC-55")]
#[test]
fn tc_048_a_region_or_locus_source_equal_to_no_lock_row_is_an_invalid_source_map() {
    let edits = [
        ("authority", json!("other")),
        ("identity", json!("other")),
        ("digest_domain", json!("quire.definition.bytes/v1")),
        ("digest", json!("2".repeat(64))),
        // Malformed members differ from every lock row too: a region is not
        // checked as a row, it is compared to the rows.
        ("authority", json!("")),
        ("digest", json!("x".repeat(64))),
        ("digest_domain", json!("")),
    ];
    for (package, place) in source_places().into_iter().skip(1) {
        for (member, value) in &edits {
            let mut mutated = package.clone();
            at(&mut mutated, &place)[member] = value.clone();
            expect(
                &mutated,
                CheckedPackageRefusalCode::InvalidSourceMap,
                &place,
                &format!("{place} {member} {value}"),
            );
        }
    }
}

/// Tracing: TC-048, FR-038-AC-57
#[trace("TC-048", "FR-038-AC-57")]
#[test]
fn tc_048_a_profile_role_law_must_be_the_pair_of_the_locks_profile_selection() {
    let base = v2_all_families();
    let node = temporal_node(&base);
    let definition = law_definition(&base);
    assert_eq!(
        base["lock"]["profile_selections"]
            .as_array()
            .expect("selections")
            .iter()
            .filter(|selection| selection["definition"]
                == base.pointer(&definition).cloned().unwrap_or(Value::Null))
            .count(),
        1,
        "the law is the lock's own temporal_profile pair"
    );
    admitted(&base);
    // A known FR-250 profile the lock does not select, and a law whose
    // authority differs from the lock row's: neither is `unknown_profile`
    // (FR-038-AC-108 refuses an identity outside the five), both are
    // unselected.
    for (member, value) in [
        (
            "identity",
            json!("quire.temporal.fixed-sample.false-extension/v1"),
        ),
        ("authority", json!("other")),
    ] {
        let mut mutated = base.clone();
        at(&mut mutated, &definition)[member] = value;
        rekey_application_node(&mut mutated, node);
        refresh_identity(&mut mutated);
        // The refusal also carries the calling node as its locus.
        let actual = refused(&mutated);
        let expected = refusal_cause(
            CheckedPackageRefusalCode::InvalidPackage,
            &definition,
            CheckedPackageRefusalCause::OperationLawUnselected,
        );
        assert_eq!(
            (actual.code, actual.path, actual.cause),
            (expected.code, expected.path, expected.cause),
            "{member}"
        );
    }
}

/// Tracing: TC-048, FR-038-AC-59
#[trace("TC-048", "FR-038-AC-59")]
#[test]
fn tc_048_a_definition_row_that_differs_from_its_preimage_mirror_is_stale() {
    let base = v2_nominal();
    for member in DEFINITION_MEMBERS {
        let mut mutated = base.clone();
        at(&mut mutated, "/lock/definition_selections/0")[member] = json!("changed");
        expect(
            &mutated,
            CheckedPackageRefusalCode::StaleDependency,
            &format!("/lock/definition_selections/0/{member}"),
            member,
        );
    }
}

/// Tracing: TC-048, FR-038-AC-60
#[trace("TC-048", "FR-038-AC-60")]
#[test]
fn tc_048_a_rederived_definition_row_edit_admits_and_the_old_package_id_is_stale() {
    let mut base = v2_nominal();
    // A definition row no node owns, so editing its identity breaks no join.
    let mut rows = base["lock"]["definition_selections"].clone();
    rows.as_array_mut()
        .expect("rows")
        .push(json!({"authority": "agent-ix", "identity": "quire.fixture.definition.extra/v1"}));
    base["lock"]["definition_selections"] = rows;
    refresh_identity(&mut base);
    let recorded = admitted(&base).package_id().clone();

    let mut edited = base.clone();
    at(&mut edited, "/lock/definition_selections/1")["identity"] =
        json!("quire.fixture.definition.extra.next/v1");
    refresh_identity(&mut edited);
    let next = admitted(&edited).package_id().clone();
    assert_ne!(next, recorded);

    // The same edit mirrored into the preimage but kept under the old id.
    let mut stale = edited.clone();
    stale["package_id"]["digest"] = json!(recorded.digest);
    expect(
        &stale,
        CheckedPackageRefusalCode::StaleDependency,
        "/package_id/digest",
        "the old package id",
    );
}

/// Tracing: TC-048, FR-038-AC-61
#[trace("TC-048", "FR-038-AC-61")]
#[test]
fn tc_048_a_nominal_owner_naming_no_lock_row_is_an_invalid_semantic_graph() {
    let base = v2_nominal();
    let owner_of = |package: &Value, kind: &str| -> String {
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["nominal_identity_preimage"]["owner"]["kind"] == kind)
            .map(|index| format!("/semantic_graph/nodes/{index}/nominal_identity_preimage/owner"))
            .unwrap_or_else(|| panic!("the fixture has a {kind}-owned node"))
    };

    // A definition owner whose pair is no `definition_selections` row.
    let mut definition = base.clone();
    at(&mut definition, "/lock/definition_selections/0")["identity"] = json!("other-definition");
    refresh_identity(&mut definition);
    expect(
        &definition,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        &owner_of(&base, "definition"),
        "definition owner",
    );

    // A source owner whose pair is no `sources` row: the row and every region
    // naming it move together, so only the owner join is left to fail.
    let mut source = base.clone();
    let old = base["lock"]["sources"][0].clone();
    let mut new = old.clone();
    new["identity"] = json!("quire.fixture.source.other/v1");
    replace_everywhere(&mut source, &old, &new);
    expect(
        &source,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        &owner_of(&base, "source"),
        "source owner",
    );
}

/// The member sets of the artifact references (FR-019-AC-6, TC-018's
/// member-set probes). A struct literal builds only when it names every member
/// and no other, so these literals fail to compile if a member is added or
/// removed; the further-member and `CheckedRevision` probes, which must fail
/// to compile, are `compile_fail` doctests on the model crate root
/// (`crates/quire-contract-model/src/lib.rs`), the form its fault-injection
/// probes use.
///
/// Tracing: TC-018, FR-019-AC-6
///
/// Traced to TC-018, not TC-058: TC-058 verifies the planned FR-019-AC-5
/// clauses, and tracing it here would make the strict gate count AC-5 backed.
#[trace("TC-018", "FR-019-AC-6")]
#[test]
fn tc_018_the_artifact_reference_member_sets_are_exact() {
    use quire_contract_model::{CheckedArtifactLocator, CheckedArtifactRef, CheckedSourceRef};
    let definition = CheckedArtifactRef {
        authority: "a".into(),
        identity: "i".into(),
    };
    let source = CheckedSourceRef {
        authority: "a".into(),
        identity: "i".into(),
        digest_domain: "d".into(),
        digest: "x".into(),
    };
    let locator = CheckedArtifactLocator {
        authority: "a".into(),
        identity: "i".into(),
        domain: "d".into(),
    };
    assert_eq!(
        members(&serde_json::to_value(&definition).expect("serializes")),
        sorted(&DEFINITION_MEMBERS)
    );
    assert_eq!(
        members(&serde_json::to_value(&source).expect("serializes")),
        sorted(&SOURCE_MEMBERS)
    );
    assert_eq!(
        (&*locator.authority, &*locator.identity, &*locator.domain),
        ("a", "i", "d")
    );
}

/// Replaces every value equal to `from` inside `value` with `to`.
fn replace_everywhere(value: &mut Value, from: &Value, to: &Value) {
    if value == from {
        *value = to.clone();
        return;
    }
    match value {
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| replace_everywhere(item, from, to)),
        Value::Object(members) => members
            .values_mut()
            .for_each(|member| replace_everywhere(member, from, to)),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}
