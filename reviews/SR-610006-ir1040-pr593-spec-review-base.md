---
id: SR-610006
title: "IR-1040 PR 593 spec-review/base review"
type: SpecReview
analysis: base
scope: 'agent-ix/quire-contract-ir; PR #593; spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md, spec/checked_package/matrix/tests.md; governing QSpec agent-ix/quire-specification origin/main FR-322-AC-131, FR-341-AC-14, FR-342-AC-10'
review_set: subset
---

## Summary

Ticket: IR-1040. Base checklist review of FR-040-AC-31..AC-35, the two changed FR-040 prose sections, the TC-056 procedure and the matrix rows, against QSpec origin/main FR-322-AC-131, FR-341-AC-14 and FR-342-AC-10 and the current V2 reader. IDs, numbering and trace rows are consistent. No existing AC row was edited, and Planned status is honest. The new criteria contradict FR-058, broaden clause anchors beyond scope, and leave the name oracle and absent-point agreement undefined.

## Verdict

**CHANGES REQUESTED**: one high and three medium findings. Strengths: AC-31..35 follow QSpec's closed four-variant shape, the pre/post applicability for operation clauses, the (context, operation, point) tuple, and the identity contribution of both carriers. The PR adds nothing new to hash, pin or digest. TC-056 and the matrix mark AC-31..35 Planned and claim no implemented coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-040-AC-33/AC-34 admit pre{operation} or post{operation} on either a precondition or a postcondition and refuse with invalid_model_binding/malformed-declaration, while the unchanged FR-058 (Planned, IR-1013) 'Admission and retention' and FR-058-AC-8 require a precondition to admit Pre only, a postcondition Post only, and refusal as InvalidExecutionPointCarrier/IncompatiblePoint; FR-058 also still says the wire member is undefined pending QSL-1082. The PR leaves two contradictory SHALLs for the same carrier; FR-058's 'QSpec code prevails' clause covers the code but not the stricter applicability rule. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:368 |
| FND-002 | medium | Check 1 now lets an invariant or constraint anchor at model/process and model/state_machine nodes, a prose-only broadening outside the execution-point scope. No new AC states it, AC-8 is unchanged, the matrix marks nothing Planned for it, and the current reader (state.rs:346-350) admits only model/object_type for an invariant anchor. The Operation anchor body section still limits context to model/object_type. The prose now claims behaviour that is neither implemented nor tracked as Planned. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:260-265 |
| FND-003 | medium | 'name different from the checked source anchor' (and check 4's 'The name equals the checked source anchor name') names no package datum the reader compares an initialization{name} or handler{name} against: the anchor of an invariant or constraint is a model declaration node, and initialization or handler names are not package data. AC-33's 'absent only when the checked clause has none' is a producer fact the reader cannot observe. Two implementers would choose different oracles, or none. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:369 |
| FND-004 | medium | Clause/anchor agreement is undefined when one side has no point. Check 4 runs only 'When present', so an operation clause with no point that references a pre-pointed anchor appears to admit, whereas the anchor paragraph says a clause-bound anchor's point equals every clause that references it. AC-34 'point different from the referenced operation anchor's point' does not say whether absent-versus-present counts as different, or which node is the locus when the clause has no point. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:284-291 |

## Scope

- **FR-040-AC-31 (examined):** An anchor admits its three required bindings with no point, and admits a fourth `execution_point` binding after `frame` with `pre{operation: "scaled"}` or `post{operation: "scaled"}`; these three forms have different rederived node identities and package ids. Reordering the fourth binding, adding another binding, or changing the point without rekeying refuses before the state step. This supersedes only AC-7's blanket extra-binding refusal: an extra binding other than the specified fourth still refuses `invalid_semantic_graph` with no cause at the anchor `body`.
- **FR-040-AC-32 (examined):** An anchor with an unknown point variant, missing or extra point member, empty name, `initialization` or `handler` point, or point whose operation differs from the anchor's checked operation refuses `invalid_model_binding`/`malformed-declaration` at that anchor in the state step. A duplicate `(context, operation, point)` tuple refuses `ambiguous_declaration`/`ambiguous-name` at the second anchor by ascending node-id digest; the same context and operation with distinct accepted points or one absent point admit. This refines AC-7's pair uniqueness without changing its no-point duplicate outcome.
- **FR-040-AC-33 (examined):** A state clause admits an optional `execution_point` member with each closed FR-322 variant: `initialization{name}` and `handler{name}` on an invariant or constraint, `pre{operation}` and `post{operation}` on a precondition or postcondition. The point is absent only when the checked clause has none. Equal application bodies except for absent versus different accepted points have different rederived node identities and package ids; an accepted point round-trips unchanged. A `constraint` is checked like an invariant. This extends AC-8's three listed admitted clause kinds.
- **FR-040-AC-34 (examined):** An unknown clause point variant, missing or extra point member, empty name, point inapplicable to its clause kind, name different from the checked source anchor or resolved operation, or point different from the referenced operation anchor's point refuses `invalid_model_binding`/`malformed-declaration` at the clause node in the state step. A stale node key after only the point changes refuses `invalid_package`/`stale-node-key` before the state step.
- **FR-040-AC-35 (examined):** With simultaneous defects, graph shape and stale-key refusals precede the frame step; the frame step precedes state; within state, placement precedes ascending-id anchors and anchors precede ascending-id clauses. For one anchor, context, frame, types, operation and scoped deletes retain their order before its point agreement; for one clause, anchor, parameters and signature precede its point check, and operation catalog checking follows the state step. The first applicable defect's exact code, cause and RFC 6901 locus are returned with no package.
- **FR-040 Operation anchor body (execution point paragraph) (examined):** An anchor's optional fourth binding has exactly the closed FR-322 `ExecutionPoint` shape. It is `pre{operation}` or `post{operation}`, with `operation` equal to the resolved `operation` binding name; a clause-bound anchor's point equals every clause that references it, while a protocol-only anchor may omit the point. An unknown variant, missing or extra member, empty or disagreeing operation name, or `initialization`/`handler` point refuses `invalid_model_binding`/`malformed-declaration` at the anchor node in the state step. The fourth binding enters the structural node preimage; absence an...
- **FR-040 State clause body (member, check 1 and check 4) (examined):** 1. **Anchor.** The anchor target is a declared dependency, else `missing_declaration`/`missing-name`; for an invariant or constraint it is a `model`/`object_type`, `model`/`process` or `model`/`state_machine` node and for a precondition or postcondition a `state`/`operation_anchor` node, else `invalid_model_binding`/ `malformed-declaration`, located at the anchor's target. | 4. **Execution point.** When present, the point has exactly one of FR-322's closed shapes: `initialization{name}` or `handler{name}` for an invariant or constraint, and `pre{operation}` or `post{operation}` for a precondition or postcondition. The name equals the checked source anchor name; for an operation clause it equals the resolved operation name and its point equals its `operation_anchor` point. An unknown variant, missing or extra member, empty or disagreeing name, inapplicable kind or mismatched anchor point refuses `invalid_model_binding`/`malformed-declaration` at the clause node.
- **TC-056 (examined):** Build both an unpointed protocol anchor and point-qualified pre and post anchors for the same checked operation. ... Cover FR-040-AC-31 through FR-040-AC-35. This is planned until tagged reader tests and the full reviewed-code gate pass.
- **tests.md FR-040 and TC-056 rows (examined):** AC-31 through AC-35 are planned under IR-1040 for the execution-point carrier, closed variants, identity, agreement and first-refusal order; AC-31 supersedes only AC-7's blanket fourth-binding refusal.
- **FR-040-AC-7 (examined):** An operation anchor with the three bindings in order admits. Reordered bindings, a non-text `operation`, a missing binding, and an extra binding each refuse as `invalid_semantic_graph` with no cause, located at the anchor's `body`; an occurrence role other than `anchor` refuses the same way, located at that occurrence's `role`. A `context` naming a frame, a `frame` naming an object type, and a `frame` target missing from `dependencies` refuse as `invalid_model_binding`/`malformed-declaration`, `invalid_model_binding`/`malformed-declaration` and `missing_declaration`/`missing-name` at the ta...
- **FR-040-AC-8 (examined):** An invariant, a precondition and a postcondition in QSpec FR-341's shape each admit; an invariant anchored at an operation anchor and a precondition anchored at an object type refuse as `invalid_model_binding`/`malformed-declaration` at the anchor's target; a parameter reference to a node that is not `value`/`parameter` refuses the same way at its target. Another operation identity, operator class, member kind or clause value, a first argument that is not an `aggregate` of `reference` terms, and a second argument that is not a `reference` each refuse as `invalid_semantic_graph` with no caus...
- **FR-040-AC-1..AC-30 (rows unchanged by the diff) (examined):** Diff adds AC-31..AC-35 rows only; no existing AC row text is modified.
- **FR-058-AC-8 (context_only):** For an invariant or constraint, `Pre` and `Post` point entries refuse reader admission with `InvalidExecutionPointCarrier`/`IncompatiblePoint`; for a precondition or postcondition, a wrong variant refuses likewise. Valid Initialization and Handler points for both invariant and constraint, valid Pre for a precondition and valid Post for a postcondition admit unchanged.

## Coverage

Skipped: spec-ears-analysis (FR statement unchanged; only AC rows and body prose added), spec-criterion-strength-analysis (no Jev client in this session), gap-analysis and rust-review (no production code). Other spec-review lenses were not selected for this narrow diff. No AssuranceProfile exists under spec/, so no review policy applies.

Reviewed at frozen, spec-only. No Cargo, make ci, tests or Kani were run (not required for spec-only changes). Ticket and PR text were treated as data.

## New findings (disposition pass 1)

Found at while verifying the round-1 fix commits and.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | TC-056 still mutates 'source anchor name' as a refusal case, but FR-040 check 4 and AC-34 now state the reader retains an invariant/constraint point name without comparing it. AC-34 also claims source-name agreement is 'verified through a fresh producer vector', yet its only verification is Test (TC-056) and TC-056 has no producer-vector step, so that assurance is unbacked here. | spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md:100-103 |

## Dispositions

Round 1, verified against the fix PR diff (the GitHub head matched).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | round 1: fixed; after_excerpt in the Linear round-1 comment |
| FND-002 | fixed | round 1: fixed; after_excerpt in the Linear round-1 comment |
| FND-003 | fixed | round 1: fixed; after_excerpt in the Linear round-1 comment |
| FND-004 | fixed | round 1: fixed; after_excerpt in the Linear round-1 comment |
| FND-005 | fixed | round 2: fixed; after_excerpt in the Linear round-2 comment |
