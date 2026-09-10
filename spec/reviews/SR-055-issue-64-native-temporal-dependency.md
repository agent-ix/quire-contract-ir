---
id: SR-055
title: "Dependency review of issue 64 native temporal TL correspondence"
type: SpecReview
analysis: dependency
scope: "FR-025, FR-026, issues 52/57/63/64, and external native/TL authorities"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: reviews
---
# SR-055: Dependency review of issue 64

## Summary

The dependency review separates semantic authorities from Contract IR bridge
work at exact snapshot `558c4dccbed3128922517e2ec49cf6779817e9b6`.
FR-026 is specification-complete but implementation-blocked. Its branch is
stacked on FR-025/#66, while broader M0 release work remains an independent
admission hold.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6431 | high | **Fixed:** the dependency list omitted native result, progress, closure, completeness, observation, clock, capture, and correction authorities. FR-026 now names the relevant quire-specification requirements and requires their accepted public contracts/readers. | FR-026 relationships and Dependencies; quire-specification FR-061/094/110/112/113 | missing-requirement |
| FND-6432 | high | **Fixed:** internal TL Rust types could have been mistaken for public immutable contracts. FR-026 now requires selected formula, semantic, trace, evaluator-request, evaluator-report, and normalized-result contracts/readers; branch heads and copied schemas do not satisfy admission. | FR-026 Inputs and Dependencies; tl-syntax FR-003/004; tl-mltl FR-001/003/007 | missing-requirement |
| FND-6433 | medium | **Controlled:** #65 depends on FR-025/#66 and must remain stacked until #66 is independently reviewed and merged. The exact snapshot has #66 head `de101b9` as an ancestor; publication must retarget #65 to that branch before review. | quire-contract-ir #65/#66; FR-026 Dependencies | correct-requirement-no-evidence |
| FND-6434 | medium | **Controlled:** mergeable M0 heads, dependent rebases, v0.1 epics, and tags remain outside this specification PR. FR-026/TC-039 stays planned and cannot be represented as implementation or release closure while those admission rules remain open. | TM-002 FR-026/TC-039; TL v0.1 release lane | correct-requirement-no-evidence |

## Dependency Order

1. Independently review and land #66/FR-025.
2. Accept the native temporal, result, observation, clock, capture, progress,
   completeness, correction, and availability contracts/readers.
3. Publish immutable TL formula, semantic, trace, request, evaluator-report,
   and normalized-result contracts/readers.
4. Implement FR-026 and execute TC-039 against those exact selections.
5. Consume the bridge from issue #57's output-only FRETish mapping.

Issue #52 coordinates integration but supplies no missing semantic authority.
Ticket existence, a draft PR, a Rust type, or agreeing prose does not satisfy a
dependency.

## Result

**PASS with implementation parked.** The dependency graph and resume
conditions are explicit; no missing authority is inferred as coverage.
