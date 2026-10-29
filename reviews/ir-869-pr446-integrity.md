---
id: SR-12402
title: "IR-869 spec-integrity of IR PR #446"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

# IR-869 spec-integrity of IR PR #446

## Summary

Ticket IR-869. AC-195 now names the two Clippy lanes and the repository lint gate as an Analysis obligation. The initial PR added a dedicated lint row but retained AC-195 inside the broad FR-038 row's AC-186 through AC-196 range with TC-048 in its Test Cases cell. That assigned one criterion to two incompatible verification routes. The author fixed the range in the reviewed PR: it now ends at AC-194 and names AC-196 separately, leaving AC-195 only in the lint row. Both edited Quire documents validate grammar-clean (2/2).

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4756 | examined | AC-195 Analysis (`make lint`) is explicit and verifiable |
| spec/checked_package/matrix/tests.md:21 | examined | TC-048 range excludes AC-195 after fix |
| spec/checked_package/matrix/tests.md:22 | examined | dedicated AC-195 lint row |
| spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1995 | context | procedure calls for actual Clippy exits without asserting a Rust test claim |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Initial head assigned AC-195 to both TC-048 and the lint gate by retaining AC-186 through AC-196 in the broad matrix row; readers could treat a Rust test as discharge of the lint criterion. | spec/checked_package/matrix/tests.md:21-22 |

## Dispositions

| Finding | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed | splits the broad range into AC-186 through AC-194 and AC-196; the dedicated lint row is the only explicit AC-195 row. |

## Verdict

PASS after the fix. AC-195 has one stated verification route, the workflow invokes it, and the matrix does not claim a TC-048 Rust test for it.
