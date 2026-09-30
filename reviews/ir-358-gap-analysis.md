---
id: SR-616
title: "gap analysis of PR 225 (IR drops QSL)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@dbc07f9fe556f25fec0e3b3851628dd8ca459385; spec/contract/FR-028, spec/interface/FR-039, spec/contract-test-matrix.md against Cargo.toml, tests/it/cycle_free_model.rs, tests/it/kani_replay.rs"
review_set: subset
---
# SR-616: gap analysis of PR 225

## Summary

Ticket: IR-358. This is a planless gap analysis, so plan completion was not assessed. `make spec`: validate passes (214/214 grammar-clean). `coverage --strict` exits 2 with the known 22 unbacked rows and 0 contradicted, so the PR adds no unbacked row. The row count hides one real gap: FR-028-AC-3 keeps a clause that nothing tests anywhere any more.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-028-AC-3's first clause ("QSL builds against quire-contract-model with no quire-contract-ir package in its production graph") is now asserted by no test in any repository. The PR deleted `assert_spec_language_production_graph_excludes`. agent-ix/quire-integration main holds only a `hello_is_non_empty` test, and its only open PR (#2) is a spec PR. Even so, FR-028 Status and both matrix rows still count AC-3 as implemented and backed by TC-041. The AC also says the composition build's verification "lives in agent-ix/quire-integration", which is not true today | spec/contract/FR-028-separate-cycle-free-contract-model.md:63 |

## Finding Detail

- FND-001: Split AC-3. Keep the in-repo clause (the root manifest names no QSL crate) as FR-028-AC-3, which the rewritten tc_041 tests and which is not vacuous: adding any `quire-spec-language` or `qsl-*` dependency fails it (see SR-615 FND-007 for `quire-exact`). Take the QSL-graph clause and the composition build out of this repo's AC, because they are obligations on QSL's graph and on quire-integration. Record them as planned with a quire-integration ticket, and stop counting them as implemented in the FR-028 Status and in the matrix FR-028 and TC-041 rows.

## Scope

- `FR-028-AC-1`, examined: Cargo metadata for every production feature combination is acyclic and contains no owner or TL dependency reachable from `quire-contract-model`. Still backed by `tc_041_model_dependency_graph_is_cycle_free_and_owner_free`.
- `FR-028-AC-3`, examined: QSL builds against `quire-contract-model` with no `quire-contract-ir` package in its production graph, while the root package's own manifest names no `quire-spec-language` or `qsl-*` dependency in any kind. (FND-001)
- `FR-028-AC-4`, examined: Default, all-feature and minimum-version builds prove that no optional, dev or historical dependency leaks into the production graph. Unchanged by this PR.
- `TC-042` `tc_042_counterexample_replays_through_native_runtime_execute`, examined: deleted with `replay_with_native_runtime`. It was the mis-tagged FR-028-AC-3 trace that SR-604 FND-002 flagged, so removing it is correct.
- `TC-048` tests in `checked_package_v2_parameters.rs`, examined: renamed only, with trace ids unchanged.
- `FR-039-AC-2`, context_only: No public item names a `quire_spec_language::runtime` type. This is now true by construction.
