---
id: SR-11831
title: "Integrity review of IR-824 AC-252 / AC-172 supersession (PR #420)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-172, FR-038-AC-252), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md (items 10-11), spec/checked_package/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-824. Consistency of the AC-252 supersession of AC-172's Set/Bag
clause against the repo's convention (FR-040-AC-7/AC-28, FR-052-AC-8/AC-17).
TC-048 item 10 no longer asserts the Set/Bag refusal and points at item 11;
the FR-038 matrix row and the new AC-252 row both state the supersession; no
other IR spec text still requires a single-valued end for `reaches` (grep for
`reaches`, `Set or Bag`, `Set/Bag`, `single-valued`, `Reference or Option`).
No existing test asserts the Set/Bag refusal (the AC-172 tagged tests at
`tests/it/checked_package_v2_model_members.rs:1294` and
`operations.rs:4021` cover single-valued admission and the heterogeneous
refusal only), so no tagged test must be retargeted. The defect is in the
superseded AC itself.

## Findings

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-001 | medium   | FR-038-AC-172 is left verbatim with no pointer to FR-038-AC-252, so read alone it still requires "otherwise ineligible Set/Bag edge refuses `ill_typed`/`operator-ineligible`" while AC-252 admits the same input: two live criteria disagree. The repo convention puts the pointer in the superseded row too (FR-040-AC-7 "(FR-040-AC-28 supersedes the former shared-frame refusal)"; FR-052-AC-8 "FR-052-AC-17 supersedes this criterion"). Add a pointer to AC-172 (e.g. "FR-038-AC-252 supersedes the Set/Bag clause for `quire.op.model.reaches`") without rewriting its outcomes. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4348 | wrong-requirement |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix commit 1840592 appends to FR-038-AC-172 (line 4356) "FR-038-AC-252 supersedes the otherwise-ineligible Set/Bag clause for `quire.op.model.reaches`." with outcomes and status unchanged, matching the FR-040-AC-7 / FR-052-AC-8 convention; TC-048 item 10 and the matrix rows assert only the new behaviour. |
