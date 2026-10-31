---
id: SR-2026103033
title: "Code and Rust review for IR-1186 PR 736"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/shared.rs; crates/quire-contract-model/src/checked_package/v2/; tests/conformance_qspec/node_index.rs; tests/it/"
review_set: subset
---

## Summary

Code review now finds one high-severity resource-bound regression at head `17e961d90fc76f55df4eefcf0f0c1250aac1ba52`. The indexed-wire joins and fixture migration were inspected against the same PR scope.

## Verdict

**PASS after disposition round 3** — FND-002 is fixed by the selected-bound early return and shallow structural tokens; final aggregate gates remain outside this reviewer pass.

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

## Recheck after fix head

At `9a989415ce479eb2b2f7bfffb5f8fbaa922cf11c`, the only change since the reviewed head is a documented `#[ix_trace_rs::string_edge]` on `node_index::check`. Its closed-name comparisons do not accept an unmatched member or discriminator as an alternate valid form; the marker applies only to that function. `make string-edge` passed on this head. The original review had no real findings, so there is no disposition row or Linear disposition comment under the reviewer workflow.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | In `validate_selected_type_bounds`, collecting every deep rebuilt key into a `BTreeMap` performs at least one increasingly deep `CheckedNodeId::cmp` for each insertion, even with no selected range. On NI-13 this is quadratic unmetered work before the operation step; the 100,000-node admission control can stall or exhaust resources. | crates/quire-contract-model/src/checked_package/v2/structural.rs:683 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | still-open | Current fix head retains the unmetered deep-key map; a bounded lookup or selective scan and a focused NI-13 control are needed. |
| FND-002 | fixed | The selected-bound join now returns before indexing any graph key when there are no selected integer ranges, and otherwise interns structural descriptors rather than sorting deep keys. The structural dependency join uses those tokens for resolved graph-key equality. |
| FND-003 | fixed | At `4b8280cc`, `from_structural` recursively rejects malformed package/path/ordinal values, and `from_v2_request` resolves absent indexed keys through the admitted source graph; a focused test covers the absent derived query. |
| FND-004 | still-open | At `ba08b8dd`, one absent body target sends every dependency through deep-key sorting and comparison without charging that work. A bounded/tokenized fallback is needed. |
| FND-004 | fixed | At `d222f8c9`, absent listed dependencies join a graph-wide structural interner and rank table once; matching and ordering compare shallow tokens/ranks rather than sorting deep keys. Expected absent roots are retained for stable pointer caching. |

## Recheck at integrated fix head

`77a153d55aed863766db1c2e614daea93ccb551c`: The selected-bound join now returns before indexing any graph key when there are no selected integer ranges, and otherwise interns structural descriptors rather than sorting deep keys. The structural dependency join uses those tokens for resolved graph-key equality. The fix changes only `structural.rs`; no compatibility path, hash, pin, or test-only product bypass was introduced. The unrelated broad-filter failures remain for the code lead to classify through the final gate.

## Recheck at V1 projection fix head

`1116bfca85b3d1c5bf6d42deebb40d86f704a3ab`: Reviewed the delta from `77a153d5` and the current PR diff. V1 producer and reader now order graph nodes by rebuilt structural keys, remap actual `CheckedNodeId` path indexes into the projected graph, preserve FR-375 external package keys, and compare projected raw bodies with remapped originating bodies during source handoff. The source map, accounting ID fields, admitted terms, operation views, and dependency-call readback were inspected. No independent new code or Rust finding was established. The intentionally red absent indexed request case under FR-330-AC-2 remains a stated specification hold and is outside this disposition. In the same held family, `read_with_source` still looks up an unsupported record's projected `node_id` directly in the original source; an unsupported indexed request with reordered represented operands can miss its source kind. This caveat needs resolution with FR-330-AC-2 before qualification. No compatibility decoder, hash/pin identity, or test-only production bypass appeared in this delta.

## Recheck at indexed unsupported handoff head

`b10edf6ea03a6fba5cd9afa102916c1d1f5a78ec`: The `read_with_source` unsupported-record lookup now reverses each projected graph index through the already authenticated `source_positions` map before calling `source.node_kind`. The map is built from structural keys of every represented node, so a reordered indexed operand resolves to the originating source position. The new `v1_indexed_unsupported_request_authenticates_its_source_kind` control includes an unsupported application and a lowered operand, asserts the V1 key actually differs from the source key, and checks the exact `Expression(Binary)` source kind after handoff. No new defect was established in this narrow delta. The intentionally red absent indexed `InvalidInput` case remains held under FR-330-AC-2.

## New findings (disposition pass 6)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The public `ContractSourceKeyV1::from_structural` accepts any no-index `CheckedNodeId`, including empty package/path and an out-of-domain ordinal, without requiring an admitted owning graph or resolved V1 table entry. No public constructor rebuilds an absent derived indexed request against its source graph; callers can only manually synthesize its resolved nested key. Thus the typed source-frame key promised by FR-035-AC-35 is not reliably constructible for that case and can be constructed from malformed input. | crates/quire-contract-model/src/checked_package/v2/contract_package.rs:731; FR-035-AC-35 |

