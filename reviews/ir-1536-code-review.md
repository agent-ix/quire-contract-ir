---
id: SR-202611081512300014
title: "IR-1536 code review (rust lane): remove SccClosureIndex complexity"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@7fddc1a269424c77b24f58cf9b23e69d91b83b3a; crates/quire-contract-model/src/checked_package/v2/lower.rs, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs, crates/quire-contract-model/src/checked_package/v2/contract_package.rs, crates/quire-contract-model/src/checked_package/v2/retained_index.rs, crates/quire-contract-model/src/checked_package/v2/monitor_plan/layout_context.rs, tests/read_storage_alloc.rs, tests/it/support/read_allocation_proof.rs, tests/it/checked_package_v2_whole_read_accounting.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-050
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-472
    type: references
---

## Summary

Ticket: IR-1536, PR quire-contract-ir#1021, head 7fddc1a, scoped to `git diff origin/main...HEAD` (8 files, +185/-280). The rust-review lane is folded into this file. The PR deletes `SccClosureIndex` and the per-call whole-graph node index and source-map grouping, takes each request's reached closure from `lower_one`'s own walk, projects lowered packages through sparse `BTreeMap<usize, usize>` position maps, and narrows the adverse-shape whole-read comparisons in `read_storage_alloc.rs` from `ReadViewScope::Full` to `ReadViewScope::Core`.

Behaviour preservation of the closure deletion was traced by hand. `assign_components_charged` builds SCCs over exactly `fact.successors`, and the old `SccClosureIndex` condensation edges came from the same `fact.successors`. Every member of the start's SCC is reachable from the start, so the union of members over condensation-reachable components equals the set reachable from `start` by successors. The memoised and uncacheable paths both returned that same closure. `lower_one` already walked those successors before calling `reachable`, with the same early returns. In production, `facts`, `nodes` and `kinds` all have graph length, so the walk's `continue` guards never skip a node. The result is the same set, in the same order. The `include_source_operands` bounds check and the `from_lowering` / handoff position maps are equivalent rewrites. Focused run: `cargo test -p quire-contract-model --lib lower::` passed 12 of 12.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| lower.rs `lower_one` | examined | "The completed walk visited exactly the closure reachable from `start`." |
| lower.rs `NodeSourceMaps` | examined | "A node's source-map entries found through the package's retained source index" |
| lower.rs `assign_components_charged` | examined | "Assign SCCs from the exact lowering edges." |
| retained_index.rs `node_sources` | examined | "The positions of every source-map entry of `id` among `entries`, in index order." |
| contract_package.rs `from_lowering`, `source_frame_wire`, `include_source_operands`, handoff | examined | "type ProjectedPositions = BTreeMap<usize, usize>;" |
| ceiling_tests.rs | examined | "one_request_compares_ids_by_its_closure_not_by_graph_size" |
| read_storage_alloc.rs `whole_read_adverse` | examined | "on these shapes it is quadratic in node count (a chain node's lowering carries its whole dependency chain)." |
| FR-038-AC-308 | context_only | "lowering performs no further body grammar validation, reuses closure reachability per strongly connected component" |
| FR-038-AC-309 | context_only | "graph/source indexes and component closures are built once" |
| FR-038-AC-511 | context_only | "Whole-read accounting ... non-refusal-diagnostic bytes requested during the call exceed the read-storage delta the budget admitted by at most 4096 bytes" |
| FR-038-AC-512 | context_only | "Over the TC-048 corpus, every budget ... returns the same outcome as the unlimited budget: an equal admitted package (equal graph, terms, models, selected documents, source map, diagnostics and lowering of every node)" |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `LoweringNodeFacts::component` and `assign_components_charged` are now dead in production: no production code reads `component`, but admission still runs a full SCC pass at `mod.rs:4352` and charges the caller's work and read-storage budget for six graph-sized vectors (incoming counts, reversed adjacency, seen, postorder, two stacks). The PR's stated reason to keep it ("removing them would change read-storage and work accounting") is not a constraint: AC-511 measures actual allocations against charges, and no test pins absolute charge values. Delete the field, `assign_components_charged`, the `#[cfg(test)] assign_components` helper and the `component` asserts in ceiling_tests.rs. | crates/quire-contract-model/src/checked_package/v2/lower.rs:52-134, crates/quire-contract-model/src/checked_package/v2/mod.rs:4336, crates/quire-contract-model/src/checked_package/v2/mod.rs:4352 |
| FND-002 | medium | `lowering_components_exclude_admission_only_extension_edges` (traced to FR-038-AC-308) lost its only behavioural assertion. The deleted `closure.reachable(...)` checks proved that a request for node 0 does not inherit node 1. What remains asserts only `facts[0].component != facts[1].component`, which nothing in lowering reads any more. As written, the test is trace evidence for code that no longer affects lowering. Replace it with a `lower()`-level assertion that node 0's lowered dependencies exclude node 1, or delete it together with FND-001. | crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs:437-456 |
| FND-003 | low | The `whole_read_adverse` doc comment and the commit say per-node lowering is quadratic "on these shapes". That holds for the two reference chains (chain_100000 and the over-1 MiB chain of 4000). It does not hold for `string_one_mebibyte`, `diagnostics_10000`, `direct_dependencies_64` or `selected_models_two_512k`, whose graphs are small, so `Full` is cheap there and was dropped without the stated reason. The narrowing itself is spec-consistent (see Verdict). Either keep `Full` for the small-graph shapes, or reword the comment to state the real rule: AC-512's lowering equality is scoped to the TC-048 corpus. | tests/read_storage_alloc.rs:808-820, tests/read_storage_alloc.rs:867-895 |
| FND-004 | low | Test names and docs still describe the deleted mechanism. `independently_lowered_records_share_one_closure_and_equal_single_requests` no longer shares a closure. The `shared_closure_body_walks_scale_with_nodes_not_requests` doc says "The same shared closure is parsed once per call", which now holds only for body validation. | crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs:238-243, crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs:316-319 |

