---
id: SR-202610310122
title: "IR-1218 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-1218 stack; protocol Location projection and eight changed paths"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
---

## Summary

Independent integrity pass on the captured protocol Location reader and its owner-backed conformance evidence.

## Verdict

FAIL: Do not merge while the findings remain open; shared-base validation also remains pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Required-term completeness assumes a term census absent from the canonical producer carrier | spec/checked_package/functional/FR-354-project-admitted-runtime-context.md:110 |

## Scope and evidence

Ticket: IR-1218, PR #759, application version 0.1.0. Scope is the stack delta against the IR-1214 sidecar branch, not its entire delta against main. Examined all eight changed paths: protocol_locations.rs, dispatch_projection.rs, V2 mod.rs, model lib.rs, FR-019 inventory row, FR-354, conformance main.rs and conformance protocol_locations.rs. The canonical checked_term_data types and shared envelope belong to the base owner and are not duplicated here. No applicable AssuranceProfile or required enforced review policy was found under spec/. No workflow or dependency changes, shims, evaluator, QSL dependency, copied fixture, unsafe operation or mirror semantic type are introduced.

Owner vectors were read directly from the QSpec checkout. Initial parent-run Location conformance log contains nine passing tests; review found that this was false-green for PL-R2 because its expected refusal was discarded. Parent has corrected that test to assert the recorded code, cause and locus; no current green result is claimed after that correction. The same known base Clippy failure is E0063 at model_fields/tests.rs:1049, missing checked_term_data in a shared envelope initializer, already reported to IR-1214. Initial format drift was corrected by the parent; final format and final merged-base checks remain pending. This reviewer ran no Cargo or Kani command. Final base integration and disposition remain mandatory before merge.

## Analysis

Identity, ownership, refusal precedence, unavailable versus missing, complete borrowed output and finite resource behavior were checked. The positive and negative owner values are equal, so the assumption that admission can recognize the absent second semantic term is not observable from this wire. The positive PL-01-root row, after removing its id, is exactly equal to PL-R2-missing-term-mapping.value[0], yet PL-R2 requires invalid_package / invalid-value at /checked_term_data/protocol_locations. The carrier contains no required-term census or omitted term witness. Thus admission cannot satisfy both owner oracles without additional owner evidence or a clarified selection-time refusal contract. Do not infer a missing term or narrow the requirement; keep this PR blocked until the owner resolves the contradiction.

## Dispositions

Round 1 of the same independent review.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | Owner positive/negative rows are indistinguishable with no census; requires owner witness or explicit clarified contract. Planner fallback is recorded on IR-1218 comment fcc69782. No merge or AC narrowing is accepted. |

## Same-review final validation

The parent added `reversed_distinct_term_keys_refuse_at_the_first_out_of_order_row`; the inspected focused log reports one passed, zero failed or ignored. It reverses distinct term keys and asserts `invalid_package`, `invalid-value` and the second row's exact pointer. The corrected PL-R2 test was also executed separately: zero passed, one failed because the production reader returned Admitted rather than the owner-recorded refusal. That red is retained honestly; it is not a passing conformance result or a completed feature. The parent confirmed that positive PL-01-root, after removing its id, equals the entire sole PL-R2 row. No additional owner census distinguishes them.

The draft remains unmerged and blocked pending an owner witness or clarified contract; no inference, shim, skipped oracle or acceptance narrowing was introduced. Shared-base E0063 remains separately tracked on IR-1214. This same-review disposition ran no Cargo or Kani command and made no source edit.

Round 2 of the same independent review.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | The corrected owner-oracle test actually fails because read returns Admitted. PL-01-root equals the entire sole PL-R2 row after id removal; the canonical carrier has no expected-term census. Owner witness or explicit clarified refusal contract remains required; IR-1218 comment fcc69782 records the planner handoff. Shared-base Clippy E0063 is separately recorded on IR-1214 ac485467. No merge or AC narrowing is accepted. |
