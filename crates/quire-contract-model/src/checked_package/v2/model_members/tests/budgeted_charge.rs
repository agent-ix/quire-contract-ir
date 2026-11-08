//! The ModelDocument stage charges every allocation of the selected model's
//! read to the caller's budget before it is made (FR-038 "Budget-charged
//! strict read").

use super::*;
use crate::checked_package::v2::native_join_budget::{
    NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource,
};
use crate::checked_package::v2::PackageReadStage;

/// The position of the selected model whose read these tests charge.
const POSITION: usize = 3;

/// What one read under a read-storage allowance did.
#[derive(Debug, Eq, PartialEq)]
struct Outcome {
    admitted: bool,
    consumed: u64,
    allocated: u64,
    locus: Option<NativeJoinLocus>,
}

/// Reads `document` as the selected model at [`POSITION`] under `allowance`
/// bytes of read storage, measuring every byte the read allocates.
fn run(document: &Value, allowance: u64) -> Outcome {
    let bytes = serde_json::to_vec(document).expect("authored bytes");
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document("acme/orders", bytes)
        .expect("first domain evidence insertion succeeds");
    let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX).with_read_storage_limit(allowance);
    let unit_refs = UnitRefs::default();
    let mut admitted = false;
    let measured = allocation_counter::measure(|| {
        let mut meter = WorkMeter::over(&mut budget);
        let mut read =
            Budget::new(&mut meter, 0, u64::MAX).at(PackageReadStage::ModelDocument, POSITION);
        admitted =
            admit_selection("acme/orders", "checked", &evidence, &mut read, &unit_refs).is_ok();
    });
    Outcome {
        admitted,
        consumed: budget.read_storage_consumed(),
        allocated: measured.bytes_total,
        locus: budget.first_unpaid_locus(),
    }
}

/// The bytes the refusal's own pointer to the root owner takes to build. The
/// pointer is the diagnostic of the stop, which the budgeted outcome charges
/// on its own (the `diagnostic` form), not an allocation of the read.
fn denial_pointer_bytes() -> u64 {
    allocation_counter::measure(|| {
        drop(crate::checked_package::common::node_pointer(0).key("owner"));
    })
    .bytes_total
}

fn operation(owner: &str, name: &str) -> Value {
    let identity = format!("{owner}/{name}");
    json!({
        "identity": identity, "name": name,
        "params": [{
            "identity": format!("{identity}/amount"), "name": "amount",
            "typeRef": "ix://quire/native/Integer",
            "multiplicity": multiplicity(1, Some(1)),
        }],
        "returns": {"typeRef": "ix://quire/native/Integer", "multiplicity": multiplicity(1, Some(1))},
        "frame": {"modifies": ["amount"], "creates": [], "deletes_receiver": false},
    })
}

/// A document that exercises the object, field, operation, relationship,
/// supertype and origin reads.
fn fixture() -> Value {
    let mut widget = object_type(
        WIDGET,
        &[],
        vec![
            field(WIDGET, "code", "ix://quire/native/Integer"),
            field(WIDGET, "label", "ix://quire/native/String"),
        ],
    );
    widget["operations"] = json!([operation(WIDGET, "bump"), operation(WIDGET, "reset")]);
    widget["relationships"] = json!([relationship()]);
    widget["origin"] = json!({"source": {
        "sourceIdentity": WIDGET, "path": "models/Widget.md", "startLine": 1, "startColumn": 1}});
    let gadget = object_type(
        GADGET,
        &[WIDGET],
        vec![field(GADGET, "size", "ix://quire/native/Integer")],
    );
    document(vec![widget, gadget])
}

