---
id: SR-612070
title: "IR-1059 PR 638 intake refusal code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 638 tests/conformance_qspec/model_tuple_union.rs and crates/quire-contract-model/src/checked_package/v2/model_members.rs; FR-069-AC-6"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-069
    type: reviews
---

## Summary

Ticket: IR-1059. Reviewed PR #638 at 75674cc3164010fe8a2ceb9b1a9e41a8f46db286 against QSpec's published TK-01..TK-10 intake rows and FR-069-AC-6. The production reader now carries a full-document pointer for selected tuple refusals, and the test compares the published expected locus; the test still bypasses part of the published mutation.

## Verdict

**CONDITIONAL** — the test reaches production intake with rederived digest and package identity, but published mutation drift can pass undetected. Focused Cargo test and hosted checks were pending during this source review. No CI workflow, copied QSpec file, unsafe code, dependency or public wire shape changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The test ignores the published patch operation and uses a literal empty positions array for TK-01. If QSpec changes TK-01's value or TK-03's operation, this test still executes the old mutation and can pass while claiming the new published row. Apply or assert each row's operation and use its value; check that TK-03 removes the expected member. | tests/conformance_qspec/model_tuple_union.rs:35 |

## Coverage

- FR-069-AC-6, role: examined. Excerpt: “Every published intake and wire mutation returns its exact QSpec code, cause and location at its specified stage, including duplicate or gapped tuple positions, absent/unsupported/unresolved position `typeRef`, illegal tuple members, unresolved/unsupported `Variant.payloadType`, malformed owner, stale tuple key and owner-kind mismatch.” This PR addresses only the ten intake rows; the wire and combined-defect matrix stays open.
- FR-069-AC-8, role: context_only. Excerpt: “`make conformance-qspec` loads the positive fixture, `model-tuple-union-vectors.json`, its full Semantic IR `selected_document` ... and runs every tuple/union case.” The full projection, identity and wire conformance is outside this PR.
- Source checked: the changed test helper and intake loop, selected tuple refusal location helper and all changed call sites, plus the public selection-to-refusal conversion in `v2/mod.rs`.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At 94cb7acd, the helper asserts each published operation (`remove` TK-03, `add` TK-04, `replace` otherwise), applies TK-01's published value, and asserts TK-03 removed an existing `typeRef`. A change to the published operation or TK-01 value now changes the test outcome or the applied document. |

## Disposition verdict

Round 1 at 94cb7acd5a3c0c30257d67eca27c5643ed25c196: FND-001 fixed on source inspection. The focused Cargo test remained queued at this review point; the lead owns the final gate result.
