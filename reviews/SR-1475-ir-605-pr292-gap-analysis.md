---
id: SR-1475
title: "gap analysis of PR 292 (IR-605 typed STD-001 code as the Kani outcome code)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@fa67cf58512e2908256eef029345aa4989abcaac; spec/core/functional/FR-044-typed-std001-code.md, spec/core/functional/STD-001-diagnostic-registry.md, spec/core/matrix/TC-442-typed-std001-code.md, spec/core/matrix/tests.md, spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md, spec/kani/matrix/TC-223-kani-outcome-fr331-result-map.md, spec/kani/matrix/tests.md, spec/tests.md, spec/assurance/AD-006-codegen-consumption-seam.md, crates/quire-contract-model/src/code.rs, src/kani/outcome.rs, tests/it/std001_code.rs, tests/it/kani_shared.rs"
review_set: subset
---
# SR-1475: gap analysis of PR 292

## Summary

Ticket: IR-605. Plan completion: not assessed. This review checks the merged spec (#291,
7e4dc54579e6d859e7fa224a63a15c4aeec4580d) against the code and tests at
fa67cf58512e2908256eef029345aa4989abcaac. It covers FR-044-AC-1 through AC-5, FR-030-AC-6,
TC-442, TC-223 and the STD-001 rows the PR touches. For each test, it asks whether the
test would fail on a plausible mutant.

| AC | Test | Result | Mutant check |
| --- | --- | --- | --- |
| FR-044-AC-1 | `tc_442_new_accepts_the_form_and_refuses_everything_else` | all 13 inputs as the AC lists, including the 64-byte accept and the 1 MiB refuse | dropping the trailing-`_`, `__`, leading-digit, leading-`_` or length check, or a limit of 63, each fails a named input |
| FR-044-AC-2 | `tc_442_serializes_as_the_bare_string_and_deserializes_through_new` | exact bytes `"kani_vacuous_proof"`; round-trip; `"Bad"`, `""`, `7`, `null` refused | a `String`-backed non-validating `Deserialize` fails on `"Bad"`/`""` |
| FR-044-AC-3 | `tc_442_registered_codes_are_spelled_once_and_diagnostic_codes_convert` | `REGISTERED` equals the 15 written-out codes in order; each constant equals its lower-cased name; every `DiagnosticCode::ALL` converts unchanged and is registered; `kani_corpus_identity_collision` is not | a missing, misspelled or unsorted code, an always-true or `DiagnosticCode`-blind `is_registered`, or a broken `From` each fail |
| FR-044-AC-4 | `tc_442_the_macro_builds_a_const_code_from_a_literal`, plus `compile_fail` doctests on `code.rs` | `const` evaluation, text, unregistered; the probes fail for the stated reasons (E0080 const panic; `no rules expected`), measured with throwaway probes | gated: model-crate doctests run in `make test` |
| FR-044-AC-5 | `tc_442_from_static_is_a_const_new` | `const` items; `Ok` equal to `new`; `Err(invalid_code_form)` for empty, `Kani_x`, 65 bytes | sound |
| FR-030-AC-6 | `tc_223_the_outcome_and_its_error_carry_a_typed_std001_code`, plus `compile_fail` doctests on `non_success` | vacuous code and `"code":"kani_vacuous_proof"`; `"Bad-Code"` refused on read; error code checked through `non_success(Proved, ..)` | error clause is the AC's in substance only (FND-001); probes not gated (SR-1474 FND-001) |

Other checks:

- STD-001's rows equal `DiagnosticCode` ∪ `Std001Code::REGISTERED` exactly. No row lacks a
  spelling, and no constant lacks a row.
- `kani_outcome_kind_invalid` is gone from code and from the touched specs. AD-006 still
  describes it as current behavior (FND-004).
- (d) FR-044's old Status said "`DiagnosticCode` has no `Deserialize`". That was already
  false on main: `identity.rs` has a hand-written `Deserialize for DiagnosticCode`.
  Removing it corrects the status and claims nothing new.
- (c) The `CapabilityDisposition` code becoming a `Std001Code` is covered by FR-044 "Break
  and order", which says any field that carried a code as `String` takes the type.