/// A document that adds tuples, variants, dimensions, units and an alias.
fn wide_fixture() -> Value {
    const PAIR: &str = "ix://acme/orders/Pair";
    const MAYBE: &str = "ix://acme/orders/Maybe";
    const ALIAS: &str = "ix://acme/orders/Count";
    let mut selected = fixture();
    let types = selected["types"].as_array_mut().expect("types");
    types.push(json!({
        "identity": PAIR, "kind": {"module": "acme/orders", "name": "tuple"},
        "positions": [
            {"identity": format!("{PAIR}/p0"), "typeRef": "ix://quire/native/Boolean"},
            {"identity": format!("{PAIR}/p1"), "typeRef": ALIAS},
        ]
    }));
    types.push(json!({
        "identity": MAYBE, "kind": {"module": "acme/orders", "name": "variant"},
        "variants": [
            {"identity": format!("{MAYBE}/Some"), "name": "Some", "payloadType": PAIR},
            {"identity": format!("{MAYBE}/None"), "name": "None"},
        ]
    }));
    types.push(json!({
        "identity": ALIAS, "kind": "alias", "target": "ix://quire/native/Integer"
    }));
    selected["constructs"]
        .as_array_mut()
        .expect("constructs")
        .extend([
            json!({"kind":{"module":"acme/orders","name":"tuple"},
            "construct":{"meaning":"quire.meaning.model.tuple-value-type/v1"}}),
            json!({"kind":{"module":"acme/orders","name":"variant"},
            "construct":{"meaning":"quire.meaning.model.variant-type/v1"}}),
        ]);
    selected["dimensions"] = json!([
        {"identity":"ix://acme/orders/dimension/Time","name":"Time","base":true,"terms":[]},
        {"identity":"ix://acme/orders/dimension/Rate","name":"Rate","base":false,
         "terms":[{"dimension":"ix://acme/orders/dimension/Time","exponent":"-1"}]},
    ]);
    selected["units"] = json!([{
        "identity":"ix://quire/unit/acme/orders/s","name":"s",
        "dimension":"ix://acme/orders/dimension/Time","scale":"1","offset":"0"}]);
    selected
}

/// Both retained population indexes own their identity, and each copy must be
/// reserved before the selected-model reader constructs it.
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn population_keys_are_paid_before_both_retained_copies_are_allocated() {
    const POPULATION: &str = "ix://acme/orders/widgets";
    let mut selected = document(vec![object_type(WIDGET, &[], vec![])]);
    selected["constructs"]
        .as_array_mut()
        .expect("construct rows")
        .push(json!({
            "kind": {"module": "acme/orders", "name": "population"},
            "construct": {"meaning": "quire.meaning.model.population/v1"},
        }));
    selected["populations"] = json!([{
        "identity": POPULATION,
        "kind": {"module": "acme/orders", "name": "population"},
        "members": [WIDGET],
    }]);
    let model = read(&selected).expect("selected population reads");
    assert!(model.populations.contains(POPULATION));
    assert!(model
        .population_members
        .get(POPULATION)
        .expect("retained population members")
        .contains(WIDGET));

    let unlimited = run(&selected, u64::MAX);
    assert!(unlimited.admitted);
    assert_eq!(
        unlimited.allocated, unlimited.consumed,
        "both owned population keys must be paid before allocation"
    );
    assert_eq!(run(&selected, u64::MAX), unlimited, "the read repeats");
    assert_eq!(
        run(&selected, unlimited.consumed),
        unlimited,
        "the exact charged allowance admits the same read"
    );
    let short = run(&selected, unlimited.consumed - 1);
    assert!(!short.admitted, "one byte short must stop the read");
    assert_eq!(short.allocated, short.consumed);
    assert!(short.consumed < unlimited.consumed);
    assert_eq!(
        short.locus.map(|locus| locus.source),
        Some(NativeJoinLocusSource::PackageRead(
            PackageReadStage::ModelDocument,
            POSITION
        ))
    );
    assert_eq!(
        run(&selected, unlimited.consumed - 1),
        short,
        "the short read repeats its exact accounting and denial locus"
    );
    let zero = run(&selected, 0);
    assert!(!zero.admitted);
    assert_eq!((zero.allocated, zero.consumed), (0, 0));
    assert_eq!(run(&selected, 0), zero, "the zero allowance repeats");
}

#[derive(Debug, Eq, PartialEq)]
struct PopulationFaultOutcome {
    failure: ValidationFailure,
    consumed: u64,
    allocated: u64,
    work: u64,
    unpaid: Option<(crate::checked_package::v2::PackageReadResource, u128)>,
    locus: Option<NativeJoinLocus>,
    paid: Option<crate::checked_package::v2::CheckedPackagePendingOutcome>,
    pending: Option<crate::checked_package::v2::CheckedPackagePendingOutcome>,
}

