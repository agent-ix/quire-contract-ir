---
id: SR-1587
title: "failure-domain review, disposition round 1, of quire-contract-ir PR #298 (IR-627 skip predicate)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@49b85f5d96d54b655660117c5fff54ba4973eade; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Derived-shape nodes inside a recursion group; FR-038-AC-145 to AC-150; Q4)"
review_set: subset
---

## Summary

Ticket: IR-627. These are new findings from disposition round 1 over the
fix-round head 49b85f5. The rewritten predicate closes the original hole:
- a lone label or a self-dependency no longer escapes verification;
- `integer_range`, `boolean`, `integer` and `reference` are always verified.

Re-measured against the code and fresh QSL main (a9cfe11):
- `MemberType::node_key` (model_members.rs:445-490) keys `integer_range` with
  `semantic_type` Integer and a body of Integer-typed literals.
- Integer and Boolean are self-typed with an empty body.
- QSL FR-094 (lines 120-122) gives model declaration nodes an empty body "so
  Reference attributes key without a cycle".
- The reader's `recover` refuses a model node that carries a `recursion_group`
  (model_members.rs:911).

So none of the four always-verified shapes can be on a names cycle without
changing a member its key binds. The forged two-node group of AC-148 is an
honest stated limit, and no node read as `integer_range` is reachable through it.

The reader has the data to compute the names-graph SCC. Body references,
literal types and `semantic_type` are on the wire and already sit in the
adjacency of `validate_graph`.

Six defects remain. The predicate's "exactly the labelled set" condition
contradicts three ACs. One mutation row cannot catch its mutant. The shape test
reads an unverified wire form. Three precision nits.

## Findings

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-001 | high | The predicate's fourth condition requires the names-graph component to have exactly the label's nodes as members. That un-skips a genuine group as soon as any labelled node is off its cycle. The genuine members are then derived as ungrouped and refuse `stale-node-key`. This contradicts three ACs: (a) AC-147 row 3 says a group with one extra off-cycle labelled node admits when that node holds its ungrouped key, but the group's `Sequence` and `collection_bounds` refuse; (b) AC-148 row 3 (`Sequence<Tree>` re-pointed to another on-cycle node, which leaves the labelled record off the cycle) says no `stale-node-key`, but the remaining members refuse; (c) AC-146 row 2 (Tree's label added to `Int[0, 1000]`) pins the refusal at the Int node, but the Tree members also refuse and digest order picks the locus. Fix: require that every member of the component carries the node's label (component is a subset of the labelled set). That keeps every attack closed, because a skip still needs a names cycle, and makes all three rows hold as written. Otherwise rewrite the three rows to the refusals the predicate produces. | FR-038 Decided: skip, FR-038-AC-145, FR-038-AC-146, FR-038-AC-147, FR-038-AC-148 | wrong-requirement |
| FND-002 | medium | AC-146 row 3 adds to `Int[0, 1000]` a body `reference` naming a Tree group node. Its mutation row says "a reader that skips every node on a names cycle admits the third and fails". No Tree node names the Int node, so the row forms no cycle. That mutant still verifies the node, finds the body not closed, and refuses, so the mutant is not caught. Close the cycle: also re-point a labelled collection's body `reference` at the Int node, and label the Int node. | FR-038-AC-146 | wrong-requirement |
| FND-003 | medium | The shape test reads the wire `node_tag` and `semantic_form`, and for a skipped node the key that binds them is not verified. `collection_bounds` and `integer_range` share one body form (`bounds(&integer, ...)` in `MemberType::node_key`). So a node keyed `Int[0, 1000]` can be retagged `collection_bounds`, with `max` changed and `semantic_type` re-pointed at a labelled collection whose body references it, and the pair forms a names cycle and is skipped. "`integer_range` is never skipped" holds for the wire form only. A consumer that dispatches on form reads no range from that node. A consumer that takes the field type from the domain document and the node from the graph by key gets a type-confused node. State this in the limit and add an AC-146 row, or cross-check the node's tag and form against the domain-derived `MemberType` at the trust-root join. | FR-038 Decided: skip, Stated soundness limit, FR-038-AC-146 | missing-requirement |
| FND-004 | low | The names graph is defined as body `reference`, `semantic_type` and literal `type`, and the spec says the reader holds it "as the edges of its own cycle graph less the `dependencies` edges". IR's cycle graph also carries `result_type` and frame-entry edges (`ReferenceMember::{ResultType, FrameEntry}`, common.rs:609). QSL FR-092's graph includes `result_type` and member `declaration`. No SCC holding a skippable node changes, because type nodes name no expression or frame node, but the definition should match one source exactly. | FR-038 Decided: skip | wrong-requirement |
| FND-005 | low | AC-147 is imprecise in three places. Row 2 (self-dependency) "refuses identically" only if the node also carries the label: without it, IR's dependency-counting cycle check refuses `invalid_semantic_graph` first (AC-18). Row 1 reads "Over the Tree package: a `collection_bounds` node of an acyclic package", which contradicts itself. The last clause (`semantic_type` re-pointed off the cycle) states no outcome, and that mutation dissolves the whole group. | FR-038-AC-147 | wrong-requirement |
| FND-006 | low | AC-149's limit clause ("a `work` limit set to the work the whole read used minus one returns `incomplete`") holds for any implementation, the mutant included, when the used work is measured from that same run. Only the counter row catches the uncharged skip. Record the expected work as a constant, or drop the clause. | FR-038-AC-149 | correct-requirement-no-evidence |

