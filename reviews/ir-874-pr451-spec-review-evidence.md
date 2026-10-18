---
id: SR-12423
title: "IR-874 PR 451 spec-review-evidence review"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Checked whether the specified independent oracle and test inputs exist. The machine vectors and regenerated protocol fixtures are not yet published. Ticket: IR-874. Frozen head: e3c91ccff561f7a80402c07d79c594e16957b978.

## Examined units

- **TC-048** (`examined`): Use QSpec FR-464, FR-465 and FR-466 and their TC-946, TC-947 and TC-948 vectors as the independent expected outcomes.
- **FR-038-AC-281** (`examined`): A reader-admitted protocol member has a `quire.op.protocol.control` body root, exactly one `protocol_profile` law matching the selected lock row, `null` mode, no leaves, Boolean result and semantic types, and a `profile_operator` member. Under the published `quire.protocol.finite-global/v1` selection the admitted operators are exactly `protocol`, `role`, `channel`, `queue`, `finish`, `terminal`, `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit`, `fence` and `compensate`; an unknown operator refuses `operation-member-mismatch
- **FR-038-AC-289** (`examined`): A `compensate` body admits the thirteen FR-466 bindings in order, including distinct registration and activation captures, a trigger, interval, at-least-one attempt, retry relation, optional commit and recovery predicate. `commit never` is an absent optional Group, not a fabricated commit. The reader refuses a forward target that is not an `effect`, a commit target that is not a `commit`, an inverted interval or zero attempts, a commit causally before its forward effect, and a read outside any of the five compensation stages with the corresponding FR-466 code, cause and locus; the package crea

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-048 requires protocol machine vectors and regenerated fixtures that are absent from QSpec main. Measured QSpec origin/main fd7a16b: proposals/checked-package-v2 contains schemas and older fixtures but no protocol machine-vector file. FR-464 calls implementation and qualification separately tracked. The step is not executable as written. State that TC-946..948 textual tables are the current oracle and make machine-vector execution conditional on producer publication; keep fixtures in QSpec. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1503-1507 |

## Verdict

Checked whether the specified independent oracle and test inputs exist. The machine vectors and regenerated protocol fixtures are not yet published.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at PR #451 head bd0f2fdef5793d005952505b0e456a629eb76f70 against QSpec FR-464/466 and TC-946/948; the corresponding TC-048 control was updated. |
