// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FSpec FR-831 / TC-2113: the published `protocol-correlation-vectors.json`
//! through the production strict reader and the typed correlation read.
//!
//! The packages are FSpec's: the base fixture plus the vectors' published RFC
//! 6902 edits, read from the `filament-specification` checkout the harness
//! names. Nothing of them is copied into this repository. Each case's `expect`
//! is compared with what the reader and the typed read return, never with a
//! value derived from the reader's own output.

use super::{
    cause_word, code_word, fixtures_dir, fspec_inputs, proposal_dir_of, read_json_wire,
    DOMAIN_DOCUMENT,
};
use filament_contract_model::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageV2, CheckedPackageV2BudgetedReadResult, CheckedProtocolRefusalDetail,
    NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
    PackageReadResource, PackageReadStage, ProtocolCorrelationProvenance,
    ProtocolCorrelationReadError, UnavailableCorrelationProvenance,
};
use ix_trace_rs::trace;
use serde_json::{json, Value};
use std::sync::Arc;

const MEMBER: &str = "protocol_correlation_occurrences";

fn vectors() -> Value {
    read_json_wire(&proposal_dir_of(&fixtures_dir()).join("protocol-correlation-vectors.json"))
        .expect("FSpec protocol correlation vectors")
}

/// Applies published RFC 6902 `add`, `replace` and `remove` operations.
fn apply(package: &mut Value, ops: &[Value]) {
    for op in ops {
        let path = op["path"].as_str().expect("path");
        let (parent, last) = path.rsplit_once('/').expect("absolute pointer");
        let target = package
            .pointer_mut(parent)
            .unwrap_or_else(|| panic!("{path}: parent exists"));
        match (op["op"].as_str().expect("op"), target) {
            ("add" | "replace", Value::Object(map)) => {
                map.insert(last.to_owned(), op["value"].clone());
            }
            ("add", Value::Array(items)) => {
                let at = if last == "-" {
                    items.len()
                } else {
                    last.parse().expect("index")
                };
                items.insert(at, op["value"].clone());
            }
            ("replace", Value::Array(items)) => {
                items[last.parse::<usize>().expect("index")] = op["value"].clone();
            }
            ("remove", Value::Object(map)) => {
                map.remove(last).expect("member exists");
            }
            ("remove", Value::Array(items)) => {
                items.remove(last.parse::<usize>().expect("index"));
            }
            (other, _) => panic!("{path}: unsupported {other}"),
        }
    }
}

/// The complete package a vectors package names: the published base (wire form,
/// operand rows included) plus its edits, or the complete fixture file.
fn package(vectors: &Value, name: &str) -> Value {
    let entry = &vectors["packages"][name];
    let fixture = entry["fixture"].as_str().expect("fixture");
    let file = fixture.strip_prefix("fixtures/").expect("fixtures/ path");
    let mut package = read_json_wire(&fixtures_dir().join(file)).expect("published fixture");
    apply(&mut package, entry["ops"].as_array().expect("ops"));
    package
}

fn case_package(vectors: &Value, case: &Value) -> Value {
    let mut package = package(vectors, case["package"].as_str().expect("package"));
    apply(&mut package, case["edit"].as_array().expect("edit"));
    package
}

fn evidence(package: &Value) -> CheckedPackageEvidence {
    let document = proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT);
    fspec_inputs::evidence_for_published(
        package,
        "protocol correlation",
        &document,
        true,
        None,
        &fixtures_dir(),
    )
    .expect("evidence for the published package")
}

fn bytes(package: &Value) -> Vec<u8> {
    crate::fixture_support::canonical(package)
}

