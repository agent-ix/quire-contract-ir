---
id: SR-610007
title: "IR-1040 PR 593 spec-review/integrity review"
type: SpecReview
analysis: integrity
scope: 'agent-ix/quire-contract-ir; PR #593; spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, spec/checked_package/matrix/TC-056-checked-package-v2-frame-entries-and-state-clauses.md, spec/checked_package/matrix/tests.md; governing QSpec agent-ix/quire-specification origin/main FR-322-AC-131, FR-341-AC-14, FR-342-AC-10'
review_set: subset
---

## Summary

Ticket: IR-1040. Integrity analysis (consistency and completeness) of the same scope. It covers refusal classification against FR-040's own convention, the wire/term grammar the fourth binding needs, reader order between point checks and tuple uniqueness, writer obligations, and the exactness of refusal codes.

## Verdict

**CHANGES REQUESTED**: four medium and one low finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Point-shape defects (unknown variant, missing or extra member, empty name) are assigned to the state step as invalid_model_binding/malformed-declaration. FR-040 elsewhere maps QSpec schema-failing body shapes to invalid_semantic_graph at the body before the frame step (AC-7, AC-8), so an extra member on the state_clause member refuses one way and an extra member inside its execution_point the other. QSpec's ExecutionPoint oneOf rejects these at schema while FR-341/FR-342 prose says malformed-declaration; the PR resolves that QSpec tension silently instead of marking an explicit IR reading, as FR-040 does for AC-30. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:367 |
| FND-002 | medium | QSpec's fourth anchor binding value is a bare ExecutionPoint object ({kind, operation}) rather than a term. The reader's typed body grammar stores a binding value as a CheckedTerm (typed_terms.rs:117-121), and FR-038 owns the flat-wire/term grammar. Neither FR-040 nor FR-038 states how a non-term binding value is admitted, nor which step refuses one that fails to decode. AC-31 and AC-35's 'graph shape' precedence are therefore unanchored, and FR-040's claim to add 'no shape of its own' is not reconciled with the grammar change. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:366 |
| FND-003 | medium | AC-35 orders an anchor's point agreement after its own joins, but never orders it against the package-wide (context, operation, point) uniqueness check, which keys on the point. Two anchors that share context and operation, where one carries an unknown or disagreeing point, can be reported either as ambiguous_declaration/ambiguous-name or as invalid_model_binding/malformed-declaration. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:370 |
| FND-004 | medium | IR-1040 covers reader and writer support, but no AC states the writer/encode obligation: that a re-encoded admitted anchor and clause emit the point unchanged, and that an absent point emits nothing. AC-31 has no anchor-point retention or round-trip at all; only AC-33 states round-trip for clauses, while TC-056 says it round-trips 'the retained point'. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:366 |
| FND-005 | low | 'Reordering the fourth binding, adding another binding, or changing the point without rekeying refuses before the state step' gives no exact code, cause or locus for the reorder and stale-key cases, although the prose implies invalid_semantic_graph at the body and invalid_package/stale-node-key. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:366 |

## Scope