/// Measures the actual selected-model read and its public refusal projection.
fn run_population_fault(document: &Value, allowance: u64) -> PopulationFaultOutcome {
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(
            "acme/orders",
            serde_json::to_vec(document).expect("authored selected document"),
        )
        .expect("first selected document");
    let roots = [("acme/orders", 4)];
    let unit_refs = UnitRefs::default();
    let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX).with_read_storage_limit(allowance);
    let mut result = None;
    let measured = allocation_counter::measure(|| {
        result = Some(crate::checked_package::v2::validate_domain_packages(
            &roots,
            "checked",
            &evidence,
            &mut WorkMeter::over(&mut budget),
            u64::MAX,
            &unit_refs,
        ));
    });
    PopulationFaultOutcome {
        failure: result
            .expect("the selected-model read ran")
            .expect_err("the malformed population refuses"),
        consumed: budget.read_storage_consumed(),
        allocated: measured.bytes_total,
        work: budget.consumed().work,
        unpaid: budget.first_unpaid_resource(),
        locus: budget.first_unpaid_locus(),
        paid: budget.take_paid(),
        pending: budget.take_pending(),
    }
}

/// A bare population refusal pays its private carrier and public record at
/// their respective construction boundaries, without exposing intake metadata.
#[trace("TC-229", "FR-038-AC-304", "FR-038-AC-511")]
#[test]
fn bare_population_faults_prepay_the_private_carrier_and_public_diagnostic() {
    use crate::checked_package::shared::{CheckedPackageRefusal, CheckedPackageRefusalLocation};
    use crate::checked_package::v2::{CheckedPackagePendingOutcome, PackageReadResource};

    const POPULATION: &str = "ix://acme/orders/widgets";
    const PATH: &str = "/semantic_graph/nodes/4/owner";
    let mut selected = document(vec![object_type(WIDGET, &[], vec![])]);
    selected["constructs"]
        .as_array_mut()
        .expect("construct rows")
        .push(json!({
            "kind": {"module": "acme/orders", "name": "population"},
            "construct": {"meaning": "quire.meaning.model.population/v1"},
        }));
    selected["populations"] = json!([{
        "identity": POPULATION,
        "kind": {"module": "acme/orders", "name": "population"},
        "members": [WIDGET],
    }]);
    let model = read(&selected).expect("the unmodified population is genuine");
    assert!(model.populations.contains(POPULATION));

    let mut missing_identity = selected.clone();
    missing_identity["populations"][0]
        .as_object_mut()
        .expect("population row")
        .remove("identity");
    let mut forbidden_node = selected.clone();
    forbidden_node["populations"][0]["node"] = json!(POPULATION);
    let mut unresolved = selected.clone();
    unresolved["constructs"]
        .as_array_mut()
        .expect("construct rows")
        .retain(|entry| entry["kind"]["name"] != "population");
    let mut wrong_meaning = selected.clone();
    wrong_meaning["populations"][0]["kind"]["name"] = json!("entity");
    let mut duplicate = selected.clone();
    duplicate["populations"] = json!([
        selected["populations"][0].clone(),
        selected["populations"][0].clone(),
    ]);
    let mut wrong_members = selected.clone();
    wrong_members["populations"][0]["members"] = json!(0);
    let mut wrong_member = selected;
    wrong_member["populations"][0]["members"] = json!([0]);

    let code = Code::InvalidModelBinding;
    let pending = CheckedPackagePendingOutcome::Refused(code);
    let private_bytes = std::mem::size_of::<SelectionRefusal>() as u128;
    let public_bytes = (std::mem::size_of::<CheckedPackageRefusal>()
        + std::mem::size_of::<CheckedPackageRefusalLocation>()
        + PATH.len()) as u64;
    for (name, document) in [
        ("missing identity", missing_identity),
        ("forbidden node", forbidden_node),
        ("unresolved construct", unresolved),
        ("wrong meaning", wrong_meaning),
        ("duplicate population", duplicate),
        ("wrong members", wrong_members),
        ("wrong member", wrong_member),
    ] {
        let unlimited = run_population_fault(&document, u64::MAX);
        assert_eq!(unlimited.allocated, unlimited.consumed, "{name}");
        let ValidationFailure::Refused(refusal) = &unlimited.failure else {
            panic!("{name}: the selected population must refuse semantically");
        };
        assert_eq!(refusal.code, code, "{name}");
        assert_eq!(refusal.cause, Some(Cause::MalformedDeclaration), "{name}");
        assert_eq!(refusal.loci.len(), 1, "{name}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some(PATH),
            "{name}"
        );
        assert!(refusal.document_pointer.is_none(), "{name}");
        assert!(refusal.locus.is_none(), "{name}");
        assert!(refusal.contract_version.is_none(), "{name}");
        assert!(refusal.extension_detail.is_none(), "{name}");
        assert!(refusal.protocol_detail.is_none(), "{name}");
        assert!(refusal.terminal_duplicate.is_none(), "{name}");
        assert_eq!(unlimited.paid, Some(pending), "{name}");
        assert_eq!(unlimited.pending, None, "{name}");
        assert_eq!(unlimited.unpaid, None, "{name}");
        assert_eq!(
            run_population_fault(&document, u64::MAX),
            unlimited,
            "{name}: repeat"
        );
        assert_eq!(
            run_population_fault(&document, unlimited.consumed),
            unlimited,
            "{name}: exact"
        );

        let private_exact = unlimited.consumed - public_bytes;
        for (allowance, first_unpaid) in [
            (unlimited.consumed - 1, u128::from(public_bytes)),
            (private_exact, u128::from(public_bytes)),
            (private_exact - 1, private_bytes),
        ] {
            let short = run_population_fault(&document, allowance);
            assert!(
                matches!(short.failure, ValidationFailure::Budget(_)),
                "{name}"
            );
            assert_eq!(short.allocated, short.consumed, "{name}: short {allowance}");
            assert!(short.consumed <= allowance, "{name}");
            assert_eq!(
                short.work, unlimited.work,
                "{name}: no changed work at projection"
            );
            assert_eq!(
                short.unpaid,
                Some((PackageReadResource::ReadStorage, first_unpaid)),
                "{name}"
            );
            assert_eq!(short.pending, Some(pending), "{name}");
            assert_eq!(short.paid, None, "{name}: the public record was never paid");
            assert_eq!(
                short.locus.map(|locus| locus.source),
                Some(NativeJoinLocusSource::PackageRead(
                    PackageReadStage::ModelDocument,
                    0
                )),
                "{name}"
            );
            assert_eq!(
                run_population_fault(&document, allowance),
                short,
                "{name}: short repeats"
            );
        }
        let zero = run_population_fault(&document, 0);
        assert!(
            matches!(zero.failure, ValidationFailure::Budget(_)),
            "{name}"
        );
        assert_eq!((zero.allocated, zero.consumed), (0, 0), "{name}");
        assert_eq!((zero.paid, zero.pending), (None, None), "{name}");
        assert_eq!(
            run_population_fault(&document, 0),
            zero,
            "{name}: zero repeats"
        );
    }
}

