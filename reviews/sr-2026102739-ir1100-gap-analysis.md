---
id: SR-2026102739
title: IR-1100 PR 679 gap analysis
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-contract-ir; PR #679; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  and tests/it/checked_package_v2_protocol_body.rs'
review_set: subset
---

## Summary

Reviewed IR-1100 PR #679 exact against merged QSpec FR-464 and IR FR-038-AC-344 through AC-356. No applicable AssuranceProfile or CI workflow changes were found. Rust review used repository conventions; aggregate gate remains with the code lead.

## Verdict

**CONDITIONAL** — acceptance evidence gaps require a fix round.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The tests verify four charges for a two-atom tautology and one for `a or b`, but never add an unread `visible` atom and require eight charges, as TC-048 specifies. An implementation that gathers only guard-read atoms would pass these tests. | tests/it/checked_package_v2_protocol_body.rs:8883-9080 |
| FND-002 | medium | The budget probe only counts a run of four incomplete results at the nested locus. It does not assert the exact V-1 denial, the V later-charge locus, and the T admitted outcome from TC-048. A wrong later-charge location or outcome at V could pass. | tests/it/checked_package_v2_protocol_body.rs:9026-9102 |
| FND-003 | medium | The sibling control exercises sequence, parallel and choice, but no `await` around a nested repeat. AC-348 explicitly requires admission through await; a regression in that branch would remain unobserved. | tests/it/checked_package_v2_protocol_body.rs:9211-9338 |

## Coverage

Plan completion: not assessed
- FR-038-AC-344 (examined): An outer `repeat` whose sole body control is a nested `repeat` around an `attempt` admits when the nested guard exit is infeasible and its maximum is absent or positive.
- FR-038-AC-345 (examined): The same outer `repeat` refuses `unsupported_construct`/`declaration-form` at the outer control when the nested guard exit is feasible, regardless of an absent or positive maximum.
- FR-038-AC-346 (examined): The same outer `repeat` refuses `unsupported_construct`/`declaration-form` at the outer control when the nested maximum is zero, even if its guard exit is infeasible.
- FR-038-AC-347 (examined): A nested `repeat` with a nonstepping body supplies no progress to its outer `repeat`, even when `exhausted` contains an `attempt`; the first progress refusal is at the outer control.
- FR-038-AC-348 (examined): An outer `repeat` containing a nested control with a feasible exit admits when a sequence sibling, parallel or await provides the necessary step; a choice admits only when all cases step.
- FR-038-AC-349 (examined): A nested guard whose listed visible entry is the composite expression `E = a and b` treats `E` as one independent atom beside `a` and `b`; `E or not a or not b` therefore has a feasible exit and leaves its sole-step outer `repeat` refused.
- FR-038-AC-350 (examined): A nested guard `eq(a,b) or not eq(a,b)` has an infeasible exit, while `eq(a,b) or not eq(b,a)` has a feasible exit when the two distinct equality nodes are listed; the outer sole-step `repeat` admits only the first.
- FR-038-AC-351 (examined): An inline or referenced Boolean literal `true` is a constant rather than an atom, so the outer sole-step `repeat` admits when its nested guard is either `true` or `a or true`.
- FR-038-AC-352 (examined): A nested two-atom tautology charges four valuation work units, while a nested `a or b` charges only its first, all-false valuation; atom collection and comparison add no work units.
- FR-038-AC-353 (examined): A valuation charge denied by `CheckedPackageReadLimits.work` returns incomplete `stage_limit_exceeded`/`work-budget-exceeded` at the nested control before an enclosing progress refusal, including for a nested maximum of zero.
- FR-038-AC-354 (examined): A standalone `repeat` guard with 30 visible atoms consumes the same reader work as its `true` replacement, because no valuation runs outside another `repeat` body.
- FR-038-AC-355 (examined): A nested `repeat max 1 while true` around an `attempt` with an `exhausted` `check` supplies the outer body's step on every outer iteration because its counter resets on entry.
- FR-038-AC-356 (examined): A causal-cycle refusal precedes nested valuation and progress outcomes; when the cycle is removed, denied valuation precedes the first nonprogressing `repeat` in tree order.
- TC-048 (examined): Construct each case as an independently keyed, otherwise valid V2 package and read it through the public strict reader.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The budget probe adds a distinct unread `disjunction` node to `visible` and asserts that the nested valuation run grows from four to eight. |
| FND-002 | fixed | The probe now asserts V-1 denies the fourth valuation at the nested locus, V denies the next later control charge, and T admits. It also checks maximum-zero valuation denial. |
| FND-003 | fixed | The new await control places a feasible-exit nested repeat in the success branch of an await inside the outer repeat and asserts admission through the public reader. |

## Current verdict

**PASS** after disposition pass 1 on PR #679. The original findings above remain as the historical review record; all are fixed. Aggregate gate remains with the code lead.
