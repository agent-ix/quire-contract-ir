---
id: SR-12160
title: "IR-852 code review (rust lane) of IR PR #442"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/{selected_population.rs,v2/reaches_population.rs,v2/relationship_end.rs,v2/operations.rs,v2/structural.rs,v2/mod.rs}, crates/quire-contract-model/src/lib.rs, tests/it/checked_package_v2_model_members.rs, tests/it/checked_package_v2_model_members/reaches_population.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-852 code review (rust lane) of IR PR #442

## Summary

Ticket: IR-852. PR quire-contract-ir#442 is the reviewed change (`gh pr view 442` confirms the head). It has three commits on merge, which is the merged spec PR #439. The `/rust-review` lane is folded into this file, and repository conventions (CLAUDE.md, `#[trace]` only, no `/// Trace:` lines) take precedence over the skill's defaults. The diff (`origin/main...HEAD`) touches 13 files, +1237/-96. It adds:

- `v2/reaches_population.rs`: the accessor `CheckedPackageV2::reaches_population`, the closed enum `CheckedReachesPopulationError` and one crate-internal unit test.
- The population check as the last step of `check_relationship_member` (`operations.rs`).
- The `population` id in `application_dependencies` for reaches and navigate (`structural.rs`).
- Sharing: `is_model_population`, `least_type_occurrence`, `population_maximum` and `population_object_type` become `pub(super)` in `selected_population.rs`, and `population_object_type` now takes a node lookup. `CheckedPackageV2::node` and `reaches_application` (with `ReachesLookup`) are added in `relationship_end.rs`, and `relationship_end` uses them.
- The root re-export, the FR-019 row, 13 tests/it tests and the spec status flips.

