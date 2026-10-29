---
id: SR-11740
title: "IR-759 Slice A base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir (PR #411); spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-759 (Slice A). PR agent-ix/quire-contract-ir#411,, base origin/main, diff `git diff origin/main...HEAD` (3 files, +76/-58). Base checklist: AC id stability (no add, renumber or removal), status claims against traced tests, factual claims against `crates/quire-contract-model/src/source_obligation.rs` and `tests/it/source_obligation.rs`, cross-requirement agreement with FR-019, four-owner wording, and gates. `quire validate` on the three touched files exits 0 (only module-level warnings). `quire matrix` shows FR-052-AC-12 and AC-13 tagged by `terminal_boundary_distinguishes_empty_complete_partial_and_invalid_count` (tests/it/source_obligation.rs:326) and `terminal_boundary_preserves_typed_stops_and_rejects_mismatched_rows` (:379), and AC-8 tagged by three provisional-port tests (:75,:127,:219). `quire coverage --scope. --strict` reports 49 unbacked and 0 contradicted at head, and the same 49 rows (only line numbers shift) on origin/main measured in a temporary worktree that was then removed.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-052 Outputs | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | The provisional IR-784 port exports only `SourceObligationInput`, `SourceInventoryEnvelope`, `SourceInventoryRow`, typed owner identities and stop causes, and authored assertion/provenance types. FR-019 lists that current public surface exactly. The provisional `SourceInventoryEnvelope` is not a census input: it carries no completeness state, and `SourceInventoryOutcome` is the only completeness carrier. |
| FR-052 status paragraph | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | FR-052-AC-4 and FR-052-AC-8 have provisional-port evidence. FR-052-AC-12 and AC-13 are Implemented with traced tests for the neutral synthetic boundary only. |
| FR-052-AC-12 | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | Implemented by `SourceInventoryOutcome::from_enumerator`; verified by the TC-452 synthetic-boundary tests. A synthetic `SourceInventoryEnumerator` producing zero or multiple `Ok(Some(row))` events yields a neutral `Complete` outcome only after its `Ok(None)` terminal event. |
| FR-052-AC-13 | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | Implemented by `SourceInventoryOutcome` and `SourceInventoryOutcomeError`; verified by the TC-452 synthetic-boundary tests and the `compile_fail` doctests. Mismatched observed count, owner/scope pair, row owner, row source, or stop-cause owner returns the corresponding typed structural error and no outcome. |
| FR-052-AC-14 | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | A default-feature neutral consumer distinguishes a typed Quire criterion identity containing obligation declaration, scoped document and authored criterion ID from `ClauseRef` and QSL `RequirementRecord`. (unchanged) |
| FR-052-AC-15 | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | An explicit source-author binding connects the selected Quire corpus/obligation scope to the selected IR `SourceIdentity`. (unchanged) |
| FR-052-AC-16 | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | With PLAT-1165's real Quire producer, normal exhaustion of the validated selected scope issues `Complete` with every authored criterion and its provenance. (unchanged) |
| TC-452 Execution Status | examined | spec/model/matrix/TC-452-source-obligation-correspondence.md | The separate synthetic neutral-boundary checks for FR-052-AC-12 and AC-13 are implemented and traced (tests/it/source_obligation.rs and the `SourceInventoryOutcome` `compile_fail` doctests); they cannot discharge AC-9 through AC-11. |
| TM row TC-452 | examined | spec/model/matrix/tests.md | TC-452... FR-052-AC-1... FR-052-AC-8... FR-052-AC-16 — Planned/UNRUN: AC-8 is covered by provisional-port tests; AC-12/13 are Implemented and traced for the synthetic neutral terminal and structural boundary only. |
| FR-019 source_obligation inventory | context_only | spec/model/functional/FR-019-rust-library-interface.md | `source_obligation` types... `SourceInventoryCompleteness`, `SourceInventoryEnvelope`... `SourceInventoryOutcome`, `SourceInventoryOutcomeError`...; trait `SourceInventoryEnumerator` |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-052 still says the provisional port "exports only `SourceObligationInput`, `SourceInventoryEnvelope`, `SourceInventoryRow`, typed owner identities and stop causes, and authored assertion/provenance types" and that "FR-019 lists that current public surface exactly" (:79-82), and later that "The provisional port has no `SourceInventoryCompleteness`" (:245). The code exports `SourceInventoryCompleteness` (source_obligation.rs:435), `SourceInventoryEnumerator` (:446), `SourceInventoryOutcomeError` (:461) and `SourceInventoryOutcome` (:550). FR-019:191 lists all four. This PR marks AC-12/13 Implemented on that surface. FR-052 now gives two different current public surfaces: one in these sentences and another in FR-019 and the AC-12/13 status. | spec/model/functional/FR-052-source-obligation-correspondence.md:79-84, 245-247 |
| FND-002 | low | AC-12 and AC-13 criterion text now begins with status prose ("Implemented by...; verified by..."). The criterion is the obligation, not its status. `quire matrix` now shows "Implemented by `SourceInventoryOutcome::from_enumerator`; verified by the TC-..." as the criterion text, which hides the criterion. The same status is already stated in the FR status paragraph (:339-341), TC-452 Execution Status and the tests.md row. | spec/model/functional/FR-052-source-obligation-correspondence.md:333-334 |

