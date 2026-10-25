---
id: SR-565877
title: IR-874 PR 566 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: 'agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/flat_wire.rs,
  crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs,
  tests/it/checked_package_v2_protocol_body.rs, tests/it/support/checked_package.rs;
  IR-874 PR #566 head 30ac95e4'
review_set: subset
---

## Summary

New trace tags claim criteria beyond the first protocol validation slice, hiding the remaining acceptance gaps.

## Verdict

**FAIL** — New trace tags claim criteria beyond the first protocol validation slice, hiding the remaining acceptance gaps.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-284 trace marks full protocol ordering backed using only two strict-wire tests | crates/quire-contract-model/src/checked_package/v2/flat_wire.rs:690 |
| FND-002 | high | AC-286/287 trace marks full role/channel/queue/finish admission backed using a local shape-only test | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:1321 |

## Scope

- FR-038-AC-281 (examined): A reader-admitted protocol member has a `quire.op.protocol.control` body root, exactly one `protocol_profile` law matching the selected lock row, `null` mode, no leaves, Boolean result and semantic types, and a `profile_operator` member. Under the published `quire.protocol.finite-global/v1` selection the admitted operators are exactly `protocol`, `role`, `channel`, `queue`, `finish`, `terminal`, `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit`, `fence` and `compensate`; an unknown operator refuses `operation-member-mismatch
- FR-038-AC-282 (examined): Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- FR-038-AC-284 (examined): The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- FR-038-AC-286 (examined): A role admits the FR-465 static and replicated shapes only, including a model of an eligible form, replication maximum at least one, a reference-typed binder and an `until` term exactly for an `until` lifetime; a channel admits only same-clause role endpoints, an eligible message type, explicit delivery and ordering, and a FIFO key with equality-stable type; a queue admits one channel, literal or symbolic capacity, explicit overflow and a loss selector exactly for `loss`. The reader admits the positive variants and refuses the wrong form, missing premise, wrong binder type, non-Boolean lifetim
- FR-038-AC-287 (examined): A clause names exactly one `finish` obligation whose closing-record binder and Boolean constraint read only the binders available after the root control; its optional `terminal` obligation admits `when` with one Boolean condition or `any` with none. A missing or extra condition refuses `malformed_wire`, a non-Boolean condition or constraint refuses `ill_typed`/`non-boolean-root`, an unjoined branch binder read by finish refuses `missing_declaration`/`missing-name`, and a terminal target in the finish position or an unattached finish refuses by the FR-464 membership rules.

## Coverage

Plan completion: not assessed. Full matrix was unavailable during this review; a scoped source and trace audit identified the two false bindings. Semantic review not requested.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at PR #566 head 156e59c0; see new term-pointer logic or narrowed trace tag. |
| FND-002 | fixed | Verified at PR #566 head 156e59c0; see new term-pointer logic or narrowed trace tag. |
