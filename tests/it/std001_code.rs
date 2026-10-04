//! `Std001Code` (FR-044, TC-442). The compile-time refusals of `std001_code!`
//! are `compile_fail` doctests on the model crate's `code` module, the form the
//! model crate's other compile probes use, paired there with the passing
//! `const` form.

use ix_trace_rs::trace;
use quire_contract_model::{std001_code, DiagnosticCode, Std001Code, Std001CodeError};

const MACRO_CODE: Std001Code = std001_code!("kani_corpus_identity_collision");
const FROM_STATIC_OK: Result<Std001Code, Std001CodeError> =
    Std001Code::from_static("kani_vacuous_proof");
const FROM_STATIC_EMPTY: Result<Std001Code, Std001CodeError> = Std001Code::from_static("");
const FROM_STATIC_UPPER: Result<Std001Code, Std001CodeError> = Std001Code::from_static("Kani_x");
/// 65 bytes of `a`: one past the longest code.
const LONG: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const _: () = assert!(LONG.len() == 65);
const FROM_STATIC_LONG: Result<Std001Code, Std001CodeError> = Std001Code::from_static(LONG);

/// The fifteen codes FR-044-AC-3 lists, in order, written out.
const REGISTERED: [&str; 15] = [
    "invalid_code_form",
    "kani_backend_absent",
    "kani_bound_exhausted",
    "kani_bound_invalid",
    "kani_capability_missing",
    "kani_capability_request_invalid",
    "kani_counterexample",
    "kani_identity_invalid",
    "kani_outcome_invalid",
    "kani_population_incomplete",
    "kani_population_invalid",
    "kani_proved",
    "kani_reference_invalid",
    "kani_solver_absent",
    "kani_vacuous_proof",
];

fn refused(input: &str) -> Std001CodeError {
    match Std001Code::new(input) {
        Ok(code) => panic!("{:?} is not a code but built {code}", input.len()),
        Err(error) => error,
    }
}

/// Tracing: TC-442, FR-044-AC-1
#[trace("TC-442", "FR-044-AC-1")]
#[test]
fn tc_442_new_accepts_the_form_and_refuses_everything_else() {
    for valid in ["kani_vacuous_proof", &"a".repeat(64)] {
        let code = Std001Code::new(valid).expect("a well-formed code");
        assert_eq!(code.as_str(), valid);
        assert_eq!(code.to_string(), valid);
    }
    let invalid = [
        String::new(),
        "a".repeat(65),
        "Kani_x".into(),
        "kani-x".into(),
        "_x".into(),
        "x_".into(),
        "x__y".into(),
        "1x".into(),
        "kani x".into(),
        "kan\u{ef}".into(),
        "a".repeat(1_048_576),
    ];
    for input in &invalid {
        let error = refused(input);
        assert_eq!(error.code(), Std001Code::INVALID_CODE_FORM);
        assert_eq!(error.code().as_str(), "invalid_code_form");
    }
}

/// Tracing: TC-442, FR-044-AC-2
#[trace("TC-442", "FR-044-AC-2")]
#[test]
fn tc_442_serializes_as_the_bare_string_and_deserializes_through_new() {
    let text = serde_json::to_string(&Std001Code::KANI_VACUOUS_PROOF).expect("serializes");
    assert_eq!(text, "\"kani_vacuous_proof\"");
    let back: Std001Code = serde_json::from_str(&text).expect("a well-formed code reads");
    assert_eq!(back, Std001Code::KANI_VACUOUS_PROOF);
    for refused in ["\"Bad\"", "\"\"", "7", "null"] {
        assert!(
            serde_json::from_str::<Std001Code>(refused).is_err(),
            "{refused} must not deserialize"
        );
    }
}

/// Tracing: TC-442, FR-044-AC-3
#[trace("TC-442", "FR-044-AC-3")]
#[test]
fn tc_442_registered_codes_are_spelled_once_and_diagnostic_codes_convert() {
    assert_eq!(Std001Code::REGISTERED, REGISTERED.as_slice());
    let constants = [
        ("INVALID_CODE_FORM", Std001Code::INVALID_CODE_FORM),
        ("KANI_BACKEND_ABSENT", Std001Code::KANI_BACKEND_ABSENT),
        ("KANI_BOUND_EXHAUSTED", Std001Code::KANI_BOUND_EXHAUSTED),
        ("KANI_BOUND_INVALID", Std001Code::KANI_BOUND_INVALID),
        (
            "KANI_CAPABILITY_MISSING",
            Std001Code::KANI_CAPABILITY_MISSING,
        ),
        (
            "KANI_CAPABILITY_REQUEST_INVALID",
            Std001Code::KANI_CAPABILITY_REQUEST_INVALID,
        ),
        ("KANI_COUNTEREXAMPLE", Std001Code::KANI_COUNTEREXAMPLE),
        ("KANI_IDENTITY_INVALID", Std001Code::KANI_IDENTITY_INVALID),
        ("KANI_OUTCOME_INVALID", Std001Code::KANI_OUTCOME_INVALID),
        (
            "KANI_POPULATION_INCOMPLETE",
            Std001Code::KANI_POPULATION_INCOMPLETE,
        ),
        (
            "KANI_POPULATION_INVALID",
            Std001Code::KANI_POPULATION_INVALID,
        ),
        ("KANI_PROVED", Std001Code::KANI_PROVED),
        ("KANI_REFERENCE_INVALID", Std001Code::KANI_REFERENCE_INVALID),
        ("KANI_SOLVER_ABSENT", Std001Code::KANI_SOLVER_ABSENT),
        ("KANI_VACUOUS_PROOF", Std001Code::KANI_VACUOUS_PROOF),
    ];
    assert_eq!(constants.len(), Std001Code::REGISTERED.len());
    for (name, constant) in &constants {
        assert_eq!(constant.as_str(), name.to_lowercase());
        assert!(
            Std001Code::REGISTERED.contains(&constant.as_str()),
            "{name}"
        );
        assert_eq!(Std001Code::new(constant.as_str()).as_ref(), Ok(constant));
        assert!(constant.is_registered(), "{name}");
    }
    for diagnostic in DiagnosticCode::ALL {
        let code = Std001Code::from(*diagnostic);
        assert_eq!(code.as_str(), diagnostic.as_str());
        assert!(code.is_registered(), "{diagnostic:?}");
    }
    let unregistered = Std001Code::new("kani_corpus_identity_collision")
        .expect("a well-formed code is a valid Std001Code");
    assert!(!unregistered.is_registered());
}

/// Tracing: TC-442, FR-044-AC-4
#[trace("TC-442", "FR-044-AC-4")]
#[test]
fn tc_442_the_macro_builds_a_const_code_from_a_literal() {
    assert_eq!(MACRO_CODE.as_str(), "kani_corpus_identity_collision");
    assert!(!MACRO_CODE.is_registered());
}

/// Tracing: TC-442, FR-044-AC-5
#[trace("TC-442", "FR-044-AC-5")]
#[test]
fn tc_442_from_static_is_a_const_new() {
    assert_eq!(
        FROM_STATIC_OK,
        Ok(Std001Code::new("kani_vacuous_proof").expect("a well-formed code"))
    );
    for refused in [FROM_STATIC_EMPTY, FROM_STATIC_UPPER, FROM_STATIC_LONG] {
        let error = refused.expect_err("not a well-formed code");
        assert_eq!(error.code(), Std001Code::INVALID_CODE_FORM);
    }
}
