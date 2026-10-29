---
id: SR-612140
title: gap-analysis review of IR-1103 step relation admission
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/,
  tests/it/checked_package_v2_frame_entries.rs, spec/checked_package/functional/FR-350-admit-step-relation-claim-body.md
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-350
  type: reviews
---
## Summary

Tagged tests cover AC-1 through AC-5, but their assertions do not establish every stated behavior. Plan completion: not assessed. QSL-552 is a genuine external producer dependency.

## Verdict

**FAIL** — Required acceptance evidence remains incomplete.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-1 fixture uses direct self-node equality, not pre(x.self.l) = pre(y.self.l) | tests/it/checked_package_v2_frame_entries.rs:4397 |
| FND-002 | medium | No assertion proves model and code claim items have distinct FR-402 obligation identities | tests/it/checked_package_v2_frame_entries.rs:4913 |
| FND-003 | medium | Whitespace and comment identity invariance has no test | tests/it/checked_package_v2_frame_entries.rs:4605 |
| FND-004 | high | External QSL producer readback remains unverified pending QSL-552 | tests/it/checked_package_v2_frame_entries.rs:4321 |

## Coverage

All FR-350 AC-1 through AC-5 examined against the frozen PR diff, QSpec FR-477 and TC-962. Plan completion: not assessed. The external producer readback is blocked on QSL-552. Final aggregate gates are assigned to the code lead; they were not rerun in this review.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The admitted equality now traverses pre, l projection and deref for x and y and asserts distinct receiver binder nodes. |
| FND-002 | fixed | The public obligation view now yields model temporal-satisfaction and code operation-contract identities over the same claim, asserted in the bound and unbound tests. |
| FND-003 | fixed | Two source spellings with distinct source digests now admit with equal typed body, node id and package id. |
| FND-004 | deferred: QSL-552 | QSL-552 has not produced canonical external checked-package bytes; IR-authored fixtures do not discharge this readback. |
