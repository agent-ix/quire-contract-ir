---
id: SR-567895
title: "IR-874 PR 580 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, crates/quire-contract-model/src/checked_package/v2/structural.rs, tests/it/checked_package_v2_protocol_body.rs; PR #580 frozen "
review_set: subset
---

## Summary

Reviewed only PR #580's clause reference, activation, and capture changes against QSpec FR-464 and IR FR-038. Rust review and code review were one method.

## Verdict

**CONDITIONAL** — two captures with distinct levels and the same readable name pass the new clause check, contrary to the ambiguous-name rule.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The capture loop checks each binder's level but never compares its parameter name with `over`, the `each` trigger, or earlier captures. Two distinct parameter nodes named `x` at successive levels can therefore be admitted as one clause's captures instead of refusing `ambiguous_declaration`/`ambiguous-name`. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:3171 |

## Scope

- **FR-464-AC-5 (examined, QSpec)**: The clause body is thirteen bindings in the order the "Clause body" table lists; `activation` is `origin` with no trigger or `each` with a trigger and an optional guard; a protocol with no relationship, requirement, compensation, `terminal` member or `scheduling` has empty sets and absent optionals and is admitted; a body with arguments reordered, missing or of another term shape refuses `malformed_wire`.
- **FR-464-AC-6 (examined, QSpec)**: A listed role, channel, queue, compensation or obligation node that is not of the stated tag and form refuses `invalid_model_binding`/`malformed-declaration` at the target, and a `relationships` target that is not a `relation`/`relationship` node `invalid_model_binding`/`wrong-export`; a target absent from `dependencies` refuses `missing_declaration`/`missing-name`; a declaration node named by two arguments of one clause or by two clauses, a node owned by no clause, a channel with no queue or with two, and a queue naming another clause's channel refuse `invalid_semantic_graph`.
- **FR-464-AC-7 (examined, QSpec)**: Binder levels run from the introducing construct's entry count in layout order: `over` `0`, the `each` trigger `1`, the captures `2` and up in order, a control's binder `size(In(n))`; a wrong level, a binder body of another shape and two readable binders of one name refuse (`invalid_semantic_graph`, `malformed_wire`, `ambiguous_declaration`/`ambiguous-name`).
- **FR-464-AC-12 (examined, QSpec)**: The protocol step runs after the state, temporal, abstraction and refinement steps and before the operation step and reports, in order, placement, operator, declaration, member, membership, clause and refinement class-catalog graph-join defects; within a clause the cycle and progress defects precede the slot checks, and within a slot the `over`/model-state refusal precedes the binder-scope refusal, which precedes the decision-fact refusal; a state-step defect and a protocol defect in one package report the state-step defect.
- **FR-038-AC-284 (examined)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- **FR-038-AC-285 (examined)**: Every role, channel, queue, obligation and compensation is listed by exactly one owning protocol clause, and every control is held once in its root control tree with the declaration path of its holder followed by its own name. A member listed twice or unattached, a control held twice or by nothing, a channel without exactly one queue, or a queue attached to another clause refuses `invalid_semantic_graph` at that member. A target not listed in its referencing node's `dependencies` refuses `missing_declaration`/`missing-name`; a target of the wrong tag or form refuses `invalid_model_binding`/`ma
- **FR-038-AC-291 (examined)**: Binder levels start at the introducing construct's entry count; protocol `over` is level 0, an `each` trigger follows, captures advance in their authored order, and a control binder is at the size of its readable input set. The reader admits reads within the FR-464/466 stage tables and refuses a wrong level, duplicate readable name, forward or foreign binder, or read after a branch, choice, repeat or await path where the binder is unavailable with the specified code, cause, stage and owner. In a binders-only slot, a read of `over` or model state refuses `ill_typed`/`operator-ineligible` before
- **FR-038-AC-324 (context_only)**: For an admitted protocol clause the reader derives one activation-relation row per distinct required temporal clause from the clause's `activation` and `requirements` alone, in ascending required-clause node-id digest order. The row contains the protocol clause key, `origin` or `each` with the trigger key, and the required temporal clause key. No requirement gives no row; two source `requires temporal` declarations encoded as one canonical set entry give one row; two duplicate wire entries refuse `invalid_semantic_graph` under the clause's set rule and expose no relation; a second distinct req

## Coverage

The three changed files were inspected; no CI, dependency, spec, or plan file changed. `quoin matrix --repo . --json` completed; focused public-reader tests cover references, trigger dependency, guard root and capture levels. The new clause code has an owning requirement and no stub. No full gate was run; the lead runs final exact-head gates. Full FR-038 protocol criteria remain Planned.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, `check_clause` introduces `over`, the optional `each` trigger, then each capture into one readable-name set and refuses the second occurrence as `AmbiguousDeclaration`/`AmbiguousName` at the second parameter node. The new public-reader test uses two distinct parameter nodes named `captured` at levels 1 and 2 and asserts the exact refusal. |

## Disposition verdict

**PASS ** — the recorded code finding is fixed. This disposition covers only the frozen #580 review scope.
