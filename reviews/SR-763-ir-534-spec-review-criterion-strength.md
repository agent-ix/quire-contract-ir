---
id: SR-763
title: "criterion-strength review of PR 252 (IR-534)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@31d2f9654bd4136ae64c1500074ce01793382af8; spec/checked_package/functional/FR-038-consume-checked-package-v2.md AC-2/10/19/20/27/31/32/45/62/63/64; main's AC-19/AC-20/AC-32 for weakening comparison"
review_set: subset
---
# SR-763: criterion-strength review of PR 252 (IR-534)

## Summary

Ticket: IR-534. I compared each amended criterion with its text on origin/main (eedc378) for
silent weakening, and checked that each new criterion can fail.

- AC-2: gains `version`.
- AC-19, AC-32: lose only the version-specific cases, which no longer exist.
- AC-27: drops the version mismatch and gains a positive version-independence clause.
- AC-31: states the shape.
- AC-45: changes the invariance axis from version to row digest. QSpec FR-322-AC-28 makes the key
  content-only, so that is a deliberate reformulation.
- AC-62/63/64: each fails under today's reader (a version-less row is `malformed_wire`), so none
  is vacuous.

The one real weakening is in AC-20 (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | On main, AC-20 pinned that the identity-uniqueness class outranks the selection-evidence class: a same-identity pair refused `malformed_wire` even when one of its entries named a digest no supplied document satisfies. The amended prose puts the same-identity, different-digest check inside class 4. It says that check runs "whatever documents the evidence supplies, before any document is read", so it outranks a missing, forged or wrong-identity document on any row. The amended AC-20 pins only the case where both documents are supplied, plus the `digest_domain_mismatch` outranking. A reader that admits row 0's document first, and refuses `missing_import` there before it sees a same-identity pair at rows 1-2, passes every criterion. Add a clause: the pair refuses `stale_dependency` at the later entry even when an entry of the pair, or an earlier row, has no supplied document or a mismatched one, in both orders. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:874,219-229 |
| FND-002 | low | "The same package supplied as two documents at different versions admits under each one's own digest" can be read as one lock selecting both documents. AC-20 refuses that as `stale_dependency`. AC-64 says "each selected in its own package". Say the same in AC-27. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:881 |
| FND-003 | low | AC-20 was fully rewritten but still names the location as `lock.model_selections`. Class 4's prose says the later entry refuses "at its `digest`". The reader returns `/lock/model_selections/<i>/digest` (mod.rs:1113-1116), and AC-24 requires the pointer of the value. The author says AC-10's identical wording predates this PR. That is true (main's AC-10 and AC-19 use the same family-location style), so this is not a regression. Because AC-20 is new text, though, it should state the pointer exactly (the later entry's `digest`), so that a reader pointing at the array still fails it. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:874 |

## Verdict

Mostly strong, with one guarantee that lost its criterion (FND-001, medium) and two wording
weaknesses (low).

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@a1f44f02c78fa786dc3e5582a45f46a069d74bd5 (delta from 31d2f9654bd4136ae64c1500074ce01793382af8; base origin/main eedc378d7fe879e0d107e04318773528f9595422 unchanged; merge clean). `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/204 rows backed, FR-038 42/62. No new finding.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | a1f44f02c78fa786dc3e5582a45f46a069d74bd5 |
| FND-002 | fixed | a1f44f02c78fa786dc3e5582a45f46a069d74bd5 |
| FND-003 | fixed | a1f44f02c78fa786dc3e5582a45f46a069d74bd5 |
