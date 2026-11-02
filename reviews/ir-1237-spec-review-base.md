---
id: SR-202611010122
title: "IR-1237 PR 803 FR-019 inventory spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 803; spec/model/functional/FR-019-rust-library-interface.md inventory row"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
---

## Summary

Ticket: IR-1237. Reviewed the sole changed FR-019 inventory row against the actual `CheckedProbabilisticClaim` root export, typed members, `ClaimForm` variant and FR-019-AC-5/18. No applicable AssuranceProfile was found. The unchanged FR-051 table-format gap is recorded in gap-analysis.

## Verdict

**PASS** — the changed public-item row accurately identifies the implemented type and sidecar field.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-019-AC-5, role: examined. Excerpt: The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands; exporting a planned item fails the inventory until its row is marked implemented. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path. The retired `PACKAGE_DOMAIN_V2` export is absent.
- FR-019-AC-18, role: examined. Excerpt: The default-feature model crate exports `ClaimForm::ProbabilisticClaim` and the public `CheckedProbabilisticClaim` row with exact typed `declaration: CheckedNodeId` and `occurrence: CheckedOccurrence` members, usable in the fourth required `CheckedTermDataV2::probabilistic_claims` array and borrowed `encode_checked_package_v2` input. A public consumer constructs nonempty rows without importing a private wire type or passing display name or source span as identity, and reads the same typed values from a strictly admitted package. The existing public-item inventory includes the row type when implementation lands.
- FR-019 Public items row for `CheckedProbabilisticClaim`, role: examined. Excerpt: type `CheckedProbabilisticClaim` with exact typed declaration and occurrence; the sidecar has its fourth required probabilistic claims array.
