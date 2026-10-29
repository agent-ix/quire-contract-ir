---
id: SR-11402
title: "IR-800 PR 405 spec-review base review (fix round 1)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir (PR delta); spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Ticket IR-800, PR quire-contract-ir#405, fix round 1. This is a base spec-review of the two spec edits in `git diff origin/main...HEAD`:

- FR-038 statement item 22 now lists the `CheckedMemberType` variants String, Uuid and Timestamp.
- AC-242..245 rows and the TC-048 notes and section heading moved from PLANNED/UNRUN to IMPLEMENTED.

The status prose matches the existing convention of AC-225..227 ("IMPLEMENTED (IR-739; tagged reader and conformance tests passed)"). Each of the four criteria has tagged tests that pass, and `quoin matrix` reports each as tagged. The item 22 amendment agrees with `model_fields.rs` for the three new variants.

## Examined units

- FR-038 item 22 (closed `CheckedMemberType` variant list). (examined)
- FR-038-AC-242, FR-038-AC-243, FR-038-AC-245 status prefixes. (examined)
- FR-038-AC-244: its status prefix, and its statement compared with the refusal order the fix round now pins in a test traced to it. (examined)
- TC-048 notes (lines 73-76) and the section "Native String, UUID and Timestamp admission". (examined)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-244 and TC-048 do not state that arity and other-operand family defects come before the String mode refusal, although a new test traced to AC-244 pins that order | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4340 |

## Verdict

**PASS** with one low finding. Both requested edits are accurate and follow the file's conventions.

Observation, not a finding against this PR: item 22 still omits the existing `Record` and `Variant` variants while saying it "mirrors `MemberType` variant for variant". That omission predates this PR.

FND-001 detail: `text_comparison_checks_arity_and_other_families_before_string_mode` (tests/it/checked_package_v2_operand_families.rs:626) is traced to AC-244. It asserts two refusals:

- `ill_typed`/`operator-ineligible` at `/arguments/0` for `text.eq(integer, string)`.
- The arity refusal at `/arguments` for a three-argument `text.eq` that includes a String.

AC-244 states only that a String operand refuses at the mode value, and the TC-048 procedure gives no order either. A reader following the AC literally could implement the opposite order. Add one sentence to AC-244 and the TC-048 paragraph.

## Dispositions

| FND | Outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-244 and TC-048 now state that an arity defect or another wrong-family operand refuses ill_typed/operator-ineligible at the arguments or the first offending argument, before the String mode refusal. This matches check_operands (the arity check, then the position loop that skips String operands) and the pinned test (round 2) |
