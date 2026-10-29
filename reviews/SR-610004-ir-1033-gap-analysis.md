---
id: SR-610004
title: Gap analysis — IR-1033 owner model carrier
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/{application_identity,encode,intake,mod,owner,protocol_body,structural}.rs;
  tests/{it,conformance_qspec}
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-064
  type: references
---

## Summary

IR-1033 PR #592; examined FR-064-AC-1..6 and FR-038 owner/catalog context.

## Verdict

**FAIL** — FR-064-AC-5 remains unimplemented and untagged for PR #586.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-064-AC-5 has no tagged test and no owner-based complete class-catalog graph join in this PR; it remains Planned for the designated #586 follow-up and is not claimed as delivered here. | spec/checked_package/functional/FR-064-bind-protocol-clause-to-owner-model.md:92 |

## Coverage

Plan completion: not assessed. Reconciliation: quoin matrix 0.29.1 on this checkout; 717 criteria, 607 tagged, 92 untagged, 17 method-without-symbol, 1 tagged-by-ignored-test; all 717 have no run evidence in Quoin. FR-064: AC1-4 tagged, AC5 untagged, AC6 method-without-symbol. Repository baseline gaps predate this PR and are outside this diff review. No new unowned production behavior or stub found in the diff. Semantic review not separately opted in.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | IR-874 / draft PR #586 owns the full owner-based class-catalog join and its FR-064-AC-5 test after this carrier. AC-5 remains Planned, unclaimed and untagged on this PR. |

## Current verdict

The open catalog obligation is explicitly deferred to IR-874/#586 and is not counted as passing.
