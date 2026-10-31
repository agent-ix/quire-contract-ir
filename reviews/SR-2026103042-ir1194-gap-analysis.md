---
id: SR-2026103042
title: "IR-1194 PR 744 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #744; FR-352 scalar projection and tests in crates/quire-contract-model/src/checked_package/v2/exact_projection.rs"
review_set: subset
---

## Summary

The two traced unit tests verify scalar values and several refusal codes through the new projection method. They construct `CheckedPackageV2` directly with private fields and do not exercise the public reader or an external-crate caller. Quoin matrix was available; this review maps changed tests to FR-352 manually because the full repository matrix contains pre-existing open criteria outside this partial PR.

## Verdict

**CONDITIONAL** — One evidence gap in the delivered scalar slice. FR-352 AC2, AC3, AC9 and full admitted-package acceptance remain open pending the QSL identity producer; this partial PR does not satisfy them.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Scalar tests bypass public admission and external caller path | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:283-351 |

## Coverage

- **FR-352-AC-1 (examined):** The test checks Boolean, Integer, String, Uuid, Timestamp, and three inclusive ranges; external caller and reader admission are untested.
- **FR-352-AC-4 (examined):** The test checks changed path, ordinal, and package; declaration witness cases remain open.
- **FR-352-AC-5 (examined):** The test checks three scalar variants and one alias refusal; declared units, union, dimension cases remain open.
- **FR-352-AC-6 (examined):** One unsupported form refuses; other admitted forms use the fallback branch.
- **FR-352-AC-7 (examined):** One clone equality and exhausted-work outcome are checked; owner admission is untested.
- **FR-352-AC-8 (examined):** Cargo dependency and public exports inspected; no QSL mirror is added.
- **FR-352-AC-2, AC-3, AC-9 (context only):** Identity-bearing acceptance remains open.
- Plan completion: not assessed.

## Evidence

The fixture at lines 299-351 deserializes a private wire and directly initializes `CheckedPackageV2` with empty `structural_keys`, default retained models, and empty closure index. That bypasses `CheckedPackageV2::read` and its admission checks. Both new tests use this fixture. Consequently, the tests can pass if the new method works only on an unadmitted internal package and fails for a package returned by the public reader. A focused integration test through the public reader and public projection API would close the scalar-slice evidence gap. Semantic review beyond the changed code and test intent was not run.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `tests/it/checked_package_v2_reader.rs:52-108` now admits a package through the public reader and calls the public projection with Boolean, Integer and Int[0,1000], checks clone equality and finite exhaustion, and verifies a malformed candidate refuses admission. |

## Disposition Verdict

**PASS for the original finding** — The public admitted-package scalar path is tested. Complete FR-352 record view and the QSL SelectedExactIdentities producer remain open outside this partial PR.


## New findings (gate evidence recheck)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | AC-1 is marked backed by tests that do not cover all admitted range cases or external conversion | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:708-735; tests/it/checked_package_v2_reader.rs:52-95 |
| FND-003 | high | AC-2 is marked backed by a field-row test with no Composite or SelectedRecordView | crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs:234-294 |
| FND-004 | high | AC-3 is marked backed without a positive admitted Node reference or foreign/dangling controls | crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs:153-216; tests/it/checked_package_v2_derived_reference.rs:71-100 |
| FND-005 | high | AC-4 is marked backed without ambiguous-owner and alias/display-name join controls | crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs:19-80 |
| FND-006 | high | AC-5 is marked backed without declared-unit, union, dimension and compound-unit checks | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:708-735 |
| FND-007 | medium | AC-6 is marked backed by one unsupported-shape example, not the admitted-form partition | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:737-781 |
| FND-008 | medium | AC-7 is marked backed without nested or cyclic depth and no-reparse controls | crates/quire-contract-model/src/checked_package/v2/exact_projection.rs:737-781; tests/it/checked_package_v2_reader.rs:52-110 |
| FND-009 | medium | AC-8 is marked backed by a scalar projection test that never inspects inventories | tests/it/checked_package_v2_reader.rs:52-110 |
| FND-010 | high | AC-9 is marked backed without the QSL-side producer or external authoritative rows | crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs:19-319; tests/it/checked_package_v2_derived_reference.rs:71-100 |

## Exact trace-tag disposition

Remove every `FR-352-AC-*` trace tag from the eight current test functions, in both their `/// Trace:` lines and `#[trace(...)]` attributes. Retain all test bodies as partial regression checks; retain any unrelated trace IDs. Rebind an AC only when a test or cohesive test set exercises its full criterion.

| Test function | Remove FR-352 tags | Retain FR-352 tags | Reason |
| --- | --- | --- | --- |
| `scalar_and_inclusive_range_projection` | AC-1, AC-5 | none | private constructed package; no admitted unit/union controls |
| `unknown_foreign_unsupported_and_exhausted_are_distinct` | AC-4, AC-6, AC-7 | none | shallow refusal examples only |
| `exact_stage_rows_join_selected_original_declaration` | AC-4, AC-9 | none | local row constructor and selected-identity subcases only |
| `distinct_selected_objects_cannot_claim_one_exact_identity` | AC-3, AC-4 | none | duplicate-identity subcase only |
| `exact_reference_uses_the_selected_object_row` | AC-3, AC-9 | none | positive reference uses a privately mutated package and local row |
| `exact_field_rows_require_selected_record_and_original_field` | AC-2, AC-9 | none | field-row join only; no complete record projection |
| `exact_scalar_projection_uses_publicly_admitted_package` | AC-1, AC-4, AC-7, AC-8 | none | admitted scalar Int[0,1000] path only; no inventories |
| `admitted_reference_requires_selected_exact_identity_row` | AC-3, AC-9 | none | admitted missing-witness refusal only |

