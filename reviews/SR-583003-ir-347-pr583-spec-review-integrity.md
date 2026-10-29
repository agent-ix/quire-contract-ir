---
id: SR-583003
title: "IR-347 PR #583 spec-review/integrity (cross-artifact outcome status consistency)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; PR #583 against origin/main; diff files spec/core/matrix/tests.md, spec/kani/matrix/tests.md, spec/tests.md, checked against spec/kani/matrix/TC-443-kani-outcome-typed-code.md and spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md; IR-347"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/TC-443
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/STD-001
    type: reviews
---

## Summary

Ticket: IR-347. This integrity sub-analysis checks that the status this PR writes into the
matrices and the index agrees with the per-artifact status prose those rows summarize. The
diff is `origin/main...HEAD` at. The TC-223
artifact and the FR-030 AC-4/AC-5/AC-6 status paragraph already agree with the new rows. The
TC-443 artifact does not, and one FR-030 sentence and the STD-001 status marker are left
inconsistent.

`quire coverage --strict` reports 0 contradicted statuses. It compares status markers with
trace tags. It does not read TC body prose, so it cannot catch FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The kani matrix now marks TC-443 ✅ and says it covers the zero-count error code, but the TC-443 artifact still says "Partly implemented" and that "no such request exists", so the zero-count clause "is not counted as met". Its Description still says FR-030-AC-4 and AC-5 "have no test". The closeout leaves the artifact contradicting its own matrix row, and the test it describes (`tests/it/kani_shared.rs:541-544`) now makes the zero-count `proved` request directly. | spec/kani/matrix/TC-443-kani-outcome-typed-code.md:15-17; spec/kani/matrix/TC-443-kani-outcome-typed-code.md:38-43; spec/kani/matrix/tests.md:25 |
| FND-002 | low | FR-030's Status paragraph ends "This does not implement AC-4, AC-5, the live cancellation producer in AC-7, or downstream codegen adoption." It means the AC-8/AC-9 work, but read beside the new ✅ rows it says AC-4 and AC-5 are unimplemented. Two paragraphs earlier, the same section says TC-223 exercises them. | spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:196-197; spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:178-180 |
| FND-003 | low | The STD-001 row stays 🚧 and the Core index still lists STD-001 as partial. After this edit, every clause of the row says implemented and verified, and the only gap the Core index names is TC-019, which is NFR-001's. Neither row says what keeps STD-001 partial. | spec/core/matrix/tests.md:39; spec/tests.md:12 |

## Verdict

Changes requested before merge. FND-001 should be fixed in this PR: update TC-443's Description
and Status to match the row the PR writes. FND-002 and FND-003 are low and can be fixed in the
same round, by scoping the FR-030 sentence to AC-8/AC-9 and by naming STD-001's remaining gap or
marking it implemented. Neither status direction is an overclaim. FND-001 and FND-002 understate
at the artifact level what the matrix now (correctly) claims. No acceptance criterion is narrowed.

## Dispositions

Round 1. Checked at (equal to
`origin/spec/ir-status-closeout`) against fix diff `..`. The fix diff touches four
Markdown files: `spec/kani/matrix/TC-443-kani-outcome-typed-code.md`,
`spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md`, `spec/core/matrix/tests.md` and
`spec/tests.md`. It changes no code, no AC text and no TC trace tag. The checks were static only:
`quire validate --scope. "spec/**/*.md"` exits 0, with module-registry warnings only, and
`quire coverage --scope. --strict` reports 68 unbacked rows and 0 contradicted statuses, the same
counts as round 0 (all from `origin/main`). `quire matrix` shows FR-030-AC-4/5 tagged in
`tests/it/kani_outcome_tc223.rs` and FR-030-AC-6 tagged at `tests/it/kani_shared.rs:540`. AC-7 is
still untagged and still marked partial.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-443's Status now reads "Implemented" and says the test calls `KaniOutcome::proved(0...)` and checks its `KaniOutcomeError` code. Its Description says TC-223 separately tests AC-4/AC-5, and its Procedure and Expected Results add the zero-count `proved` request. This is re-measured at `tests/it/kani_shared.rs:541-544`: the test calls `proved(0...)`, `expect_err`s it and asserts `code == KANI_OUTCOME_INVALID`, `requested_kind == Proved` and `rejected_success_checks == Some(0)`. `src/kani/outcome.rs:237-242` returns that error, and `:75-77` fixes the error code. Every clause of FR-030-AC-6 (the vacuous code and wire member, the zero-count error code, the `&str`/`String` compile-fail doctests on `non_success`, and the `"Bad-Code"` refusal) has a matching assertion or doctest. The artifact, the kani matrix row and FR-030's Status now agree. |
| FND-002 | fixed | The sentence now reads "The AC-8/AC-9 implementation does not supply the live cancellation producer in AC-7 or downstream codegen adoption." It no longer names AC-4 or AC-5, and it agrees with the earlier paragraph saying TC-223 exercises them. AC-7 still "remains incomplete" (IR-701), so the edit does not upgrade AC-7. |
| FND-003 | fixed | The STD-001 row is now ✅, and the Core index says "STD-001 is implemented" instead of listing it as partial. The Core row stays 🚧 for FR-045/TC-446, TC-019 and the other named gaps. The upgrade was re-measured, not taken on trust. STD-001 has no AC rows. TC-017 and TC-018, which the row cites, are still 🚧, but their open items are FR-016-AC-1/AC-3, FR-017-AC-2 and FR-019 boundary obligations, not registry obligations. The registry's own conditions have evidence: TC-454 (`tests/it/nfr003_invalid_input.rs:339-354`) ties STD-001's semantic code tables to `DiagnosticCode::ALL` and drives every class through the corpus. TC-456 reconciles all seven registry tables. The over-limit coverage code `semantic_input_too_large` is asserted at `tests/it/canonicalization.rs:1031-1041`. The Kani codes are covered by TC-442, TC-223 and TC-443. No overclaim was found. |

