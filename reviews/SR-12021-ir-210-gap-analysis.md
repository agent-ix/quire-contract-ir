---
id: SR-12021
title: "IR-210 slice 1 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@95e3df16fa6a19143273d7a4ac2f703f0d8374ae; tests/it/canonicalization.rs, spec/model/functional/FR-016-canonicalization-digests.md, spec/model/functional/FR-017-version-orphan-coverage.md, spec/core/non-functional/NFR-003-diagnostic-integrity.md, spec/core/stakeholder/StR-002-portable-interchange.md, spec/model/matrix/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-017
    type: references
  - target: ix://agent-ix/quire-contract-ir/NFR-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: references
---

## Summary

Ticket: IR-210 (slice 1), PR quire-contract-ir#426. This is a planless gap analysis; plan completion was not assessed. The computed matrix was measured with quire 0.36.2 (engine 0.50.2) on origin/main 299a126 (a detached scratch worktree) and on head 95e3df1.

`quire coverage --scope . --strict`:

- Base: 507/564 rows backed, 43 unbacked, 0 contradicted.
- Head: 500/564 rows backed, 45 unbacked, 0 contradicted.

Seven rows lose backing:

| File | Base | Head | Rows |
| --- | --- | --- | --- |
| NFR-003 | 2/3 | 0/3 | AC-1, AC-2 |
| StR-002 | 2/2 | 0/2 | VC-1, VC-2 |
| FR-016 | 9/9 | 7/9 | AC-1, AC-3 |
| FR-017 | 2/6 | 1/6 | AC-2 |

Only the NFR-003 pair enters the `--strict` unbacked list. On base, all seven had only `rust-doc-comment-id` / `legacy` credit from the removed doc lines; `quire trace --id FR-016-AC-1` on base shows exactly that. The tag-honesty defects in the changed lines are in SR-12020. This file records the coverage gaps and the TC-017 trace results.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-017-AC-5 | examined | Every repeated artifact ID produces exactly one `orphaned`/`duplicate_artifact` artifact row and contributes no coverage, including when its first occurrence would otherwise be valid. |
| FR-017-AC-3 | examined | With package requirements and traces authored out of key order, the single-package report orders its current requirement rows by requirement ID and artifact rows by artifact ID, both in Unicode scalar value order. |
| FR-017-AC-4 | examined | For the same current requirement, a valid deep trace yields `deep` coverage regardless of whether a valid shallow trace appears before or after it. |
| FR-017-AC-6 | examined | For every artifact ID occurring more than once, exactly one `duplicate_artifact_trace` diagnostic is emitted per occurrence after the first; all coverage diagnostics retain authored trace order, including when duplicate and reference errors are interleaved. |
| NFR-003-AC-1 | examined | Every declared invalid-input class returns a stable diagnostic code and no public panic. |
| NFR-003-AC-2 | examined | No malformed, unsupported, stale, undefined, or orphaned input contributes successful canonical identity or covered status. |
| NFR-003-AC-3 | examined | Given artifact traces whose source and target-reference spans differ, each cross-package, missing-requirement, and stale-revision diagnostic retains the exact target-reference span of its trace; each `duplicate_artifact_trace` diagnostic retains the exact source span of the later duplicate occurrence. |
| FR-016 | context_only | FR-016 matrix row: "✅ implemented: AC-1 through AC-4 including the caller byte budget on all five closed object kinds ..." |
| TC-017 | examined | Canonical bytes, digests, version preflight, and orphan classes conform |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Hand-written matrix status now contradicts the honest tags. The FR-016 row is marked "✅ implemented: AC-1 through AC-4". The TC-017 row says "FR-016-AC-1 through AC-4 in `tests/it/canonicalization.rs`". The FR-017 row says "AC-1 and AC-2 have existing TC-017 evidence". After this PR, no test is tagged FR-016-AC-1, FR-016-AC-3 or FR-017-AC-2, and none could be. No allocation-failure harness exists for FR-016-AC-3, and no over-limit, Shallow-row or no-digest assertion exists for FR-017-AC-2. `--strict` reports 0 contradicted statuses because the prose is not machine-checked. The slice brief keeps spec edits out of this PR, so the lead should either correct those rows here or record the follow-up on IR-210. | spec/model/matrix/tests.md:15-16, spec/model/matrix/tests.md:29 |
| FND-002 | low | FR-017-AC-5 is asserted but not tagged. `tc_017_coverage_classes_orphans_diagnostics_and_sorting_conform` passes 7 traces with 6 distinct ids and asserts `artifacts().len() == 6`, so `art_duplicate` has exactly one row. It asserts that row is `Orphaned` / `DuplicateArtifact`, and that REQ_b, whose only traces are the duplicate pair with a valid first occurrence, is `Uncovered`. That is every leg of AC-5. The matrix still says AC-5 "await[s] ... duplicate-collapse assertions". Neither the old doc lines nor this PR claimed it, so this is an omission, not a regression. The same test only partly asserts FR-017-AC-3 (requirements are authored in key order), FR-017-AC-4 (shallow before deep only), FR-017-AC-6 (duplicates are not interleaved with reference errors) and NFR-003-AC-3 (only the duplicate span is checked), so leaving those untagged is right. | tests/it/canonicalization.rs:699-807, spec/model/functional/FR-017-version-orphan-coverage.md:117, spec/model/matrix/tests.md:16 |

