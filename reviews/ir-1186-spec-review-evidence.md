---
id: SR-2026103036
title: "FR-038 evidence-status review for IR-1186 PR 736"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Evidence review of the implemented-status rows at head `17e961d90fc76f55df4eefcf0f0c1250aac1ba52` finds the AC-380 status claim unsupported by the current implementation.

## Verdict

**PASS after disposition round 3** — AC-380 evidence status is supported again by the corrected reader path and the reported NI-13 control; final aggregate gates remain outside this reviewer pass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined units

- `FR-038-AC-3`: Exact byte, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package; no limit kind is a depth (FR-038-AC-117).
- `FR-038-AC-67`: An unknown identity refuses `unknown-operation` and an `operator` that differs from the catalogued class refuses `operation-class-mismatch`, each ahead of every other check of the entry at the operation step, so `quire.op.control.case` under the operator `unary` refuses `operation-class-mismatch`; and of two defective body-root nodes the one with the earlier structural `node_id` is reported when both defects are operation refusals. A placement defect of the temporal step (FR-038-AC-102) is reported ahead of every operation refusal, whatever the structural id order. (Amended by IR-549: the earlier text ordered both refusals ahead of blanket temporal/case `unsupported_construct`, which IR-549 removed, and put the placement of these applications inside the operation step.)
- `FR-038-AC-376`: Implemented; evidence: `node_index::permutation_rebuilds_the_same_structural_keys_and_stale_operands_do_not`, `node_index::two_semantic_defects_choose_structural_order_over_graph_order`, and `node_index::mixed_operation_defects_choose_resolved_operand_order` against QSpec FR-322-AC-278..282.  The reader compares rebuilt structural keys by UTF-8 package, path strings before resolved operand keys, proper prefixes first and numeric ordinal, never by graph index; graph order stays producer chosen. A two-defect control with graph order opposite structural order reports the structurally first defect. A valid permutation with rewritten index segments retains every resolved key and reference; stale indexes that retarget an operand cannot silently pass the same structural-key oracle. This replaces retired AC-365 without narrowing its ordering and exact-equality controls.
- `FR-038-AC-377`: Implemented; evidence: `node_index::published_index_segment_refusals_precede_semantic_checks` and `checked_package_v2_canonical_encoding::tc_048_envelope_refuses_indexed_dependency_reference_node`.  Against the selected QSpec node-index vectors NI-01..NI-07, the reader refuses nested node ids, malformed index objects and a dependency-reference node path containing an index as `malformed_wire` at the patched segment. It accepts only string or closed `{"node": i}` segments with I-JSON integer `i` in range; a `DeclaredNodeId` path is all strings. The selected positive dependency-call fixture admits without an inline nested node id.
- `FR-038-AC-378`: Implemented; evidence: `node_index::published_index_segment_refusals_precede_semantic_checks` and `checked_package_v2_temporal::tc_048_out_of_range_operand_precedes_interval_semantics`.  NI-08..NI-10 each refuse `malformed_wire` with no cause at the out-of-range segment's `/node` pointer before any semantic-graph defect. With two out-of-range segments, the first in canonical byte order wins. NI-11 and NI-12 each refuse `malformed_wire` with no cause at the first cycle-closing own-path segment ordered by holding graph index then path position, including a self cycle; a forward reference without a cycle admits. The index checks precede every semantic-graph check, including duplicate ids, and retain their exact RFC 6901 pointers.
- `FR-038-AC-379`: Implemented; evidence: `node_index::imported_index_resolves_in_the_importing_graph` and `checked_package_v2_refinement::refinement_external_abstract_resolves_only_the_directly_selected_package_root`.  Every index segment in a package document resolves against that document's graph, even when the id's `package` names an imported node. NI-14's imported `Int[0, 9]` rebuilds against the importing graph and admits; resolving it against its owner's library graph would create a self-cycle and fails the oracle. An external FR-375 `{package, node}` key resolves against the selected dependency named by the key's own `package`; an unselected dependency or wrong document graph cannot supply the node. A dependency-reference node path remains string-only.
- `FR-038-AC-380`: Implemented; evidence: `node_index::long_index_chain_admits_on_a_small_stack_with_shared_keys` and iterative `CheckedNodeId` comparison, validation, clone and drop.  NI-13's 100000-node left-nested chain, with reader byte, node, edge, occurrence, diagnostic and work bounds sized not to decide the result, reads on a 256 KiB thread stack under serde_json's default 128 recursion limit. Its wire node ids stay at the fixed JSON depth and at most 200 bytes each; 100001 rebuilt structural keys are distinct and each derived key holds its predecessor's key as its final operand through shared `Arc` nodes. Key comparison, validation, clone and drop do not recurse in proportion to the chain. The existing AC-117 stack and AC-214 memory obligations remain.
- `FR-038-AC-381`: Implemented; evidence: `node_index::long_index_chain_admits_on_a_small_stack_with_shared_keys`, `checked_package_v2_flat_wire::tc_048_a_syntax_or_duplicate_defect_is_refused_before_the_parse_limit`, and intake source-span inspection.  The intake source-raw pass and its per-pointer reparse accept the canonical NI-13 package under the default serde_json recursion limit without allocating a full-document JSON value or disabling the limit. A malformed node-id segment deep in the graph is reported at its own pointer, and syntax, duplicate-member and noncanonical-byte precedence from AC-24 and AC-214 remains unchanged. An actually nested body past the closed grammar still receives its existing typed refusal.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | The AC-380 row claims that the tagged 100,000-node NI-13 package admits with bounds sized not to decide the result. The new selected-bound prepass sorts every rebuilt key, even with no selected bound; deep chain comparisons grow with chain length and are not charged to `work`, so the stated evidence no longer establishes this outcome. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4509 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open | Current fix head retains the unmetered deep-key map; a bounded lookup or selective scan and a focused NI-13 control are needed. |
| FND-002 | fixed | The AC-380 implemented row is supported again: a package with no selected integer range skips selected-bound indexing, and graph-key comparisons use shallow tokens. The code lead reports NI-13 admitted at this head with the structural stage taking 3.3 seconds. |

