---
id: SR-612136
title: "IR-1100 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

The added FR-038 criteria are internally consistent with FR-038-AC-290 and the merged QSpec rule. The base review records the test coverage defects. Reviewed PR #671 against merged QSpec FR-464-AC-21 to 23 and TC-946 R13 to R24. No applicable AssuranceProfile was found in the changed module.

## Examined units

- `FR-038-AC-290` (examined): For each clause the reader constructs the QSpec FR-464 derived causal relation from control containment, sequencing, receive/send, effect/attempt and await edges, not source order or timestamps. A receive before its send or an await anchored at a later control refuses `invalid_package`/`definition-cycle` naming the cycle; a same pair on independent parallel branches admits. A repeat whose body can complete with no progress on a path refuses `unsupported_construct`/`declaration-form`, while a repeat with a real step admits with or without a maximum and loop proof.
- `FR-038-AC-344` (examined): An outer `repeat` whose sole body control is a nested `repeat` around an `attempt` admits when the nested guard exit is infeasible and its maximum is absent or positive.
- `FR-038-AC-345` (examined): The same outer `repeat` refuses `unsupported_construct`/`declaration-form` at the outer control when the nested guard exit is feasible, regardless of an absent or positive maximum.
- `FR-038-AC-346` (examined): The same outer `repeat` refuses `unsupported_construct`/`declaration-form` at the outer control when the nested maximum is zero, even if its guard exit is infeasible.
- `FR-038-AC-347` (examined): A nested `repeat` with a nonstepping body supplies no progress to its outer `repeat`, even when `exhausted` contains an `attempt`; the first progress refusal is at the outer control.
- `FR-038-AC-348` (examined): An outer `repeat` containing a nested control with a feasible exit admits when a sequence sibling, parallel or await provides the necessary step; a choice admits only when all cases step.
- `FR-038-AC-349` (examined): A nested guard whose listed visible entry is the composite expression `E = a and b` treats `E` as one independent atom beside `a` and `b`; `E or not a or not b` therefore has a feasible exit and leaves its sole-step outer `repeat` refused.
- `FR-038-AC-350` (examined): A nested guard `eq(a,b) or not eq(a,b)` has an infeasible exit, while `eq(a,b) or not eq(b,a)` has a feasible exit when the two distinct equality nodes are listed; the outer sole-step `repeat` admits only the first.
- `FR-038-AC-351` (examined): An inline or referenced Boolean literal `true` is a constant rather than an atom, so the outer sole-step `repeat` admits when its nested guard is either `true` or `a or true`.
- `FR-038-AC-352` (examined): A nested two-atom tautology charges four valuation work units, while a nested `a or b` charges only its first, all-false valuation; atom collection and comparison add no work units.
- `FR-038-AC-353` (examined): A valuation charge denied by `CheckedPackageReadLimits.work` returns incomplete `stage_limit_exceeded`/`work-budget-exceeded` at the nested control before an enclosing progress refusal, including for a nested maximum of zero.
- `FR-038-AC-354` (examined): A standalone `repeat` guard with 30 visible atoms consumes the same reader work as its `true` replacement, because no valuation runs outside another `repeat` body.
- `FR-038-AC-355` (examined): A nested `repeat max 1 while true` around an `attempt` with an `exhausted` `check` supplies the outer body's step on every outer iteration because its counter resets on entry.
- `FR-038-AC-356` (examined): A causal-cycle refusal precedes nested valuation and progress outcomes; when the cycle is removed, denied valuation precedes the first nonprogressing `repeat` in tree order.
- `TC-048` (examined): Nested repeat progress (FR-038-AC-344 through FR-038-AC-356): construct independently keyed, otherwise valid V2 packages; compare typed refusals, incomplete results, work and loci.

## Verdict

**PASS** — 0 finding(s).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