fn read(package: &Value, budget: &mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult {
    CheckedPackageV2::read_with_budget(
        &bytes(package),
        CheckedPackageReadLimits::bounded(),
        &evidence(package),
        budget,
    )
}

fn read_with_evidence(
    package: &Value,
    evidence: &CheckedPackageEvidence,
    budget: &mut NativeJoinBudget,
) -> CheckedPackageV2BudgetedReadResult {
    CheckedPackageV2::read_with_budget(
        &bytes(package),
        CheckedPackageReadLimits::bounded(),
        evidence,
        budget,
    )
}

fn admitted(package: &Value) -> CheckedPackageV2 {
    match read(package, &mut NativeJoinBudget::unlimited()) {
        CheckedPackageV2BudgetedReadResult::Admitted(package) => *package,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn refused(package: &Value) -> CheckedPackageRefusal {
    match read(package, &mut NativeJoinBudget::unlimited()) {
        CheckedPackageV2BudgetedReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// The dependency a `PC-ROOT` read selects: `PC-DEP` read as its own package.
fn dependency(vectors: &Value, member: Option<&Value>) -> Arc<CheckedPackageV2> {
    let mut dep = package(vectors, "PC-DEP");
    if let Some(rows) = member {
        dep[MEMBER] = rows.clone();
    }
    Arc::new(admitted(&dep))
}

/// Reads `package`, supplying the `PC-DEP` dependency when it selects one.
fn read_case(
    vectors: &Value,
    package: &Value,
    budget: &mut NativeJoinBudget,
) -> CheckedPackageV2BudgetedReadResult {
    let mut evidence = evidence(package);
    if package["lock"]["dependency_selections"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["identity"] == "acme/flow"))
    {
        evidence
            .insert_dependency_package("acme/flow", dependency(vectors, None))
            .expect("dependency evidence");
    }
    read_with_evidence(package, &evidence, budget)
}

fn admitted_case(vectors: &Value, package: &Value) -> CheckedPackageV2 {
    match read_case(vectors, package, &mut NativeJoinBudget::unlimited()) {
        CheckedPackageV2BudgetedReadResult::Admitted(package) => *package,
        other => panic!("expected admission, got {other:?}"),
    }
}

/// A typed read as the vectors write it.
fn typed_read(package: &CheckedPackageV2) -> Value {
    match package.protocol_correlation_provenance() {
        ProtocolCorrelationProvenance::Unavailable(unavailable) => {
            json!({"outcome": unavailable.as_str()})
        }
        ProtocolCorrelationProvenance::Complete(table) => {
            let groups: Vec<Value> = table
                .groups()
                .map(|group| {
                    let clauses: Vec<Value> = group
                        .clauses()
                        .iter()
                        .map(|row| {
                            json!({
                                "source_ordinal": row.source_ordinal,
                                "canonical_index": row.canonical_index,
                                "alias": row.alias,
                            })
                        })
                        .collect();
                    json!({
                        "control": group.control(),
                        "occurrence": group.occurrence(),
                        "source_map_entry": group.source_entry(),
                        "clauses": clauses,
                    })
                })
                .collect();
            json!({"outcome": "complete", "groups": groups})
        }
    }
}

/// The expected read of a vector, without the `package` annotation the
/// dependency case adds.
fn expected_read(expect: &Value) -> Value {
    let mut read = expect["read"].clone();
    read.as_object_mut().expect("read").remove("package");
    read
}

fn pointer_of(refusal: &CheckedPackageRefusal) -> String {
    refusal.loci[0]
        .path
        .as_ref()
        .expect("refusal locus")
        .to_string()
}

fn assert_refusal(id: &str, refusal: &CheckedPackageRefusal, expect: &Value) {
    assert_eq!(code_word(refusal.code), expect["code"], "{id}: code");
    assert_eq!(
        refusal.cause.map(cause_word),
        expect["cause"].as_str(),
        "{id}: cause"
    );
    assert_eq!(
        pointer_of(refusal),
        expect["locus"].as_str().unwrap(),
        "{id}: locus"
    );
}

fn case<'a>(vectors: &'a Value, group: &str, id: &str) -> &'a Value {
    vectors[group]
        .as_array()
        .expect("group")
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("{id}"))
}

/// Trace: FR-378-AC-1
/// Provenance: FSpec FR-831-AC-1, FR-831-AC-2, FR-831-AC-3, FR-831-AC-9
#[trace("FR-378-AC-1")]
#[test]
fn correlation_positive_vectors_read_as_published() {
    let vectors = vectors();
    let positive = vectors["positive"].as_array().expect("positive");
    assert_eq!(positive.len(), 12, "every published positive case runs");
    for case in positive {
        let id = case["id"].as_str().expect("id");
        let package = case_package(&vectors, case);
        // A dependency case reads the dependency as its own package; the other
        // cases read the root named by the case.
        let admitted = if case["package"] == "PC-ROOT" {
            admitted_case(&vectors, &package)
        } else {
            admitted(&package)
        };
        let expected = &case["expect"];
        let expected_read = expected_read(expected);
        let actual = if id == "CO-P12-root-member-dependency-without" {
            // The root's table is `[]`; the dependency it selects carries none.
            let root = typed_read(&admitted);
            let dependency = admitted
                .direct_dependencies()
                .next()
                .expect("selected dependency");
            assert_eq!(
                typed_read(dependency),
                json!({"outcome": UnavailableCorrelationProvenance::NAME}),
                "{id}: a dependency without a member is unavailable whatever the root carries"
            );
            root
        } else {
            typed_read(&admitted)
        };
        assert_eq!(actual, expected_read, "{id}");
        if id == "CO-P11-root-without-member-over-dependency" {
            assert_eq!(admitted.direct_dependencies().count(), 1, "{id}");
        }
    }
}

/// Trace: FR-378-AC-1
/// Provenance: FSpec FR-831-AC-9
#[trace("FR-378-AC-1")]
#[test]
fn correlation_dependency_is_read_from_its_own_member() {
    let vectors = vectors();
    let case = case(&vectors, "positive", "CO-P10-dependency-own-table");
    // The dependency's own member, edit applied to PC-DEP.
    let package = case_package(&vectors, case);
    let own = admitted(&package);
    assert_eq!(typed_read(&own), expected_read(&case["expect"]));
    // A root that names the dependency's control refuses; the root never
    // supplies a dependency control from its own member.
    let root_case = case_named(
        &vectors,
        "reader_refusals",
        "CO-R03-root-names-dependency-control",
    );
    let root = case_package(&vectors, root_case);
    let mut evidence = evidence(&root);
    evidence
        .insert_dependency_package("acme/flow", Arc::new(own))
        .expect("dependency evidence");
    let CheckedPackageV2BudgetedReadResult::Refused(refusal) =
        read_with_evidence(&root, &evidence, &mut NativeJoinBudget::unlimited())
    else {
        panic!("a root member naming a dependency control refuses");
    };
    assert_refusal("CO-R03", &refusal, &root_case["expect"]);
}

fn case_named<'a>(vectors: &'a Value, group: &str, id: &str) -> &'a Value {
    case(vectors, group, id)
}

