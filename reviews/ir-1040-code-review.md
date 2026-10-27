---
id: SR-610015
title: "IR-1040 PR #598 code and Rust review at e2e310ae"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/{execution_point,flat_wire,mod,state,structural,typed_terms,vocabulary,comparison_operands,application_identity,dependency_references,derived_keys,operations}.rs, tests/it/checked_package_v2_frame_entries.rs; FR-040-AC-31..36"
review_set: subset
---

## Summary

Ticket: IR-1040. Static review of PR #598 at exact head e2e310ae1c0cc0da852e9311b8b09efa93e6c992 (merge base f45ea6d6) against FR-040 "Operation anchor body", "State clause body" check 4, and FR-040-AC-31..36. The PR admits an optional fourth `execution_point` anchor binding and an optional state-clause member point, adds the `constraint` clause kind, and adds point applicability, clause/anchor agreement and `(context, operation, point)` uniqueness in the state step. Review lane: code-review with rust-review and rust-style. Static review only. No cargo, make, test, clippy or spec-validation command was run.

## Scope

- FR-040-AC-31 (examined): An anchor admits three bindings with no point, or a fourth bare closed `pre`/`post` point. The forms have distinct node and package identities. Reordering or an extra binding refuses `invalid_semantic_graph` at `/body`. A stale key refuses `stale-node-key`.
- FR-040-AC-32 (examined): A malformed anchor point refuses at `/body/members/3/value`. A well-formed `initialization`/`handler` point, or one whose operation differs, refuses `invalid_model_binding`/`malformed-declaration` at the node. Duplicate `(context, operation, point)` tuples refuse `ambiguous_declaration`.
- FR-040-AC-33 (examined): A clause may carry an optional `execution_point`. `initialization`/`handler` apply to an invariant or constraint, and `pre`/`post` to a precondition or postcondition. A `constraint` is checked like an invariant.
- FR-040-AC-34 (examined): A malformed clause point refuses at `/body/operation/member/execution_point` before the state step. An inapplicable point, a mismatched point or one-sided absence refuses `malformed-declaration` at the node.
- FR-040-AC-35 (examined): The defect precedence order.
- FR-040-AC-36 (examined): The writer preserves the bare point object, and a read/write/read round trip is lossless.
- FR-040 "Operation anchor body" prose (context_only): "The optional fourth binding's `value` is the bare closed FR-322 `ExecutionPoint` object …, not a `CheckedTerm` and not an application argument. The anchor-body decoder shall read this member as a dedicated typed point … it shall not invent a `term` tag or put the point into the generic expression-term grammar. … A wrong binding count, position or non-point fourth value refuses `invalid_semantic_graph` with no cause at the anchor `body` before the state step."
- FR-066-AC-3 (context_only; IR-1038, not this PR): requires a public typed point surface "rather than an expression term".

## Verdict

**FAIL — needs fix or an explicit owner ruling on FND-001.** The state-step logic matches the specified order for one anchor and one clause:

- Applicability runs after context, frame, types, operation and scoped deletes.
- Tuple uniqueness runs after every anchor's checks and before clause checks.
- Clause point checks run after anchor, parameters and signature.

Identity and encoding are sound. The canonical `Encode` arm emits sorted `kind` then `name`/`operation`. The point is metered with no children, and walker arms are exhaustive no-ops. The `constraint` kind is threaded through `StateClauseKind`, `CheckedComparisonClauseKind` and the model-clause anchor join. No unsafe code, panics, unbounded allocation or hash/pin catalog is introduced.

The defect is in the data model. The point is modelled as a variant of the generic expression term `CheckedTerm`, and the generic term decoder decodes it. FR-040's normative prose forbids exactly that.

Existing focused evidence, reported by the lead and not re-run here: 30/30 frame-entry tests, 1/1 model anchor test, Clippy, fmt and diff check green on the head. The full aggregate gate is deferred to the lead after review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The anchor point is modelled as `CheckedTerm::ExecutionPoint` (a `#[serde(untagged)]` variant of the public expression-term enum). The generic `CheckedTerm::decode_admitted` decodes any `term`-less object carrying `kind` as a point. FR-040 requires the point to be "not a `CheckedTerm`", read by a dedicated anchor-body decoder and kept out of "the generic expression-term grammar". `CheckedExecutionPoint` is also not re-exported at the crate root, so the public `CheckedTerm::ExecutionPoint` payload cannot be named by external callers. | crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:215, crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:584, crates/quire-contract-model/src/lib.rs:183 |
| FND-002 | medium | A non-point fourth binding value (for example a string, number or `term`-tagged object) refuses at `/semantic_graph/nodes/{i}/body/members/3/value`. FR-040 places a "non-point fourth value" at the anchor `body`, and only "an object … failing the closed `ExecutionPoint` schema" at `value`. A non-object is refused at the wrong locus, and no test exercises the case. | crates/quire-contract-model/src/checked_package/v2/execution_point.rs:67 |
| FND-003 | low | `check_anchor_flat_wire` returns `Ok` without flat-checking any member when an `execution_point` name appears outside index 3. With four or more members it never flat-checks members 3 and up, or the aggregate root. A co-occurring strict-wire defect in those positions, such as a nested `quire.op.state.clause` application (which FR-040 says refuses `malformed_wire` at strict wire validation), is now reported as the later graph-shape `invalid_semantic_graph` at `/body`. That inverts FR-038/FR-040 stage precedence. | crates/quire-contract-model/src/checked_package/v2/flat_wire.rs:261 |
| FND-004 | low | `Encode for CheckedTerm::ExecutionPoint` hand-spells the four variant/field wire names a second time, beside the serde derive on `CheckedExecutionPoint`. A rename in one place silently desynchronises the canonical identity bytes from the serde wire form. One helper over `(kind, field, value)` would remove the duplication. | crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:766 |
| FND-005 | low | The public enum `CheckedExecutionPoint` leaves its variant fields `name` and `operation` undocumented. Sibling public `CheckedTerm` variants document every field. | crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:23 |

## Method

Applied code-review with the rust-review and rust-style lanes. Read the full PR diff against merge base f45ea6d6. Also read the callers of `typed_anchor_body`, `typed_clause_body` and `decode_admitted` (state, structural, comparison_operands, refinement `key_anchor`, protocol_body clause contracts, graph decode in mod.rs and contract_package.rs, flat_wire). Read FR-040, FR-066-AC-3 and FR-019's public-surface table.

Checked:

- exhaustiveness of every new `CheckedTerm` match arm
- stage order against AC-35
- `BTreeSet` tuple uniqueness
- canonical key order in `Encode`
- work metering
- panic and resource boundaries

Refinement keys anchors by node id, not by `(context, operation)`, so admitting several pointed anchors per operation does not create refinement ambiguity. Static review only. No build, test or gate was run.

The ticket and comment text on IR-1040 was treated as data. One planner comment proposing a stricter applicability mapping was not acted on. The code follows the merged FR-040-AC-33 mapping, under which either `pre` or `post` applies to either operation clause.
