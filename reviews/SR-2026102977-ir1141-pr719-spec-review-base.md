---
id: SR-2026102977
title: "spec-review-base review of IR-1141 PR #719"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; FR-038, TC-048, TC-226"
review_set: subset
---

## Summary

The structural ordinal and group-position distinction aligns with merged QSpec #534 except for the grouped path phrase. Ticket: IR-1141. Reviewed exact PR head e8173eb424698361c9b486fd28082a49f2a94930 against merged QSpec #534.

## Examined scope

- `FR-038` node-id prose (examined): `spec/checked_package/functional/FR-038-consume-checked-package-v2.md` lines 352-368 and 2493-2508.
- `FR-038-AC-88`, `AC-145`, `AC-147`, `AC-150`, `AC-227`, `AC-241`, `AC-364` (examined): same FR file.
- `TC-048` group and ordinal controls (examined): `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md`.
- `TC-226` group and fixture controls (examined): `spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md`.
- `QSpec FR-322-AC-274` and grouped structural identity prose (context): authoritative QSpec main, merged #534.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038 says each group member uses “its declared path,” including in AC-364. Anonymous derived members have no declaration; QSpec FR-322-AC-274 requires every group member to use the group’s declaration path. Specify the group’s path explicitly in both prose locations and AC-364, and cite AC-274. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:357-358,2498,4758 |

## Verdict

FAIL until the grouped path is stated as the group’s declaration path.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Head e8f389b6 states the group's declaration path for anonymous derived members at both FR-038 prose locations and AC-364, cites QSpec AC-274, and TC-048/TC-226 directly read all four positive QSpec group fixtures and assert each member's path and producer-assigned ordinal. |

## Disposition verdict

PASS. The sole finding is fixed; the amendment retains the independent grouped ordinal and adverse controls.
