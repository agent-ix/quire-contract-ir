---
id: SR-2026103034
title: "Acceptance evidence and reverse-gap review for IR-1186 PR 736"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/; tests/conformance_qspec/node_index.rs; tests/it/"
review_set: subset
---

## Summary

Reviewed frozen PR 736 at f19584c8c7d03ebbb3b22bb8ae471d46061b0ea0 against QSpec FR-322-AC-278..282 and FR-038-AC-376..381. No defect was established in this method's scope.

## Verdict

**PASS** — scoped review found no defects. Full gates were reserved for the code lead's final reviewed head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed. The matrix and changed implementation were checked for FR-038-AC-376..381; no full repository gate was run in this review.

## Examined units

- `FR-038-AC-3`: Exact byte, node, edge, occurrence, diagnostic and work limits admit a package; each one-over limit returns `incomplete` with that limit kind, the limit and the consumed counter and no package; no limit kind is a depth (FR-038-AC-117).
- `FR-038-AC-67`: An unknown identity refuses `unknown-operation` and an `operator` that differs from the catalogued class refuses `operation-class-mismatch`, each ahead of every other check of the entry at the operation step, so `quire.op.control.case` under the operator `unary` refuses `operation-class-mismatch`; and of two defective body-root nodes the one with the earlier structural `node_id` is reported when both defects are operation refusals. A placement defect of the temporal step (FR-038-AC-102) is reported ahead of every operation refusal, whatever the structural id order. (Amended by IR-549: the earlier text ordered both refusals ahead of blanket temporal/case `unsupported_construct`, which IR-549 removed, and put the placement of these applications inside the operation step.)
- `FR-038-AC-376`: Implemented; evidence: `node_index::permutation_rebuilds_the_same_structural_keys_and_stale_operands_do_not`, `node_index::two_semantic_defects_choose_structural_order_over_graph_order`, and `node_index::mixed_operation_defects_choose_resolved_operand_order` against QSpec FR-322-AC-278..282.  The reader compares rebuilt structural keys by UTF-8 package, path strings before resolved operand keys, proper prefixes first and numeric ordinal, never by graph index; graph order stays producer chosen. A two-defect control with graph order opposite structural order reports the structurally first defect. A valid permutation with rewritten index segments retains every resolved key and reference; stale indexes that retarget an operand cannot silently pass the same structural-key oracle. This replaces retired AC-365 without narrowing its ordering and exact-equality controls.
- `FR-038-AC-377`: Implemented; evidence: `node_index::published_index_segment_refusals_precede_semantic_checks` and `checked_package_v2_canonical_encoding::tc_048_envelope_refuses_indexed_dependency_reference_node`.  Against the selected QSpec node-index vectors NI-01..NI-07, the reader refuses nested node ids, malformed index objects and a dependency-reference node path containing an index as `malformed_wire` at the patched segment. It accepts only string or closed `{"node": i}` segments with I-JSON integer `i` in range; a `DeclaredNodeId` path is all strings. The selected positive dependency-call fixture admits without an inline nested node id.
- `FR-038-AC-378`: Implemented; evidence: `node_index::published_index_segment_refusals_precede_semantic_checks` and `checked_package_v2_temporal::tc_048_out_of_range_operand_precedes_interval_semantics`.  NI-08..NI-10 each refuse `malformed_wire` with no cause at the out-of-range segment's `/node` pointer before any semantic-graph defect. With two out-of-range segments, the first in canonical byte order wins. NI-11 and NI-12 each refuse `malformed_wire` with no cause at the first cycle-closing own-path segment ordered by holding graph index then path position, including a self cycle; a forward reference without a cycle admits. The index checks precede every semantic-graph check, including duplicate ids, and retain their exact RFC 6901 pointers.
- `FR-038-AC-379`: Implemented; evidence: `node_index::imported_index_resolves_in_the_importing_graph` and `checked_package_v2_refinement::refinement_external_abstract_resolves_only_the_directly_selected_package_root`.  Every index segment in a package document resolves against that document's graph, even when the id's `package` names an imported node. NI-14's imported `Int[0, 9]` rebuilds against the importing graph and admits; resolving it against its owner's library graph would create a self-cycle and fails the oracle. An external FR-375 `{package, node}` key resolves against the selected dependency named by the key's own `package`; an unselected dependency or wrong document graph cannot supply the node. A dependency-reference node path remains string-only.
- `FR-038-AC-380`: Implemented; evidence: `node_index::long_index_chain_admits_on_a_small_stack_with_shared_keys` and iterative `CheckedNodeId` comparison, validation, clone and drop.  NI-13's 100000-node left-nested chain, with reader byte, node, edge, occurrence, diagnostic and work bounds sized not to decide the result, reads on a 256 KiB thread stack under serde_json's default 128 recursion limit. Its wire node ids stay at the fixed JSON depth and at most 200 bytes each; 100001 rebuilt structural keys are distinct and each derived key holds its predecessor's key as its final operand through shared `Arc` nodes. Key comparison, validation, clone and drop do not recurse in proportion to the chain. The existing AC-117 stack and AC-214 memory obligations remain.
- `FR-038-AC-381`: Implemented; evidence: `node_index::long_index_chain_admits_on_a_small_stack_with_shared_keys`, `checked_package_v2_flat_wire::tc_048_a_syntax_or_duplicate_defect_is_refused_before_the_parse_limit`, and intake source-span inspection.  The intake source-raw pass and its per-pointer reparse accept the canonical NI-13 package under the default serde_json recursion limit without allocating a full-document JSON value or disabling the limit. A malformed node-id segment deep in the graph is reported at its own pointer, and syntax, duplicate-member and noncanonical-byte precedence from AC-24 and AC-214 remains unchanged. An actually nested body past the closed grammar still receives its existing typed refusal.

