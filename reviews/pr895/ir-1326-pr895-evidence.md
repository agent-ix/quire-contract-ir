---
id: SR-2026110414560007
title: "IR-1326 PR 895 evidence review"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; PR #895; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: reviews
---

## Summary

Ticket IR-1326. Reviewed PR #895 initial head cf6b0484 and final head 18003a84 against QSpec FR-370 and its published temporal aggregate argument vectors by reference. The initial TC-048 procedure built each mutation from its positive base and mutation argument but omitted the reader row profile. AR-01, AR-17 and AR-18 change profile, so a test could skip their intended refusal path.

## Verdict

PASS. The changed documents were checked with scoped Quire validation; the one grammar warning at an unchanged FR-038 criterion exists on the base. No Cargo gate applies to this spec-only PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Reader case profile override missing in the initial test procedure | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:661 |

## Coverage

- FR-038-AC-461, examined: the strict reader admits the published aggregate positives under their published profiles.
- FR-038-AC-462, examined: each published reader mutation returns its exact code, cause and prefixed pointer, with temporal-step precedence.
- TC-048 Window aggregate atom, examined: by-reference construction and exact positive/refusal oracles.
- QSpec FR-370-AC-17 and AC-18, context only: catalog shape and five ordered refusal checks.

## Evidence

QSpec main 860e089028fddad4f18058ca280f613c6fe58695 publishes 9 positive and 43 reader argument fragments. Three reader rows override their positive base profile. The IR spec PR copies no published vector or schema and adds no hash, digest or pin. Scoped Quire validation exits 0 for both changed files with the inherited warning; git diff --check exits 0. No applicable AssuranceProfile review policy was found under this repository's spec tree.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The updated TC-048 procedure uses each reader case’s published profile and argument, including profile overrides. |