## Verdict

CHANGES REQUESTED (one medium, one low).

**NFR-003-AC-1 / AC-2 (strict count 43 -> 45).** Removing these claims is right; the credit should not be restored.

- The only base credit was `rust-doc-comment-id` / `legacy` from `/// NFR-003-AC-1.` on the preflight test and `/// NFR-003-AC-2.` on the coverage test.
- AC-1 quantifies over every declared invalid-input class; the preflight test covers one.
- AC-2 lists malformed, unsupported, stale, undefined and orphaned input. The tests show unsupported versions yield no canonical identity and duplicates yield no coverage. They do not show that stale input fails to cover, because REQ_a is covered by a valid deep trace anyway, and they have no undefined input.
- Both ACs verify through TC-019, which is unbacked in `--strict` (an `Analysis` TC that mints no symbol). The +2 is the true state, previously hidden by the over-claim. It belongs to TC-019's own work, not to a restored tag.

**`quire trace --id TC-017 --json`.**

- Base: the four canonicalization tests are credited `rust-test-name-id` / `legacy` only.
- Head: each of the four is `rust-trace-attribute` / `canonical`.
- FR-016-AC-2, FR-016-AC-4 (two tests) and FR-017-AC-1 resolve the same way on head.
- The PR's evidence items 1 and 3 reproduce.

**Rename mutation.** Renaming `tc_017_canonical_bytes_...` to `tc_999_...` was re-run on a scratch copy:

- TC-017 still credits the renamed test through the attribute, so this file's coverage no longer depends on the function name. That meets this slice's share of the IR-210 "Done means".
- The name form still mints a TC-999 credit, because `rust-test-name-id` is still declared in spec-artifacts-process `trace_tags.legacy`. That is the ecosystem half of IR-210 and correctly out of scope here, so it is not a defect of this PR.

**Out of diff, for later slices.** `crates/quire-contract-model/src/canonical.rs` `tests::tc_017_a_limit_and_an_encoder_fault_...` carries `#[ix_trace_rs::trace("TC-017", "FR-016-AC-6")]`, yet base `quire trace --id TC-017` credits it only by `rust-test-name-id`. The path-qualified attribute form appears not to be read, so that test still depends on its name.

**Gate.** `cargo test --locked --workspace --test it canonicalization` on head: 4 passed, 0 failed (reviewer re-run, scratch target dir since deleted). PR CI runs only the CLA checks, so this re-run is the test evidence.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head 1eb5872: `spec/model/matrix/tests.md` was corrected, prose only.<br>FR-016 is now 🚧: AC-4 is tagged, and AC-1, AC-2 and AC-3 "have no test tagged and await direct assertions".<br>FR-017 is now "AC-1 and AC-5 tagged; AC-2 awaits over-limit, shallow-row and no-digest assertions".<br>TC-017 now lists FR-016-AC-4, FR-017-AC-1 and FR-017-AC-5 as tagged, and FR-016-AC-1/2/3, FR-017-AC-2..4, FR-017-AC-6 and NFR-003-AC-1..3 as planned (IR-836 tracks NFR-003).<br>Each claim matches the tags on head, and no other spec file names those ids. |
| FND-002 | fixed | Round 1, head 1eb5872: `tc_017_coverage_classes_orphans_diagnostics_and_sorting_conform` now carries `/// Trace: TC-017, FR-017-AC-5` / `#[trace("TC-017", "FR-017-AC-5")]`. The body was re-read against the criterion and is unchanged.<br>Exactly one row: 7 traces over 6 distinct ids, with `artifacts().len() == 6`.<br>`orphaned`/`duplicate_artifact`: the `art_duplicate` row asserts `CoverageClass::Orphaned` and `OrphanReason::DuplicateArtifact`.<br>No coverage, even though the first occurrence is valid: REQ_b's only traces are the pair, a valid shallow reference to the current REQ_b revision 2, and REQ_b is asserted `Uncovered`. |
