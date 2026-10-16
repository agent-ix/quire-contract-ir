---
id: SR-11816
title: "IR-759 Slice C source-obligation census code review (with Rust review)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #419 diff origin/main...32fd1e8: crates/quire-contract-model/src/source_obligation.rs, tests/it/source_obligation.rs, spec/model/functional/FR-019-rust-library-interface.md, spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md"
review_set: subset
---

# IR-759 Slice C source-obligation census code review (with Rust review)

## Summary

Ticket: IR-759. PR: quire-contract-ir#419 at 32fd1e8fa7d3739bc7fcde854af0bc1fafbda95f. This is the independent code review, with the rust-review lane folded in. It is not the coder's own pre-handoff review `reviews/26-10-16-source-obligation-census-code-review.md` (SR-11815). Authority: merged FR-052 "Census construction rules", AC-15 and AC-17 to AC-23, TC-452, and ADR-0056 for the retirement of AC-8. The branch `code/ir-759-neutral-census-v2` was not used as an oracle.

Focused gates, run with `CARGO_TARGET_DIR` set to the reviewed worktree's own `target/` and with `-j2` under `flock /tmp/luna-ir-heavy.lock`:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy -p quire-contract-model --all-targets -D warnings` and `cargo clippy -p quire-contract-ir --test it -D warnings`: clean.
- `cargo test --test it -- source_obligation public_interface`: 19 passed. This includes `tc_058_model_root_exports_exactly_the_documented_public_items`, which compares the `pub` items of `source_obligation.rs` with FR-019's row.
- `cargo test -p quire-contract-model --doc source_obligation`: 10 passed, including the `compile_fail` probes.
- 25 source mutants were run in a throwaway worktree, which has since been removed. 18 were killed, 1 is equivalent and 6 survived on test gaps (FND-001 to FND-005).
- The head merges cleanly with origin/main 8b6e5d0.

## Verdict

**NOT MERGEABLE.** There is one medium finding (FND-001), a fixture gap with a small fix. The other findings are low.

What holds:

- **Exhaustive, typed.** Every `match` over `SourceInventoryIdentity` and `SourceInventoryOwner` lists all variants. No wildcard arm maps to a value. Dispositions, missing-mapping reasons and rejection reasons are closed enums, and identities are typed newtypes. No string is used as a discriminant. The census is immutable and has only borrowed accessors (`source`, `criterion_source_binding`, `inventory_rows`, `correspondence_rows`, `rejected_assertions`), as the construction rules list them. The public interface test passes against FR-019:191.
- **Envelope port deleted.** `SourceInventoryEnvelope`, its constructor and accessors are gone. There is no re-export, shim or second constructor. `SourceObligationInput::new` takes four `Option<SourceInventoryOutcome>`. A repo-wide grep finds no `SourceInventoryEnvelope` anywhere.
- **Refusal precedence.** The order is missing binding, foreign binding, Quire outcome, conflicting binding (least criterion via `.min()`), IR V2, QSL, `quire-code-rs`. Within a position it is missing, scope, source, incomplete. Mutants killed: foreign binding moved after Quire, IR V2 checked before the conflicting binding, incomplete before source, source before scope, and each binding check skipped or defaulted.
- **Assertion pipeline.** The relation-conflict check precedes provenance (the swap mutant is killed). Foreign is checked before dangling. The step-4 relation group rejects differing provenance with `ConflictingProvenance` and otherwise yields one row (the mutant that ignores it is killed).
- **Endpoint resolution.** An occurrence under a conflicting node, an occurrence with no node row, and a function with no module row do not resolve. All three mutants are killed. Orphan and conflicting rows stay in the census.
- **Dispositions and order.** The disposition rules match the spec for each kind. QSL rows are always `ZeroMappings(NoAuthoredMapping)`. Inventory rows sort by `(owner, identity)`, and since every row of an outcome has one source, that key is total. `dedup` collapses exact repeats. Removing the sort or the dedup is caught. The accepted rows are already in `BTreeMap` key order, so removing `accepted.sort()` is an equivalent mutant.
- **Bounds and panics.** The census is O(n log n), built from sort, dedup and `BTree` sets and maps. There is no quadratic scan, no recursion, and no `unwrap`, `expect` or indexing in production code.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-17 requires "the AC-2 fixture shape", which includes "explicit many-to-many assertions". The fixture has only one-to-many assertions: one criterion maps to three occurrences, each occurrence maps to at most one function, and each function to one occurrence. A mutant that drops `production_function` from the step-4 relation key survives. It collapses distinct (criterion, occurrence, function) relations into one row. Add one occurrence mapped to both functions, or one function reached from two occurrences, and extend the hand-written expectation. | tests/it/source_obligation.rs:651-798; crates/quire-contract-model/src/source_obligation.rs:1318-1330 |
| FND-002 | low | Step 1, the collapse of exact duplicate assertions, is never observable. Removing `group.dedup()` survives, because the one duplicated assertion in the suite is accepted and step 4 collapses it anyway. A duplicated assertion that is rejected (for example a dangling one) would appear twice in `rejected_assertions()` with no test to notice. | crates/quire-contract-model/src/source_obligation.rs:1267; tests/it/source_obligation.rs:844-885 |
| FND-003 | low | No test covers the provenance-only branch of `ForeignSource`, where the assertion source matches and the provenance source differs. Removing `\|\| assertion.provenance.source != input.source` survives. | crates/quire-contract-model/src/source_obligation.rs:1289 |
| FND-004 | low | Two rejection-order rules are untested. Swapping `DanglingCriterion` and `DanglingOccurrence` in step 3 survives. Removing `rejected.sort_by(...)` also survives, because no test interleaves a step-3 rejection with a step-4 `ConflictingProvenance` rejection across assertion ids. | crates/quire-contract-model/src/source_obligation.rs:1288-1300, 1357-1359 |
| FND-005 | low | The `expected` scope that `InventoryScopeMismatch` reports is asserted only for the Quire position. A wildcard mutant `_ => outcome.scope` for the non-Quire positions survives: the redundant `outcome.owner != owner` check still refuses, but the error then reports `expected == observed`. A more natural wildcard default (`_ => QuireSelectedAuthoredCriteria`) is killed by many tests. | crates/quire-contract-model/src/source_obligation.rs:1080-1096; tests/it/source_obligation.rs:1082-1147 |
| FND-006 | low | Wrong trace tag. `assertion_conflicts_and_relation_dedup_are_deterministic` carries `FR-052-AC-19`, but it only exercises AC-22 (dedup of equal provenance and `ConflictingProvenance` across ids). AC-19's foreign-before-dangling and relation-before-provenance precedence is in `assertion_reason_precedence_rejects_...`, which is tagged correctly. | tests/it/source_obligation.rs:841-844 |
| FND-007 | low | Test and code hygiene. `with_positions` takes a `base` parameter it discards (`let _ = base;`). `binding_refusals_are_typed` binds `let mut input` and overwrites it immediately. `provisional_input_has_no_result_or_completeness_fields` keeps the name of the deleted provisional port. `build_source_obligation_census` is one 320-line function holding four separable phases: refusals, endpoint sets, the assertion pipeline and dispositions. | tests/it/source_obligation.rs:279, 804-813, 887-898; crates/quire-contract-model/src/source_obligation.rs:1132-1455 |

## Examined scope

