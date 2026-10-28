---
id: SR-2026102732
title: "Gap analysis — tuple and union intake mapping rows"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-069-AC-3, FR-069-AC-4, FR-069-AC-6, TC-940; tests/conformance_qspec/model_tuple_union.rs and production reader"
review_set: subset
---

## Summary

Reviewed the changed public reader replay against FR-069 and QSpec's TK-11 through TK-35 published cases. The intake matrix compares admission and exact refusal code, cause and pointer, but it does not test using the newly admitted Duration type.

## Verdict

**CONDITIONAL** — add a public reader control that uses an admitted Duration tuple position and variant payload where the selected s unit resolves, and checks the resulting type or operation admission. FR-069-AC-6 correctly remains open and untagged for the complete intake and wire matrix.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TK-25/26 replay proves only intake admission, so it misses unusable Duration members | tests/conformance_qspec/model_tuple_union.rs:438 |

## Coverage

Plan completion: not assessed

The test loops over every published `intake_cases` row and asserts all numbered TK-01 through TK-35 are present. It checks exact refusal code, cause, selection row and selected-document pointer for refused rows. It retains only model graph nodes, which permits intake replays to isolate stage 1, but leaves no operation or accessor control for TK-25/26 Duration use. The full FR-069-AC-6 intake and wire matrix is not claimed by this PR. Reverse source-to-spec ownership is FR-069 and QSpec FR-322; no copied upstream fixture is introduced. Optional semantic review was not requested.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 9816a2d8, `duration_tuple_position_projects_to_resolved_second_unit` uses the public reader and checks admitted `tuple.project` with source fallback and selected package `s`. |
| FND-002 | still-open | The new control omits missing `s`, invalid selected `s` and both-units-present cases. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | No public Duration use control distinguishes absent `s` from invalid `s` or checks selected unit precedence with both present | tests/conformance_qspec/model_tuple_union.rs:537 |

## Fix round coverage

The new control has two positives: a source `s` with no selected `s`, and a selected `s` without a source `s`. It does not test no `s`, invalid selected `s` with a valid source `s`, or both valid units with different identities. QSpec FR-322-AC-104 and AC-116 make these branches observable. This test gap is limited to the changed Duration resolution path; the full TC-280 field matrix remains outside the claimed partial AC-6 slice.
