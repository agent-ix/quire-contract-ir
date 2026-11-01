---
id: SR-202610310119
title: "IR-1218 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1218 stack; protocol Location projection and eight changed paths"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
---

## Summary

Independent code-review pass on the captured protocol Location reader and its owner-backed conformance evidence.

## Verdict

FAIL: Do not merge while the findings remain open; shared-base validation also remains pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Owner requires both admission and refusal for an identical captured row; required-term absence cannot be authenticated by this carrier | crates/quire-contract-model/src/checked_package/v2/protocol_locations.rs:44; tests/conformance_qspec/protocol_locations.rs:147 |
| FND-002 | medium | The original PL-R2 regression discarded expected_code, expected_cause and locus and asserted admission instead | tests/conformance_qspec/protocol_locations.rs:147 |

## Scope and evidence

Ticket: IR-1218, PR #759, application version 0.1.0. Scope is the stack delta against the IR-1214 sidecar branch, not its entire delta against main. Examined all eight changed paths: protocol_locations.rs, dispatch_projection.rs, V2 mod.rs, model lib.rs, FR-019 inventory row, FR-354, conformance main.rs and conformance protocol_locations.rs. The canonical checked_term_data types and shared envelope belong to the base owner and are not duplicated here. No applicable AssuranceProfile or required enforced review policy was found under spec/. No workflow or dependency changes, shims, evaluator, QSL dependency, copied fixture, unsafe operation or mirror semantic type are introduced.

Owner vectors were read directly from the QSpec checkout. Initial parent-run Location conformance log contains nine passing tests; review found that this was false-green for PL-R2 because its expected refusal was discarded. Parent has corrected that test to assert the recorded code, cause and locus; no current green result is claimed after that correction. The same known base Clippy failure is E0063 at model_fields/tests.rs:1049, missing checked_term_data in a shared envelope initializer, already reported to IR-1214. Initial format drift was corrected by the parent; final format and final merged-base checks remain pending. This reviewer ran no Cargo or Kani command. Final base integration and disposition remain mandatory before merge.

## Analysis

Borrowed canonical views, local application/occurrence joins, shape limits, ordinal normalization, exact-key lookup, error distinctions, finite row/path charging and public exports were examined. No additional concrete source defect was found.

The positive PL-01-root row, after removing its id, is exactly equal to PL-R2-missing-term-mapping.value[0], yet PL-R2 requires invalid_package / invalid-value at /checked_term_data/protocol_locations. The carrier contains no required-term census or omitted term witness. Thus admission cannot satisfy both owner oracles without additional owner evidence or a clarified selection-time refusal contract. Do not infer a missing term or narrow the requirement; keep this PR blocked until the owner resolves the contradiction.

FND-002 records the original reviewed test. The parent already replaced its admission expectation with the owner-recorded refusal assertions; this makes the mismatch honest rather than completing FND-001.

## Dispositions

Round 1 of the same independent review.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | Owner positive/negative rows are indistinguishable with no census; requires owner witness or explicit clarified contract. Planner fallback is recorded on IR-1218 comment fcc69782. No merge or AC narrowing is accepted. |
| FND-002 | fixed | Owner-recorded refusal assertions replace the false admission expectation; functional owner contradiction remains separately blocked. |

## Same-review final validation

The parent added `reversed_distinct_term_keys_refuse_at_the_first_out_of_order_row`; the inspected focused log reports one passed, zero failed or ignored. It reverses distinct term keys and asserts `invalid_package`, `invalid-value` and the second row's exact pointer. The corrected PL-R2 test was also executed separately: zero passed, one failed because the production reader returned Admitted rather than the owner-recorded refusal. That red is retained honestly; it is not a passing conformance result or a completed feature. The parent confirmed that positive PL-01-root, after removing its id, equals the entire sole PL-R2 row. No additional owner census distinguishes them.

The draft remains unmerged and blocked pending an owner witness or clarified contract; no inference, shim, skipped oracle or acceptance narrowing was introduced. Shared-base E0063 remains separately tracked on IR-1214. This same-review disposition ran no Cargo or Kani command and made no source edit.

Round 2 of the same independent review.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | The corrected owner-oracle test actually fails because read returns Admitted. PL-01-root equals the entire sole PL-R2 row after id removal; the canonical carrier has no expected-term census. Owner witness or explicit clarified refusal contract remains required; IR-1218 comment fcc69782 records the planner handoff. Shared-base Clippy E0063 is separately recorded on IR-1214 ac485467. No merge or AC narrowing is accepted. |

FND-002 remains fixed from round 1: the test checks owner code, cause and locus. Its now-executed red result confirms that the false-green expectation is gone, not that the functional blocker is resolved.