## Recheck after large fix delta

At `17e961d90fc76f55df4eefcf0f0c1250aac1ba52`, the changed fixture conversions, structural joins, and FR-038-AC-376..381 trace tags were rechecked. The computed matrix binds each changed criterion; no independent defect was established in this method. The NI-13 implementation regression is recorded in code-review and spec-review/evidence.

The current-head computed Test Matrix reports `tagged` for all six changed criteria: AC-376 has 5 binders, AC-377 has 4, AC-378 has 2, AC-379 has 2, AC-380 has 1, and AC-381 has 1. This is static trace coverage; the NI-13 implementation regression remains the code-review finding.

## Recheck at V1 projection fix head

`1116bfca85b3d1c5bf6d42deebb40d86f704a3ab`: The new V1 structural-order positive/adverse test binds FR-035-AC-23 and exercises a producer/read/read-with-source round trip; the dependency-call control reads back emitted bytes. The fixture helper expands indexed IDs for source-graph joins. Existing FR-038-AC-376..381 bindings remain in place, and no test assertion was weakened by this delta. No independent gap finding was established. The intentionally red FR-330-AC-2 absent indexed request regression remains held pending a QSpec ruling; it is not counted as passing evidence.

## Recheck at indexed unsupported handoff head

`b10edf6ea03a6fba5cd9afa102916c1d1f5a78ec`: The new FR-330-AC-2 positive sibling test proves an actual projected-index change and the exact authenticated unsupported source kind. Existing assertions were not weakened. The separate absent indexed `InvalidInput` regression stays intentionally red pending the QSpec ruling and is not passing evidence. No independent new test gap was established.

## New findings (disposition pass 6)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The FR-035-AC-35 test exercises a present indexed key and a name-only absent key, but never queries an absent derived indexed source key or constructs a raw-id collision whose source and output keys rebuild unequally. Those are the adverse cases that prove frame-safe selection and typed-key creation. | tests/it/complete_v1_contract_package.rs:805; FR-035-AC-35 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open | The frozen head has no absent-derived indexed query or unequal rebuilt-key collision control. |
| FND-002 | still-open | At `4b8280cc`, the absent derived indexed request and malformed constructor controls were added, but no test creates equal raw indexed IDs in two frames whose rebuilt structural keys differ. |
| FND-002 | fixed | At `965fd7b3`, the AC-35 test uses the exact output root raw indexed ID as a source-frame query, proves it resolves to an unequal structural key in the admitted V2 graph, and asserts `Absent` instead of the output closure. |
| FND-003 | still-open | At `ba08b8dd`, no adverse control pairs an absent body target with many indexed deep dependencies and verifies bounded comparison work or preserved typed refusal precedence. |
| FND-003 | fixed | At `d222f8c9`, a focused 4,096-link absent indexed dependency test bounds deep ID comparisons and checks matching and unequal expected sets; the existing eight missing-target cases verify typed refusal precedence. |

