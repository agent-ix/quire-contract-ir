---
id: SR-2026102960
title: "Evidence review of IR-1141 structural public-interface controls"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; IR-1141 PR #714 head 6eb0b5e1; FR-019, TC-048, TC-058; FR-046 and FR-048 context"
review_set: subset
---

## Summary

TC-048 identifies structural positive and adverse controls and remains PLANNED/UNRUN. TC-058 correctly separates existing inventory tests from planned consumer probes, but its unchanged typed-operation fixture procedure retains one obsolete digest-specific probe.

## Examined units

TC-048 structural test plan (examined); TC-058 fixture procedure, Status and structural consumer plan (examined); FR-019-AC-5/15, FR-046/048 controls (context).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-058 still requires fixture member keys with wrong domain, wrong digest length, uppercase or nonhexadecimal digest. Structural `CheckedNodeId` has `{package,path,ordinal}` and none of those error dimensions. Replace with typed package/path/ordinal invalid controls and keep the same refusal/no-operation oracle. | spec/model/matrix/TC-058-model-crate-public-interface.md:72-73 |

## Verdict

FAIL until the obsolete fixture procedure is updated. No applicable AssuranceProfile found in the affected spec tree.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | TC-058 now tests empty structural package/path and out-of-range ordinals, including a nested path node ID, with typed error and no operation; the digest-key controls are gone. |