/// An exact allowance admits the read and a byte less refuses it at the
/// ModelDocument locus, so every allocation is requested before it is made
/// and none is made that was not requested.
#[trace("TC-472", "FR-038-AC-497")]
#[test]
fn tc_472_model_document_read_allocates_exactly_what_it_requests() {
    for document in [fixture(), wide_fixture()] {
        let unlimited = run(&document, u64::MAX);
        assert!(unlimited.admitted, "the fixture reads");
        assert_eq!(
            unlimited.allocated, unlimited.consumed,
            "every byte the read allocates is a byte it requested"
        );

        let exact = run(&document, unlimited.consumed);
        assert!(exact.admitted, "the exact allowance admits the read");
        assert_eq!(exact.consumed, unlimited.consumed);
        assert_eq!(exact.locus, None);

        let short = run(&document, unlimited.consumed - 1);
        assert!(!short.admitted, "one byte short refuses");
        assert_eq!(
            short.locus.map(|locus| locus.source),
            Some(NativeJoinLocusSource::PackageRead(
                PackageReadStage::ModelDocument,
                POSITION
            ))
        );
        assert!(short.allocated <= unlimited.consumed - 1 + denial_pointer_bytes());
    }
}

/// No allowance is ever overdrawn: whatever the read allocates before it
/// refuses was requested within the allowance.
#[trace("TC-472", "FR-038-AC-498")]
#[test]
fn tc_472_model_document_read_never_allocates_past_its_allowance() {
    for document in [fixture(), wide_fixture()] {
        let total = run(&document, u64::MAX).consumed;
        let mut allowance = 0;
        while allowance < total {
            let outcome = run(&document, allowance);
            assert!(!outcome.admitted, "allowance {allowance} is short");
            assert!(
                outcome.allocated <= allowance + denial_pointer_bytes(),
                "allowance {allowance}: allocated {}",
                outcome.allocated
            );
            assert_eq!(
                outcome.locus.map(|locus| locus.source),
                Some(NativeJoinLocusSource::PackageRead(
                    PackageReadStage::ModelDocument,
                    POSITION
                )),
                "allowance {allowance}"
            );
            allowance += 61;
        }
    }
}

use crate::checked_package::v2::monitor_plan::model_document_test_fixture as public_fixture;

