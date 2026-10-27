---
id: SR-610016
title: "IR-1040 PR #598 acceptance evidence gap review at e2e310ae"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-040-admit-frame-entries-and-state-clauses.md, tests/it/checked_package_v2_frame_entries.rs, crates/quire-contract-model/src/checked_package/v2/typed_terms.rs, crates/quire-contract-model/src/checked_package/v2/{execution_point,state,structural,flat_wire}.rs; FR-040-AC-31..36"
review_set: subset
---

## Summary

Ticket: IR-1040. Static gap review of PR #598 at exact head e2e310ae1c0cc0da852e9311b8b09efa93e6c992 for FR-040-AC-31..36. Every criterion has at least one trace-tagged test. Several AC clauses have no behavioural oracle, the AC-35 precedence clauses most of all. QSpec's producer-origin TC-056 vector (the fresh QSL producer vector that FR-040-AC-34 names) is not in this PR. It is a separate open follow-up, QSL-1082, and this review does not claim it passes.

## Scope

- FR-040-AC-31 (examined): An anchor admits no point or a fourth bare `pre`/`post` point. The forms have distinct identities. Reordering or an extra binding refuses at `/body`, and a stale key refuses `stale-node-key`.
- FR-040-AC-32 (examined): A malformed anchor point refuses at `/body/members/3/value`. A well-formed `initialization`/`handler` point, or a point whose operation differs from the checked operation, refuses `malformed-declaration` at the node. Duplicate `(context, operation, point)` tuples are refused, and distinct points admit.
- FR-040-AC-33 (examined): The optional clause point admits each variant on its applicable clause kind. Absence is explicit. Points produce distinct identities and round-trip unchanged. A `constraint` is checked like an invariant.
- FR-040-AC-34 (examined): A malformed clause point refuses at `/body/operation/member/execution_point` before the state step. An inapplicable point, a mismatched operation name, or an anchor/clause point mismatch (including one-sided absence) refuses `malformed-declaration`. The source-name agreement is QSL-owned. A point-only stale key refuses `stale-node-key`.
- FR-040-AC-35 (examined): Precedence across graph shape, stale key, frame, state and catalog. Within one anchor, context, frame, types, operation and scoped deletes come before point applicability, and tuple duplicates are detected after all anchors and before clauses. Within one clause, anchor, parameters and signature come before point checks.
- FR-040-AC-36 (examined): The writer preserves the exact anchor and clause point, absence emits nothing, and read/write/read returns equal points, keys and package id.
- QSL-1082 producer-origin TC-056 vector (context_only): open follow-up. Not present and not claimed.

## Verdict

**CONDITIONAL** — no criterion is untagged and no high-severity gap was found. The gaps are medium and low: precedence and inapplicable-point branches are implemented but not asserted. No stubs or unowned production behaviour were found in the diff. Every new production branch traces to FR-040 "Operation anchor body" or "State clause body" check 4. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-040-AC-35 is tagged only on `tc_056_execution_point_shape_and_agreement_refuse_at_their_stage`. That test asserts just one precedence pair: a stale key before point agreement. No test combines a point defect with an anchor operation or scoped-deletes defect, a duplicate tuple with a later anchor's point defect, or a clause signature defect with a clause point defect. A reordering of `check_anchor`, `validate_state` or `check_clause` would pass every test. | tests/it/checked_package_v2_frame_entries.rs:1378; crates/quire-contract-model/src/checked_package/v2/state.rs:325 |
| FND-002 | medium | FR-040-AC-32 refusal branches are untested for an anchor `initialization` point and for an anchor `pre`/`post` point whose `operation` differs from the checked operation. Only `handler` on an anchor is asserted. | tests/it/checked_package_v2_frame_entries.rs:1378; crates/quire-contract-model/src/checked_package/v2/state.rs:325 |
| FND-003 | low | FR-040-AC-36's encoder read/write/read round trip covers only the anchor and precondition points. Invariant and constraint clause points (`initialization`/`handler`) are checked through `serde_json::to_value` of the admitted body, never through `encode_checked_package_v2` and a re-read. | tests/it/checked_package_v2_frame_entries.rs:1123 |
| FND-004 | low | The FR-040-AC-33 clause "A `constraint` is checked like an invariant" is asserted only by admission. No test refuses a constraint carrying a `pre`/`post` point, a constraint whose anchor is not a `model`/`object_type`, or a constraint whose signature has more than `self`. No test asserts that a constraint's package id differs from the absent-point form. | tests/it/checked_package_v2_frame_entries.rs:1123 |
| FND-005 | low | The FR-040 "non-point fourth value" case (a fourth `execution_point` binding whose value is not an object) has no test. Its expected locus is the anchor `body`, and the code refuses at `/members/3/value`; see SR-610015 FND-002. | tests/it/checked_package_v2_frame_entries.rs:1378; crates/quire-contract-model/src/checked_package/v2/execution_point.rs:67 |

