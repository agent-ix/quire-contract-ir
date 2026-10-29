---
id: SR-567885
title: "IR-874 PR 572 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs; PR #572 "
review_set: subset
---

## Summary

The new local control checks follow the typed reference and refusal patterns, but the owning-clause lookup adds unmetered whole-graph work for each qualifying control.

## Verdict

**CONDITIONAL — One bounded-work defect requires a fix before merge.**

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | same_clause scans every graph node for each attempt, event or commit without a WorkMeter charge; a package with many step controls can do quadratic unmetered work under a small validation-work limit. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:1353 |

## Scope

- **FR-038-AC-3 (examined)**: Exact byte, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package; no limit kind is a depth (FR-038-AC-117).
- **FR-038-AC-282 (examined)**: Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- **FR-038-AC-284 (examined)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- **FR-038-AC-288 (examined)**: Each of the thirteen control operators admits its FR-466 ordered arguments and local reference, type, bounds, label and ordering checks: `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit` and `fence`. A choice or parallel branch list with fewer than two entries, repeated labels, a receive naming a non-send, an effect naming a non-attempt, an await naming an ineligible event, a non-Boolean guard or constraint, an invalid join count or an ineligible memory or ordering spelling refuses with the FR-466 code, cause and locus, with
- **FR-466-AC-1 (examined)**: Each of the thirteen control operators is admitted with exactly its layout; the arguments reordered, a missing or surplus binding, an unknown operator spelling and an enumerated value outside its set (`memory`, `outstanding`, `policy`, `ordering`, `kind`) refuse `malformed_wire` or refuse `operation-member-mismatch`.
- **FR-466-AC-8 (examined)**: An `attempt` is admitted with a role, an operation anchor, each memory ordering or none, and `contracts` that are the pre- and postconditions of that anchor; a `contracts` target anchored at another operation, an `invariant` clause and a non-clause node refuse `invalid_model_binding`/`malformed-declaration`; an empty `contracts` list is admitted. An `effect` is admitted of an `attempt` and refuses `invalid_model_binding`/`malformed-declaration` of any other control.
- **FR-466-AC-9 (examined)**: An `event` is admitted with and without a compensation; a `commit` and a `fence` are admitted; `related` entries name a relationship the clause lists with end terms of the endpoint types, and an unlisted relationship refuses `missing_declaration`/`missing-name` and an ill-typed end `ill_typed`/`type-mismatch`.

## Coverage

The change touches Rust model code and public-reader tests only; no CI workflow, dependency, spec or plan file changed. Reviewed malformed-input paths, local refusal order, nested-clause lookup and changed tests. Focused test run is recorded by the code lead; no aggregate gate was run in review.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The clause scan now charges one work unit per node before inspecting it; exhaustion returns through `Result` at the control node. |

## Disposition verdict

**PASS ** — All original findings were fixed on this PR; the complete FR-038-AC-282/284/288 criteria remain Planned.
