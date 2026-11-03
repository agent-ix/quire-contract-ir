---
id: SR-20261103150648
title: "IR-1273 canceled source-integrity specification cleanup review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #858 FR-038, FR-019, AD-004, AD-007"
review_set: subset
---
# IR-1273 canceled source-integrity specification cleanup review

## Summary

Reviewed PR #858 at `3b56e327` against current IR main and the user's decision to reject the proposed source-byte digest as needless ceremony. No AssuranceProfile applies under `spec/`. The functional requirements correctly remove the proposed reader behavior, but both edited architecture decisions retain literal commit SHAs as revision records.

## Verdict

**FAIL** — remove the recorded revisions from the two changed architecture decisions under the repository's hash/digest/pin rule.

## Scope

Examined the removed FR-038 source-byte-integrity section and AC-437 through AC-442; the removed FR-019 planned public API section, inventory row and AC-25; AD-004's checked-package seam and risks; AD-007's dependency direction and risks. Searched the remaining IR specification for `source_integrity`, `source-integrity`, source-byte integrity and IR-1273 references. The remaining mentions describe the still-published QSpec/QSL conflict and assign correction to those owners; they impose no IR reader implementation. No test or other spec artifact still cites the removed acceptance criteria. The current two-member `CheckedSourceRef` and unrelated reader rules remain intact.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The edited checked-package seam retains literal commit SHAs as measured-source revision records, contrary to the repository's hash/digest/pin prohibition and the spec-review checklist's ban on recorded or reviewed revisions. | spec/assurance/AD-004-checked-package-seam.md:214 |
| FND-002 | high | The edited cross-repo dependency decision retains literal commit SHAs as revision records for IR, QSL, QSpec, CG, RT and QVC, contrary to the same rule. | spec/assurance/AD-007-cross-repo-dependency-graph.md:30 |

## Dispositions

At PR #858 head `9cb66f63d0636fcb5f8acdf644c16b19fee5ade3`, both findings are fixed in the same PR. The original review verdict and findings above remain as the record of the first review pass.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AD-004 now cites QSpec, QSL, CG and IR source paths and lines without literal commit SHAs or recorded/reviewed revisions. |
| FND-002 | fixed | AD-007 now cites repository paths and line locations without literal commit SHAs or recorded/reviewed revisions. |

The FR-038 source-byte-integrity section and AC-437 through AC-442 remain removed, as do FR-019's planned API section, inventory row and AC-25. The current two-member `CheckedSourceRef` stays intact. AD-004 and AD-007 continue to state the unresolved QSpec/QSL published-text conflict and route it to those owners. Current disposition: **PASS**.
