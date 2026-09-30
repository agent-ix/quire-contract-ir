---
id: SR-628
title: "PR #231 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@a567ba99e8601849c77984f19c723a000e664fca; FR-038 requires-bound rules, crates/quire-contract-model/src/checked_package/v2/lower.rs, tests/it/checked_package_v2_require_bounds.rs, tests/it/complete_v1_contract_package.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---
# SR-628: PR #231 gap analysis

## Summary

Ticket: IR-283 (and IR-284). PR agent-ix/quire-contract-ir#231.
Plan completion: not assessed.

Examined: FR-038-AC-39, FR-038-AC-8 and FR-038-AC-6 against the four new
TC-050 tests and the changed TC-047 test. Also examined: the tickets'
acceptance statements (untrusted ticket text, re-measured here) against QSL
TC-440 run with this PR patched in.

Matrix: `quire coverage --scope . --strict` at a567ba9 and at merge-base
70792e7 both report 22 unbacked rows, the same rows (pre-existing, IR-448).
Bound evidence goes from 202 to 206 symbols. The PR adds no unbacked row.

## Verdict

**CHANGES REQUESTED.**

- IR-284's acceptance (QSL's RangedTree fixture agrees) is met, measured.
- IR-283's record fixtures (Flags, Mixed) agree, measured.
- IR-283 is not fully delivered: QSL's TC-440 application fixture still
  disagrees.
- The new behaviour has no AC of its own and borrows AC-39's trace.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | IR-283's acceptance is that QSL's TC-440 agreement fixtures agree. It is not met. `tc_440_an_unbounded_application_record_requires_a_bound_in_ir_pending_ir_283` (QSL `qsl-package/src/emit/extent_agreement.rs`) still fails against this PR: IR lowers the outer `+` of `(x + 1) + n`, which QSL classifies `Unbounded` at `n`. See SR-627 FND-001. | crates/quire-contract-model/src/checked_package/v2/lower.rs:464-468 |
| FND-002 | medium | The four new tests (position rule and recursion rule) are traced `#[trace("TC-050", "FR-038-AC-39")]`. AC-39 states only `x + 1` over an `integer_range` parameter and the literal-annotation exemption. No AC states that a composite element or field position is covered only by its own type, or that a recursive type always requires a bound. The matrix therefore claims AC-39 backing for behaviour AC-39 does not state, and the new rules have no criterion a reviewer can check. | tests/it/checked_package_v2_require_bounds.rs:316-409; spec/contract/FR-038-consume-checked-package-v2.md:651 |
| FND-003 | low | The quantity/unit divergence is left with no ticket. QSL ADR-014 §4 classifies a quantity as unbounded and unboundable, while IR `requires_bound` returns `false` for `unit` and `compound_unit`. An IR-283 comment asked for it to be measured when the ticket was worked. It cannot be measured yet (QSL's emitter omits the record), and the PR is right to leave FR-038's rule alone. No IR ticket tracks the divergence (only IR-263, a Kani row). | crates/quire-contract-model/src/checked_package/v2/lower.rs:548-557 |

## Dispositions

Round 1, reviewed at edb862819349fae5ba5bf2a19ea565ab1a3f96dd.

- **`make spec`** at edb8628 and at origin/main 7c70041 (which includes #229):
  both report 22 unbacked rows, the same rows (IR-448). The PR adds no
  unbacked row. Bound evidence goes from 204 to 209 symbols, and docs from
  223 to 226, all grammar-clean.
- **QSL TC-440:** see SR-627 Dispositions.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | edb8628 |
| FND-002 | fixed | edb8628 |
| FND-003 | deferred | FR-038 now states the unit/quantity divergence explicitly (line 575-576). No IR ticket tracks it yet, and filing one is the lead's action, outside this PR's code. |
