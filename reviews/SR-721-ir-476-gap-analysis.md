---
id: SR-721
title: "gap analysis of PR 247 (IR-476 test trace hygiene)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@f512aac4330ad47cf474d23af9f2407a32c45f19; tests/it/executable_binding.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs, spec/checked_package/matrix/tests.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---
# SR-721: gap analysis of PR 247

## Summary

Ticket: IR-476. Plan completion: not assessed. Measured with `quire coverage --json` at base
4233b56 and head f512aac. Coverage 161/184 both; 23 unbacked matrix rows both, identical
set; FR-023 5/5 both; candidates/tagged/bound 267/243/239 before and 267/245/239 after. The
shared-trace-id symbol sets (which list all 13 TC-035 symbols) are byte-identical before and
after. The only JSON differences are `binding_census` (tagged +2), `metrics`, and four new
`unmatched_tags` rows, all TC-048 on the four operations.rs tests. `make spec` stderr
diagnostics are identical (no new warning).

FR-038-AC-43 and FR-038-AC-44 are `backed: false` in `minted_targets` at both base and head
(they are not among the 23 strict matrix rows). Their 18 operations.rs tests carry
`Tracing: TC-048, FR-038-AC-4x` doc lines, which are reported as unmatched.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The PR body's stated reason that FR-038-AC-43/44 stay unbacked is wrong. It says unit tests inside quire-contract-model "can only carry doc Tracing lines (which do not bind)". Measured at head: 113 symbols bind TC-048, including about 40 unit tests in `crates/quire-contract-model/src/**` that bind through the `tc_048_` test-name form, so evidence symbols in that crate bind. AC-43/44 are unbacked only because `Tracing:` is not a declared form. A declared form would bind them: `#[trace(...)]` after adding `ix-trace-rs` as a dev-dependency of the model crate (already in Cargo.lock, so no new package) or a legacy bare id. That fix is within IR-476's scope and needs no spec PR. The follow-up recorded in the body points at the wrong blocker | crates/quire-contract-model/src/checked_package/v2/operations.rs:3493-4480 |
| FND-002 | low | The about 28 operation-law validation unit tests in operations.rs still have no owning criterion. I agree with the coder: no FR-038 AC covers unknown-operation, operator-class, law missing/mismatch/unselected outside leaves, mode/member, arity or malformed-wire operation checks. AC-43/44 cover leaves only, and AC-12..15 cover frames. This needs a spec PR adding an FR-038 AC (plus a TC-048 case). IR-476 stays open, but no follow-up ticket records this work yet | crates/quire-contract-model/src/checked_package/v2/operations.rs |

## Verdict

The `executable_binding.rs` half of the change is coverage-neutral, as claimed. The
operations.rs half adds no coverage. File a Linear subticket for the FR-038 operation-law AC
(spec work). Correct the AC-43/44 follow-up to say that binding is the fix, not a spec PR.

## Dispositions

Round 1, reviewed agent-ix/quire-contract-ir@f6cc2e9a7f9a387a073211e653fddfac5f4dd54c.
Measured with `quire coverage --scope . --strict --json` at head: coverage 163/184 (base
161/184), FR-038 42/42, candidates/tagged/bound 267/245/243. FR-038-AC-43 and AC-44 are now
backed in `minted_targets`. Every one of the 24 tagged tests (10 on AC-43, 14 on AC-44) was
read against the AC text. Each exercises a clause of the AC it names, and each already cited
that AC in its `Tracing:` doc line, so no tag was invented.

Reconciling the "23 unbacked" line: two different lists both happened to hold 23 entries at
base. (a) The coverage denominator (`minted_targets`, 184 rows) had 23 unbacked at base,
FR-038-AC-43 and AC-44 among them. At head it has 21 (= 184 - 163), and the two removed are
exactly AC-43 and AC-44. (b) The strict `unbacked_rows` list ("23 unbacked row(s)" in
`make spec`) is a separate check. It holds FR-036, FR-037 and FR-039 on kani/matrix,
NFR-001-AC-2, NFR-002-AC-1, TC-019, TC-045, TC-055, TC-223, TC-058, FR-030-AC-4/5,
FR-036-AC-1..5, FR-037-AC-6, FR-039-AC-1..4 and FR-019-AC-5. It never contained AC-43 or
AC-44, and it is identical at base and head. So the two newly backed rows were among the
former 23 coverage rows, not among the 23 strict rows. That is why the strict line did not
move.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 743407e13fdfc7a3fc3c4f8a865834d4ae81326f |
| FND-002 | deferred | Needs a spec PR adding an FR-038 operation-law AC plus a TC-048 case first, which is out of scope for this test-hygiene PR. The PR body records it under "Not done", and the team leader files the Linear subticket (none exists yet). |
