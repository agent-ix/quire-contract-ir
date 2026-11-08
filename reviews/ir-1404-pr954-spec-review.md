---
id: SR-202611080004003
title: IR remediation batch 1 (PR 954) spec review
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 954 head b3363171 vs main; spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md, spec/temporal/matrix/TC-461-native-temporal-tl-correspondence.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-519..529 renumbering), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Tickets: IR-1005 (FR-350 correction), IR-1396, IR-1405 and IR-1398 (FR-038 renumbering), IR-1396 (FR-019 inventory). One spec-review pass over the batch-only spec text. The renumbering moves the batch criteria past merged main's IR-1404 range: IR-1396 to AC-519, IR-1405 to AC-520..523, IR-1398 to AC-524..529. The new IDs are unique in FR-038, TC-048's relationships list each one, and TC-048's section headings use the new numbers. FR-019 now lists `CatalogReadError`. The FR-350 correction marks R4 and the owner byte-ceiling mapping as defensive, defines `L` for an absent side, and states the `max_bytes` fixed point.

## Verdict

Pass with one low ambiguity in FR-350-AC-8.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-350-AC-8 says, for `max_bytes`, that the smallest `s` with `record_len(s) <= s` admits and `s - 1` refuses. It does not say what the refusal's `value` is, and the general clause in the same AC (`limit-reached{limit, value: n, setting: n - 1}`) does not fit. At `s - 1`, the measured length is `record_len(s - 1)`. That equals `s` only when `s - 1` has as many digits as `s`. When `s` is a power of ten, the embedded settings lose a digit, and `value` is not `s`. Two implementers could assert different values. Fix: say `value: record_len(s - 1)`. | spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md:614 |

## Coverage

- FR-350 R4 defensive row and the AC-5 exclusion, examined: consistent with the row table and TC-461 step 9.
- FR-350 `L` with an absent side, examined: clear and testable.
- FR-350 `max_bytes` fixed point (prose, AC-8, TC-461 step 11), examined: FND-001.
- FR-038-AC-519..529, examined: unique IDs, TC-048 relationships and headings updated, matrix binds each.
- FR-019 `checked_package` row, examined: lists `CatalogReadError`, `initialize_operation_catalog` and the budgeted entries.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-350 prose, AC-8 and TC-461 step 11 now state `value: record_len(s - 1)`, equal to `s` only when `s - 1` and `s` have the same number of digits. |
