---
id: SR-15432
title: "IR-276 PR 510 shared closure and projection cache specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-276; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md; spec/model/functional/FR-023-executable-projection-binding.md"
review_set: base
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-050
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-023
    type: references
---

## Summary

Ticket: IR-276. Reviewed PR #510 at bfb5ba1aba9212f3290f307cd086d1cba4f2603c. The retained-adjacency and closure wording permits result-preserving memoized lowering. Its per-request node, term and edge charges, limit-stop consumed value, record contents and canonical bytes remain explicit obligations. The revised main-base measurement in Linear comment 36228c29 confirms the nine pre-change lowering counts K*(R+1): 10/18/34, 20/36/68 and 40/72/136 for K=2/4/8 and R=4/8/16.

## Scope

- FR-038 admission paragraph (examined): "Admission retains the validated body-reference adjacency and term counts for lowering; lowering uses those admitted facts rather than parsing a body again for each request."
- FR-038 lowering paragraph (examined): "Reusing an admitted graph or closure does not discount any request's work: each request reports the same consumed counter and terminal outcome it would report when traversed independently."
- FR-038-AC-308, new shared-closure row (examined): "The counted grammar validations are independent of K; a grouped recursive component and a stop within the shared closure preserve the same outcomes."
- TC-050 shared closure section (examined): "Admission validates each body once; lowering validates none again and reuses the retained graph."
- FR-023 input paragraph (examined): "The decoder compiles this fixed embedded schema once per process and reuses it for later projections. Each decode indexes its declaration population once; lookups by declaration name use that index while preserving diagnostic order."
- FR-023-AC-7 (examined): "Across repeated valid and invalid projection decodes, the fixed embedded schema is compiled once per process, and a decode with D declarations and B binding/declaration lookups performs at most D+B declaration search visits; the same inputs yield byte-identical bound identities or the same ordered diagnostics as before caching and indexing."
- TC-050 and FR-023 rows in the respective matrix indexes (context only): both still enumerate only pre-change criteria.

## Verdict

**CONDITIONAL** — The proposed mechanism and preservation invariants are coherent, but the new AC states an impossible K-independent total validation count. The test mappings also need to name the new criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The claim that total counted grammar validations are independent of K contradicts K distinct leaf bodies validated once each: the expected admission count is R+K, with zero lowering validations, versus the measured old lowering count K*(R+1). State the R+K total and zero lowering bound, or limit K independence explicitly to validations of the R shared bodies. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5024; spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md:74 |
| FND-002 | medium | TC-050's Description and its row in the checked-package matrix still enumerate only old FR-038 criteria, so FR-038-AC-308 is absent from the declared TC-050 coverage mapping. | spec/checked_package/matrix/TC-050-checked-package-v2-lowering.md:13; spec/checked_package/matrix/tests.md:43 |
| FND-003 | medium | The model matrix still maps FR-023 only through AC-6, leaving the new AC-7 schema-cache and declaration-visit criterion absent from the declared TC-035 coverage range. | spec/model/matrix/tests.md:18; spec/model/functional/FR-023-executable-projection-binding.md:139 |

## Review notes

The revised main-base measurement is a claimed test result in the Linear ticket, not a gate run by this reviewer. The arithmetic and duplicate ID were independently checked against the rebased spec. Scope was restricted to the PR's three Markdown changes and their declared matrix links; no Cargo, Kani or full test gates were run.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 578ca1a, FR-038-AC-308 says: "Admission performs R+K body grammar validations for the R shared nodes and K distinct leaves, and lowering performs zero additional validations"; this matches the K distinct leaves and removes the impossible K-independent total. |
| FND-002 | fixed | At 578ca1a, TC-050's Description names FR-038-AC-308, and the checked-package matrix adds a planned/unrun FR-038-AC-308 row and includes it in TC-050's trace list. |
| FND-003 | fixed | At 578ca1a, the model matrix adds a planned/unrun FR-023-AC-7 row mapped to TC-035 while leaving AC-1 through AC-6 implemented credit intact. |
