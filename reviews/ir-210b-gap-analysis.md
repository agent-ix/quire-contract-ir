---
id: SR-12066
title: "IR-210 slice B gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@406e8d735d0a51efaadc9e9364ac291bdf784ac1; tests/it/identity.rs, tests/it/integration.rs, tests/it/conformance.rs, spec/core/stakeholder/StR-001-authoritative-contract.md, spec/core/stakeholder/StR-003-reviewable-assurance.md, spec/core/functional/FR-011-package-identity.md, spec/core/functional/FR-012-anchors-clauses-dependencies.md, spec/conformance/functional/FR-018-conformance-corpus.md, spec/conformance/functional/FR-020-json-conformance-interface.md, spec/model/functional/FR-019-rust-library-interface.md, spec/core/non-functional/NFR-002-portability-independence.md, spec/core/non-functional/NFR-003-diagnostic-integrity.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/StR-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-011
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: references
---

## Summary

Ticket: IR-210 (slice B), PR quire-contract-ir#430. This is a planless gap analysis, so plan completion was not assessed. Everything was measured with quire 0.36.2 (engine 0.50.2), on origin/main 1bd6749 in a detached scratch worktree and on head 406e8d7.

`quire coverage --scope . --strict --format json` (exit 1 on both, from the same 43 pre-existing unbacked rows):

- **Matrix reference rows.** Base has 43 unbacked and head has 43 unbacked, and the unbacked set is identical. Contradicted statuses are 0 on both.
- **Minted targets backed.** Base is 508/565 and head is 506/565. Two targets flip from backed to unbacked: **StR-001-VC-2** and **StR-003-VC-1**. The StR-001 VC group goes from 2/2 to 1/2, and the StR-003 VC group from 1/1 to 0/1.
- **Coverage matrix (`coverage_matrix` and `quire matrix --format tsv`).** All 532 rows keep their status: 469 tagged, 54 untagged, 7 method-without-symbol, 1 status and 1 tagged-by-ignored-test on both. The only binder-count changes are FR-011-AC-1 (3 -> 1) and FR-018-AC-1 (3 -> 2). VC rows are not in the coverage matrix.

`quire trace --file` over the three files, base vs head, comparing (id, symbol) pairs:

- Base has 64 pairs and head has 58.
- Six pairs were lost:
  - FR-011-AC-1 on `untrusted_json`
  - FR-011-AC-1 on the integration test
  - FR-018-AC-1 on `unsafe_paths`
  - StR-001-VC-1 and StR-001-VC-2 on the identity conformance test
  - StR-003-VC-1 on `mismatch`
- No pair was gained.
- Base binds 36 distinct non-TC ids in these files and head binds 33. The three missing at head are the three StR VC ids.
- On the seven touched tests, all 26 pairs are `rust-trace-attribute`.
- The tests that still use the fully-qualified `#[ix_trace_rs::trace]` stay `rust-doc-comment-id`, `rust-comment-id` or `rust-test-name-id`: eight in all, seven in `conformance.rs` and one in `identity.rs`.
- STD-001 and NFR-003-AC-1 were `evidence_near_miss` citations on base and never credited a row.

`quire trace --id` on head:

- StR-001-VC-1 is still bound by `tests/it/expression.rs` `tc_016_closed_types_expressions_scopes_and_dependencies_conform`.
- StR-001-VC-2 has no binder.
- StR-003-VC-1 has no binder.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| StR-001-VC-1 | examined | Every executable clause has a stable requirement revision, named anchor, typed expression, dependency set, and source location. |
| StR-001-VC-2 | examined | Downstream artifacts can cite the same package, requirement, clause, and expression identities. |
| StR-003-VC-1 | examined | Malformed, ill-typed, undefined, unsupported, and orphaned inputs yield stable source-located diagnostic codes. |
| FR-011-AC-1 | examined | A package round trip through the non-canonical JSON value representation preserves namespace, schema version, requirement ID, requirement revision, and source-document identity/revision with structural equality. |
| FR-018-AC-1 | examined | Running the corpus directory yields one matching row per input with non-empty trace targets, and the union of observed tokens equals the published inventory ...; a corpus missing a token's only fixture, a fixture with an unknown operation prefix or a missing expectation, and oversize, over-count and over-budget corpora fail before any row is written. |
| FR-011-AC-2 | examined | Incrementing one requirement revision changes its clause and dependency identities without changing unrelated requirement identities. |
| FR-011-AC-3 | examined | Empty or malformed ... identities; a zero schema major; zero source or requirement revisions and a revision above 9007199254740992 ...; duplicate requirement or clause identifiers; non-increasing revision advances; and cross-package references fail with their registered structured diagnostic codes. Schema minor zero is valid. |
| FR-012-AC-1 | examined | A floating executable clause is rejected at its source span with a stable diagnostic code. |
| FR-012-AC-2 | examined | The recursive reference body returns each exposed structural dependency identity exactly once in deterministic order. |
| FR-012-AC-3 | examined | Malformed, cross-package, stale-revision, orphaned-requirement, and orphaned-clause references produce distinct STD-001 diagnostic codes and the narrowest available source span. |
| FR-012-AC-4 | examined | Every executable clause kind accepts only its declared anchor kinds ...; informational clauses reject any anchor ...; malformed anchor identifiers use `invalid_identifier`. |
| FR-012-AC-6 | examined | Source spans reject zero line/column positions, decreasing byte offsets or positions, a byte offset above 9007199254740992 ..., and endpoints from different source-document identities/revisions using `invalid_source_span` ... |
| NFR-002-AC-3 | examined | Public JSON field names, identity kinds, anchors, clause kinds, dependency kinds, and diagnostic codes contain no Rust, GUMBO, AADL, HAMR, solver, runtime, pointer-width, or operating-system-path vocabulary. |
| FR-018-AC-2 | examined | Mutation fixtures independently alter schema validity, diagnostic code/path/order/span/obligation, canonical byte, digest, dependency, and coverage row/reason; each produces the exact mismatch result without message parsing. |
| FR-018-AC-3 | examined | Raw package probes pin exact and one-past wire depth, and quoted delimiters do not count toward depth. |
| FR-019-AC-2 | examined | The complete negative corpus executes ... through `catch_unwind`; exact-at-limit and one-past-limit type depth, semantic node, and semantic collection cases return the specified result with no public panic, partial result, or message parsing. |
| FR-020-AC-2 | examined | Process fixtures pin exit 1 with all six mismatch kinds in fixed order and exit 2 for each of the six closed operational codes ... |
| NFR-003-AC-1 | examined | Every declared invalid-input class returns a stable diagnostic code and no public panic. |
| NFR-003 Scope | context_only | "the six runner operational codes of FR-020 ... are outside the declared classes ... NFR-003's statement concerns rejected semantic conditions; TC-018 asserts them." |
| spec/core/matrix/tests.md StR-003 row | context_only | StR-003 traced to TC-015, TC-016, TC-017, TC-018, TC-058 |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | StR-003-VC-1 loses its only credit (StR-003 VC 1/1 -> 0/1). The drop from the mismatch test is honest, but no test that asserts the VC was given the tag, and the PR body states that no at-risk row loses its credit | tests/it/conformance.rs:221-224 |

## Verdict

CHANGES REQUESTED on one medium finding, together with SR-12065 FND-001 (StR-001-VC-2, the same credit loss, where the drop itself was wrong). This is a repeat of the SR-12020 FND-002 lesson from #426. The `strict` gate cannot see it: its 43 unbacked rows are matrix reference rows, and stakeholder VC rows are counted only in `minted_targets` and `groups`.

FND-001 detail.

- **The drop itself is right.** `tc_018_all_mismatch_kinds_and_exit_classes_are_stable` asserts runner operational codes and exit classes, which NFR-003's Scope places outside the semantic, source-located diagnostics. It asserts no source-located diagnostic for a malformed, ill-typed, undefined, unsupported or orphaned input.
- **The row still needs an owner.** On main, that doc line was the VC's only binder, and the VC's verification column names TC-016 and TC-018 ("Negative corpus and diagnostic review").
- **Tests that do assert it.**
  - `tc_018_the_corpus_runs_deterministically_and_every_fixture_matches` runs the whole negative corpus and requires every fixture's exact diagnostics (code, path, span) to `match`.
  - The TC-454 tests in `tests/it/nfr003_invalid_input.rs` show that every declared class returns a registered code.