## Verdict

CHANGES REQUESTED for FND-001, a stale public-surface claim in the paragraph this PR edits. The rest of the base checks pass:
- AC ids FR-052-AC-1..16 are unchanged: none added, renumbered or removed. The diff touches the rows for AC-2, 3, 6, 7, 12 and 13 only. AC-8, 14, 15 and 16 text is byte-identical.
- No three-owner wording remains in the touched spec files. Four owners (Quire, IR V2, QSL, `quire-code-rs`) appear at:40,:126-128,:263-264 and:269. The code module doc (source_obligation.rs:8, "three-owner census") and the three-variant `SourceInventoryOwner` are pre-existing code outside this diff. They are the IR-795 correction that FR-052 already names.
- "Implemented" on AC-12/13 is backed:
  - Both traced tests cover zero, one and two-row `Complete`, typed `Incomplete` with the preceding row, and each of ObservedCountMismatch, OwnerScopeMismatch, RowOwnerMismatch, RowSourceMismatch and StopOwnerMismatch.
  - The `compile_fail,E0616`/`E0451` doctests at source_obligation.rs:520-548 and the positive accessor doctest at:487-515 cover field privacy. The doctests are cited in prose only and carry no trace tag.
  - No other AC was promoted. AC-1..3, 5..7, 9..11 and 14..16 remain Planned/UNRUN.
- Coverage is unchanged against the measured baseline: 49 unbacked and 0 contradicted on both sides.

## Dispositions

Round 1. Reviewed, diff `git diff..`. Each outcome was checked against that head and against the code in `crates/quire-contract-model/src/source_obligation.rs`.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-052:79-87 now splits the current compiled surface into two parts. Part one is the IR-784 envelope port: Input, Envelope, Row, identities, stop causes and assertion types. Part two is the IR-794 outcome boundary: `SourceInventoryEnumerator`, `SourceInventoryOutcome`, `SourceInventoryOutcomeError` and `SourceInventoryCompleteness`. FR-052 says FR-019 lists the surface exactly, and FR-019:191 agrees. FR-052:249 now reads "The provisional envelope port has no completeness state", which matches `SourceInventoryEnvelope` (source_obligation.rs:377-383). "Neither part exports a census" is true: there is no `SourceObligationCensus` or `build_source_obligation_census` in the module. |
| FND-002 | fixed | The AC-12 and AC-13 rows no longer start with "Implemented by …". `quire matrix` shows the criterion text itself and keeps both tests tagged (tests/it/source_obligation.rs:326,:379). The status is still stated in the FR status paragraph, TC-452 Execution Status and the tests.md row. |