Earlier clean results were re-checked at this head. SR-583001 (code-review) and SR-583002 (base
checklist) stay clean. AD-001 and AD-006 are unchanged since. No requirement or AC text
changed across `origin/main...HEAD`. FR-030-AC-7, TC-449, FR-029-AC-2 and FR-036 stay partial or
planned, and nothing calls IR-347 complete. No hash, pin, shim or cross-repo copy is added. No new
findings.

Verdict after round 1: all three findings are fixed, and this review method has no objection to
merge.

## Coverage

```yaml
scope:
- id: TC-443
  path: spec/kani/matrix/TC-443-kani-outcome-typed-code.md
  role: examined
  excerpt: "Partly implemented: `tc_443_the_outcome_and_its_error_carry_a_typed_std001_code` in `tests/it/kani_shared.rs`... FR-030-AC-6's clause that reads the error's code for a `proved` request with a count of zero is planned with FR-030-AC-4: no such request exists, so the test reads the error through the non-success request for a `proved` outcome, and the clause is not counted as met."
- id: TC-223
  path: spec/kani/matrix/TC-223-kani-outcome-fr331-result-map.md
  role: examined
  excerpt: "Implemented by the TC-223 tagged count, cause and wire controls plus the `KaniOutcome` compile-fail probe."
- id: FR-030
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: "AC-4 and AC-5 are exercised by TC-223: `KaniOutcome`'s fields are private, `proved` retains a nonzero SUCCESS-check count, and the broad constructor and wire reader reject invalid fixed causes and counts.... This does not implement AC-4, AC-5, the live cancellation producer in AC-7, or downstream codegen adoption."
- id: kani-matrix:TC-443
  path: spec/kani/matrix/tests.md
  role: examined
  excerpt: "✅ `tc_443_the_outcome_and_its_error_carry_a_typed_std001_code` in `tests/it/kani_shared.rs` covers the zero-count error code, vacuous-proof code and wire member, malformed-code refusal and Boolean-kind errors"
- id: core-matrix:STD-001
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "🚧 closed code catalogs implemented and verified by TC-015 through TC-018; the typed `Std001Code`, the registered Kani cause codes and `invalid_code_form` are implemented and verified by TC-442; `kani_outcome_invalid` is raised for zero-count `proved` requests and invalid broad non-success requests, verified by TC-223 and TC-443 (FR-030-AC-4 through AC-6)"
- id: tests-index:Core
  path: spec/tests.md
  role: examined
  excerpt: "StR-001, StR-003, NFR-001..NFR-003 and STD-001 are partial: cross-platform determinism (TC-019) is planned, while TC-223 covers the `kani_outcome_invalid` raise sites in `src/kani/outcome.rs`"
- id: TC-443:round1
  path: spec/kani/matrix/TC-443-kani-outcome-typed-code.md
  role: examined
  excerpt: "Implemented: `tc_443_the_outcome_and_its_error_carry_a_typed_std001_code` in `tests/it/kani_shared.rs`... The test calls `KaniOutcome::proved(0...)` and checks its `KaniOutcomeError` code, as well as the zero-check outcome code, serialization, malformed-code refusal and broad-constructor errors."
- id: tc443-test:round1
  path: tests/it/kani_shared.rs
  role: examined
  excerpt: "let zero = KaniOutcome::proved(0, \"clause:demo\", \"profile\").expect_err(\"zero is not proved\"); assert_eq!(zero.code, Std001Code::KANI_OUTCOME_INVALID);"
- id: FR-030:round1
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: examined
  excerpt: "The AC-8/AC-9 implementation does not supply the live cancellation producer in AC-7 or downstream codegen adoption."
- id: FR-030-AC-6
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: context_only
  excerpt: "the `code` of the `KaniOutcomeError` for a `proved` outcome with a count of zero equals `Std001Code::KANI_OUTCOME_INVALID`"
- id: core-matrix:STD-001:round1
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "✅ closed code catalogs implemented and verified by TC-015 through TC-018; the typed `Std001Code`, the registered Kani cause codes and `invalid_code_form` are implemented and verified by TC-442"
- id: tests-index:Core:round1
  path: spec/tests.md
  role: examined
  excerpt: "StR-001, StR-003 and NFR-001..NFR-003 are partial: cross-platform determinism (TC-019) is planned; STD-001 is implemented, with TC-223 covering the `kani_outcome_invalid` raise sites in `src/kani/outcome.rs`"
- id: tc454-registry-tie
  path: tests/it/nfr003_invalid_input.rs
  role: context_only
  excerpt: "STD-001's semantic code tables and DiagnosticCode::ALL must name the same classes"
```
