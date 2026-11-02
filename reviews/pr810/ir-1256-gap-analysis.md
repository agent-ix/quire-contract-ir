---
id: SR-202611010316
title: "Gap analysis of carried quantity acceptance evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1256; FR-038-AC-401 through FR-038-AC-405; FR-019-AC-20; tests/conformance_qspec/carried_quantity.rs"
review_set: subset
---
## Summary

Reviewed the QSpec referenced wire and evaluation vectors against the new tests and production reader. The initial review found two incomplete acceptance assertions; both were fixed in the disposition pass.

## Verdict

PASS after both recorded gaps were fixed. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Admitted carried body test never compares typed unit and carrier references with the QSpec vector, so incorrect retained bindings can pass AC-401 and AC-20. | tests/conformance_qspec/carried_quantity.rs:322 | correct-requirement-no-evidence |
| FND-002 | medium | Vector loops do not assert the required CQ-W01..CQ-W15 and CQ-E01..CQ-E20 ID sets; omission from upstream JSON silently reduces AC-404 and AC-405 coverage. | tests/conformance_qspec/carried_quantity.rs:278 | correct-requirement-no-evidence |

## Coverage

Examined FR-038-AC-401 through FR-038-AC-405 and FR-019-AC-20, QSpec's 15 wire and 20 evaluation cases, the `#[trace]` bindings, test package builder and reader assertions. Current QSpec main contains duplicate FR-322-AC-301 entries for carried quantity and operation effects; this review uses CQ IDs, TC-187 U41..U44 and TC-280 MU-31..MU-34 until that upstream numbering is resolved. The repository matrix command did not produce a readable result in this pass; trace coverage was checked directly in the changed tests. Plan completion: not assessed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The admitted-node test now resolves the expected carried node and compares both typed `unit` and `carrier` references against the QSpec vector identities; it also checks both nodes in the distinct case. |
| FND-002 | fixed | The test now compares the wire and evaluation vector ID sets with the complete CQ-W01..CQ-W15 and CQ-E01..CQ-E20 inventories, and rejects duplicate IDs. |

## Disposition evidence

The fix was checked on the frozen post-fix head. The original carried-quantity implementation commit has the same stable patch content after rebase. The focused CQ conformance, Clippy and formatting results were supplied by the lead and were not rerun in this reviewer pass. No additional finding arose from the fix slice.
