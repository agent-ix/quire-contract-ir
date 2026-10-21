---
id: SR-15323
title: "IR-571 evidence review"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038, FR-346; spec/checked_package/matrix/TC-225; current TC-225 implementation; QSpec FR-154-AC-14"
review_set: subset
---

## Summary

The TC-225 test plan lacks QSpec’s positive case-sensitive name distinction control. The lead's exact-base comparison reports unchanged inherited strict coverage: 45 unbacked rows and zero contradicted statuses on both the base and draft branch; the changed criterion's new selection-row cases are still unimplemented.

## Verdict

**CONDITIONAL** — The TC-225 test plan lacks QSpec’s positive case-sensitive name distinction control. The lead's exact-base comparison reports unchanged inherited strict coverage: 45 unbacked rows and zero contradicted statuses on both the base and draft branch; the changed criterion's new selection-row cases are still unimplemented.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | QSpec FR-154-AC-14 explicitly admits distinct valid parameter names differing only in letter case. TC-225 adds a baseline from/to and invalid mutations but no case-only positive control; a case-folding reader could satisfy all new negative cases. Add an admitted same-operation pair such as from/From with matching distinct identities and a frame binding both. | spec/checked_package/matrix/TC-225-checked-package-v2-abstraction-relation-body.md |

## Method note

The full-repository `quoin advise --mismatch-only` query did not complete during the scoped review and was stopped. The changed FR-346-AC-7 retains `Test` as its authored verification method; this finding is a manual check of TC-225's ability to distinguish case-sensitive names, not a catalog mismatch claim.

## Dispositions

- FND-001 — fixed. TC-225 now requires an admitted `rebase(from, From)` with
  distinct identities ending in `from` and `From` and a frame entry binding
  both names.
