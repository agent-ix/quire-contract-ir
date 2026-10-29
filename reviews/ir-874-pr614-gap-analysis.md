---
id: SR-610063
title: "IR-874 PR 614 trace and gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/shared.rs; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs; tests/it/checked_package_v2_protocol_body.rs; FR-038-AC-291/292; TC-048; QSpec FR-466-AC-14/16"
review_set: subset
---

## Summary

Ticket: IR-874. Planless, diff-scoped trace and test-intent audit of test-only PR #614. The four new tests carry the FR-038-AC-291/292 and TC-048 tags appropriate to their behavior. No new production code needs an owner; the location and owner assertion deficit is recorded in the code-review artifact SR-610062.

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

## Expanded STD-515 coverage

The new `FR-038-AC-292` text and TC-048 procedure own the production pass in `protocol_body.rs`; the existing `ModelStateRead` variant remains its typed detail. `bare_activation_record_read_refuses_in_each_decision_slot`, `bare_receive_record_read_refuses_in_each_decision_slot` and `bare_activation_precedes_sent_field_in_either_guard_operand_order` have FR-038-AC-292 and TC-048 tags and assert the public reader's typed refusal, path, locus and slot. The preserved receive-field and activation-field tests cover admitted and ineligible projected facts. Plan completion: not assessed. No new production source lacks a stated owner; repository-wide strict coverage is not claimed.

## New findings (expanded review)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-002 | medium | No test covers a record-typed protocol capture read bare, although the criterion covers any record binder; the implementation explicitly skips captures. | tests/it/checked_package_v2_protocol_body.rs:953; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:3898 | correct-requirement-no-evidence |

## Expanded verdict

**CONDITIONAL** — the tagged cases cover DF7g/DF7h but not the full “any record binder” obligation. The production defect is detailed in SR-610062 FND-002.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed |, the new tagged public-reader test covers a record-typed protocol capture bare in all four decision slots, with exact typed refusal/path/locus/slot and field-projection plus scalar-capture admission controls. The implementation now checks captures too. |

## Final disposition

**PASS** for the expanded diff's FR-038-AC-292 trace and evidence. Plan completion: not assessed. The repository-wide strict-coverage backlog remains outside this diff's claim.
