---
id: SR-20261107060894603
title: IR-1396 PR 946 carried_member spec-review/base
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 946 head 186b6d2b; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (L2154-2161, FR-038-AC-495), spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1396. Spec review of the added lines only. `quire validate` passes on the three changed files; its one `ac:non-canonical-shape` warning, at L5908, is outside the diff. The prose cites QSpec FR-322 as the owner of the meaning and does not copy it.

## Verdict

CONDITIONAL: AC-495 does not record the owner's refusal order, and its no-panic clause reads as broader than what it requires.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-495 states the `carried_member` refusal without the owner's precedence. QSpec FR-322 L3300-3311 decides it only "with the pinned mode agreeing", after `operation-mode-type-mismatch`. As written, the AC is satisfied by an implementation that refuses `ill_typed` where the owner requires `invalid_package`, and the code does exactly that (SR-20261107060894601 FND-002). Add the precedence clause and a case for it. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6010 |
| FND-002 | low | AC-495's third sentence ("returns the typed catalog read error ... and never panics") repeats FR-038-AC-65. It speaks only of supplied catalog bytes, while the production catalog path still panics under FR-038-AC-58. A reader can take it to satisfy ticket AC2, but it does not. Drop the sentence, or scope it so it plainly differs from the production read. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6010 |

## Coverage

- FR-038 L2154-2161 prose, examined: adds `carried_member` to the constraint-kind list and states its FR-322 meaning.
- FR-038-AC-495, examined.
- TC-048 Catalog words paragraph, examined: consistent with AC-495.
- tests.md FR-038-AC-495 row, examined.
- FR-038-AC-65 and FR-038-AC-58, context_only.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | high | The new FR-038 text contradicts itself. L964-966 says the production catalog "is a build-time input and not a caller's bytes, so an entry of another shape is never a package refusal", and the added L967-971 says every application then "refuses `invalid_package`/`unknown-operation`". FR-038-AC-58 likewise says "this is a catalog read failure and not a package refusal", then makes unreadable production bytes a package refusal. Two implementers would read it in opposite ways, and as written it requires the cause misreport in SR-20261107060894601 FND-004. State instead that unreadable production catalog bytes return a distinct reader error naming the unreadable entry and word, never a package refusal and never a panic. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:964-971 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | AC-495 now reads "with the pinned mode agreeing ... A pinned rounding mode that disagrees with the types refuses invalid_package/operation-mode-type-mismatch first", and TC-048 adds the nearest-even case. |
| FND-002 | fixed | The repeated AC-65 sentence is gone. AC-495 now says "a package read never panics on the production catalog bytes (FR-038-AC-58)"; the cause it should report is FND-003. |