/// Trace: FR-378-AC-2
/// Provenance: FSpec FR-831-AC-3, FR-831-AC-4, FR-831-AC-5, FR-831-AC-9
#[trace("FR-378-AC-2")]
#[test]
fn correlation_reader_refusals_name_code_cause_and_locus() {
    let vectors = vectors();
    let cases = vectors["reader_refusals"].as_array().expect("refusals");
    assert_eq!(cases.len(), 21, "every published refusal case runs");
    for case in cases {
        let id = case["id"].as_str().expect("id");
        let package = case_package(&vectors, case);
        let result = read_case(&vectors, &package, &mut NativeJoinBudget::unlimited());
        let CheckedPackageV2BudgetedReadResult::Refused(refusal) = result else {
            panic!("{id}: expected a refusal, got {result:?}");
        };
        let expect = &case["expect"];
        assert_refusal(id, &refusal, expect);
        if let Some(retained) = expect.get("retained") {
            assert_eq!(
                refusal
                    .locus
                    .as_ref()
                    .map(|control| serde_json::to_value(control).unwrap()),
                Some(retained["control"].clone()),
                "{id}: retained control"
            );
            let Some(detail) = refusal.protocol_detail.as_deref() else {
                panic!("{id}: retained occurrence and index");
            };
            let CheckedProtocolRefusalDetail::CorrelationCoverage {
                control,
                occurrence,
                canonical_index,
            } = detail
            else {
                panic!("{id}: coverage detail, got {detail:?}");
            };
            assert_eq!(
                serde_json::to_value(control).unwrap(),
                retained["control"],
                "{id}"
            );
            assert_eq!(
                serde_json::to_value(occurrence).unwrap(),
                retained["occurrence"],
                "{id}"
            );
            assert_eq!(json!(canonical_index), retained["canonical_index"], "{id}");
        }
    }
}

