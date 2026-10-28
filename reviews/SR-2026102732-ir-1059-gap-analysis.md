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
