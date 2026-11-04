---
id: SR-2026110414560011
title: IR-1326 PR 896 spec-review/base review
type: SpecReview
analysis: base
scope: 'agent-ix/quire-contract-ir; PR #896; IR-1326; FR-038-AC-251/461/462; spec-review/base'
review_set: subset
---

## Summary

Independent review of PR #896 at 254da53f against QSpec main 860e0890.

## Verdict

PASS after review of the edited AC-251, AC-461/462, TC-048 and matrix text on d651e47b. The historical AC-251 finding is fixed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

FR-038-AC-461 and FR-038-AC-462 examined; QSpec FR-370-AC-17/18 and FR-322 temporal binding occurrence rule used as context. Plan completion: not assessed. The published QSpec vector is read by reference; no case count, digest, or source revision is pinned. No applicable AssuranceProfile found under spec/assurance. The diff only changes Planned to Implemented in existing AC and TC cells and adds one matrix row; no new normative requirement text or hash/digest/pin ceremony.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | AC-251 and TC-048 required an exact unavailable catalog identity inventory, and the first catalog-delta fix extended that copied list. This pinned the current catalog and caused churn without adding reader behavior. Require a reason from each live entry's metadata and fail only when a member/type-argument entry lacks an admitted control or an unavailable entry has no specific fixture reason. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5577; tests/it/checked_package_v2_catalogue_differential.rs:1223 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | AC-251 and TC-048 now require catalog-derived reasons, and d651e47b removes expected_unavailable() and all count pins. The classifier has no generic fallback; an unclassified entry fails. The 12 member/type-argument conversion entries retain real admitted controls. |

## Follow-up coverage

FR-038-AC-251, TC-048's catalogue differential section, and the FR-038 matrix row were examined against QVC 5070338. Scope extension found no new hash, digest, pin, copied catalog inventory, or compatibility layer. Quire validation of the changed requirement, TC, and matrix passes with one pre-existing unrelated grammar warning at FR-038 line 5714.