/// Trace: FR-378-AC-2
/// Provenance: FSpec FR-831-AC-6
#[trace("FR-378-AC-2")]
#[test]
fn correlation_schema_decisions_are_malformed_wire_at_the_first_error() {
    let vectors = vectors();
    let invalid = vectors["schema_invalid"]
        .as_array()
        .expect("schema_invalid");
    assert_eq!(invalid.len(), 12, "every published schema case runs");
    for case in invalid {
        let id = case["id"].as_str().expect("id");
        let package = case_package(&vectors, case);
        let refusal = refused(&package);
        assert_eq!(code_word(refusal.code), "malformed_wire", "{id}");
        assert_eq!(
            pointer_of(&refusal),
            case["expect"]["locus"].as_str().expect("locus"),
            "{id}: first schema error path"
        );
    }
    let valid = vectors["schema_valid"].as_array().expect("schema_valid");
    assert_eq!(valid.len(), 3);
    // Schema-valid is not reader-admitted: `[]` over nonempty related (CO-R20)
    // and a safe-integer ordinal gap (CO-S02) are schema-valid and refused by
    // the reader; only the rows of CO-P04 admit.
    let expected = [
        (
            "CO-S00-present-empty",
            Some(("invalid-value", "/protocol_correlation_occurrences")),
        ),
        ("CO-S01-every-form", None),
        (
            "CO-S02-max-safe-ordinal",
            Some((
                "invalid-value",
                "/protocol_correlation_occurrences/0/source_ordinal",
            )),
        ),
    ];
    for case in valid {
        let id = case["id"].as_str().expect("id");
        let package = case_package(&vectors, case);
        let (_, refusal) = expected
            .iter()
            .find(|(name, _)| *name == id)
            .map(|(n, r)| (n, r))
            .expect(id);
        match refusal {
            None => {
                assert!(
                    matches!(
                        admitted(&package).protocol_correlation_provenance(),
                        ProtocolCorrelationProvenance::Complete(_)
                    ),
                    "{id}"
                );
            }
            Some((cause, locus)) => {
                let refusal = refused(&package);
                assert_eq!(code_word(refusal.code), "invalid_package", "{id}");
                assert_eq!(refusal.cause.map(cause_word), Some(*cause), "{id}");
                assert_eq!(pointer_of(&refusal), *locus, "{id}");
            }
        }
    }
}

/// Trace: FR-378-AC-3
/// Provenance: FSpec FR-831-AC-1, FR-831-AC-12
#[trace("FR-378-AC-3")]
#[test]
fn correlation_consumer_refuses_an_absent_member_over_nonempty_related_only() {
    let vectors = vectors();
    let cases = vectors["consumer"]["cases"].as_array().expect("consumer");
    assert!(!cases.is_empty());
    for case in cases {
        let id = case["id"].as_str().expect("id");
        let package = case_package(&vectors, case);
        let admitted = admitted(&package);
        let control: CheckedNodeId =
            serde_json::from_value(case["needs"]["control"].clone()).expect("needed control");
        let result = admitted.protocol_correlation_clauses(&control);
        let expect = &case["expect"];
        match expect["outcome"].as_str().expect("outcome") {
            "refuse" => {
                assert_eq!(
                    expect["name"],
                    UnavailableCorrelationProvenance::NAME,
                    "{id}"
                );
                assert_eq!(
                    result.err(),
                    Some(ProtocolCorrelationReadError::Unavailable(
                        UnavailableCorrelationProvenance
                    )),
                    "{id}"
                );
            }
            "proceed" => {
                let rows = result.unwrap_or_else(|error| panic!("{id}: {error}"));
                let actual: Vec<Value> = rows
                    .iter()
                    .map(|row| {
                        json!({
                            "source_ordinal": row.source_ordinal,
                            "canonical_index": row.canonical_index,
                            "alias": row.alias,
                        })
                    })
                    .collect();
                assert_eq!(json!(actual), expect["clauses"], "{id}");
            }
            other => panic!("{id}: unknown outcome {other}"),
        }
    }
}

fn resource_word(resource: PackageReadResource) -> &'static str {
    match resource {
        PackageReadResource::Work => "work",
        PackageReadResource::ReadStorage => "read_storage",
    }
}

