---
id: SR-062
title: "FR-025 native predicate projection gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "quire-contract-ir#70; FR-025; TC-038; TM-002; QCI allocation of tl-syntax PLAN-010 Task-006"
review_set: subset
relationships:
  - target: ix://agent-ix/tl-syntax/Task-006
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TM-002
    type: references
---

## Summary

The targeted QCI allocation of PLAN-010 Task-006 is implemented at candidate
`db6bbf551d5e321c8051011d63221e03e9bfaece`: checked native predicates project
bijectively into strict-read TL Boolean artifacts, and runtime values come only
from exact owner availability and mapped-result views. Projection and valuation
fail closed without partial output, alternate parsing/evaluation or Boolean
coercion.

## Verdict

**PASS** — every FR-025 criterion and TC-038 matrix row is backed by executing
Rust, and no scoped task, matrix, reverse-trace, stub or code/test alignment gap
remains. PLAN-010 Tasks 007 and 011 remain separate planned campaign work and
are not counted as complete here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No scoped implementation, matrix, traceability, stub, reverse-trace or semantic-alignment gap remains after the remediations recorded in SR-061. | FR-025; TC-038; #70 |

## Coverage

- Reconciliation: `quire coverage` 0.32.0, engine
  `a874fb641cb70da83c8c8b23f9fea0a44255b88a`.
- Scoped implementation allocation: 1/1 complete for PLAN-010 Task-006.
- FR-025 acceptance criteria backed: 8/8.
- TC-038 source symbols: 16 executing Rust tests across artifact join,
  projection and valuation.
- Repository-wide rows backed: 121/152. All 16 reported functional/criterion
  gaps belong to explicitly planned FR-026/TC-039 and FR-027/TC-040; FR-025 has
  no unbacked row, status lie or untracked symbol.
- Rust source census: 67/67 candidate test symbols tagged, bound and
  self-named; TC-038 uses bare trace attributes.
- Changed production behaviors inventoried: owner contract admission, checked
  predicate identity, deterministic TL artifacts, strict projection reading,
  owner-only valuation, completeness gaps, correction identity, stable causes
  and resource fail-closed behavior. Untraced scoped behavior: 0. Source/test
  stubs: 0.
- Semantic alignment was evaluated as part of SR-061: tests use real owner
  views and assert the observable FR-025 oracle rather than restating local
  implementation helpers.

## Reverse trace

`predicate::project` owns FR-025-AC-1/2/6/8; the constructor-private definition
and exact target selections bind owner identity without copying owner wire
types. `predicate::value` owns FR-025-AC-2 through AC-7 and derives Boolean or
typed non-value decisions only from observation/protocol owner views.
`read_projection` and `read_valuation` re-derive expected decisions and compare
canonical bytes, covering strict-reading and replay/cross-wire refusal. TC-038
tests exercise each path and the matrix now reports FR-025 implemented.
