---
id: SR-20261104020013
title: "IR-1322 PR 884 protocol refusal failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; PR 884; protocol mutation refusal precedence and loci"
review_set: subset
---

## Summary

Scoped failure-domain analysis of the planned protocol reader refusal rules.

## Verdict

PASS. The criteria separate strict-schema refusal from semantic-reader refusal, keep the primary pointer distinct from document_pointer, and name the first winning refusal for missing temporal-profile selection and unattached control membership. They require no admitted or partial protocol view after refusal. AC-459 asserts the published diagnostic fields rather than inferring success from QSL's replay.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

- FR-038-AC-455: schema versus reader stage and first schema path.
- FR-038-AC-456: node-level primary locus, field document_pointer and no partial view.
- FR-038-AC-457: missing selection before operation-law refusal.
- FR-038-AC-458: target-kind retention and membership before clause run-reference check.
- FR-038-AC-459: unexpected admission and wrong diagnostic fields fail the by-reference test.

## Verification and Limits

Checked QSpec protocol manifest rows V2, V5, V8-V11, RP6, V16, CP28, C11 and C12; QSpec FR-464 Membership and containment and Reader order; FR-272 document_pointer semantics. M1's vector correction and V3/V4/V7/L19 prose reconciliation remain QSpec-owned and were not specified as IR reader obligations here.
