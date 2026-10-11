---
id: SR-7761
title: "Gap analysis of IR-490 structural forwarding evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@e29d89390ca017d9ee6652464f8bf34cdd03b1f1; FR-038-AC-224; crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---

## Summary

The changed tests carry the FR-038-AC-224 trace tag and the resolver's isolated boundaries are exercised. Two acceptance-criterion controls lack decisive evidence at the operation and reader boundaries.

## Verdict

**CONDITIONAL** — The implementation appears sound, but both medium evidence gaps should be closed before calling FR-038-AC-224 verified.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Two-pass lawful case does not measure the second pass's structural Work delta | crates/quire-contract-model/src/checked_package/v2/operations.rs:5644 | correct-requirement-no-evidence |
| FND-002 | medium | Inner, result-inner, and leaf-edge controls do not exercise otherwise valid operations through the reader at exact and one-less limits | crates/quire-contract-model/src/checked_package/v2/operations.rs:6030 | correct-requirement-no-evidence |

## Coverage

Examined FR-038-AC-224 against all seven new tagged tests and the changed resolver callers. Plan completion: not assessed. The matrix computation and focused tests were used as trace and execution evidence; no optional repository-wide semantic audit was run.

FND-001: `structural_forwarding_is_paid_once_across_both_leaf_passes` checks that `resolved.len()` remains 300 after placement, but a second pass could charge those 300 entries again and leave that size unchanged. The admitted operation's total limit is measured from the same implementation, so its exact and one-less assertions also pass with that overcharge. Measure `meter.consumed()` before and after each leaf pass and separate the expected leaf-visit/pin charges from the structural delta.

FND-002: `inner_result_and_leaf_edges_propagate_structural_work` invokes `inner_type`, `LeafWalk::edge`, and `check_leaf_count` directly. Its `contains` and `flatten` application nodes have empty argument arrays, and the exact helper cases use a generous limit of 100 rather than a measured operation limit. A regression in `operation_defect` ordering or public-reader propagation could leave these assertions green. Use otherwise valid applications and assert the public reader's ordinary exact-limit result and typed `incomplete`/`work` at one less for each route.
