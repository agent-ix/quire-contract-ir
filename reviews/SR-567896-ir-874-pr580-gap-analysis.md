---
id: SR-567896
title: "IR-874 PR 580 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, crates/quire-contract-model/src/checked_package/v2/structural.rs, tests/it/checked_package_v2_protocol_body.rs; PR #580 frozen "
review_set: subset
---

## Summary

The computed matrix and changed public-reader tests were checked against the clause-level obligations. The new tests trace activation-relation AC-324 without asserting a relation, and the binder-level and duplicate-name criterion has no bound test.

## Verdict

**CONDITIONAL** — the changed test evidence does not establish the level/name or activation-relation obligations it invokes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `FR-038-AC-291` is untagged in the computed matrix although this PR adds capture/trigger level checks. The wrong-level tests trace only AC-284 and assert an `InvalidSemanticGraph` outcome, leaving the explicit binder level and duplicate readable-name criterion without bound evidence. | tests/it/checked_package_v2_protocol_body.rs:157 |
| FND-002 | medium | Three new tests trace `FR-038-AC-324`, but they only assert admission, duplicate-set or missing-dependency outcomes; none reads or compares activation-relation rows. The tags imply evidence for AC-324's distinguishing row content and ordering that these assertions cannot supply. | tests/it/checked_package_v2_protocol_body.rs:154 |

## Scope

- **FR-038-AC-284 (examined)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- **FR-038-AC-285 (examined)**: Every role, channel, queue, obligation and compensation is listed by exactly one owning protocol clause, and every control is held once in its root control tree with the declaration path of its holder followed by its own name. A member listed twice or unattached, a control held twice or by nothing, a channel without exactly one queue, or a queue attached to another clause refuses `invalid_semantic_graph` at that member. A target not listed in its referencing node's `dependencies` refuses `missing_declaration`/`missing-name`; a target of the wrong tag or form refuses `invalid_model_binding`/`ma
- **FR-038-AC-291 (examined)**: Binder levels start at the introducing construct's entry count; protocol `over` is level 0, an `each` trigger follows, captures advance in their authored order, and a control binder is at the size of its readable input set. The reader admits reads within the FR-464/466 stage tables and refuses a wrong level, duplicate readable name, forward or foreign binder, or read after a branch, choice, repeat or await path where the binder is unavailable with the specified code, cause, stage and owner. In a binders-only slot, a read of `over` or model state refuses `ill_typed`/`operator-ineligible` before
- **FR-038-AC-324 (examined)**: For an admitted protocol clause the reader derives one activation-relation row per distinct required temporal clause from the clause's `activation` and `requirements` alone, in ascending required-clause node-id digest order. The row contains the protocol clause key, `origin` or `each` with the trigger key, and the required temporal clause key. No requirement gives no row; two source `requires temporal` declarations encoded as one canonical set entry give one row; two duplicate wire entries refuse `invalid_semantic_graph` under the clause's set rule and expose no relation; a second distinct req
- **FR-464-AC-7 (examined, QSpec)**: Binder levels run from the introducing construct's entry count in layout order: `over` `0`, the `each` trigger `1`, the captures `2` and up in order, a control's binder `size(In(n))`; a wrong level, a binder body of another shape and two readable binders of one name refuse (`invalid_semantic_graph`, `malformed_wire`, `ambiguous_declaration`/`ambiguous-name`).
- **FR-464-AC-13 (context_only, QSpec)**: The activation relation of an admitted protocol is derived from its clause node's `activation` and `requirements` alone: `activation on each (opened)` with `requires temporal OrderClosureBounded` gives exactly one row, the clause key, `each` with `opened`'s node key and `OrderClosureBounded`'s node key; under `activation on origin` the one row carries `origin` and no trigger; a protocol with no requirement has no row; a second `requires temporal OrderClosureBounded` leaves one row; a second required temporal clause gives two rows in ascending node-id digest order; a temporal clause that shares

## Coverage

Plan completion: not assessed. `quoin matrix --repo . --json` completed: FR-038-AC-284 has 15 bound tests, AC-285 has 6, AC-291 has 0 (`untagged`), and AC-324 has 3. All three AC-324 tests are new in this diff; none asserts relation rows. Changed source has an owning requirement and no stub. Optional repository-wide semantic review was not requested; this scoped review compared the changed assertions to their trace tags. The remaining FR-038 protocol criteria stay Planned.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, the trigger and ordered-capture level tests now trace `FR-038-AC-291`, and a new public-reader test traced to AC-291 asserts `AmbiguousDeclaration`/`AmbiguousName` for duplicate readable capture names. The full AC remains Planned for later binder-scope work. |
| FND-002 | fixed | In the reviewed PR, the three changed tests no longer trace `FR-038-AC-324`; they assert only their actual clause and binder behavior. Activation-relation row content and ordering remain unimplemented and unverified under Planned AC-324, so these tests no longer claim that evidence. |

## Disposition verdict

**PASS** — both recorded test-evidence defects are fixed; AC-324's relation work remains Planned.
