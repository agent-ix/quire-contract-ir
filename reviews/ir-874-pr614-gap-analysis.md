---
id: SR-610063
title: "IR-874 PR 614 trace and gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_protocol_body.rs; FR-038-AC-291/292; TC-048; QSpec FR-466-AC-16"
review_set: subset
---

## Summary

Ticket: IR-874. Planless, diff-scoped trace and test-intent audit of test-only PR #614 at df7e4ee68987856433aaef7baa9c1463392fded3. The four new tests carry the FR-038-AC-291/292 and TC-048 tags appropriate to their behavior. No new production code needs an owner; the location and owner assertion deficit is recorded in the code-review artifact SR-610062.

## Examined units

- **FR-038-AC-291** (`examined`): “In a binders-only slot, including each `choice` or `repeat` `visible` entry and guard, a direct read of `over` or model state refuses `ill_typed`/`operator-ineligible` at the read before binder-scope or decision-fact checks.”
- **FR-038-AC-292** (`examined`): decision fact eligibility, all four direct `over` slots, finish and template scope, activation projection, and paired precedence.
- **TC-048** (`examined`): decision-slot public-reader procedure, including retention of stage, owner, slot and reason.
- **QSpec FR-466-AC-16** (`context_only`): published precedence oracle for visible entries and guards.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Coverage

- Plan completion: not assessed.
- The new test functions have `/// Trace:` criterion tags and `#[trace("TC-048")]`; the function at line 406 claims AC-291/292, and the three later functions claim AC-292.
- Choice/repeat by visible/guard is explicit for `over`, finish, template and activation field projection. Both precedence combinations are exercised in guards. The bare activation parameter read is unclaimed under STD-515.
- Reverse gap: no production source was added. The test behavior is owned by FR-038-AC-291/292 and TC-048. No stub or tautological assertion was found.
- The review is limited to this PR diff; a repository-wide strict-coverage PASS is not claimed. The separate code review records the missing location/owner assertion.
- Optional repository-wide semantic review: not run; the changed test intent was inspected here and in SR-610062.

## Verdict

**PASS** for trace ownership and the diff's evidence shape. This does not discharge the code-review finding or the repository-wide strict-coverage backlog.
