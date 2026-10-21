---
id: SR-15324
title: "IR-571 failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038, FR-346; spec/checked_package/matrix/TC-225; current TC-225 implementation; QSpec FR-154-AC-14"
review_set: subset
---

## Summary

The collision-first rule is stated but its decisive adverse oracle is absent.

## Verdict

**CONDITIONAL** — The collision-first rule is stated but its decisive adverse oracle is absent.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-225 repeats one parameter identity but leaves both names otherwise valid. That cannot distinguish the specified collision-first suppression from a reader that checks malformed names before identity collisions. Add a repeated-identity case with a malformed name on one colliding parameter and require exactly one conflicting-binding refusal at the later parameter path, with no malformed-declaration. | spec/checked_package/matrix/TC-225-checked-package-v2-abstraction-relation-body.md |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | rejected | The admitted control has rebase(from, to) with different names and matching identities. Repeating one identity as the sole mutation necessarily makes one unchanged name differ from that shared identity's final segment. TC-225 requires conflicting-binding at selection before the name check, so the case is already a decisive collision-precedence oracle. |
