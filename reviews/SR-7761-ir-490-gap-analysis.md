---
id: SR-7761
title: "Gap analysis of IR-490 structural forwarding evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir and follow-ups,; FR-038-AC-224; crates/quire-contract-model/src/checked_package/v2/operations.rs; tests/it/checked_package_v2_recursive_leaves.rs"
review_set: subset
---

## Summary

The changed tests carry the FR-038-AC-224 trace tag and the resolver's isolated boundaries are exercised. Two acceptance-criterion controls lack decisive evidence at the operation and reader boundaries.

## Verdict

**PASS after dispositions** — The initial evidence gaps and the follow-up test finding are fixed on the reviewed follow-up heads. The original findings and their outcomes remain recorded below.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Two-pass lawful case does not measure the second pass's structural Work delta | crates/quire-contract-model/src/checked_package/v2/operations.rs:5644 | correct-requirement-no-evidence |
| FND-002 | medium | Inner, result-inner, and leaf-edge controls do not exercise otherwise valid operations through the reader at exact and one-less limits | crates/quire-contract-model/src/checked_package/v2/operations.rs:6030 | correct-requirement-no-evidence |

## Coverage

Examined FR-038-AC-224 against all seven new tagged tests and the changed resolver callers. Plan completion: not assessed. The matrix computation and focused tests were used as trace and execution evidence; no optional repository-wide semantic audit was run.

FND-001: `structural_forwarding_is_paid_once_across_both_leaf_passes` checks that `resolved.len()` remains 300 after placement, but a second pass could charge those 300 entries again and leave that size unchanged. The admitted operation's total limit is measured from the same implementation, so its exact and one-less assertions also pass with that overcharge. Measure `meter.consumed()` before and after each leaf pass and separate the expected leaf-visit/pin charges from the structural delta.

FND-002: `inner_result_and_leaf_edges_propagate_structural_work` invokes `inner_type`, `LeafWalk::edge`, and `check_leaf_count` directly. Its `contains` and `flatten` application nodes have empty argument arrays, and the exact helper cases use a generous limit of 100 rather than a measured operation limit. A regression in `operation_defect` ordering or public-reader propagation could leave these assertions green. Use otherwise valid applications and assert the public reader's ordinary exact-limit result and typed `incomplete`/`work` at one less for each route.

## Dispositions

Follow-up source reviewed: (the later commit changes only the committed review text).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The test now asserts 903 count-pass units, including 300 structural units, and exactly 301 placement-pass units, with no repeated structural or pin charge. |
| FND-002 | fixed | The new unit and integration controls construct valid `contains`, `flatten`, and `structural.eq` operations and exercise both operation validation and public reading at measured exact/one-less total work limits. |
| FND-003 | fixed | Each route now has a valid direct control, the forwarded operation must consume exactly one more unit, and a shared limit must yield direct Work at `operation/leaves` versus forwarded Work at `operation`. This excludes the common base charge. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-003 | medium | Operation-path Work assertion can pass before structural resolution | tests/it/checked_package_v2_recursive_leaves.rs:802 | correct-requirement-no-evidence |

FND-003: The new public-reader `find_map` accepts any `Incomplete(Work)` whose path equals the operation. `operation_defect` unconditionally charges one unit at that same path before structural resolution (`operations.rs:411`); the whole-operation unit test uses the same path search. Therefore removing the inner/result/edge structural charge or converting its Work into another outcome still leaves an operation-path Work limit from that base charge. The exact/one-less total checks compute their limit from the modified implementation and also remain green. Assert a threshold attributable specifically to each structural forwarding call, with a no-forwarding control or an explicit measured pre-lookup meter delta, through the reader.