## Recheck at source-frame and producer-order head

At frozen `667b6dfbd81d16ea794d8384a07520705436607d`, reviewed the delta from `b10edf6e` and the full current PR against QSpec FR-330-AC-14..16 and FR-322-AC-95/278..282. Source references use the closed frame, output indexes stay output-local, source table indexes stay table-local, duplicate structural keys are refused, and producer-chosen graph order is accepted. The rank-order treap compares resolved operand positions and handles separately allocated equal keys. No compatibility decoder, content hash/pin, assertion weakening, or test-only production bypass was found. FND-003 remains open pending a same-PR fix.

## Recheck at typed source-key fix head

`4b8280cc8703f110cb76ba21e9a15ba00ee58970`: The constructor now checks every nested structural operand and exposes an owning-graph indexed request constructor. An out-of-range index and malformed nested operand are rejected; the new absent derived query returns `Absent`. No new code defect was established in this narrow fix.

## Recheck at operations lookup head

`5079cbdb8c927ca315677f9fc61b7886c0a9a3ff`: Reviewed only the `operations.rs` delta from `965fd7b3`. `FlatWireIndex` hashes flat wire IDs by package, ordinal, path length and tagged name/index segments, using value equality; its nested-key fallback delegates to the original ordered index when nested keys exist. A nested key cannot equal any flat key, so skipping the ordered lookup when the index is entirely flat preserves `None`. The ordered index still determines temporal placement and first structural refusal; the fast path is confined to operand-family and same-type lookups. It changes no meter charge, refusal type, or wire identity. This is a local hash lookup, not a content hash or pin. No new code/Rust finding was established. Final aggregate gates remain outside this reviewer pass.

## Recheck at string-edge annotation head

`1346b77bf38870fa87ee3c981a757c98b3fd2baa`: The only production delta from committed review head `b6636e99` is `#[ix_trace_rs::string_edge]` on `source_ref_paths` and `validate_shapes`. Both functions enumerate the fixed FR-330 closed-member vocabulary; `source_ref_paths` branches on the fixed `nodes` collection name, and `validate_shapes` branches on that same fixed member while calling the typed shape helpers. The annotations cover those functions, with no changed branch, acceptance path, pointer, or refusal. The code lead reports the focused string-edge scanner, format and diff checks passed. No new code/Rust finding was established.

## New findings (disposition pass 11)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | high | In `matches_dependencies`, one absent target selects a fallback that sorts and compares all resolved dependencies as deep `CheckedNodeId` values. On an indexed 100000-node chain with many dependency edges and one absent target, each comparison can revisit a long shared operand spine; the structural stage can do quadratic work outside the reader's work meter. This reintroduces the resource-bound failure class previously fixed for the normal path. | crates/quire-contract-model/src/checked_package/v2/structural.rs:384-422; FR-038-AC-3/117 |

## Recheck at eleven-regression fix head

`ba08b8ddb7dcb3b2432f956b5f88488090daefef`: The eight missing-target precedence failures stemmed from structural dependency matching refusing an absent body target at `/dependencies`; the fallback now reconstructs such keys so their owning body joins can issue their typed refusals. The byte-limit sibling test changes only the output profile identity, leaving the admitted source at its exact input byte limit. FR-019's public inventory adds the actually exported `ContractSourceKeyV1`, matching both model and crate-root exports and the two public-inventory failures. The code lead reports all 11 named focused regressions pass; this reviewer also ran one exact missing-target test, which passed. FND-004 remains open for the fallback's unmetered deep comparisons.

## Recheck at bounded missing-target fix head

`d222f8c9c8b786128139dd55e78d23865fc548ff`: The dependency matcher now builds one graph-wide token/rank table containing present keys and resolved absent listed IDs. It compares tokens and ranks for all actual dependencies, retaining expected absent roots so the interner's pointer cache cannot alias a later allocation. `resolve_unlisted_dependency` rejects mixed inline/index paths; an indexed absent ID resolves against the same graph as all other local IDs. The 4,096-link adverse test exercises a present deep chain plus one absent indexed target, bounds `CheckedNodeId::cmp` calls below `64 * LENGTH`, and checks both matching and unequal expected keys. Existing missing-target tests retain the owning body's typed-refusal assertions; the code lead reports protocol 154/154, population 14/14, temporal 50/50, model Clippy, format and diff checks passing. FND-004 is fixed; no new defect was established in this delta.

## Recheck at nominal-body indexed-wire test head

`a8172f0598231293bba79d02c6aeea3b49f59b6a`: The only delta from gate head `b906ffad` is the test in `identity.rs`. Its former 10,000-level inline nested node ID is outside the closed indexed V2 wire grammar and could recursively drop on an assertion panic. The replacement admits and canonically encodes a flat `{"node":0}` path segment, checks exact `MalformedWire` and `/terms/0/dimension_node_id` for the old inline spelling, and drops the deeply nested test value with `quire_canonical::drop_value`. No product code, acceptance branch, or compatibility reader changed. NI-13 remains the 100,000-node small-stack evidence for FR-038-AC-117/380; this test now binds FR-038-AC-24. The code lead reports the named test, model Clippy, format and diff checks pass. No new finding was established.