origin/main has moved on to (IR-861, #441). `git merge-tree --write-tree origin/main HEAD` reports a content conflict in `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`, so the branch must be rebased before merge (see Verdict).

Focused review checks ran with a separate target directory, serialized by `flock /tmp/luna-ir-heavy.lock` at `-j2`, with `QUIRE_SPEC_LANGUAGE_DIR=/home/peter/dev/worktrees/qsl-ir-gates`:

- `cargo fmt --check` is green.
- `cargo test -j2 --locked --workspace reaches_population -- --nocapture`: 13 integration tests and one crate unit test passed.
- The new test at `tests/it/checked_package_v2_model_members/reaches_population.rs:708` still asserts the pinned `REACHES_ADMISSION_WORK = 97` value; a full suite run is the lead's gate.

The full gates (`make -k ci`, twice) belong to the lead. Isolated review mutants and workspace Clippy are recorded after their runs complete.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| crates/quire-contract-model/src/checked_package/v2/reaches_population.rs | examined | pub fn reaches_population(&self, application: &CheckedNodeId) -> Result<Option<CheckedSelectedPopulationSelector>, CheckedReachesPopulationError>; closed enum UnknownNode / NotReachesApplication / UnresolvedPopulation |
| crates/quire-contract-model/src/checked_package/v2/operations.rs:2277-2313 | examined | population check after the result_type check: navigate refusal, node-id/resolve/model_population/Type occurrence/population_maximum filter, receiver key, population_object_type comparison |
| crates/quire-contract-model/src/checked_package/v2/structural.rs:647-688 | examined | application_dependencies adds member_id("population") only for quire.op.model.reaches / quire.op.model.navigate |
| crates/quire-contract-model/src/checked_package/selected_population.rs:210-328 | examined | is_model_population, least_type_occurrence, population_maximum (doc comment on the binding check), population_object_type(population, node: impl Fn(&CheckedNodeId) -> Option<&'a Node>) |
| crates/quire-contract-model/src/checked_package/v2/relationship_end.rs:52-135 | examined | ReachesLookup, CheckedPackageV2::node, reaches_application; relationship_end refactored onto them; pub(super) mod tests, raw_id/admitted pub(in v2) |
| crates/quire-contract-model/src/lib.rs | examined | root pub use adds CheckedReachesPopulationError only |
| tests/it/checked_package_v2_model_members/reaches_population.rs | examined | 13 tests tagged #[trace("TC-048", "FR-038-AC-269..280")] |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The population check derives the receiver end's key a second time. On a reaches application the two ends share one declared type: the `source.type_ref != target.type_ref` check above refuses otherwise. So `end_declaration_key(&receiver.type_ref)` always returns the same result as the `destination_key` computed 50 lines earlier. Its `else` arm (`ineligible(member_at("population"))`) is unreachable. If that arm ever were reached, it would map `Some(Err(_))` to `ineligible`, while the destination derivation maps the same `Err` to `key_refused()` (`invalid_semantic_graph`). This recomputation is also the source of the coder's one equivalent mutant (receiver vs destination type). FR-038 itself names the destination end's type ("the destination end's type, which a reaches application shares with the receiver end"). Keep a clone of `destination_key` before `navigation_type` consumes it, and compare against that. This removes the dead arm, the inconsistent `Err` mapping and the equivalent mutant. | crates/quire-contract-model/src/checked_package/v2/operations.rs:2248-2252,2301-2309 |
| FND-002 | low | The new general node lookup `CheckedPackageV2::node` is placed in `relationship_end.rs`, although `selected_population.rs` also uses it. It has the same body as `parameter_declaration.rs:47-58` (`kind_of` plus the inline lookup) and `comparison_operands.rs:249-257`, which stay as they were. And `select_admitted` still finds the selector's binding by a linear `nodes.iter().find(...)` (`selected_population.rs:217-224`), while the chain beneath it now goes through `package.node`. The PR's aim is one rule in one place. A package-wide helper belongs in `v2/mod.rs` beside `node_index`, and `select_admitted`'s own lookup could use it. This needs no behaviour change. | crates/quire-contract-model/src/checked_package/v2/relationship_end.rs:60-72; crates/quire-contract-model/src/checked_package/selected_population.rs:217-224 |
| FND-003 | low | AC-277's "charges no work" assertion cannot fail. `least_admitting_work(&scenario)` re-reads the canonical bytes from scratch on each call, so calling `first.reaches_population(..)` between the two measurements cannot change either. The real evidence that the accessor charges nothing is its `&self` signature and the `first == clone` equality. The test should say that, or drop the two lines, so it does not suggest a guard that does not exist. | tests/it/checked_package_v2_model_members/reaches_population.rs:699-702 |

## Verdict

**CONDITIONAL** — three low findings. None shows an incorrect result, but the branch also conflicts with current `origin/main` and cannot merge until rebased. What was checked and found clean:

1. **Public surface.** It adds exactly one new public type, `CheckedReachesPopulationError`, and one new public method, `CheckedPackageV2::reaches_population`. The enum is closed: three variants, no `#[non_exhaustive]`, `Clone, Copy, Debug, Eq, Error, PartialEq` and a `thiserror` Display, which matches `CheckedRelationshipEndError`. The root `pub use` adds only that type. The FR-019 row `` `checked_package` (V2 reaches population) | type `CheckedReachesPopulationError` | FR-038 `` is exact, and tc_058 passes. `CheckedSelectedPopulationSelector` is still reachable by module path only, with no root or `quire-contract-ir` re-export. Everything else the PR adds is `pub(super)`, `pub(in crate::checked_package)` or `#[cfg(test)]`. The test helpers widened to `pub(in crate::checked_package::v2)` are under `#[cfg(test)]` and are acceptable.
2. **Accessor semantics.** An unknown id gives `UnknownNode`. A body that is not `term: application` with identity `quire.op.model.reaches` gives `NotReachesApplication`, and navigate is included in that. The key-absent case gives `Ok(None)` and never infers anything. Present-but-not-a-node-id (`null` included), an absent node, a non-population node, or no `Type` occurrence each give `UnresolvedPopulation`. On success the result is the package's identity, `binding` exactly as the member holds it, and the least-ordinal `Type` occurrence. The accessor reads no `max` and charges no work, and every lookup goes through `Option`/`?`. Production code has no `unwrap`, `expect`, `panic!` or unchecked index. The `ReachesLookup` mapping has explicit arms and no wildcard.
3. **Reader check placement and order.** The block sits after the `result_type` check, the last statement before `Ok(None)`. It runs in this order: key absent → `Ok(None)`; navigate → refuse at `member.population` before reading the value; then node-id, resolution, `is_model_population`, `least_type_occurrence`, `population_maximum`; then the object type through FR-053's `population_object_type`, compared with the receiver key's model declaration node id. Every refusal uses the existing `ill_typed`/`operator-ineligible` pair at `member.population`, so no new vocabulary is introduced. A package without the key returns at the same point as before, so its outcome and its charged work are unchanged (the 97-unit pin passes).
4. **Dependency join.** `member_id("population")` is added only when the identity is reaches or navigate. A value that is not a node id is skipped and left to the operation stage, as for `declaration`. The `BTreeSet` keeps the join unique and digest-ascending.
5. **Sharing.** `population_object_type` taking `impl Fn(&CheckedNodeId) -> Option<&'a CheckedSemanticNodeV2>` is the right seam. Admission resolves through the operation-stage `Graph` index, before any `CheckedPackageV2` exists, and the selector resolves through `package.node`. So there is one derivation over two lookups, and the old linear `find` over all nodes goes away. `relationship_end` now uses `reaches_application`, with the same refusal mapping as before.
6. **The non-binding branch of `population_maximum` (kept with a doc comment).** This is acceptable, not dead code to delete. The function is FR-053's: `select_population` calls it, and it can also be reached on a package a crate-internal mutation changed. Its contract (reject a `max` member that is not a binding) is pinned by the existing unit test `maximum_requires_the_selected_integer_literal_binding`, whose `{"name": "max", "term": "literal"}` row would fail without it. Deleting it would widen FR-053's reader to accept an unadmitted shape. The doc comment accurately says it cannot fail on a reader-admitted body.
7. **Work and complexity.** The population check charges no work. That matches FR-038, which pins work only for packages without the key and requires no charge for the check. The cost per reaches application is O(1) index lookups plus one scan of the named population node's `members` (`population_maximum`). That is bounded by the node's admitted size, so the total is at most applications × population size, within the reader's byte and node limits. Under `bounded()` (1 MiB, 10,000 nodes) that is negligible. No new allocation grows with input beyond one `CheckedNodeId` clone.
8. **Tests follow repository conventions.** Every new test carries `#[trace("TC-048", "FR-038-AC-...")]`, with no fully qualified spelling and no `/// Trace:` doc line (the head commit removed them). Expected values come from the fixture builders and `structural_key`, never from the accessor.

Merge readiness: not mergeable at this head because `git merge-tree --write-tree origin/main HEAD` reports a content conflict in FR-038. Re-apply the AC-269..280 status flips and the AC-279/reader-check amendment on top of IR-861's Verification-cell rewrite, then run the gates. The gap-analysis artifact SR-12161 also records two missing test families.

## Acceptance criteria examined

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-038-AC-269 | examined | Implemented (IR-852). When `reaches_population` is called with the id of an admitted `quire.op.model.reaches` application whose `operation.member` carries `population` naming a `bounded_domain` / `model_population` node of the receiver end'... |
| FR-038-AC-270 | examined | Implemented (IR-852). When `reaches_population` is called with an admitted reaches application whose `operation.member` has no `population` key it returns `Ok(None)`, also in a package that holds a `model_population` node of the receiver en... |
| FR-038-AC-271 | examined | Implemented (IR-852). `reaches_population` over an id absent from the graph refuses `UnknownNode`; over an admitted `quire.op.model.navigate` application, an application of `quire.op.integer.add`, the relationship declaration node, a `model... |
| FR-038-AC-272 | examined | Implemented (IR-852). A reaches application whose `operation.member.population` is not a node id (a number, an object and `null`), or is a node id with no node in the graph and listed in `dependencies`, is refused by the reader with `ill_ty... |
| FR-038-AC-273 | examined | Implemented (IR-852). A reaches application whose `operation.member.population` names a node that is not a `bounded_domain` / `model_population` node, or a `model_population` node with no `Type` occurrence, is refused by the reader with `il... |
| FR-038-AC-274 | examined | Implemented (IR-852). A reaches application whose `operation.member.population` names a `model_population` node over an object type other than the declared type of the selected receiver end, including a subtype of that type, is refused by t... |
| FR-038-AC-275 | examined | Implemented (IR-852). A `quire.op.model.navigate` application whose `operation.member` carries a `population` key that names a node id its `dependencies` list, whatever the node is (an absent node, an integer-range node or a valid populatio... |
| FR-038-AC-276 | examined | Implemented (IR-852). A reaches or navigate application whose `operation.member.population` is a node id is admitted by the dependency join only when its `dependencies` hold that id in the unique, digest-ascending list; a package that omits... |
| FR-038-AC-277 | examined | Implemented (IR-852). When `reaches_population` is called twice on an admitted reaches application and on a clone of the admitted package, the results are equal, the package equals its pre-call self and no work is charged; every package of ... |
| FR-038-AC-278 | examined | Implemented (IR-852). An external Rust API fixture calls `reaches_population` with `&CheckedNodeId`, binds the result as `Result<Option<CheckedSelectedPopulationSelector>, CheckedReachesPopulationError>` and exhaustively matches `CheckedRea... |
| FR-038-AC-279 | examined | Implemented (IR-852). A reaches application whose `operation.member.population` names a `model_population` node over the receiver end's declared type with a missing `max`, two `max` members, a `max` that is not an integer `literal`, a negat... |
| FR-038-AC-280 | examined | Implemented (IR-852). A reaches application with an invalid `population` (a node id absent from the graph, listed in its `dependencies`) and an earlier defect in `check_relationship_member` (an operand 1 that does not conform to the receive... |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in the reviewed PR; see the after excerpt in the Linear disposition. |
| FND-002 | fixed | Verified in the reviewed PR; see the after excerpt in the Linear disposition. |
| FND-003 | fixed | Verified in the reviewed PR; see the after excerpt in the Linear disposition. |
