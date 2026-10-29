---
id: SR-610061
title: "IR-874 PR 610 STD-496 decision-refusal spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #610 reviewed, disposition; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: base
---

## Summary

Reviewed the two changed FR-038 criteria and TC-048 protocol-node step 5 against QSpec (FR-466-AC-14/16 and TC-948 DF6b, DF7d..DF7f). The refusal classes and precedence in FR-038 agree with the upstream authority. TC-048 initially omitted the complete finish-binder slot matrix; the same-PR fix now requires all four cases.

## Verdict

**PASS — FND-001 fixed in the same PR.** The original finding and its verified disposition remain below for audit history.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-048 leaves three of the four finish-binder decision slots unrequired | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1669 |

### FND-001 detail

TC-048 step 5 says, “read a finish binder and a compensation-template binder from a run slot,” so one finish-binder slot can satisfy the test plan. FR-038-AC-292 says the finish binder in **any of the four** choice/repeat visible/guard slots must refuse `missing_declaration`/`missing-name` with `read` `stage`; QSpec TC-948 DF6b explicitly enumerates those four cases. A reader that misclassifies the three unexercised slots as `wrong_snapshot` can pass the written TC. Require the finish-binder case in each of the four slots, while retaining the compensation-template case and the paired precedence cases.

## Scope

- **FR-038-AC-291 (examined):** Binder levels follow introducing constructs; binders-only choice/repeat visible entries and guards refuse direct protocol `over` or model-state reads as `ill_typed`/`operator-ineligible` before binder-scope or decision-fact checks.
- **FR-038-AC-292 (examined):** Decision facts use role-owned, individually visible eligible binders; `over` reads refuse `ill_typed` first, finish and compensation-template binder reads in any of four run slots refuse `missing_declaration`, and paired faults have defined precedence.
- **TC-048 protocol-node step 5 (examined):** Exercises causal, repeat, binder-stage and decision-fact cases, including all four `over` slots, a finish binder and compensation-template binder in a run slot, and two paired-fault precedence cases.
- **QSpec FR-466-AC-14/16 and TC-948 DF6b, DF7d..DF7f (context_only):** is the exact upstream refusal and precedence oracle.

## Review method

The base spec checklist covered identity, criterion testability, adverse cases, cross-reference and test coverage for this PR diff. No applicable AssuranceProfile was found under IR `spec/`. EARS analysis does not apply because this diff edits acceptance-criteria table rows, not FR Description/Behavior/Constraints statements. No production code changed, so Rust and gap-analysis review do not apply. Quire validation of both changed artifacts passed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, TC-048 step 5 explicitly requires the finish-binder `missing_declaration`/`missing-name` refusal at the read with `read` `stage` in each choice visible entry, choice guard, repeat visible entry and repeat guard; the compensation-template and paired-fault cases remain. |

### FND-001 after excerpt

> Read a finish binder in each `choice` `visible` entry, `choice` guard, `repeat` `visible` entry and `repeat` guard; require `missing_declaration`/`missing-name` at the read with `read` `stage` in all four slots.
