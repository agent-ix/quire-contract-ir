---
id: SR-15652
title: "Code and Rust review — IR-276 independent V2 record lowering PR 517"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/lower.rs, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs; FR-038-AC-309, FR-019-AC-5"
review_set: subset
---

## Summary

Reviewed PR #517, including the shared candidate builder, one-key package assembly, byte-ceiling failure path, and test controls. Ticket: IR-276. No code or Rust defect was found in the changed lines.

## Verdict

**PASS** — the new API preserves the per-key record semantics of a separate `lower` call while sharing its graph, source-map, and SCC closure indexes. This is a diff-scoped review; the author owns the final gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Rust review

The new public method is documented, returns an owned vector without exposing cache state, and uses borrowed indexes within the call. The candidate walker remains iterative and preserves the existing per-request work meter. Each candidate is assembled alone under the retained ceiling; failed assembly produces the same `Failed { limit_kind: Bytes, limit, consumed }` shape as the one-key `lower` path. The diff adds no unsafe, request-path panic, unchecked boundary cast, lock, async work, or CI gate change. `#[cfg(test)]` counters observe constructor calls without changing production behavior. No copied upstream code or new file digest was introduced.

## Scope and evidence

- Examined FR-038-AC-309: “`CheckedPackageV2::lower_independent_records` returns one record per requested key in input order, each byte-for-byte equal to the record from `lower` called with only that key and the same profile and retained byte ceiling. For a shared R-node closure under K requests, graph/source indexes and component closures are built once and body validation is not repeated. When every one-key package fits but their union exceeds the byte ceiling, this method retains each one-key record while `lower` on the union reports its existing whole-call `failed` result.”
- Examined FR-019-AC-5 as context: the default-feature public item inventory must match its implemented table and reject added or missing items.
- Examined both changed Rust files and their direct `assemble` and `ContractPackageWireV1::from_lowering` callees. The latter is existing code needed to verify one-key byte behavior. No test or aggregate gate was run by this reviewer, as the review brief reserved final gates for the author.