## Recheck at source-frame and producer-order head

At frozen `667b6dfbd81d16ea794d8384a07520705436607d`, the changed NI and V1 tests bind FR-330-AC-14..16 and FR-035-AC-31..35. The output/source permutation, malformed-reference, duplicate-key, and empty-output controls retain exact assertions. FND-002 is the remaining AC-35 coverage gap; no further trace or test-intent defect was established.

## Recheck at typed source-key fix head

`4b8280cc8703f110cb76ba21e9a15ba00ee58970`: The new AC-35 test proves an absent derived indexed request can be rebuilt against its owning V2 graph and queried as `Absent`; malformed top-level and nested structural IDs are rejected. FND-002 remains open only for AC-35's unequal-key raw-collision adverse.

## Recheck at raw-collision control head

`965fd7b3f699e7cb7ed23228b7ca9b3271a84ac0`: The new AC-35 test verifies a raw indexed ID names the output root, resolves in the source graph to a different structural key, and returns `Absent`. It therefore exercises the adverse that FND-002 named. The code lead reports its named focused test and format/diff checks passed. FND-002 is fixed; the later operations performance delta remains for this same reviewer to inspect before final readiness.

## Recheck at operations lookup head

`5079cbdb8c927ca315677f9fc61b7886c0a9a3ff`: The new unit controls compare flat and nested lookup results to the ordered index and show zero `CheckedNodeId::cmp` calls for 8,000 unresolved lookups against a flat index. This directly addresses the profiled `check_operands`/`argument_family` hot spot. The code lead reports 88 operation, 17 argument and 23 family focused tests, Clippy and format green. Existing NI-13 and structural-order tests remain; no independent coverage gap was established in this delta.

## Recheck at string-edge annotation head

`1346b77bf38870fa87ee3c981a757c98b3fd2baa`: The two source-frame annotation additions do not change behavior, trace tags, or assertions. Existing FR-035-AC-31..35 and FR-330-AC-14..16 controls remain bound; no additional coverage gap was established.

## New findings (disposition pass 11)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The eight named missing-target tests assert their typed refusal precedence, but none combines an absent target with a long indexed chain and many dependencies to catch the fallback's unmetered deep comparisons. A focused comparison-count adverse should cover that new branch. | crates/quire-contract-model/src/checked_package/v2/structural.rs:384-422; FR-038-AC-3/117 |

## Recheck at eleven-regression fix head

`ba08b8ddb7dcb3b2432f956b5f88488090daefef`: The 11 previously failed tests comprise eight missing-target precedence cases, one output byte-limit sibling case, and two FR-019 public inventory cases. All are still present with their original assertions; the code lead reports all 11 now pass. The structural fallback is the new path requiring an adverse resource control (FND-003).

## Recheck at bounded missing-target fix head

`d222f8c9c8b786128139dd55e78d23865fc548ff`: The new 4,096-link test exercises the formerly unmetered fallback with an absent indexed target and caps deep comparisons. Its positive, missing-key and unequal-key assertions cover the tokenized branch; existing protocol, population and temporal controls cover the exact typed refusal path. FND-003 is fixed. No test assertion was weakened or compatibility path added.

## Recheck at nominal-body indexed-wire test head

`a8172f0598231293bba79d02c6aeea3b49f59b6a`: The replaced test no longer claims that an inline 10,000-level nested node ID is valid V2 input. It now verifies indexed positive encoding and exact malformed-wire refusal for that old invalid form under FR-038-AC-24. NI-13 remains tagged for the genuine 100,000-node FR-038-AC-117/380 stack obligation, so no acceptance evidence was lost and no new test gap was established.

## Recheck at model-lib indexed-fixture repair head

`128dcf0fe01c6b5843b2e11b2bf23111f27c7c04`: The obsolete inline-wire positive was replaced by closed indexed-wire positive and malformed inline adverse assertions; the deep test now exercises the internal Arc-key operations rather than treating an invalid nested V2 wire value as readable. NI-13 remains the independent 100,000-node small-stack reader evidence for FR-038-AC-117/380. Synthetic lower, model-fields, and relationship fixtures now use graph-local indexes and retained structural-key state, matching the admitted reader contract. Trace claims were updated from retired FR-038-AC-364/FR-322-AC-270/271 to FR-038-AC-377/FR-322-AC-278. No acceptance obligation or assertion was removed, and no new test gap was established.