/// Every place pays its one exact diagnostic block before building it.
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn budget_denial_places_pay_owned_paths_before_work_and_storage_errors() {
    use crate::checked_package::v2::native_join_budget::NativeJoinChargeKind;
    for (place, root, index, stage, expected) in [
        (
            Place::Owner,
            13,
            27,
            PackageReadStage::ModelDocument,
            "/semantic_graph/nodes/13/owner",
        ),
        (
            Place::Lock("domain/~é"),
            13,
            27,
            PackageReadStage::Lock,
            "/lock/domain~1~0é/27",
        ),
        (
            Place::Node,
            13,
            27,
            PackageReadStage::Graph,
            "/semantic_graph/nodes/27/body",
        ),
        (
            Place::NodeOwner,
            13,
            27,
            PackageReadStage::Lock,
            "/semantic_graph/nodes/27/owner",
        ),
    ] {
        for work in [true, false] {
            for affordable in [true, false] {
                let length = expected.len() as u64;
                let allowance = if affordable { length } else { length - 1 };
                let mut native = NativeJoinBudget::new(0, 0).with_read_storage_limit(allowance);
                let mut result = None;
                let measured = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(&mut native);
                    let mut read = Budget {
                        meter: &mut meter,
                        root,
                        bytes: 0,
                        stage,
                        index,
                        place,
                    };
                    result = Some(if work {
                        read.charge_form(3, "model_document")
                    } else {
                        read.request(4096, "model_document")
                    });
                });
                let Err(ValidationFailure::Budget(stop)) = result.unwrap() else {
                    panic!("the original request must be denied");
                };
                assert_eq!(
                    stop.path.as_ref().map(|path| path.as_str()),
                    affordable.then_some(expected)
                );
                assert_eq!(stop.amount, if work { 3 } else { 4096 });
                assert_eq!(
                    stop.resource,
                    if work {
                        PackageReadResource::Work
                    } else {
                        PackageReadResource::ReadStorage
                    }
                );
                assert_eq!(
                    native
                        .first_unpaid()
                        .map(|charge| (charge.kind, charge.amount)),
                    work.then_some((NativeJoinChargeKind::Work, 3))
                );
                assert_eq!(
                    native.first_unpaid_read_storage(),
                    if work { None } else { Some(4096) }
                );
                assert_eq!(
                    native.first_unpaid_locus(),
                    Some(NativeJoinLocus {
                        source: NativeJoinLocusSource::PackageRead(stage, index),
                        form: "model_document"
                    })
                );
                let paid = if affordable { length } else { 0 };
                assert_eq!(
                    measured.bytes_total, paid,
                    "the denied request may allocate only its prepaid pointer"
                );
                assert_eq!(native.read_storage_consumed(), paid);
                assert_eq!(native.take_prepaid_diagnostic(), Some(affordable));
            }
        }
    }
}

