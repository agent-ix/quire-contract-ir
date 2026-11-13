---
id: SR-202611081512300015
title: "IR-1536 gap analysis: remove SccClosureIndex complexity"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@7fddc1a269424c77b24f58cf9b23e69d91b83b3a; crates/quire-contract-model/src/checked_package/v2/lower.rs, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs, tests/read_storage_alloc.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-050
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-472
    type: references
---

## Summary

Ticket: IR-1536, PR quire-contract-ir#1021, head 7fddc1a. Planless, diff-scoped gap analysis. Plan completion: not assessed. The diff changes lowering's closure mechanism and the tests traced to FR-038-AC-308, AC-309, AC-511 and AC-512. No file under `spec/` is changed. Each affected criterion was read against the changed code and tests.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-308 | examined | "lowering performs no further body grammar validation, reuses closure reachability per strongly connected component, and does not scan the full closure separately to decide each candidate bound." |
| FR-038-AC-309 | examined | "For a shared R-node closure under K requests, graph/source indexes and component closures are built once and body validation is not repeated." |
| FR-038-AC-511 | examined | "Whole-read accounting ... the adverse shapes ... For every read, non-refusal-diagnostic bytes requested during the call exceed the read-storage delta the budget admitted by at most 4096 bytes" |
| FR-038-AC-512 | examined | "Over the TC-048 corpus, every budget whose limits are at least the read's measured requirement returns the same outcome as the unlimited budget: an equal admitted package (... lowering of every node)" |
| spec/checked_package/matrix/tests.md rows 52-53 | examined | "shared-SCC closure reuse" / "share one SCC closure build" |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-308 requires that lowering "reuses closure reachability per strongly connected component", and AC-309 requires that "component closures are built once". The PR deletes the SCC closure index, and the `SHARED_CLOSURE_BUILDS == 1` and `*_INDEX_BUILDS == 1` assertions that proved those clauses, without amending either criterion. The matrix rows still claim "shared-SCC closure reuse" and "share one SCC closure build". The code is now correct and the spec describes a mechanism that no longer exists, so both ACs are false as written. Amend AC-308 and AC-309 in this PR to state the new design: each request's closure comes from its own walk over the retained adjacency, at a cost bounded by what it reaches; no per-call whole-graph index; zero additional body validations. Bind `one_request_compares_ids_by_its_closure_not_by_graph_size` to the amended text, and update matrix rows 52-53. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6345-6346, spec/checked_package/matrix/tests.md:52-53, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs:572 |

## Verdict

REQUEST CHANGES: one high spec-versus-code contradiction (FND-001), fixable by a spec edit in this PR. AC-511 and AC-512 are clean. AC-511's adverse shapes still run under unlimited, exact, repeated and one-below budgets with the unpaid-byte bound unchanged. AC-512's lowering-of-every-node equality applies to the TC-048 corpus, whose callers keep `ReadViewScope::Full` (whole_read_accounting.rs and qspec_read_proof.rs). The `Core` narrowing for adverse shapes therefore drops no criterion's oracle.