- `make spec` was re-measured on both trees with quire 0.36.1. `quire validate` passes on
  main and on the PR. `quire coverage --strict` exits 1 on both, with the same class of
  failure: "31 unbacked row(s) and 0 contradicted" on main 7e4dc54, and "20 unbacked row(s)
  and 0 contradicted" on the PR. Rows backed go from 264/292 to 272/292. Every one of the
  20 remaining unbacked rows also appears in main's list, so the PR adds no new red. The
  11 rows that became backed are:
  - FR-044 and its five ACs
  - TC-442
  - TC-223
  - FR-030-AC-6
  - FR-030-AC-4 and FR-030-AC-5, which are inflated (FND-002)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-030-AC-6 is marked implemented, but one of its clauses is not met as written. The AC requires "the `code` of the `KaniOutcomeError` for a `proved` outcome with a count of zero equals `KANI_OUTCOME_INVALID`". No count-bearing `proved` request exists, because that is AC-4, which is planned. The test reads the error through `non_success(Proved, ..)` instead, which is an AC-5 clause. The status prose is candid about the substitution. Because `code()` is constant, the substance (the error's code) is verified. But the kani and core matrices, FR-030 Status and TC-223 all record AC-6 as implemented against a condition the AC does not name. Either amend AC-6 through a spec change to name the non-success request, or keep AC-6 partial until AC-4 lands | spec/kani/matrix/tests.md:13, spec/kani/functional/FR-030-bounded-kani-domain-and-outcomes.md:82, tests/it/kani_shared.rs:282-310 |
| FND-002 | medium | Strict coverage now counts FR-030-AC-4 and FR-030-AC-5 as backed, although no test exercises either and both stay planned. On main, both appear in `quire coverage --strict` as "has no backing symbol [verification]". On the PR they do not, because both verify through "Test (TC-223)" and the new test tags `TC-223`, so quire backs every AC routed through that TC. quire's per-document count shows the truth (FR-030: 4/6 bound), but the strict gate, which is what `make spec` and reviewers read, no longer lists two unimplemented ACs. Two of the 11 newly backed rows (31→20) are inflation. The prose status stays truthful (planned), so this is not a contradicted status, but the gate cannot see them. Fix in this PR: give AC-6 a TC of its own (a spec change), or otherwise keep AC-4/AC-5 visible as unbacked. At minimum, record the inflation in the TC-223 status | tests/it/kani_shared.rs:280-281, spec/kani/matrix/tests.md:24 |
| FND-003 | medium | `non_success` now refuses `Counterexample` with `KaniOutcomeError`, and FR-030 Status (as edited by this PR) claims it ("for a `proved` or `counterexample` request"). No test exercises the `Counterexample` arm. Moving `KaniOutcomeKind::Counterexample` into the `Ok` arm passes every test: `tc_042_only_proof_and_counterexample_have_boolean_claims` iterates the eight non-success kinds only, and `tc_223_*` asks only for `Proved`. Add the `Counterexample` request to the `tc_223` test (or to the AC-6 status's stated evidence) | src/kani/outcome.rs:228, tests/it/kani_shared.rs:291-299 |
| FND-004 | low | AD-006 G-4 still says "`non_success` turns `Proved` into a `Refused` outcome with code `kani_outcome_kind_invalid`". After this PR, that is false: it returns `KaniOutcomeError` (`kani_outcome_invalid`), and `kani_outcome_kind_invalid` no longer exists. Update the "not built" note to say what remains (the count) | spec/assurance/AD-006-codegen-consumption-seam.md:145-147 |

## Verdict

FR-044 is fully and soundly implemented. All five ACs have tests that fail on plausible
mutants, the compile probes fail for the right reasons and are gated, and the constant set
matches STD-001 exactly.

The FR-030 side overclaims in three ways:

- FND-001: AC-6 is marked implemented through a substitute request.
- FND-002: the strict gate now hides the planned AC-4 and AC-5.
- FND-003: the `Counterexample` refusal the status claims is untested.

`make spec`'s red is main's strict-coverage baseline and nothing new. FND-004 is doc
drift.

Verdict: changes requested.

## Dispositions

Round 1 at 165771b183fd96fb18791febd82f2732feb37796. I re-measured with quire 0.36.1:

- `quire validate` over spec, plan and reviews passes.
- `quire coverage --strict` reports "23 unbacked row(s) and 0 contradicted status(es)", with 272/293 rows backed. Main 7e4dc54 had 31 unbacked and 0 contradicted.
- All 23 unbacked rows also appear in main's list. FR-030-AC-4, FR-030-AC-5 and TC-223 are listed as unbacked again.
- The new TC-443 row is backed by `tc_443_*`.

FND-001: FR-030 Status, the kani and core matrices, `spec/tests.md` and TC-443 now call AC-6 "partly implemented". Its count-zero `proved` clause is "planned with AC-4", and the test reads the error through the non-success request "not counted as met". The status claim now matches the evidence.

FND-002: TC-443 was split out to verify FR-030-AC-6, and TC-223 now traces only AC-4 and AC-5 and is planned with no test. The test is retagged `#[trace("TC-443", "FR-030-AC-6")]`. `spec/spec.md`, TC-442's description and the STD-001 registry row are updated to match.

FND-003: `tc_443_*` now asks `non_success` for both `Proved` and `Counterexample`. Mutation check, applied in my review worktree and then reverted: with `Counterexample` moved into the `Ok` arm, `tc_443_the_outcome_and_its_error_carry_a_typed_std001_code` FAILED (panicked at tests/it/kani_shared.rs:302), so the mutant is killed.

FND-004: AD-006 G-4 now says `non_success` already returns `KaniOutcomeError` for `Proved`/`Counterexample`. `kani_outcome_kind_invalid` appears nowhere in `src/`, `tests/` or `spec/`; the only mentions are in historical `reviews/` files.

The unresolved conflict recorded in the FR-030 status is accurate. The `CapabilityDisposition::Inconclusive` arms carry the profile's code, and AC-5 will forbid that. The conflict already existed on main, where `non_success` accepted any code. It resolves with FR-029-AC-2, which retires the `Inconclusive` capability entry, or with IR-347, which removes the lowerings. It needs no spec ruling before merge. It does need a tracking ticket before AC-5 is built, and none names it today. The status does not say that `raise` will turn those outcomes into `refused`/`kani_outcome_invalid` once AC-5 lands; that is SR-1474 FND-006.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 165771b183fd96fb18791febd82f2732feb37796 |
| FND-002 | fixed | 165771b183fd96fb18791febd82f2732feb37796 |
| FND-003 | fixed | 165771b183fd96fb18791febd82f2732feb37796 |
| FND-004 | fixed | 165771b183fd96fb18791febd82f2732feb37796 |
