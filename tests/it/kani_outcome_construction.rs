//! External-consumer checks for the infallible non-success outcome API.

use ix_trace_rs::trace;
use quire_contract_ir::kani::{KaniCancellationSource, KaniOutcome, KaniOutcomeKind};
use quire_contract_model::{std001_code, Std001Code};

fn assert_round_trip(
    outcome: &KaniOutcome,
    kind: KaniOutcomeKind,
    code: Std001Code,
    source: &str,
    context: &str,
) {
    assert_eq!(outcome.kind, kind);
    assert_eq!(outcome.code, code);
    assert_eq!(outcome.source_id, source);
    assert_eq!(outcome.context, context);
    assert_eq!(outcome.boolean_claim(), None);

    let wire = serde_json::to_value(outcome).expect("serialize the public outcome");
    assert_eq!(wire["code"], code.as_str());
    assert_eq!(wire["source_id"], source);
    assert_eq!(wire["context"], context);
    let expected_kind = match kind {
        KaniOutcomeKind::Refused => serde_json::json!("refused"),
        KaniOutcomeKind::InvalidInput => serde_json::json!("invalid_input"),
        KaniOutcomeKind::IncompleteInput => serde_json::json!("incomplete_input"),
        KaniOutcomeKind::TimedOut => serde_json::json!("timed_out"),
        KaniOutcomeKind::ResourceExhausted => serde_json::json!("resource_exhausted"),
        KaniOutcomeKind::Cancelled { source: KaniCancellationSource::Requested } => {
            serde_json::json!({"cancelled": {"source": "requested"}})
        }
        KaniOutcomeKind::Cancelled { source: KaniCancellationSource::Deadline } => {
            serde_json::json!({"cancelled": {"source": "deadline"}})
        }
        KaniOutcomeKind::Unavailable => serde_json::json!("unavailable"),
        KaniOutcomeKind::Inconclusive => serde_json::json!("inconclusive"),
        KaniOutcomeKind::Proved | KaniOutcomeKind::Counterexample => {
            panic!("the infallible non-success route returned a Boolean outcome")
        }
    };
    assert_eq!(wire["kind"], expected_kind);
    let decoded: KaniOutcome = serde_json::from_value(wire).expect("read the same wire envelope");
    assert_eq!(&decoded, outcome);
    assert_eq!(decoded.boolean_claim(), None);
}

#[trace("FR-030-AC-8")]
#[test]
fn coded_constructors_retain_consumer_causes_and_exact_context() {
    let code = std001_code!("consumer_population_stop");
    assert!(!code.is_registered());
    let refused: KaniOutcome = KaniOutcome::refused(code, "source:refused", "kani-bounded/1; bound=1");
    let invalid: KaniOutcome = KaniOutcome::invalid_input(code, String::from("source:invalid"), String::from("kani-bounded/1; bound=2"));
    let incomplete: KaniOutcome = KaniOutcome::incomplete_input(code, "source:incomplete", "kani-bounded/1; bound=3");
    let timed_out: KaniOutcome = KaniOutcome::timed_out(code, "source:timeout", "kani-bounded/1; bound=4");
    let exhausted: KaniOutcome = KaniOutcome::resource_exhausted(code, "source:exhausted", "kani-bounded/1; bound=5");
    for (outcome, kind, source, context) in [
        (refused, KaniOutcomeKind::Refused, "source:refused", "kani-bounded/1; bound=1"),
        (invalid, KaniOutcomeKind::InvalidInput, "source:invalid", "kani-bounded/1; bound=2"),
        (incomplete, KaniOutcomeKind::IncompleteInput, "source:incomplete", "kani-bounded/1; bound=3"),
        (timed_out, KaniOutcomeKind::TimedOut, "source:timeout", "kani-bounded/1; bound=4"),
        (exhausted, KaniOutcomeKind::ResourceExhausted, "source:exhausted", "kani-bounded/1; bound=5"),
    ] {
        assert_round_trip(&outcome, kind, code, source, context);
        assert!(!outcome.code.is_registered());
    }
    for (source, source_id, context) in [
        (KaniCancellationSource::Requested, "source:requested", "kani-bounded/1; bound=6"),
        (KaniCancellationSource::Deadline, "source:deadline", "kani-bounded/1; bound=7"),
    ] {
        let outcome: KaniOutcome = KaniOutcome::cancelled(source, code, source_id, context);
        assert_round_trip(&outcome, KaniOutcomeKind::Cancelled { source }, code, source_id, context);
        assert!(!outcome.code.is_registered());
    }
}

#[trace("FR-030-AC-8")]
#[test]
fn fixed_cause_constructors_retain_the_registered_cause_and_context() {
    let checker: KaniOutcome = KaniOutcome::checker_unavailable("source:checker", "kani-bounded/1; bound=8");
    let backend: KaniOutcome = KaniOutcome::backend_unavailable("source:backend", "kani-bounded/1; bound=9");
    let vacuous: KaniOutcome = KaniOutcome::vacuous_proof("source:zero-checks", "kani-bounded/1; bound=10");
    for (outcome, kind, code, source, context) in [
        (checker, KaniOutcomeKind::Unavailable, Std001Code::KANI_CHECKER_ABSENT, "source:checker", "kani-bounded/1; bound=8"),
        (backend, KaniOutcomeKind::Unavailable, Std001Code::KANI_BACKEND_ABSENT, "source:backend", "kani-bounded/1; bound=9"),
        (vacuous, KaniOutcomeKind::Inconclusive, Std001Code::KANI_VACUOUS_PROOF, "source:zero-checks", "kani-bounded/1; bound=10"),
    ] {
        assert_round_trip(&outcome, kind, code, source, context);
        assert!(outcome.code.is_registered());
    }
}

/// The argument-type and arity compile-fail probes, paired with passing public
/// calls, are doctests on the nine `KaniOutcome` infallible constructors.
#[trace("FR-030-AC-9")]
#[test]
fn broad_fallible_constructor_still_rejects_both_boolean_kinds() {
    let code = std001_code!("consumer_boolean_refusal");
    for kind in [KaniOutcomeKind::Proved, KaniOutcomeKind::Counterexample] {
        let error = KaniOutcome::non_success(kind.clone(), code, "source:boolean", "kani-bounded/1; bound=11")
            .expect_err("a Boolean outcome cannot use the non-success route");
        assert_eq!(error.code(), Std001Code::KANI_OUTCOME_INVALID);
        assert_eq!(error.requested_kind(), &kind);
        assert_eq!(error.requested_code(), code);
    }
}
