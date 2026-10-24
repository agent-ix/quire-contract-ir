//! The count and cause rules of the public bounded-Kani outcome.

use ix_trace_rs::trace;
use quire_contract_ir::kani::{KaniCancellationSource, KaniOutcome, KaniOutcomeKind};
use quire_contract_model::{std001_code, Std001Code};

#[trace("TC-223", "FR-030-AC-4")]
#[test]
fn proof_count_is_retained_and_zero_is_refused_or_classified_vacuous() {
    let proved = KaniOutcome::proved_from_checks(3, "clause:demo", "profile");
    assert_eq!(proved.kind(), KaniOutcomeKind::Proved);
    assert_eq!(proved.success_checks(), Some(3));
    assert_eq!(proved.boolean_claim(), Some(true));
    let proved_wire = serde_json::to_value(&proved).expect("wire");
    assert_eq!(proved_wire["success_checks"], 3);
    assert_eq!(
        serde_json::from_value::<KaniOutcome>(proved_wire).expect("read"),
        proved
    );

    let vacuous = KaniOutcome::proved_from_checks(0, "clause:demo", "profile");
    assert_eq!(vacuous.kind(), KaniOutcomeKind::Inconclusive);
    assert_eq!(vacuous.code(), Std001Code::KANI_VACUOUS_PROOF);
    assert_eq!(vacuous.success_checks(), None);
    assert_eq!(vacuous.boolean_claim(), None);
    assert!(serde_json::to_value(&vacuous)
        .expect("wire")
        .get("success_checks")
        .is_none());

    let error = KaniOutcome::proved(0, "clause:demo", "profile").expect_err("zero is not proof");
    assert_eq!(error.code(), Std001Code::KANI_OUTCOME_INVALID);
    assert_eq!(error.requested_kind(), &KaniOutcomeKind::Proved);
    assert_eq!(error.rejected_success_checks(), Some(0));
}

#[trace("TC-223", "FR-030-AC-5")]
#[test]
fn broad_non_success_rejects_wrong_fixed_causes_and_boolean_kinds() {
    let checker = KaniOutcome::checker_unavailable("s", "p");
    let backend = KaniOutcome::backend_unavailable("s", "p");
    assert_eq!(checker.code(), Std001Code::KANI_CHECKER_ABSENT);
    assert_eq!(backend.code(), Std001Code::KANI_BACKEND_ABSENT);
    assert_eq!(checker.boolean_claim(), None);
    assert_eq!(backend.boolean_claim(), None);

    for (kind, code) in [
        (KaniOutcomeKind::Unavailable, std001_code!("wrong_absence")),
        (
            KaniOutcomeKind::Inconclusive,
            std001_code!("wrong_inconclusive"),
        ),
        (KaniOutcomeKind::Proved, Std001Code::KANI_PROVED),
        (
            KaniOutcomeKind::Counterexample,
            Std001Code::KANI_COUNTEREXAMPLE,
        ),
    ] {
        let error = KaniOutcome::non_success(kind, code, "s", "p").expect_err("invalid cause");
        assert_eq!(error.code(), Std001Code::KANI_OUTCOME_INVALID);
        assert_eq!(error.requested_kind(), &kind);
        assert_eq!(error.requested_code(), code);
    }
    for (kind, code) in [
        (KaniOutcomeKind::Refused, std001_code!("refusal")),
        (KaniOutcomeKind::InvalidInput, std001_code!("invalid")),
        (KaniOutcomeKind::IncompleteInput, std001_code!("incomplete")),
        (KaniOutcomeKind::TimedOut, std001_code!("timeout")),
        (KaniOutcomeKind::ResourceExhausted, std001_code!("limit")),
        (
            KaniOutcomeKind::Cancelled {
                source: KaniCancellationSource::Requested,
            },
            std001_code!("cancelled"),
        ),
    ] {
        let outcome = KaniOutcome::non_success(kind, code, "s", "p").expect("non-success");
        assert_eq!(outcome.kind(), kind);
        assert_eq!(outcome.code(), code);
        assert_eq!(outcome.boolean_claim(), None);
    }
}

#[trace("TC-223", "FR-030-AC-5")]
#[test]
fn outcome_kind_match_is_exhaustive() {
    fn spelling(kind: KaniOutcomeKind) -> &'static str {
        match kind {
            KaniOutcomeKind::Proved => "proved",
            KaniOutcomeKind::Counterexample => "counterexample",
            KaniOutcomeKind::Refused => "refused",
            KaniOutcomeKind::InvalidInput => "invalid_input",
            KaniOutcomeKind::IncompleteInput => "incomplete_input",
            KaniOutcomeKind::Unavailable => "unavailable",
            KaniOutcomeKind::TimedOut => "timed_out",
            KaniOutcomeKind::ResourceExhausted => "resource_exhausted",
            KaniOutcomeKind::Cancelled { .. } => "cancelled",
            KaniOutcomeKind::Inconclusive => "inconclusive",
        }
    }
    assert_eq!(
        spelling(KaniOutcome::checker_unavailable("s", "p").kind()),
        "unavailable"
    );
}

#[trace("TC-223", "FR-030-AC-4", "FR-030-AC-5")]
#[test]
fn wire_reader_cannot_bypass_count_or_fixed_cause_rules() {
    for wire in [
        serde_json::json!({"kind":"proved","code":"kani_proved","source_id":"s","context":"p","success_checks":0}),
        serde_json::json!({"kind":"proved","code":"kani_proved","source_id":"s","context":"p"}),
        serde_json::json!({"kind":"unavailable","code":"wrong_absence","source_id":"s","context":"p"}),
        serde_json::json!({"kind":"inconclusive","code":"wrong_inconclusive","source_id":"s","context":"p"}),
    ] {
        assert!(
            serde_json::from_value::<KaniOutcome>(wire.clone()).is_err(),
            "accepted {wire}"
        );
    }
}
