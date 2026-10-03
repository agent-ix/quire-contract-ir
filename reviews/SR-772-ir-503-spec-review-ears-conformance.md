---
id: SR-772
title: "EARS and AC-shape conformance review of PR 254 (IR-503)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@731e307dc132975d3f3f6fac172c8c81bfe83b18; spec/checked_package/functional/FR-038-consume-checked-package-v2.md AC-65..68 and new prose; make spec grammar before/after"
review_set: subset
---
# SR-772: EARS and AC-shape conformance review of PR 254 (IR-503)

## Summary

Ticket: IR-503. In this repo, criteria are direct assertions of an observable outcome, and quire's
grammar flags `shall` in a criterion. None of AC-65..68 uses `shall` or an obligation shape. Each
names its input and its outcome (code, cause, pointer, admission, or a catalog read error).
`make spec` grammar: 299/300 docs grammar-clean, with 1 finding before and after (the FR-014
`ac:vague-response` baseline). No new grammar finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-67 bundles three unrelated assertions: precedence against `unknown-operation` and `operation-class-mismatch`, least-digest reporting across two nodes, and stray-member mismatch. The third already holds on main, where an unknown member kind mismatches through `member_kind_class`, and it has its own scoping defect (SR-771 FND-001). Moving the member clause into its own criterion would let it be scoped and traced on its own. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1005 |
| FND-002 | low | "The members the catalog declares beyond the earlier set" and "the `profile_operator` member the entry carried earlier" (line 662) are relative to a past state the spec does not name. Read later, "earlier" points at nothing. Name the six words as the vocabulary members (the list already follows), and describe the clause's former shape as removed rather than "earlier". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:604 |

## Verdict

Conforms, with two low wording findings.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee (delta from 731e307dc132975d3f3f6fac172c8c81bfe83b18; base origin/main af733f23f42788ea2c8dfa1eb31adaff39e85960 unchanged; GitHub MERGEABLE). `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/209 rows backed, FR-038 42/67. AC ids now run to AC-69, so the next free id is AC-70.

Notes on the round-1 checks: "the earlier set" is gone. The prose now says the catalog's operator classes "include" the six words, and the clause's former member is described as one the entry "no longer admits".

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |
| FND-002 | fixed | 7b11d6ee032c69e0ed36fcb5a40e5d25f7232eee |