## Dispositions

Round 2, reviewed at 4f953d9434d0eae651de39a2934543c9e16b34cd.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4f953d9: the predicate now requires every node of the component to carry the label, and labelled nodes outside it do not matter. AC-147's extra-node row admits, AC-148 row 3 admits, and AC-146 row 2 refuses at the Int node alone. |
| FND-002 | still-open | The cycle is now closed, but it is closed through the Tree group's Sequence node. That breaks the Tree cycle: the collection_bounds node still carries the Tree label, is now on no cycle, is verified as ungrouped, and refuses stale-node-key, because its stored key is the QSL in-group key. Digest order, not the row, then decides whether the report lands at Int[0, 1000] or at that collection_bounds node. The 'skip any shape on a cycle' mutant also refuses, at the collection_bounds node, so the claim that it admits is false. Build row 3 on the acyclic Set<Int[0, 1000]>[0, 3] package of AC-147: label the set and the Int node and close the cycle between them. |
| FND-003 | fixed | 4f953d9: the retag is now stated in the limit, with its consumer-by-key effect, and recorded as an AC-148 row. Closing it is deferred to AC-150 (IR-630). Acceptable: CG's IR-628 route reads member types from the retained domain-document table, not from node bodies. |
| FND-004 | fixed | 4f953d9: the names graph is now IR's cycle graph less dependencies, including result_type and frame entries, and the differences from QSL FR-092 are stated and argued harmless. |
| FND-005 | fixed | 4f953d9: AC-147 now builds on an acyclic Set<Int[0, 1000]>[0, 3] package, the self-dependency row carries the label (and the unlabelled case refuses per AC-18), and every row has an outcome. |
| FND-006 | fixed | 4f953d9: AC-149 now asserts only the counter against the constant K counted from the built package. |
| FND-002 | fixed | 861f997: round 3. AC-146 row 3 is now built on the acyclic Set<Int[0, 1000]>[0, 3] package. The Int node's semantic_type is re-pointed at the set node, both are labelled x, and the two form a fully labelled names cycle. The set node is skipped. The Int node is always verified, and its re-derived key differs because semantic_type and max are both in its preimage, so it refuses stale-node-key. No earlier stage refuses (no application, parameter or compound-unit node is touched), and collection_bounds is unlabelled with its key intact, so the Int node is the only refusal. The 'skip any labelled node on a cycle' mutant skips the Int node, admits, and is caught. |
