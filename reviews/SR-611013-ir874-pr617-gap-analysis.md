---
id: SR-611013
title: "gap-analysis review of IR-874 PR #617"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_protocol_body.rs; FR-038-AC-283; TC-048"
review_set: subset
---

## Summary

Planless gap analysis of the PR #617 criterion-to-test binding: FR-038-AC-283 is statically tagged by 12 tests but two explicit identity dimensions lack assertions.

## Verdict

**CONDITIONAL** — Fix the recorded coverage gaps before merging. Focused formatter, 136 protocol tests, and Clippy were reported by the coder; this review did not repeat build gates.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-283 is tagged but two identity dimensions remain unverified | tests/it/checked_package_v2_protocol_body.rs:4812 |

## Coverage

- FR-038-AC-283 (examined): A protocol node derives its `quire.application-node/v1` key from its exact tag, form, Boolean type, declaration path, `null` recursion and body, with occurrences excluded; a second reference, source reformatting or declaring roles and other set-backed members in another source order preserves the key and package id, while a changed name, referenced target, literal, ordered capture or `sequence` child changes both. Role, channel, finish, terminal, control and compensation declarations use the fixed qualified-name segments of QSpec FR-464; queues have no declaration and generated occurrences, while other protocol members have declaration occurrences. A wrong path or occurrence role refuses at the node.
- TC-048 (examined): Reorder an authored `sequence` and an ordered capture list and require changed keys; reformat source and reorder role or other set-backed declarations and require unchanged keys.
- Computed matrix: static tagged (12 binders); evidence status no run evidence at frozen head. Plan completion: not assessed. Optional semantic review: not invoked; the criterion-bound test assertions were inspected under the reviewer brief.

### FND-001

Quoin matrix reports FR-038-AC-283 statically tagged by 12 test functions with no run evidence. Inspection of those tests shows neither stable identities under set-backed declaration reordering nor changed identities under reference retargeting. The static binding overstates semantic coverage of the criterion.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Both missing dimensions are now covered: role declaration source reorder in protocol_body.rs and admitted reference retarget with identity inequality in protocol_reference.rs. |

## Disposition verdict

**PASS** — the two recorded test-coverage gaps are fixed. The fix diff changes tests only; the lead owns the aggregate gate.
