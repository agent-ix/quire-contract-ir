---
id: SR-13541
title: "IR-881 PR #485 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/code.rs; tests/it/identity.rs; tests/it/public_interface.rs; tests/it/std001_code.rs; spec/core/functional/FR-044-typed-std001-code.md; spec/core/non-functional/NFR-002-portability-independence.md"
review_set: subset
---

## Summary

Ticket: IR-881. Compared changed obligations with computed matrix binders, test assertions, and source. FR-044-AC-3, NFR-002-AC-1, and NFR-002-AC-3 are tagged; FR-030-AC-5 remains explicitly planned and untagged before this PR.

## Scope

Examined: FR-044-AC-3; NFR-002-AC-1; NFR-002-AC-3; FR-030-AC-5. PR PR diff.

## Examined units

- FR-044-AC-3 (examined, spec/core/functional/FR-044-typed-std001-code.md): "`Std001Code::REGISTERED` equals, in order, the fifteen codes `invalid_code_form`, `kani_backend_absent`, `kani_bound_exhausted`, `kani_bound_invalid`, `kani_capability_missing`, `kani_capability_request_invalid`, `kani_checker_absent`, `kani_counterexample`, `kani_identity_invalid`, `kani_outcome_invalid`, `kani_population_incomplete`, `kani_population_invalid`, `kani_proved`, `kani_reference_invalid`, `kani_vacuous_proof`; each constant's `as_str` equals its lower-case name and passes `Std001Code::new`; every `DiagnosticCode::ALL` entry converts to a code whose `as_str` equals `Diag"
- NFR-002-AC-1 (examined, spec/core/non-functional/NFR-002-portability-independence.md): "Public schema and model names contain no Rust-layout, architecture-language, solver, runtime, pointer-width, or operating-system-path vocabulary."
- NFR-002-AC-3 (examined, spec/core/non-functional/NFR-002-portability-independence.md): "Public JSON field names, identity kinds, anchors, clause kinds, dependency kinds, and diagnostic codes contain no Rust, GUMBO, AADL, HAMR, solver, runtime, pointer-width, or operating-system-path vocabulary."
- FR-030-AC-5 (context_only, spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md): "An `unavailable` outcome built for an absent checker carries cause `kani_checker_absent` and one built for an absent backend carries `kani_backend_absent`, each with no Boolean claim; a request for an `unavailable` outcome with any other cause code, an `inconclusive` outcome with a cause other than `kani_vacuous_proof`, or a `proved` or `counterexample` outcome through the non-success constructor returns `KaniOutcomeError` with code `kani_outcome_invalid` and no outcome; and a struct-literal `KaniOutcome` outside the `kani` module fails to compile."

## Verdict

**PASS** — Compared changed obligations with computed matrix binders, test assertions, and source. FR-044-AC-3, NFR-002-AC-1, and NFR-002-AC-3 are tagged; FR-030-AC-5 remains explicitly planned and untagged before this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. Review limited to PR #485's changed obligations and associated tests.