## Coverage

Plan completion: not assessed. A grep index over `#[trace(...)]` tags stood in for the computed matrix. `quoin matrix` / `quire matrix` was deliberately not run: the dispatch brief allowed static review only, with no matrix or spec-validation command.

Tags found per criterion:

| Criterion | Tagged tests |
| --- | --- |
| FR-040-AC-31 | 2 (`anchor_point_is_a_closed_bare_typed_object`, `tc_056_execution_points_rekey_and_round_trip_as_bare_wire_objects`) |
| FR-040-AC-32 | 3 (adds `tc_056_same_operation_admits_distinct_anchor_points` and `tc_056_execution_point_shape_and_agreement_refuse_at_their_stage`) |
| FR-040-AC-33 | 2 |
| FR-040-AC-34 | 2 |
| FR-040-AC-35 | 1 |
| FR-040-AC-36 | 2 |

Bindings were inspected against the assertions in each test. All are correct, and AC-35 is thin (FND-001). The optional semantic review was not invoked as a separate exercise. The intent-to-test gaps above came from the changed-code review.

Existing focused evidence, reported by the lead and not re-run here: 30/30 frame-entry tests and the 1/1 model anchor test. The full aggregate gate is deferred to the lead after review. The QSL producer-origin TC-056 fixture (QSL-1082) remains open and is not claimed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 58ffe3afebb58d572ce66ce287dda31b229bef90 the new test `tc_056_point_checks_follow_anchor_joins_and_clause_signature` (tagged FR-040-AC-35) asserts that operation resolution and scoped deletes come before anchor point applicability, and that the clause signature comes before clause/anchor point agreement. `tc_056_same_operation_admits_distinct_anchor_points` (now also tagged AC-35) asserts that anchor point applicability comes before duplicate-tuple detection. |
| FND-002 | fixed | `tc_056_execution_point_shape_and_agreement_refuse_at_their_stage` now loops over `initialization{startup}`, `handler{tick}`, `pre{withdraw}` and `post{withdraw}` on the anchor. Each expects `invalid_model_binding`/`malformed-declaration` at the anchor node. |
| FND-003 | fixed | The new `assert_pointed_writer_round_trip` helper runs the encoder plus read/write/read for invariant clause points (`initialization`/`handler`) and constraint clause points, comparing body bytes, package id and graph equality. |
| FND-004 | fixed | The constraint test now asserts a package-id and node-digest difference from an absent-point constraint. It also asserts refusals for a `pre` point on a constraint (`malformed-declaration` at the node), an operation-anchor target (`malformed-declaration` at `arguments/1/target`) and a two-parameter signature (`ill_typed`/`operator-ineligible` at `arguments/0`). |
| FND-005 | fixed | The "non-point fourth value" case covers a string, a number and a `reference` term in the fourth binding, each expecting `invalid_semantic_graph` with no cause at the anchor `body`. |

## Disposition Verdict

**PASS for FND-001..005 at 58ffe3afebb58d572ce66ce287dda31b229bef90.** No new gap findings. The QSL producer-origin TC-056 vector remains the open follow-up QSL-1082 and is not claimed. Plan completion: not assessed. Static review only, with the coder's focused tests reported green and not re-run here. The full aggregate gate is deferred to the lead.
