---
id: SR-567876
title: IR-874 PR 568 code-review review
type: SpecReview
analysis: code-review
scope: 'agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/flat_wire.rs, protocol_body.rs, structural.rs; tests/it/checked_package_v2_protocol_body.rs, tests/it/support/checked_package.rs; IR-874 PR #568 '
review_set: subset
---

## Summary

Frozen PR #568 member-prefix review.

## Verdict

**CONDITIONAL** — 2 findings in the claimed role/channel/queue/obligation prefix. Control, compensation and reader stages 5–7 remain Planned and are not assessed as completed by this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Replicated role max validation follows binder target and type validation. With max 0 and a wrong-form or wrong-type binder, the reader reports the binder error instead of FR-465 role check 2 at max. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:1217 |
| FND-002 | medium | Queue channel validation checks only protocol/channel form. A declared channel from another clause reaches the end of the local queue check without the FR-465 same-clause invalid_model_binding/malformed-declaration refusal at the target. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:1291 |

## Scope

- FR-038-AC-282 (examined): Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- FR-038-AC-284 (examined): The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- FR-038-AC-285 (examined): Every role, channel, queue, obligation and compensation is listed by exactly one owning protocol clause, and every control is held once in its root control tree with the declaration path of its holder followed by its own name. A member listed twice or unattached, a control held twice or by nothing, a channel without exactly one queue, or a queue attached to another clause refuses `invalid_semantic_graph` at that member. A target not listed in its referencing node's `dependencies` refuses `missing_declaration`/`missing-name`; a target of the wrong tag or form refuses `invalid_model_binding`/`ma
- FR-038-AC-286 (examined): A role admits the FR-465 static and replicated shapes only, including a model of an eligible form, replication maximum at least one, a reference-typed binder and an `until` term exactly for an `until` lifetime; a channel admits only same-clause role endpoints, an eligible message type, explicit delivery and ordering, and a FIFO key with equality-stable type; a queue admits one channel, literal or symbolic capacity, explicit overflow and a loss selector exactly for `loss`. The reader admits the positive variants and refuses the wrong form, missing premise, wrong binder type, non-Boolean lifetim
- FR-038-AC-287 (examined): A clause names exactly one `finish` obligation whose closing-record binder and Boolean constraint read only the binders available after the root control; its optional `terminal` obligation admits `when` with one Boolean condition or `any` with none. A missing or extra condition refuses `malformed_wire`, a non-Boolean condition or constraint refuses `ill_typed`/`non-boolean-root`, an unjoined branch binder read by finish refuses `missing_declaration`/`missing-name`, and a terminal target in the finish position or an unattached finish refuses by the FR-464 membership rules.
- FR-038-AC-288 (context_only): Each of the thirteen control operators admits its FR-466 ordered arguments and local reference, type, bounds, label and ordering checks: `sequence`, `choice`, `parallel`, `repeat`, `await`, `check`, `send`, `receive`, `attempt`, `effect`, `event`, `commit` and `fence`. A choice or parallel branch list with fewer than two entries, repeated labels, a receive naming a non-send, an effect naming a non-attempt, an await naming an ineligible event, a non-Boolean guard or constraint, an invalid join count or an ineligible memory or ordering spelling refuses with the FR-466 code, cause and locus, with
- FR-038-AC-289 (context_only): A `compensate` body admits the thirteen FR-466 bindings in order, including distinct registration and activation captures, a trigger, interval, at-least-one attempt, retry relation, optional commit and recovery predicate. `commit never` is an absent optional Group, not a fabricated commit. The reader refuses a forward target that is not an `effect`, a commit target that is not a `commit`, an inverted interval or zero attempts, a commit that dominates its forward effect, and a read outside any of the five compensation stages with the corresponding FR-466 code, cause and locus; a commit in one c
- FR-464 (context_only): When a checked package carries a protocol declaration, the producer SHALL emit it as `quire.checked-semantic-graph/v2` nodes with `node_tag: "protocol"`: one `protocol_clause` node for the declaration and one member node for each of its roles, channels, queues, obligations, controls and compensations ([FR-465](FR-465-checked-package-protocol-declaration-bodies.md), [FR-466](FR-466-checked-package-protocol-control-bodies.md)). Every such body is one application of `quire.op.protocol.control`, whose `profile_operator` member names the construct. The nodes are the only carrier of the protocol: a
- FR-465 (context_only): When a checked package carries a protocol declaration's roles, channels, queue bounds, finish and terminal members, the producer SHALL encode each as one `protocol` node whose body is the [FR-464](FR-464-checked-package-protocol-node-bodies.md) application with the operator and argument layout below, and the reader SHALL refuse a node whose checks below do not hold, with the first refusal in the order each section lists. The node family's operation, keys, declarations, membership, binders and reader order are FR-464's; this requirement fixes the member set, order and checks of the declarations
- FR-466 (context_only): When a checked package carries a protocol declaration's control tree and compensation templates, the producer SHALL encode each control and each template as one `protocol` node whose body is the [FR-464](FR-464-checked-package-protocol-node-bodies.md) application with the operator and argument layout below, and the reader SHALL refuse a node whose checks below do not hold, with the first refusal in the order each section lists. This requirement fixes the member set, order, binder scope and checks of the thirteen control operators and of `compensate`.

## Coverage

Rust lane: rust-review and rust-style applied; reviewed the frozen PR diff and repository conventions. No assurance profile found for this slice. Focused coder evidence was 9 model and 21 integration tests passing; aggregate gates await final reviewed head. This review did not rerun aggregate gates.


## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Max is checked before binder target/type; the paired public-reader test asserts the max refusal. Verified. |
| FND-002 | fixed | The queue checks its listed clause against its channel and refuses a foreign channel at the target. Verified. |
