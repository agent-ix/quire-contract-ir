// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038 "Artifact references" (FR-038-AC-46 through FR-038-AC-61): a
//! definition reference is exactly `{authority, identity}` and a source
//! reference exactly `{authority, identity}`, each
//! refused as the closed shape it is. The operation catalog's own read
//! (FR-038-AC-58) and the value-role law join (FR-038-AC-56) are unit tests in
//! `crates/quire-contract-model/src/checked_package/v2/`.

use crate::support::checked_package::{
    canonical, evidence_for, pointer, refusal, refusal_cause, settle, v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

const DEFINITION_MEMBERS: [&str; 2] = ["authority", "identity"];
const SOURCE_MEMBERS: [&str; 2] = ["authority", "identity"];

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
/// `(package, pointer)`: the lock sites and diagnostics catalog.
fn definition_places() -> Vec<(Value, String)> {
    let families = v2_all_families();
    let nominal = v2_nominal();
    let mut places = Vec::new();
    for place in [
        "/lock/edition/definition",
        "/lock/profile_selections/0/definition",
        "/diagnostics/catalog",
    ] {
        places.push((families.clone(), place.to_owned()));
    }
    places.push((nominal, "/lock/definition_selections/0".to_owned()));
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

#[trace("TC-048", "FR-038-AC-47")]
#[test]
fn tc_048_source_references_of_exactly_two_members_admit() {
    for (package, place) in source_places() {
        assert_eq!(
            members(package.pointer(&place).expect("place")),
            sorted(&SOURCE_MEMBERS),
            "{place}"
        );
    }
    admitted(&with_locus(v2_nominal()));
}

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

/// The configured clock row is checked before a malformed clause law reaches
/// the later operation-law shape check.
#[trace("TC-048", "FR-038-AC-49", "FR-322-AC-294")]
#[test]
fn tc_048_a_clock_row_precedes_a_malformed_temporal_law_definition() {
    let base = v2_all_families();
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
    for (name, mutate, path) in &cases {
        let mut mutated = base.clone();
        mutate(&mut mutated);
        let actual = refused(&mutated);
        assert_eq!(
            actual.code,
            CheckedPackageRefusalCode::InvalidPackage,
            "{name}"
        );
        assert_eq!(
            actual.loci[0].path,
            Some(pointer("/temporal_bindings/0/clock/kind")),
            "{path}"
        );
        assert_eq!(
            actual.cause,
            Some(CheckedPackageRefusalCause::TimedNodeReference),
            "{name}"
        );
    }

    // With a profile that needs no clock configuration, the same malformed
    // definitions reach the original closed graph-shape refusal.
    let mut unconfigured = base;
    super::checked_package_v2_temporal::select_profile(
        &mut unconfigured,
        "quire.temporal.infinite-trace/v1",
    );
    settle(&mut unconfigured);
    admitted(&unconfigured);
    for (name, mutate, path) in cases {
        let mut mutated = unconfigured.clone();
        mutate(&mut mutated);
        expect(
            &mutated,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &path,
            name,
        );
    }
}

#[trace("TC-048", "FR-038-AC-50")]
#[test]
fn tc_048_a_source_reference_with_an_extra_member_is_an_unknown_member() {
    for (package, place) in source_places() {
        for (member, value) in [
            (
                "revision",
                json!({"namespace": "quire-draft", "value": "1"}),
            ),
            ("export", json!("Status")),
            ("digest_domain", json!("quire.source.bytes/v1")),
            ("digest", json!("1".repeat(64))),
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
            let mut empty = package.clone();
            at(&mut empty, &place)[member] = json!("");
            expect(
                &empty,
                CheckedPackageRefusalCode::MalformedWire,
                &format!("{place}/{member}"),
                &format!("{place} {member} empty"),
            );
        }
    }
}

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

#[trace("TC-048", "FR-038-AC-53")]
#[test]
fn tc_048_a_lock_source_digest_member_is_unknown_before_row_validation() {
    let base = v2_nominal();
    let row = "/lock/sources/0";
    for (member, value) in [
        ("digest_domain", json!("quire.source.bytes/v1")),
        ("digest", json!("1".repeat(64))),
    ] {
        for empty_authority in [false, true] {
            let mut mutated = base.clone();
            let source = at(&mut mutated, row);
            source[member] = value.clone();
            if empty_authority {
                source["authority"] = json!("");
            }
            expect(
                &mutated,
                CheckedPackageRefusalCode::UnknownMember,
                &format!("{row}/{member}"),
                member,
            );
        }
    }
}

#[trace("TC-048", "FR-038-AC-54")]
#[test]
fn tc_048_a_lock_source_refuses_an_empty_authority_or_identity() {
    let base = v2_nominal();
    for member in SOURCE_MEMBERS {
        let mut mutated = base.clone();
        at(&mut mutated, "/lock/sources/0")[member] = json!("");
        expect(
            &mutated,
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/lock/sources/0/{member}"),
            member,
        );
    }
}

#[trace("TC-048", "FR-038-AC-55")]
#[test]
fn tc_048_a_region_or_locus_source_equal_to_no_lock_row_is_an_invalid_source_map() {
    let edits = [
        ("authority", json!("other")),
        ("identity", json!("other")),
        // Regions are joined to checked lock rows; an empty member cannot join.
        ("authority", json!("")),
        ("identity", json!("")),
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

/// A changed law with an unchanged clock row reaches the row's kind join
/// before the later operation-law selection check.
#[trace("TC-048", "FR-038-AC-57", "FR-322-AC-294")]
#[test]
fn tc_048_a_clock_row_precedes_an_unselected_temporal_law() {
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
        // The refusal also carries the calling node as its locus.
        let actual = refused(&mutated);
        let expected = refusal_cause(
            CheckedPackageRefusalCode::InvalidPackage,
            "/temporal_bindings/0/clock/kind",
            CheckedPackageRefusalCause::TimedNodeReference,
        );
        assert_eq!(
            (actual.code, actual.loci[0].path.clone(), actual.cause),
            (expected.code, expected.path.clone(), expected.cause),
            "{member}"
        );
        assert_eq!(
            actual
                .locus
                .as_ref()
                .map(|id| serde_json::to_value(id).expect("node id")),
            Some(base["semantic_graph"]["nodes"][node]["node_id"].clone()),
            "{member} calling node locus"
        );
    }

    let mut unconfigured = base;
    super::checked_package_v2_temporal::select_profile(
        &mut unconfigured,
        "quire.temporal.infinite-trace/v1",
    );
    settle(&mut unconfigured);
    admitted(&unconfigured);
    let claim = unconfigured["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "quire.op.claim.clause")
        .expect("claim clause");
    let claim_definition =
        format!("/semantic_graph/nodes/{claim}/body/operation/laws/0/definition");
    for (member, value) in [
        (
            "identity",
            json!("quire.temporal.fixed-sample.false-extension/v1"),
        ),
        ("authority", json!("other")),
    ] {
        let mut mutated = unconfigured.clone();
        at(&mut mutated, &claim_definition)[member] = value;
        let actual = refused(&mutated);
        assert_eq!(
            (actual.code, actual.loci[0].path.clone(), actual.cause),
            (
                CheckedPackageRefusalCode::InvalidPackage,
                Some(pointer(&claim_definition)),
                Some(CheckedPackageRefusalCause::OperationLawUnselected),
            ),
            "{member}"
        );
    }
}

#[trace("TC-048", "FR-038-AC-60")]
#[test]
fn tc_048_definition_selection_is_the_single_authoritative_row() {
    let mut base = v2_nominal();
    // A definition row no node owns, so changing its identity only changes
    // the selection that the reader joins to the supplied definition.
    let mut rows = base["lock"]["definition_selections"].clone();
    rows.as_array_mut()
        .expect("rows")
        .push(json!({"authority": "agent-ix", "identity": "quire.fixture.definition.extra/v1"}));
    base["lock"]["definition_selections"] = rows;
    admitted(&base);

    let mut edited = base.clone();
    at(&mut edited, "/lock/definition_selections/1")["identity"] =
        json!("quire.fixture.definition.extra.next/v1");
    admitted(&edited);
    assert_ne!(canonical(&base), canonical(&edited));
}

#[trace("TC-048", "FR-038-AC-61")]
#[test]
fn tc_048_nominal_owner_requires_a_selected_source_and_closed_kind() {
    let base = v2_nominal();
    let index = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "enum")
        .expect("source-owned nominal declaration");
    let mut missing_source = base.clone();
    missing_source["semantic_graph"]["nodes"][index]["owner"]["identity"] =
        json!("quire.fixture.source.unselected/v1");
    let refusal = refused(&missing_source);
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{index}/node_id").as_str())
    );

    let mut definition_owner = base;
    definition_owner["semantic_graph"]["nodes"][index]["owner"] = json!({
        "kind": "definition", "authority": "agent-ix", "identity": "quire.fixture.definition/v1"
    });
    expect(
        &definition_owner,
        CheckedPackageRefusalCode::MalformedWire,
        &format!("/semantic_graph/nodes/{index}/owner/kind"),
        "retired DefinitionOwner shape",
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
    };
    let locator = CheckedArtifactLocator {
        authority: "a".into(),
        identity: "i".into(),
    };
    assert_eq!(
        members(&serde_json::to_value(&definition).expect("serializes")),
        sorted(&DEFINITION_MEMBERS)
    );
    assert_eq!(
        members(&serde_json::to_value(&source).expect("serializes")),
        sorted(&SOURCE_MEMBERS)
    );
    assert_eq!((&*locator.authority, &*locator.identity), ("a", "i"));
}