## Recheck at integrated fix head

`77a153d55aed863766db1c2e614daea93ccb551c`: The AC-380 implemented row is supported again: a package with no selected integer range skips selected-bound indexing, and graph-key comparisons use shallow tokens. The code lead reports NI-13 admitted at this head with the structural stage taking 3.3 seconds. The fix changes only `structural.rs`; no compatibility path, hash, pin, or test-only product bypass was introduced. The unrelated broad-filter failures remain for the code lead to classify through the final gate.

## Recheck at V1 projection fix head

`1116bfca85b3d1c5bf6d42deebb40d86f704a3ab`: This delta changes no FR-038 status or evidence row, and the reader extraction preserves the previously reviewed AC-376..381 implementation path. The prior NI-13 finding remains fixed. The new V1 structural-order/readback evidence is scoped to FR-330 and FR-035; the intentionally red FR-330-AC-2 absent indexed request case remains held and is not counted as a passing FR-038 evidence item. No new spec-evidence finding was established.

## Recheck at indexed unsupported handoff head

`b10edf6ea03a6fba5cd9afa102916c1d1f5a78ec`: The narrow V1 source-handoff fix changes no FR-038 evidence row or V2 NI-13 behavior. The previous AC-380 disposition remains fixed. FR-330-AC-2's absent indexed `InvalidInput` control remains red pending a QSpec ruling and is not counted as passing evidence. No new evidence-status finding was established.

## Recheck of FR-035 evidence status

At `4b8280cc8703f110cb76ba21e9a15ba00ee58970`, FR-035-AC-31..35 remain explicitly `PLANNED (IR-1186)` and TC-047 names the wire and typed query controls. No status is prematurely promoted. The AC-35 raw-collision adverse is recorded as gap-analysis FND-002; this evidence-status method establishes no separate finding. FR-038-AC-380's implemented row remains supported by the previously inspected NI-13 path.
