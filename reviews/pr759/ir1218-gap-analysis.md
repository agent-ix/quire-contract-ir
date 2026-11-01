---
id: SR-202610310120
title: "IR-1218 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1218 stack; protocol Location projection and eight changed paths"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
---

## Summary

Independent gap-analysis pass on the captured protocol Location reader and its owner-backed conformance evidence.

## Verdict

PASS for this method after the sorted-row evidence fix. The overall PR remains blocked by the separately documented owner-contract contradiction and shared-base validation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No adverse control exercises the sorted distinct-row-key refusal; reversing two valid term keys remains untested | crates/quire-contract-model/src/checked_package/v2/protocol_locations.rs:87; tests/conformance_qspec/protocol_locations.rs:78 |

## Scope and evidence

Ticket: IR-1218, PR #759, application version 0.1.0. Scope is the stack delta against the IR-1214 sidecar branch, not its entire delta against main. Examined all eight changed paths: protocol_locations.rs, dispatch_projection.rs, V2 mod.rs, model lib.rs, FR-019 inventory row, FR-354, conformance main.rs and conformance protocol_locations.rs. The canonical checked_term_data types and shared envelope belong to the base owner and are not duplicated here. No applicable AssuranceProfile or required enforced review policy was found under spec/. No workflow or dependency changes, shims, evaluator, QSL dependency, copied fixture, unsafe operation or mirror semantic type are introduced.

Owner vectors were read directly from the QSpec checkout. Initial parent-run Location conformance log contains nine passing tests; review found that this was false-green for PL-R2 because its expected refusal was discarded. Parent has corrected that test to assert the recorded code, cause and locus; no current green result is claimed after that correction. The same known base Clippy failure is E0063 at model_fields/tests.rs:1049, missing checked_term_data in a shared envelope initializer, already reported to IR-1214. Initial format drift was corrected by the parent; final format and final merged-base checks remain pending. This reviewer ran no Cargo or Kani command. Final base integration and disposition remain mandatory before merge.

## Analysis

Plan completion: not assessed. Computed matrix baseline: 714 tagged criteria, 142 untagged, 22 method-without-symbol and one ignored-only. FR-354-AC-9 is tagged by all nine Location conformance tests; FR-019-AC-5 has its existing TC-058 public inventory controls. FR-354-AC-8 remains method-without-symbol and was inspected here, not represented as executable evidence. FR-354-AC-7 has existing nominal tests; no broader closure claim is made.

The scoped semantic evidence audit found that deleting the key<prior branch would leave every current Location test unchanged: positives use ascending term ordinals and the duplicate mutation uses equal keys. Add a reversed pair of distinct keys and assert row1 invalid_package / invalid-value. Separately, the PL-R2 owner oracle is contradictory as documented by the code-review; its trace tag cannot make it satisfied.

## Dispositions

Round 1 of the same independent review.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | A reversed distinct-row-key adverse control has not landed yet; the parent has been given this evidence gap. |

## Same-review final validation

The parent added `reversed_distinct_term_keys_refuse_at_the_first_out_of_order_row`; the inspected focused log reports one passed, zero failed or ignored. It reverses distinct term keys and asserts `invalid_package`, `invalid-value` and the second row's exact pointer. The corrected PL-R2 test was also executed separately: zero passed, one failed because the production reader returned Admitted rather than the owner-recorded refusal. That red is retained honestly; it is not a passing conformance result or a completed feature. The parent confirmed that positive PL-01-root, after removing its id, equals the entire sole PL-R2 row. No additional owner census distinguishes them.

The draft remains unmerged and blocked pending an owner witness or clarified contract; no inference, shim, skipped oracle or acceptance narrowing was introduced. Shared-base E0063 remains separately tracked on IR-1214. This same-review disposition ran no Cargo or Kani command and made no source edit.

Round 2 of the same independent review.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Reversed distinct keys now assert the exact typed refusal and row1 pointer; focused execution passes once. |