## Evidence recheck verdict

**FAIL** — On eb12804519dd410caa844d632de594aef84261f6, the computed matrix marks all nine FR-352 criteria backed, but current tests assert only subcases. The open identity producer, selected record view, and full admitted reference path make AC-2, AC-3 and AC-9 plainly incomplete. The other six tags also claim more than their assertions cover. Remove the tags now; keep the tests as partial regression checks. The full FR-352 criteria should return to unbacked until complete acceptance evidence lands. Plan completion: not assessed.

## Dispositions (gate evidence recheck)

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open | The FR-352-AC-1 tag remains on a test that verifies only part of the criterion; remove the tag until complete evidence exists. |
| FND-003 | still-open | The FR-352-AC-2 tag remains on a test that verifies only part of the criterion; remove the tag until complete evidence exists. |
| FND-004 | still-open | The FR-352-AC-3 tag remains on a test that verifies only part of the criterion; remove the tag until complete evidence exists. |
| FND-005 | still-open | The FR-352-AC-4 tag remains on a test that verifies only part of the criterion; remove the tag until complete evidence exists. |
| FND-006 | still-open | The FR-352-AC-5 tag remains on a test that verifies only part of the criterion; remove the tag until complete evidence exists. |
| FND-007 | still-open | The FR-352-AC-6 tag remains on a test that verifies only part of the criterion; remove the tag until complete evidence exists. |
| FND-008 | still-open | The FR-352-AC-7 tag remains on a test that verifies only part of the criterion; remove the tag until complete evidence exists. |
| FND-009 | still-open | The FR-352-AC-8 tag remains on a test that verifies only part of the criterion; remove the tag until complete evidence exists. |
| FND-010 | still-open | The FR-352-AC-9 tag remains on a test that verifies only part of the criterion; remove the tag until complete evidence exists. |


## Dispositions (round 3)

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | The FR-352-AC-1 trace tags were removed from the partial tests at 5504b130be6b92f91cc828083105e932d3c63d0f; no `FR-352-AC-*` tag remains in Rust test code. The tests continue to assert their subcases without claiming full criterion coverage. |
| FND-003 | fixed | The FR-352-AC-2 trace tags were removed from the partial tests at 5504b130be6b92f91cc828083105e932d3c63d0f; no `FR-352-AC-*` tag remains in Rust test code. The tests continue to assert their subcases without claiming full criterion coverage. |
| FND-004 | fixed | The FR-352-AC-3 trace tags were removed from the partial tests at 5504b130be6b92f91cc828083105e932d3c63d0f; no `FR-352-AC-*` tag remains in Rust test code. The tests continue to assert their subcases without claiming full criterion coverage. |
| FND-005 | fixed | The FR-352-AC-4 trace tags were removed from the partial tests at 5504b130be6b92f91cc828083105e932d3c63d0f; no `FR-352-AC-*` tag remains in Rust test code. The tests continue to assert their subcases without claiming full criterion coverage. |
| FND-006 | fixed | The FR-352-AC-5 trace tags were removed from the partial tests at 5504b130be6b92f91cc828083105e932d3c63d0f; no `FR-352-AC-*` tag remains in Rust test code. The tests continue to assert their subcases without claiming full criterion coverage. |
| FND-007 | fixed | The FR-352-AC-6 trace tags were removed from the partial tests at 5504b130be6b92f91cc828083105e932d3c63d0f; no `FR-352-AC-*` tag remains in Rust test code. The tests continue to assert their subcases without claiming full criterion coverage. |
| FND-008 | fixed | The FR-352-AC-7 trace tags were removed from the partial tests at 5504b130be6b92f91cc828083105e932d3c63d0f; no `FR-352-AC-*` tag remains in Rust test code. The tests continue to assert their subcases without claiming full criterion coverage. |
| FND-009 | fixed | The FR-352-AC-8 trace tags were removed from the partial tests at 5504b130be6b92f91cc828083105e932d3c63d0f; no `FR-352-AC-*` tag remains in Rust test code. The tests continue to assert their subcases without claiming full criterion coverage. |
| FND-010 | fixed | The FR-352-AC-9 trace tags were removed from the partial tests at 5504b130be6b92f91cc828083105e932d3c63d0f; no `FR-352-AC-*` tag remains in Rust test code. The tests continue to assert their subcases without claiming full criterion coverage. |

## Disposition Verdict (round 3)

**PASS for this partial API slice** — FND-002 through FND-010 are fixed. The diff removes only the eight partial test functions' FR-352 trace comments and attributes and one now-unused import; test bodies and unrelated tags remain. The repository has prior untagged partial regression precedent (`SR-612063-ir1059-gap-analysis.md`), so this repair adds no new trace-policy defect. Full FR-352 acceptance remains open and should appear unbacked until complete tests and the QSL identity producer land. No aggregate gate was run by this reviewer.
