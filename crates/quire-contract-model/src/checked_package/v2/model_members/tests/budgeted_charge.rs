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