## Verdict

REQUEST CHANGES on this method, driven by the spec contradiction recorded in the gap-analysis artifact (SR-202611081512300015, FND-001) plus FND-001 and FND-002 here. All three are cheap to fix in this PR.

The core product change is correct and behaviour-preserving. Deleting `SccClosureIndex` yields the same closure set, because `lower_one`'s walk over `fact.successors` already visited exactly the SCC-condensation closure. The retained-index lookups (`node_position`, `PositionIndex::node_sources` plus a sort that restores source-map order) are equivalent to the per-call maps they replace. The sparse projection maps are equivalent rewrites. The new `one_request_compares_ids_by_its_closure_not_by_graph_size` test is a real complexity oracle and would fail on the old per-call `BTreeMap` index.

The `ReadViewScope::Core` narrowing for adverse shapes is legitimate. AC-511 is an allocation-accounting criterion, and AC-512's "lowering of every node" equality is explicitly scoped to the TC-048 corpus, which keeps `Full`. `Core` still asserts complete retained-outcome equality (`assert_eq!` on the package and on the budgeted result). Per-node lowering equality between two reads through the same lowering code was never an oracle for lowering correctness, so no real oracle is lost. Only FND-003's comment and small-shape nit remain.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | 2a8afff2 removes `LoweringNodeFacts::component`, `assign_components_charged`, the cfg(test) `assign_components` helper, the admission call at mod.rs and the `component` asserts; clippy --all-targets -D warnings clean. |
| FND-002 | fixed | Replaced by `a_request_does_not_inherit_the_nodes_that_reach_it`, a `lower()`-level oracle: the shared node's dependencies exclude both leaves and each leaf's dependencies are exactly the shared node. |
| FND-003 | fixed | `whole_read_adverse` comment reworded to the real rule: AC-511 measures accounting on these shapes and AC-512's per-node lowering equality is scoped to the TC-048 corpus. |
| FND-004 | fixed | Test renamed `independently_lowered_records_revalidate_no_body_and_equal_single_requests`; the body-walk test doc now describes body revalidation, not a shared closure. |
