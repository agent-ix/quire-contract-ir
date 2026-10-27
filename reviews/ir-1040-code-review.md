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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | `CheckedOperationAnchorBody` is a public struct with a public `execution_point: Option<_>`, but the reader only ever builds it with `Some(point)`; a point-less anchor decodes as `CheckedTerm::Aggregate`. A caller-built `OperationAnchorBody { execution_point: None, .. }` writes the same bytes as the aggregate form, but reads back as a different `CheckedTerm` (`Aggregate`). So one wire form has two typed representations, and the caller's value is not preserved through a read/write/read round trip. Fix by giving every anchor one form (decode three-member anchors to `OperationAnchorBody` with `None`), or by making the point non-optional; the latter may belong with IR-1038's FR-066 public typed point surface. | crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:63, crates/quire-contract-model/src/checked_package/v2/execution_point.rs:87 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | medium | `decode_anchor_body` returns `MissingMember { member: "members" }` for any `operation_anchor` body without a `members` array, and it ignores the `term` tag. A well-formed non-aggregate anchor body passes `validate_anchor_body` and flat wire, because both fall back to the generic term check. Examples are a `reference` term, or a root `quire.op.state.clause` application, which FR-040 refuses `ill_typed`/`operator-ineligible`. Such a body now fails the typed decode in `validate_graph` and refuses `malformed_wire` at `/body`. On main it decoded and then refused at the structural stage, `invalid_semantic_graph` at `/body`. The fix commit 74eabb1c introduced this, and round 1 missed it. No test covers a non-aggregate anchor body. A fix is to fall back to `CheckedTerm::decode_admitted(body)` when the body is not a `members`-bearing aggregate, and add one adverse case. | crates/quire-contract-model/src/checked_package/v2/execution_point.rs:85, crates/quire-contract-model/src/checked_package/v2/mod.rs:2659 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 58ffe3afebb58d572ce66ce287dda31b229bef90 `CheckedTerm::ExecutionPoint` and the `TermTag::Missing if kind` branch of `decode_admitted` are removed. The point is now the dedicated `CheckedExecutionPoint` held by the anchor-only `CheckedOperationAnchorBody`. Only `execution_point::decode_anchor_body` builds it, selected by node kind in `decode_admitted_body`. The generic decoder now refuses a bare point (unit test). Both types are re-exported at the crate root (lib.rs:186). |
| FND-002 | fixed | `validate_anchor_body` now refuses a fourth value at the anchor `body` unless it is an object with no `term` (execution_point.rs:67). Only an object failing the closed point schema refuses at `/members/3/value`. The test "non-point fourth value" asserts the `body` locus for a string, a number and a `reference` term. |
| FND-003 | fixed | `check_anchor_flat_wire` now flat-checks every member except a binding named `execution_point`, whatever its position or the member count (flat_wire.rs:275), and checks a non-aggregate root as a term. The test "nested state application precedes wrong point position" asserts `malformed_wire` at `/members/4/value` over a misplaced point. |
| FND-004 | fixed | The canonical `Encode` arm now writes the point through `writer.serialize(point)` using the serde derive (typed_terms.rs:842), with `FixedShape` depth 1. The four hand-spelled variant/field arms are gone. |
| FND-005 | fixed | Each `CheckedExecutionPoint` variant field now has a doc comment ("Nonempty source anchor name." / "Nonempty operation name."). |
| FND-006 | fixed | Round 2, at db0c0f5b8072036f1eba002331cb6be2cef44a48. `decode_anchor_body` now decodes a three-member anchor as `OperationAnchorBody { execution_point: None }` and a four-member one as `Some(point)`, so every admitted anchor has one typed form. `typed_anchor_body` reads both through the same branch, and no consumer matches an anchor term as `Aggregate`; every other use is a node-kind check. `tc_056_execution_points_rekey_and_round_trip_as_bare_wire_objects` now asserts `OperationAnchorBody` for the absent, `pre` and `post` packages, with `execution_point.is_some()` matching the wire point, and runs a writer round trip asserting graph equality. |

## Disposition Verdict

**PASS for FND-001..005 at 58ffe3afebb58d572ce66ce287dda31b229bef90; one new low finding (FND-006).** All original findings are fixed, checked statically against the fix commits. FND-006 is low: the reader and writer behave correctly for every admitted package, and the defect only reaches caller-constructed values. The lead may route it to the coder, or record a deferral to IR-1038 (FR-066) with a reason. Static review only. The full aggregate gate is deferred to the lead.

## Disposition Verdict (round 2)

**FND-006 fixed at db0c0f5b8072036f1eba002331cb6be2cef44a48. One new medium finding, FND-007, remains open.** It is a refusal-code regression for non-aggregate operation-anchor bodies: `malformed_wire` instead of the graph-shape `invalid_semantic_graph`. It is not mergeable until it is fixed or explicitly dispositioned. Static review only. The full aggregate gate is deferred to the lead.
