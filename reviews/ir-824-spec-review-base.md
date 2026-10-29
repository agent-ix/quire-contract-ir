---
id: SR-11830
title: "Base review of IR-824 reaches over multi-valued relationship ends (PR #420)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (reaches paragraph, FR-038-AC-252), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md (items 10-11), spec/checked_package/matrix/tests.md (FR-038, TC-048 and AC-252 rows)"
review_set: subset
---

## Summary

Ticket: IR-824. PR agent-ix/quire-contract-ir#420, diff
`origin/main...HEAD`. Base checklist over the new FR-038 `reaches` paragraph,
FR-038-AC-252, TC-048 item 11 and the matrix rows, checked against merged
QSpec origin/main (FR-322 "Reaches over a field" relationship-end
paragraph, FR-322-AC-57, FR-152 "Reachability over a relationship end",
FR-152-AC-11/12, TC-281 RN-18 and RN-20..RN-27, TC-197 Y14..Y19) and the
current reader (`operations.rs` `check_relationship_member`,
`model_members.rs` `navigation_type`).

Examined and clean: admitted shapes (`[0,1]`, `[1,1]`, finite Set/Bag shaped,
unique or not) match FR-152 and FR-322-AC-57; "destination end of the
receiver endpoint's own type" matches; RN-21 (subtype operand 1 admits), RN-22
(`arguments/1`), RN-23/RN-24/RN-26 (`unsupported_construct`/`expression-form`
at `member.name`), RN-25 (`ill_typed`/`operator-ineligible` at `member.name`)
and RN-27 (direction before multiplicity) are each stated with code, cause and
locus. The stated member order direction, multiplicity, type matches QSpec and
the code: direction at `operations.rs:2242`, multiplicity inside
`navigation_type` (`operations.rs:2263`, `model_members.rs:252-275`), type at
`operations.rs:2249-2256`. The node shape is unchanged and no charge or
multiplicity member is specified (TC-197 Y18 agrees); charges are left to the
consumer. AC-252 is free and the highest FR-038 id (`git log --all -S`, one
open PR). TC-048 item 11 cites RN ids only. No hash, digest or pin is
introduced. `quire validate` is clean (916/916); `quire coverage --strict`
reports 55 unbacked / 0 contradicted on the branch against 47 / 0 on
origin/main; the 8-row delta is exactly FR-052-AC-15 and AC-17..23,
backed by #419 which the branch predates (merge is clean), so the PR adds no
unbacked row.

## Findings

| ID | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | low | The new FR-038 reaches paragraph states the check order twice in one sentence ("The checks run in the order direction, multiplicity, then type:... with multiplicity decided before type and direction before multiplicity"); the trailing restatement is redundant. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1156-1161 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix rewrote the sentence (FR-038 lines 1156-1170): the order is stated once, "The checks run in the order stated by FR-322-AC-57:... then direction; then destination multiplicity; then type", with no trailing restatement. |
