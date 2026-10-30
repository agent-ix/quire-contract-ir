---
id: SR-2026102959
title: "Integrity review of IR-1141 public-boundary reconciliation"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-1141 PR #714 head 6eb0b5e1; FR-019, TC-048, TC-058; FR-046 and FR-048 context"
review_set: subset
---

## Summary

The changed artifacts share one intended structural identity contract and leave FR-046/048 live controls intact. The base review identifies stale inventory entries; this lens checks the separate present/future consistency rule.

## Examined units

FR-019 inventory time basis (examined); TC-058 Status and Expected Results (examined); FR-046/048 live criteria (context).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-019 says the table records the compiled public surface, but its new paragraph says current helper exports are intentionally absent from the required inventory until IR-1141 code lands. TC-058 still requires equality at the tested revision. State clearly that the table is the target required surface, identify current discrepancy as Planned, and preserve the exact post-cutover inventory test. | spec/model/functional/FR-019-rust-library-interface.md:181,260,282 |

## Verdict

FAIL until the inventory time basis is stated consistently. No applicable AssuranceProfile found in the affected spec tree.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-019 now calls the table the required target public surface, excludes planned entries from the current compiled inventory, identifies current legacy exports as pending removal, and keeps TC-058 exact inventory check for the final code revision. |
