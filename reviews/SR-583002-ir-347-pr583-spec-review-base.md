---
id: SR-583002
title: "IR-347 PR #583 spec-review/base (Kani ownership and outcome status closeout)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #583 against origin/main; spec/assurance/AD-001-contract-ir-architecture.md, spec/assurance/AD-006-codegen-consumption-seam.md, spec/core/matrix/tests.md, spec/kani/matrix/tests.md, spec/tests.md; IR-347"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/AD-006
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: reviews
---

## Summary

Ticket: IR-347. This base spec-review covers the five Markdown files in
`git diff origin/main...HEAD`.

Review set: base plus the integrity sub-analysis (SR-583003). No `AssuranceProfile` exists under
`spec/`, so no profile requires an enforced spec-review run. The diff adds no requirement
statement, relationship edge or domain object, and it changes no acceptance criterion. EARS,
dependency, object, evidence, failure-domain, risk and scope-boundary analyses therefore do not
apply.

Checks:

- No requirement or AC text changed. FR-030-AC-4, AC-5 and AC-6 at head are unchanged from
  `origin/main`, so no criterion is narrowed.
- AD-001's `kani` row no longer claims the Kani-outcome-to-QSL terminal-value map. The rest of
  `spec/` already names codegen as that map's owner (AD-001 lines 105, 115 and 182; AD-002; FR-031;
  FR-039; TC-055). No residual IR ownership claim remains.
- AD-006 G-3, G-4 and seam item 4 match the code (see SR-583001).
- The FR-030 matrix row keeps AC-7 partial, with CG IR-701 as owner. TC-449 stays 🚧. FR-029-AC-2
  and FR-036 stay planned. The ✅ marks on TC-223 and TC-443 match tagged tests. No status is
  raised beyond the evidence.
- No hash, digest, pin or tracking record is introduced.
- Quire: `quire validate --scope .` over the five files, and over `spec/**`, `plan/**` and
  `reviews/**`, passes (1430/1430 docs grammar-clean; module registry warnings only).
  `quire coverage --scope . --strict` reports 68 unbacked rows and 0 contradicted statuses.
  Its seven dangling `TC-947` traces in `tests/it/checked_package_v2_protocol_body.rs` come from
  `origin/main`, not from this diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean on the base checklist. The edits correct AD-001 ownership and bring the four status
surfaces in line with the code. Requirements and ACs are intact, and nothing claims IR-347 is
complete. The cross-artifact status gaps the diff leaves are recorded under SR-583003.

## Coverage

```yaml
scope:
- id: AD-001:kani-module-row
  path: spec/assurance/AD-001-contract-ir-architecture.md
  role: examined
  excerpt: "| `kani` | the `kani-bounded/1` profile, finite input ABI, dispatch index and typed outcome | FR-029 through FR-031 |"
- id: AD-006:G-3
  path: spec/assurance/AD-006-codegen-consumption-seam.md
  role: examined
  excerpt: "A struct-literal `KaniOutcome` outside the `kani` module does not compile (FR-030-AC-5; implemented: its fields are private, with a `compile_fail` doctest in `src/kani/outcome.rs`)."
- id: AD-006:G-4
  path: spec/assurance/AD-006-codegen-consumption-seam.md
  role: examined
  excerpt: "`Proved` carries a check count and a request with count zero returns `KaniOutcomeError` (FR-030-AC-4; implemented by `proved` and verified by tagged TC-223 tests; `non_success` separately refuses a `Proved` or `Counterexample` request)."
- id: AD-006:seam-item-4
  path: spec/assurance/AD-006-codegen-consumption-seam.md
  role: examined
  excerpt: "FR-030-AC-4 and AC-5 are implemented and backed by tagged TC-223 tests for the count, fixed causes and invalid constructor requests; the private-field compile-fail probe is in `src/kani/outcome.rs`."
- id: core-matrix:StR-001
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "the typed `Std001Code` (FR-044) and its use as `KaniOutcome.code` (FR-030-AC-6, TC-443) are implemented, including the `proved`-with-count-zero error clause; the proof check count and two fixed `Unavailable` causes are implemented and backed by TC-223 (FR-030-AC-4/5); provider negotiation and the FR-344 refusals are planned"
- id: core-matrix:STD-001
  path: spec/core/matrix/tests.md
  role: examined
  excerpt: "`kani_outcome_invalid` is raised for zero-count `proved` requests and invalid broad non-success requests, verified by TC-223 and TC-443 (FR-030-AC-4 through AC-6)"
- id: kani-matrix:FR-030
  path: spec/kani/matrix/tests.md
  role: examined
  excerpt: "AC-4 and AC-5 implemented and verified by tagged TC-223 constructor, count, cause and wire-reader tests, with the private-field compile-fail probe on `KaniOutcome`; AC-6 implemented and verified by TC-443 ...; AC-7 has TC-449 IR carrier tests, while CG's live cancellation-event reversal control remains planned under IR-701"
- id: kani-matrix:TC-223
  path: spec/kani/matrix/tests.md
  role: examined
  excerpt: "✅ tagged tests in `tests/it/kani_outcome_tc223.rs` cover counts, fixed causes, broad-constructor refusals and wire-reader validation; `KaniOutcome`'s private fields have a `compile_fail` doctest"
- id: kani-matrix:TC-443
  path: spec/kani/matrix/tests.md
  role: examined
  excerpt: "✅ `tc_443_the_outcome_and_its_error_carry_a_typed_std001_code` in `tests/it/kani_shared.rs` covers the zero-count error code, vacuous-proof code and wire member, malformed-code refusal and Boolean-kind errors"
- id: tests-index:Core
  path: spec/tests.md
  role: examined
  excerpt: "cross-platform determinism (TC-019) is planned, while TC-223 covers the `kani_outcome_invalid` raise sites in `src/kani/outcome.rs`"
- id: tests-index:Kani
  path: spec/tests.md
  role: examined
  excerpt: "FR-029-AC-2 and FR-036 (TC-045) planned; FR-030-AC-4 and AC-5 are implemented with tagged TC-223 tests, and AC-6 is implemented with TC-443 and `compile_fail` doctests; FR-030-AC-7 retains the planned CG live cancellation-event reversal control"
- id: FR-030-AC-4
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: context_only
  excerpt: "A `proved` outcome built from a run with three SUCCESS checks carries the count three; a run with zero SUCCESS checks yields `inconclusive` with cause `kani_vacuous_proof`, carries no count and no Boolean claim; and a request for a `proved` outcome with a count of zero returns `KaniOutcomeError` with code `kani_outcome_invalid` and no outcome."
- id: FR-030-AC-5
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: context_only
  excerpt: "An `unavailable` outcome built for an absent checker carries cause `kani_checker_absent` and one built for an absent backend carries `kani_backend_absent` ...; and a struct-literal `KaniOutcome` outside the `kani` module fails to compile."
- id: FR-030-AC-6
  path: spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md
  role: context_only
  excerpt: "the `code` of the `KaniOutcomeError` for a `proved` outcome with a count of zero equals `Std001Code::KANI_OUTCOME_INVALID`; a call that passes a `&str` or a `String` as the code of the non-success constructor fails to compile; and a serialized `KaniOutcome` whose `code` is `\"Bad-Code\"` fails to deserialize."
```
