---
id: SR-8501
title: IR-770 base review of selected record frame fields
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md,
  spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md,
  spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Ticket IR-770. Reviewed the four-file PR diff against QSpec origin/main FR-322/FR-340 and Contract IR FR-038/FR-040 owner and refusal rules. Examined units: FR-038 intake, FR-040, FR-040-AC-3, AC-14, AC-15, AC-16, TC-056 and its matrix row.

## Verdict

**CONDITIONAL** — The owner-refusal stage and locus sentence conflicts with the upstream reader order.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new sentence says all three listed refusals, including an unrecovered or stale record owner, are FR-340 frame-entry refusals at the entry. Current QSpec FR-322 and FR-340 instead reject missing-selection or stale-node-key at the earlier derived-shape owner/key stage, before any frame entry is checked. AC-16 itself preserves that pre-frame precedence. Limit the at-entry rule to missing and ambiguous field names, and locate owner/key refusals at the earlier node/key stage. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:114 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
