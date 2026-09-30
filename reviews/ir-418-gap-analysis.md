---
id: SR-604
title: "gap analysis of PR 218 (ceremony sweep, code half)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@f21a1948936b1cdf5282ad5678bfac21748cdc8c; spec/contract/FR-028, FR-031, FR-034, FR-037, STD-003, spec/interface/FR-019, FR-039, spec/contract-test-matrix.md against Cargo.toml, crates/quire-contract-model/src/output_mapping.rs, tests/it/{cycle_free_model,kani_replay,output_mapping}.rs"
review_set: subset
---
# SR-604: gap analysis of PR 218

## Summary

Ticket: IR-418. Planless gap analysis. Plan completion: not assessed. The PR changes no spec file. This analysis checks that the code still agrees with the merged spec after the deletions, and that each AC whose backing test was deleted or re-tagged is still honestly backed. `make spec` at f21a194: validate passes (208/208 grammar-clean). `coverage --strict` exits 2 with 22 unbacked rows and 0 contradicted, the same set as main (FR-036, FR-037, FR-039, FR-344 and their ACs, FR-019-AC-5, TC-045, TC-055, TC-058, TC-222). None of them is new.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-028's dependency rules still require quire-observation, quire-protocol, tl-syntax, tl-mltl and quire-mltl to feed quire-contract-ir. This PR removes those deps and the assertions that checked them, so the spec now states a graph the code does not have | spec/contract/FR-028-separate-cycle-free-contract-model.md:43-50 |
| FND-002 | medium | The new `FR-028-AC-3` tag sits on a TC-042 Kani replay test. AC-3 is verified by TC-041, and the code the test drives (`replay_with_native_runtime`) is what FR-037-AC-6 and FR-039 say the root crate must not have. AC-3's first clause (QSL's production graph has no quire-contract-ir) is asserted by no test | tests/it/kani_replay.rs:241 |
| FND-003 | low | The matrix FR-028 row still cites `tests/it/cycle_free_model.rs:211`, but the file is now 82 lines. It also still counts AC-3 as implemented through tests this PR deleted | spec/contract-test-matrix.md:31 |

## Finding Detail

- FND-001: The deps really were unused (nothing in `src/` imports them), so removing them is right. The spec has to follow. Fix, in this PR or in a spec PR merged first: rewrite the FR-028 "Dependency and admission rules" graph to the real edges (`quire-contract-model -> quire-spec-language`, `quire-contract-model + quire-spec-language -> quire-contract-ir`). Drop the observation, protocol, TL and mltl lines.
- FND-002: This tag is now the only `FR-028-AC-3` trace in the repo. Without it, coverage would report a 23rd unbacked row. It keeps the count at main's 22 by moving the AC onto a test that checks something else. Remove `"FR-028-AC-3"` from the tc_042 trace. In `tc_041_model_dependency_graph_is_cycle_free_and_owner_free`, which already has the workspace metadata, walk `resolve.nodes` from the quire-spec-language node over normal-kind deps. Assert that no `quire-contract-ir` package is reachable, then tag that test FR-028-AC-3. Reword AC-3's last sentence ("The bounded-Kani replay path is the production root-package consumer of that API"), because FR-037 retires that path.
- FND-003: Update the row once FND-002 lands. Name the test instead of a line number.

## Scope

- `FR-028-AC-1`, examined: Cargo metadata for every production feature combination is acyclic and contains no owner or TL dependency reachable from `quire-contract-model`. Still backed by tc_041_model_dependency_graph_is_cycle_free_and_owner_free. The dropped `kind == null` check only widens the forbidden-name check to every dependency kind.
- `FR-028-AC-3`, examined: QSL builds against `quire-contract-model` with no `quire-contract-ir` package in its production graph, while a locked composition build imports both the root package and the real QSL owner API without a Cargo cycle. (FND-002)
- `FR-028-AC-4`, examined: Default, all-feature and minimum-version builds prove that no optional, dev or historical dependency leaks into the production graph. The removed `fault-injection`-only features assertion is redundant, because every model dep is non-optional and name-checked.
- `FR-034-AC-1`, examined: Equal admitted requests, mapper candidates, and generator identities produce byte-identical target output, records, raw digests, and package identities. Backed.
- `FR-034-AC-4`, examined: Mutating source/profile/generator owner/record/limit/target-byte identity inputs changes or invalidates package identity, while path/time/locale/display/observer changes do not. Backed, including the generator-owner mutation.
- `FR-034-AC-5`, examined: Structural observer acceptance, refusal, absence, and observer owner remain downstream references and cannot establish native truth or mapping preservation. Backed.
- `STD-003` rows `invalid_generator` and `invalid_observer`, examined: the declared owner is empty, unbounded, or outside visible ASCII. Only the empty case is tested (SR-603 FND-002).
- `FR-031-AC-1`, `FR-031-AC-5`, context_only: the 17 removed TC-221/FR-031-AC-4 tags backed neither. The tests are kept and now trace nothing. Per FR-037-AC-6 their subject (`src/kani/witness.rs`, `replay.rs`) is planned for removal, which is pre-existing work and not this PR's.
- `FR-019` output_mapping items row, examined: no `ObserverResultDigest`. It matches the code.

## Verdict

Not mergeable until FND-001 is resolved: the spec states a dependency graph the code no longer has. Fix FND-002 in the same round. The code direction is correct in every case. What is missing is the spec and trace follow-through.