/// A read of a case package under a budget with the given limits, after
/// `prior_work` units an earlier join of the same budget consumed. Returns the
/// outcome and the locus of the first unpaid charge, which the budget keeps.
fn budgeted(
    vectors: &Value,
    package: &Value,
    work: u64,
    storage: u64,
    prior_work: u64,
) -> (CheckedPackageV2BudgetedReadResult, Option<NativeJoinLocus>) {
    let mut budget = NativeJoinBudget::new(work, u64::MAX).with_read_storage_limit(storage);
    if prior_work > 0 {
        budget
            .accounted_charge(
                NativeJoinChargeKind::Work,
                u128::from(prior_work),
                NativeJoinLocus {
                    source: NativeJoinLocusSource::Package,
                    form: "earlier join",
                },
            )
            .expect("the earlier join is admitted");
    }
    let result = read_case(vectors, package, &mut budget);
    (result, budget.first_unpaid_locus())
}

/// The first denied charge of a budgeted read: what the reader names whether or
/// not the budget could pay for the pointer of its own diagnostic.
struct Denial {
    stage: PackageReadStage,
    resource: PackageReadResource,
    amount: u128,
    pointer: Option<String>,
    charge: &'static str,
    index: usize,
}

fn denial(
    result: CheckedPackageV2BudgetedReadResult,
    locus: Option<NativeJoinLocus>,
) -> Option<Denial> {
    let (resource, amount, pointer, locus) = match result {
        CheckedPackageV2BudgetedReadResult::BudgetIncomplete(incomplete) => (
            incomplete.denied.resource,
            incomplete.denied.amount,
            incomplete.path.map(|path| path.to_string()),
            locus.expect("the budget keeps the first unpaid locus"),
        ),
        CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) => (
            exhausted.denied.resource,
            exhausted.denied.amount,
            None,
            exhausted.locus,
        ),
        _ => return None,
    };
    let NativeJoinLocusSource::PackageRead(stage, index) = locus.source else {
        panic!("a package read charge, got {locus:?}");
    };
    Some(Denial {
        stage,
        resource,
        amount,
        pointer,
        charge: locus.form,
        index,
    })
}

/// Whether a read got as far as the member's step: no charge before it was
/// denied. The step is last, so what denies a read is the step itself, the
/// boxed result charged after it, or an earlier stage.
fn reached_member(
    result: CheckedPackageV2BudgetedReadResult,
    locus: Option<NativeJoinLocus>,
) -> bool {
    denial(result, locus).is_none_or(|denial| {
        denial.stage == PackageReadStage::Retained
            && (denial.charge.starts_with("protocol_correlation") || denial.charge == "result")
    })
}

/// The consumption of `resource` immediately before the member's step, on this
/// input: the least limit at which the read gets as far as the step. A smaller
/// limit is denied by an earlier stage of the strict read.
fn baseline(vectors: &Value, package: &Value, resource: PackageReadResource) -> u64 {
    let reaches = |limit: u64| {
        let (work, storage) = match resource {
            PackageReadResource::Work => (limit, u64::MAX),
            PackageReadResource::ReadStorage => (u64::MAX, limit),
        };
        let (result, locus) = budgeted(vectors, package, work, storage, 0);
        reached_member(result, locus)
    };
    let (mut below, mut above) = (0_u64, 1_u64 << 40);
    assert!(reaches(above), "an unbounded budget reaches the step");
    while above - below > 1 {
        let middle = below + (above - below) / 2;
        if reaches(middle) {
            above = middle;
        } else {
            below = middle;
        }
    }
    if reaches(below) {
        below
    } else {
        above
    }
}

/// `G`: the size of the typed-view block, as the reader first denies it when
/// the limit leaves it nothing to spend.
fn view_bytes(vectors: &Value, package: &Value, storage_baseline: u64) -> u64 {
    let (result, locus) = budgeted(vectors, package, u64::MAX, storage_baseline, 0);
    let shown = format!("{:?}", locus);
    let kind = match &result {
        CheckedPackageV2BudgetedReadResult::Admitted(_) => "admitted".to_owned(),
        other => format!("{other:?}").chars().take(300).collect(),
    };
    let denial = denial(result, locus).unwrap_or_else(|| {
        panic!("the view block is denied at the baseline {storage_baseline}: {kind} {shown}")
    });
    assert_eq!(denial.resource, PackageReadResource::ReadStorage);
    assert_eq!(denial.charge, "protocol_correlation_view");
    u64::try_from(denial.amount).expect("view block size")
}