/// The complete public reader charges the ModelDocument pointer once. The
/// paired difference isolates it from independent earlier allocation gaps.
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn budget_denial_model_document_public_pair_omits_unaffordable_pointer() {
    use crate::checked_package::shared::CheckedPackageReadLimits;
    use crate::checked_package::v2::native_join_budget::NativeJoinChargeKind;
    use crate::checked_package::v2::{CheckedPackageV2, CheckedPackageV2BudgetedReadResult};
    crate::checked_package::v2::initialize_operation_catalog().unwrap();
    let value = public_fixture::v2_all_families();
    let evidence = public_fixture::evidence_for(&value);
    let bytes = public_fixture::canonical(&value);
    let limits = CheckedPackageReadLimits {
        bytes: 1 << 30,
        nodes: 1 << 24,
        edges: 1 << 26,
        occurrences: 1 << 26,
        diagnostics: 1 << 24,
        work: u64::MAX,
    };
    let run = |work, storage| {
        let mut budget = NativeJoinBudget::new(work, 0).with_read_storage_limit(storage);
        let mut result = None;
        let measured = allocation_counter::measure(|| {
            result = Some(CheckedPackageV2::read_with_budget(
                &bytes,
                limits,
                &evidence,
                &mut budget,
            ))
        });
        (result.unwrap(), measured.bytes_total, budget)
    };
    let (owned, owned_requests, owned_budget) = run(0, u64::MAX);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(pointer) = owned else {
        panic!("the first work request owns a model-document diagnostic");
    };
    let expected_locus = NativeJoinLocus {
        source: NativeJoinLocusSource::PackageRead(PackageReadStage::ModelDocument, 0),
        form: "model_document",
    };
    assert_eq!(owned_budget.first_unpaid_locus(), Some(expected_locus));
    assert_eq!(
        owned_budget
            .first_unpaid()
            .map(|charge| (charge.kind, charge.amount)),
        Some((NativeJoinChargeKind::Work, 3))
    );
    let path = pointer.path.as_ref().unwrap();
    assert!(
        path.as_str().starts_with("/semantic_graph/nodes/") && path.as_str().ends_with("/owner")
    );
    let root: usize = path.as_str().split('/').nth(3).unwrap().parse().unwrap();
    assert_eq!(
        value["semantic_graph"]["nodes"][root]["semantic_form"],
        "model_root"
    );
    let identity = value["semantic_graph"]["nodes"][root]["owner"]["identity"]
        .as_str()
        .unwrap();
    let pointer_bytes = path.as_str().len() as u64;
    let prefix = owned_budget.read_storage_consumed() - pointer_bytes;
    let (without, without_requests, without_budget) = run(0, prefix);
    let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = without else {
        panic!("the exact storage prefix cannot pay for the pointer");
    };
    assert_eq!(exhausted.denied, pointer.denied);
    assert_eq!(exhausted.locus, expected_locus);
    assert_eq!(without_budget.first_unpaid(), owned_budget.first_unpaid());
    assert_eq!(without_budget.first_unpaid_locus(), Some(expected_locus));
    assert_eq!(without_budget.read_storage_consumed(), prefix);
    assert_eq!(
        owned_requests,
        without_requests + pointer_bytes,
        "unaffordable public diagnostic requests no pointer; earlier allocations cancel"
    );
    let (exact, exact_requests, exact_budget) = run(0, prefix + pointer_bytes);
    assert_eq!(
        exact,
        CheckedPackageV2BudgetedReadResult::BudgetIncomplete(pointer)
    );
    assert_eq!(exact_requests, owned_requests);
    assert_eq!(
        exact_budget.read_storage_consumed(),
        prefix + pointer_bytes,
        "public conversion must not charge the prepaid pointer again"
    );
    // The document reader's own charged total identifies the Arc request
    // immediately after its canonical read and inexact-number scan. This is
    // a bounded, actual-request prefix rather than a storage sweep.
    let mut document_budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
    {
        let mut meter = WorkMeter::over(&mut document_budget);
        let mut selected =
            Budget::new(&mut meter, root, limits.bytes).at(PackageReadStage::ModelDocument, 0);
        drop(
            super::super::document::admit_document(
                identity,
                evidence.domain_package_document(identity),
                &mut selected,
            )
            .unwrap(),
        );
    }
    let arc_bytes = super::super::charged::arc_bytes::<quire_canonical::Document>() as u64;
    assert!(arc_bytes > pointer_bytes);
    let arc_prefix = prefix + document_budget.read_storage_consumed() - arc_bytes;
    let (stored, stored_requests, stored_budget) = run(u64::MAX, arc_prefix + pointer_bytes);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(stored_pointer) = stored else {
        panic!("the selected document Arc request must refuse before allocation");
    };
    assert_eq!(stored_budget.first_unpaid(), None);
    assert_eq!(
        stored_budget.first_unpaid_read_storage(),
        Some(u128::from(arc_bytes))
    );
    assert_eq!(stored_budget.first_unpaid_locus(), Some(expected_locus));
    assert_eq!(
        stored_pointer.path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{root}/owner")
    );
    assert_eq!(
        stored_budget.read_storage_consumed(),
        arc_prefix + pointer_bytes
    );
    let (unstored, unstored_requests, unstored_budget) = run(u64::MAX, arc_prefix);
    let CheckedPackageV2BudgetedReadResult::Exhausted(storage_exhausted) = unstored else {
        panic!("a storage-denial pointer must also be omitted when unaffordable");
    };
    assert_eq!(storage_exhausted.denied, stored_pointer.denied);
    assert_eq!(storage_exhausted.locus, expected_locus);
    assert_eq!(
        unstored_budget.first_unpaid_read_storage(),
        Some(u128::from(arc_bytes))
    );
    assert_eq!(unstored_budget.first_unpaid_locus(), Some(expected_locus));
    assert_eq!(unstored_budget.read_storage_consumed(), arc_prefix);
    assert_eq!(
        stored_requests,
        unstored_requests + pointer_bytes,
        "public storage-denial conversion pays for exactly one pointer"
    );
}