```yaml
scope:
- {id: FR-052-AC-15, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "An explicit source-author binding connects the selected Quire corpus/obligation scope to the selected IR `SourceIdentity`. A missing, foreign or conflicting binding refuses a complete census with distinct typed causes; a matching label or revision alone is not treated as proof of equal source bytes across Quire, IR V2 and QSL. Foreign and dangling AC-to-occurrence/function assertions retain their separate typed dispositions."}
- {id: FR-052-AC-17, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "Given four synthetic `Complete` outcomes and a valid binding, where the outcomes hold the AC-2 fixture shape plus a QSL outcome with at least one `RequirementRecord` row and one `PendingObligation` row, `build_source_obligation_census` returns a census equal to a hand-written expected census"}
- {id: FR-052-AC-18, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "Given synthetic outcomes, two faults in adjacent positions of the order (missing or foreign binding, Quire outcome, conflicting binding, IR V2, QSL, `quire-code-rs`) yield only the error for the earlier position."}
- {id: FR-052-AC-19, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "an assertion that is both foreign and dangling is rejected only with `ForeignSource`, and one repeated assertion ID whose members differ in both relation and provenance is rejected only with `ConflictingRelation`."}
- {id: FR-052-AC-20, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "Given the FR-052-AC-17 inputs, permuting the assertion order and the row order of each outcome yields a census equal to the unpermuted one."}
- {id: FR-052-AC-21, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "an outcome in one owner position that is both in the wrong position and incomplete yields `InventoryScopeMismatch`, and one that names the wrong source and is incomplete yields `InventorySourceMismatch`."}
- {id: FR-052-AC-22, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "two assertion IDs stating one relation with equal provenance yield exactly one accepted row, and two assertion IDs stating one relation with differing provenance are both rejected with `ConflictingProvenance` and yield no accepted row."}
- {id: FR-052-AC-23, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "an assertion whose occurrence lies under a `ConflictingIdentity` node, an assertion whose occurrence has no `SemanticNode` row, and an assertion whose production function has no `ProductionModule` row are rejected with `DanglingOccurrence`, `DanglingOccurrence` and `DanglingProductionSymbol` respectively"}
- {id: FR-052-AC-2, role: context_only, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "A typed fixture containing two Quire-authored acceptance criteria, including one with no mapping, three semantic occurrences, one empty module, two production functions and explicit many-to-many assertions"}
- {id: FR-019, role: examined, path: spec/model/functional/FR-019-rust-library-interface.md, excerpt: "`source_obligation` public-items row (line 191)"}
```

## Dispositions

Round 1, checked against fix commit aa0daad on top of 32fd1e8. All 25 mutants were re-run in a throwaway worktree, which has since been removed: 24 killed and 1 equivalent (`accepted.sort()`). The six round-0 survivors (M1, M4b, M8b, M11, M12, M14) are now killed. fmt, both clippy runs (-D warnings), the 20 source_obligation and public_interface tests and the 10 doctests pass. The census builder split into `checked_census_positions`, `order_inventory_rows`, `census_endpoints`, `process_census_assertions`, `order_census_results` and `census_dispositions` moves code without changing behaviour: the bodies are textually the same apart from `input.source` becoming the threaded `source`. No new findings.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The AC-17 fixture adds `mk("d", 0, Some(second))`, so occurrence 0 maps to both functions and `verify` is reached from two occurrences. The hand-written census gains the row (AC-1, occ0, verify). The M11 mutant is killed. |
| FND-002 | fixed | A new test, `rejected_assertions_collapse_duplicates_and_sort_with_reason_precedence`, repeats a dangling assertion "a" twice and expects one rejection. M4b is killed. |
| FND-003 | fixed | The same test has assertion "z" with a matching source and a foreign provenance source, and expects `ForeignSource`. M12 is killed. |
| FND-004 | fixed | The same test has "b", which dangles in both criterion and occurrence, and expects `DanglingCriterion`. Step-4 `ConflictingProvenance` rejections ("c" and "d") arrive after the step-3 rejection "z", and the test expects sorted order. M14 and M8b are killed. |
| FND-005 | fixed | `owner_scope_and_source_errors_precede_incompleteness` now asserts `InventoryScopeMismatch` with the exact `expected` scope for the IR V2, QSL and `quire-code-rs` positions. M1 is killed. |
| FND-006 | fixed | `assertion_conflicts_and_relation_dedup_are_deterministic` is now tagged TC-452 and FR-052-AC-22 only. |
| FND-007 | fixed | The unused `base` parameter is removed, `let mut input` is replaced by `let base`, the test is renamed `census_input_has_no_result_or_completeness_fields`, and the builder is split into six named stages without any behaviour change. |