fn offset(text: &Value, g: u64) -> u64 {
    if let Some(number) = text.as_u64() {
        return number;
    }
    let text = text.as_str().expect("offset");
    let (head, tail) = match text.split_once(" + ") {
        Some((head, tail)) => (head, Some((true, tail))),
        None => match text.split_once(" - ") {
            Some((head, tail)) => (head, Some((false, tail))),
            None => (text, None),
        },
    };
    assert_eq!(head, "G", "{text}");
    match tail {
        None => g,
        Some((true, tail)) => g + tail.parse::<u64>().expect("added"),
        Some((false, tail)) => g - tail.parse::<u64>().expect("subtracted"),
    }
}

fn amount(value: &Value, g: u64) -> u128 {
    u128::from(value.as_u64().unwrap_or(g))
}

/// Trace: FR-378-AC-4
/// Provenance: FSpec FR-831-AC-7
#[trace("FR-378-AC-4")]
#[test]
fn correlation_budget_vectors_admit_exactly_and_deny_the_first_unpaid_charge() {
    let vectors = vectors();
    let cases = vectors["budget"]["cases"].as_array().expect("budget cases");
    assert_eq!(cases.len(), 16, "every published budget case runs");
    for case in cases {
        let id = case["id"].as_str().expect("id");
        let package = case_package(&vectors, case);
        let resource = match case["counter"].as_str().expect("counter") {
            "work" => PackageReadResource::Work,
            "read_storage" => PackageReadResource::ReadStorage,
            other => panic!("{id}: counter {other}"),
        };
        let prior = case["prior_work"].as_u64().unwrap_or(0);
        // The baseline is measured with no earlier charge: an earlier join on
        // the same budget only moves where the limit lies.
        let base = baseline(&vectors, &package, resource);
        let g = match resource {
            PackageReadResource::ReadStorage => view_bytes(&vectors, &package, base),
            PackageReadResource::Work => 0,
        };
        let limit = base + offset(&case["limit_over_baseline"], g);
        let (result, locus) = match resource {
            PackageReadResource::Work => budgeted(&vectors, &package, limit, u64::MAX, prior),
            PackageReadResource::ReadStorage => {
                budgeted(&vectors, &package, u64::MAX, limit, prior)
            }
        };
        let expect = &case["expect"];
        match expect["decision"].as_str().expect("decision") {
            "admit" => {
                // The vectors state the limits of the member's step. The strict
                // read charges the boxed result after it, which a storage limit
                // that is exact for the step leaves unpaid: the step itself
                // admits when nothing it charged was denied, and the read
                // completes with the result's own bytes added.
                let package = match result {
                    CheckedPackageV2BudgetedReadResult::Admitted(package) => package,
                    other => {
                        let denial = denial(other, locus).unwrap_or_else(|| {
                            panic!(
                                "{id}: expected admission at base + {}",
                                case["limit_over_baseline"]
                            )
                        });
                        assert_eq!(
                            (denial.stage, denial.charge, resource),
                            (
                                PackageReadStage::Retained,
                                "result",
                                PackageReadResource::ReadStorage
                            ),
                            "{id}: only the result's box follows the step"
                        );
                        assert_eq!(denial.resource, PackageReadResource::ReadStorage, "{id}");
                        let grown = limit + u64::try_from(denial.amount).expect("result size");
                        match budgeted(
                            &vectors,
                            &case_package(&vectors, case),
                            u64::MAX,
                            grown,
                            prior,
                        )
                        .0
                        {
                            CheckedPackageV2BudgetedReadResult::Admitted(package) => package,
                            other => {
                                panic!("{id}: the step admitted, the read does not: {other:?}")
                            }
                        }
                    }
                };
                if let Some(read) = expect.get("read") {
                    assert_eq!(typed_read(&package), *read, "{id}");
                }
            }
            "incomplete" => {
                let denial = denial(result, locus)
                    .unwrap_or_else(|| panic!("{id}: expected the budget to deny a charge"));
                let unpaid = &expect["first_unpaid"];
                assert_eq!(
                    resource_word(denial.resource),
                    expect["limit"],
                    "{id}: counter"
                );
                assert_eq!(
                    denial.amount,
                    amount(&unpaid["amount"], g),
                    "{id}: the full amount attempted"
                );
                // The pointer is built only when the budget can pay for it;
                // the locus of the charge is always known.
                if let Some(pointer) = &denial.pointer {
                    assert_eq!(
                        pointer,
                        unpaid["locus"].as_str().expect("locus"),
                        "{id}: locus"
                    );
                }
                let charge = unpaid["charge"].as_str().expect("charge");
                let wanted = match charge.split([' ', ',']).next().expect("word") {
                    "row" => "protocol_correlation_row",
                    "scan" => "protocol_correlation_scan",
                    "flags" => "protocol_correlation_flags",
                    "typed-view" => "protocol_correlation_view",
                    other => panic!("{id}: charge {other}"),
                };
                assert_eq!(denial.charge, wanted, "{id}: the charge denied");
                if wanted == "protocol_correlation_row" {
                    let row = unpaid["locus"].as_str().expect("locus");
                    let row = row.rsplit('/').next().expect("row");
                    assert_eq!(
                        denial.index,
                        row.parse::<usize>().expect("row index"),
                        "{id}"
                    );
                }
            }
            other => panic!("{id}: decision {other}"),
        }
    }
}

