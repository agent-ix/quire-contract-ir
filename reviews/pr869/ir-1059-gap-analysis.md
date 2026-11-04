---
id: SR-2026110301258
title: "Gap analysis of bounded Rational AC8 evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #869; FR-069-AC-8; tests/conformance_qspec/model_tuple_union.rs; crates/quire-contract-model/src/checked_package/v2/model_members.rs"
review_set: subset
---

## Summary

Reviewed the FR-069-AC-8 binding and the changed test path for published TK-36, TK-22, and TK-24. Dynamic replay covers admission and refusal, but no production-path assertion proves the admitted TK-36 tuple and payload retain their four exact bounds in the public package view and structural body.

## Reviewed Units

- FR-069-AC-8, role: examined. Excerpt: “The published `TK-36` case admits through both production paths with its tuple position and variant payload resolving to `Rational[-3, 7; 1, 12]` and the selected `bounded_domain`/`rational_range` shape.”
- QSpec FR-322-AC-380, role: context_only. Excerpt: “`Ratio` as a tuple position `typeRef` and as a variant case `payloadType` is admitted (`TK-36`).”

## Verdict

**CONDITIONAL** — Add a public-reader assertion for TK-36's tuple and variant member types, exact four bounds, and selected rational-range node body.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | TK-36 replay asserts admission only; no public-reader test verifies exact bound retention or selected rational-range body at both model members. | tests/conformance_qspec/model_tuple_union.rs:676; crates/quire-contract-model/src/checked_package/v2/model_members.rs:4731 | correct-requirement-no-evidence |

## Coverage

Plan completion: not assessed. The changed conformance test is tagged FR-069-AC-8 and replays all published rows, including TK-22 and TK-24 refusals. The TK-36 positive branch uses `matches!(result, CheckedPackageV2ReadResult::Admitted(_))`; the internal test checks parsing and ID derivation but never observes the public reader's tuple and variant views or the admitted node body. The lead owns the final aggregate gate.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The updated TK-36 branch reads the admitted package through `model_tuple_position_type` and `model_variant_payload_member_type`, compares both to the four exact Rational bounds, and checks the admitted `rational_range` node kind and ordered integer-literal body. |
