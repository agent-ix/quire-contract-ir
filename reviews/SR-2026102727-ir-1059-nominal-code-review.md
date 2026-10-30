---
id: SR-2026102727
title: "Code review of IR-1059 nominal rekey stage order"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #669 d937faad; crates/quire-contract-model/src/checked_package/v2/{contract_package,derived_keys,identity,mod,structural}.rs; tests/conformance_qspec/{model_tuple_union,nominal_rekey}.rs; tests/it/{checked_package_v2_owners,checked_package_v2_union}.rs"
review_set: subset
---

## Summary

Ticket IR-1059, PR #669 at d937faad, reviewed against QSpec FR-322-AC-74/169/170 and IR FR-038-AC-155/227/343. The NK-00..14 replay is complete, but the implementation still assigns two derived-shape failures to the wrong stage or omits them.

## Verdict

**FAIL** — the stage-6 derived-shape check is incomplete and an earlier scan preempts one required refusal. No full build gate was run in this diff-scoped review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A reordered compound-unit term refuses during graph shape as `invalid_semantic_graph` at the body member instead of stage 6 `invalid_package`/`invalid-derived-shape` at `node_id`. The early `compound_unit_defect` order check remains active, and the new stage-6 function only detects zero exponents. QSpec RM-CU-ORDER is the discriminating control. | crates/quire-contract-model/src/checked_package/v2/structural.rs:698 |
| FND-002 | high | A model-population node with a current key and malformed `max` has no stage-6 validation: `ModelPopulation` is excluded from `StructuralForm`, and the new stage-6 branch checks only compound units and references. QSpec FR-322-AC-74 requires `invalid_package`/`invalid-derived-shape` at its node. | crates/quire-contract-model/src/checked_package/v2/derived_keys.rs:702 |

## Coverage

Examined the full PR diff, QSpec FR-322-AC-74/169/170, NK-00..14 and TW-05..09 stage expectations, IR FR-038-AC-155/227/343, trace tags and public-reader assertions. No vendored artifact, widened gate, unsafe code, or new unbounded loop was found. The tests do not exercise QSpec RM-CU-ORDER or population malformed-max as a simultaneous stage-order case.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Head e80f3453 moves compound-unit order and uniqueness to `compound_unit_has_invalid_shape` after structural keys; the early graph check now only compares the sorted unique dependency set. The current-key and retained-key public-reader controls distinguish the two refusals. Focused tests are still queued. |
| FND-002 | fixed | Head e80f3453 adds `model_population_body_valid` and `model_population_type_valid` to stage 6 after keys; restored public-reader controls check malformed max and arbitrary Set element. Focused tests are still queued. |