/// Trace: FR-378-AC-5
#[trace("FR-378-AC-5")]
#[test]
fn correlation_member_is_the_only_difference_between_invariance_pairs() {
    let vectors = vectors();
    let cases = vectors["invariance"].as_array().expect("invariance");
    assert_eq!(cases.len(), 3, "every published invariance case runs");
    for case in cases {
        let id = case["id"].as_str().expect("id");
        let name = case["package"].as_str().expect("package");
        let build = |edit: &str| {
            let mut package = package(&vectors, name);
            apply(&mut package, case[edit].as_array().expect("edit"));
            package
        };
        let (a, b) = (build("editA"), build("editB"));
        let (read_a, read_b) = (admitted(&a), admitted(&b));
        let strip = |package: &Value| {
            let mut package = package.clone();
            package.as_object_mut().expect("object").remove(MEMBER);
            package
        };
        assert_eq!(strip(&a), strip(&b), "{id}: every other member is equal");
        assert_ne!(a.get(MEMBER), b.get(MEMBER), "{id}: the member differs");
        assert_ne!(bytes(&a), bytes(&b), "{id}: the package bytes differ");
        assert_eq!(
            read_a.direct_dependencies().count(),
            read_b.direct_dependencies().count(),
            "{id}"
        );
        assert_ne!(
            typed_read(&read_a),
            typed_read(&read_b),
            "{id}: only the provenance read differs"
        );
    }
}

/// Trace: FR-378-AC-1
#[trace("FR-378-AC-1")]
#[test]
fn correlation_group_carries_every_region_of_its_source_map_entry() {
    let vectors = vectors();
    let case = case_named(&vectors, "positive", "CO-P04-every-form");
    let mut package = case_package(&vectors, case);
    let entry = package["source_map"]
        .as_array_mut()
        .expect("source_map")
        .iter_mut()
        .find(|entry| {
            entry["role"] == "declaration"
                && entry["node_id"]["path"]
                    .as_array()
                    .and_then(|path| path.last())
                    == Some(&json!("Advised"))
        })
        .expect("the Advised declaration entry");
    let regions = entry["regions"].as_array_mut().expect("regions");
    let mut second = regions[0].clone();
    second["start"] = json!(300);
    second["end"] = json!(301);
    regions.push(second);
    let expected_entry = entry.clone();
    let read = typed_read(&admitted(&package));
    let groups = read["groups"].as_array().expect("groups");
    let group = groups
        .iter()
        .find(|group| {
            group["control"]["path"].as_array().and_then(|p| p.last()) == Some(&json!("Advised"))
        })
        .expect("Advised group");
    assert_eq!(group["source_map_entry"], expected_entry);
    assert_eq!(
        group["source_map_entry"]["regions"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
}