- **FR-040-AC-31 (examined):** An anchor admits its three required bindings with no point, and admits a fourth `execution_point` binding after `frame` with `pre{operation: "scaled"}` or `post{operation: "scaled"}`; these three forms have different rederived node identities and package ids. Reordering the fourth binding, adding another binding, or changing the point without rekeying refuses before the state step. This supersedes only AC-7's blanket extra-binding refusal: an extra binding other than the specified fourth still refuses `invalid_semantic_graph` with no cause at the anchor `body`.
- **FR-040-AC-32 (examined):** An anchor with an unknown point variant, missing or extra point member, empty name, `initialization` or `handler` point, or point whose operation differs from the anchor's checked operation refuses `invalid_model_binding`/`malformed-declaration` at that anchor in the state step. A duplicate `(context, operation, point)` tuple refuses `ambiguous_declaration`/`ambiguous-name` at the second anchor by ascending node-id digest; the same context and operation with distinct accepted points or one absent point admit. This refines AC-7's pair uniqueness without changing its no-point duplicate outcome.
- **FR-040-AC-33 (examined):** A state clause admits an optional `execution_point` member with each closed FR-322 variant: `initialization{name}` and `handler{name}` on an invariant or constraint, `pre{operation}` and `post{operation}` on a precondition or postcondition. The point is absent only when the checked clause has none. Equal application bodies except for absent versus different accepted points have different rederived node identities and package ids; an accepted point round-trips unchanged. A `constraint` is checked like an invariant. This extends AC-8's three listed admitted clause kinds.
- **FR-040-AC-34 (examined):** An unknown clause point variant, missing or extra point member, empty name, point inapplicable to its clause kind, name different from the checked source anchor or resolved operation, or point different from the referenced operation anchor's point refuses `invalid_model_binding`/`malformed-declaration` at the clause node in the state step. A stale node key after only the point changes refuses `invalid_package`/`stale-node-key` before the state step.
- **FR-040-AC-35 (examined):** With simultaneous defects, graph shape and stale-key refusals precede the frame step; the frame step precedes state; within state, placement precedes ascending-id anchors and anchors precede ascending-id clauses. For one anchor, context, frame, types, operation and scoped deletes retain their order before its point agreement; for one clause, anchor, parameters and signature precede its point check, and operation catalog checking follows the state step. The first applicable defect's exact code, cause and RFC 6901 locus are returned with no package.
- **FR-040 Operation anchor body (execution point paragraph) (examined):** An anchor's optional fourth binding has exactly the closed FR-322 `ExecutionPoint` shape. It is `pre{operation}` or `post{operation}`, with `operation` equal to the resolved `operation` binding name; a clause-bound anchor's point equals every clause that references it, while a protocol-only anchor may omit the point. An unknown variant, missing or extra member, empty or disagreeing operation name, or `initialization`/`handler` point refuses `invalid_model_binding`/`malformed-declaration` at the anchor node in the state step. The fourth binding enters the structural node preimage; absence an...
- **FR-040 State clause body (member, check 1 and check 4) (examined):** 1. **Anchor.** The anchor target is a declared dependency, else `missing_declaration`/`missing-name`; for an invariant or constraint it is a `model`/`object_type`, `model`/`process` or `model`/`state_machine` node and for a precondition or postcondition a `state`/`operation_anchor` node, else `invalid_model_binding`/ `malformed-declaration`, located at the anchor's target. | 4. **Execution point.** When present, the point has exactly one of FR-322's closed shapes: `initialization{name}` or `handler{name}` for an invariant or constraint, and `pre{operation}` or `post{operation}` for a precondition or postcondition. The name equals the checked source anchor name; for an operation clause it equals the resolved operation name and its point equals its `operation_anchor` point. An unknown variant, missing or extra member, empty or disagreeing name, inapplicable kind or mismatched anchor point refuses `invalid_model_binding`/`malformed-declaration` at the clause node.
- **TC-056 (examined):** Build both an unpointed protocol anchor and point-qualified pre and post anchors for the same checked operation.... Cover FR-040-AC-31 through FR-040-AC-35. This is planned until tagged reader tests and the full reviewed-code gate pass.
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
| FND-006 | high | Check 4 still says an unknown variant, missing or extra member or empty name refuses invalid_model_binding/malformed-declaration at the clause node in the state step. The new IR-reading paragraph (lines 237-246) and AC-34 refuse those same defects as invalid_semantic_graph with no cause at body/operation/member/execution_point before state. Two normative statements give different code, locus and step for one input. | spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md:303-313 |

## Dispositions

Round 1, verified against the fix diff.. (the GitHub head matched).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | round 1: fixed; after_excerpt in the Linear round-1 comment |
| FND-002 | fixed | round 1: fixed; after_excerpt in the Linear round-1 comment |
| FND-003 | fixed | round 1: fixed; after_excerpt in the Linear round-1 comment |
| FND-004 | fixed | round 1: fixed; after_excerpt in the Linear round-1 comment |
| FND-005 | fixed | round 1: fixed; after_excerpt in the Linear round-1 comment |
| FND-006 | fixed | round 2: fixed; after_excerpt in the Linear round-2 comment |