- **Ways to fix it in this PR.** Add StR-003-VC-1 to the corpus test's `/// Tracing:` doc line, which is how that fully-qualified test is credited today, or move that test to `#[trace]`. Either is a one-line change. The other option is to record the loss as accepted, with a follow-up ticket.

PR-body accuracy (folded into this finding, not separate):

- The body says "of the 32 at-risk rows, none loses its credit". Measured, two minted rows lose their only credit.
- It says "the five touched tests move ... to `rust-trace-attribute`". There are seven.
- It describes the fully-qualified tests as four. There are seven in `conformance.rs` and one more in `identity.rs`.

Confirmed clean:

- **FR-011-AC-1: 3 -> 1 backers.** It stays backed by the identity conformance test, which does assert the round trip. Both drops are honest, because neither `untrusted_json` nor the integration test decodes a valid package.
- **FR-018-AC-1: 3 -> 2 backers.** It stays backed by the corpus test (positive half) and the mismatch test (negative half). The unsafe-paths drop is honest.
- **Ids whose only credit was these tests' legacy doc lines.** FR-011-AC-2, FR-012-AC-1, FR-012-AC-2, NFR-002-AC-3, FR-018-AC-2, FR-018-AC-3 and FR-019-AC-2 now have `rust-trace-attribute` on the same symbol and are still backed.
- **Every other touched id** keeps its binder count. No `quire matrix` status changes.
- **STD-001 and NFR-003-AC-1.** These drops remove only uncredited `evidence_near_miss` citations. NFR-003-AC-1 stays backed by the two TC-454 tests.

Pre-existing gaps (notes, not introduced here):

- The eight fully-qualified `#[ix_trace_rs::trace]` tests are still credited through legacy doc-line forms only. That is the remainder of IR-210.
- FR-018-AC-2's "independently" leg has a single, combined-mutation backer. See SR-12065 FND-003.

Gates on head 406e8d7: `cargo fmt --all -- --check` is clean, and `cargo test --locked --test it -- identity:: integration:: conformance::` passes 21 of 21. That is the 16 tests in the three files plus 5 `application_node_identity::` tests that the filter also matches.

## Dispositions

Round 1, head 34f35dc. Re-measured with quire 0.36.2 (engine 0.50.2):

- `quire coverage --scope . --strict --format json`: 43 unbacked / 0 contradicted. The unbacked set is identical to origin/main.
- Minted targets: 508/565 backed, the same as origin/main 1bd6749, with no target differing from main. StR-001 VC is 2/2 and StR-003 VC is 1/1.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | StR-003-VC-1 is now credited to `tc_018_the_corpus_runs_deterministically_and_every_fixture_matches` (see below). |

Detail on the FND-001 fix:

- **The binding is right.** The test runs the whole published corpus and requires every row to `match`. The runner compares each invalid fixture's diagnostics structurally, by code, path and span. The test also asserts that the observed token union equals the inventory, so every STD-001 diagnostic has a failing fixture. That set includes malformed (`malformed_reference`), ill-typed (`ill_typed_expression`), undefined (`potentially_undefined`), unsupported (`unsupported_schema_version`) and orphaned (`orphaned_*_reference`) inputs. 36 expectations carry a `span`.
- **How quire credits it.** quire binds it as `rust-doc-comment-id` through the `/// Tracing:` and bare-id lines. The test keeps the fully-qualified `#[ix_trace_rs::trace]`, and quire does not read that as `rust-trace-attribute`. The id in the attribute is still checked by the macro at compile time.
- **The `Tracing:` line is not stale.** On this fully-qualified test it is the form quire actually credits, as on the other seven fully-qualified tests. Moving those tests to `#[trace]` is the remaining IR-210 work.

PR body:

- **Corrected.** A "Review round 1" section now says that VC rows are counted and that the first push dropped two only-backers. It also reports 508/565.
- **Still stale (note, not a finding).** The older Evidence lines still say "the five touched tests" and "of the 32 at-risk rows, none loses its credit". The round-1 section also says the FR-018-AC-2 follow-up is "to be filed by the lead", but IR-839 already exists. Code and coverage are unaffected.
