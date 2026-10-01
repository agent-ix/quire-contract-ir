---
id: SR-635
title: "gap analysis of PR 235 (remove digest staleness checks from the checked-package reader)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@b50cd28fd9debf730c4e3f72d9fb2abe6c17e7ce; FR-038-AC-2, FR-038-AC-4, FR-038-AC-10, FR-038-AC-20, FR-038-AC-27, FR-038-AC-37, TC-048 and their tagged tests"
review_set: base
---
# SR-635: gap analysis of PR 235

## Summary

Ticket: IR-457. Planless gap analysis (Plan completion: not assessed), scoped to the requirements and tests the diff touches. `quire coverage --scope . --strict --json` was run on origin/main 8371caa and on head b50cd28 and compared key by key: `unbacked_rows` 17 and 17 with the identical row list (FR-036, FR-036-AC-1 to AC-5, FR-037, FR-037-AC-6, FR-039, FR-039-AC-1 to AC-4, FR-019-AC-5, TC-045, TC-055, TC-058), `status_lies` 0 and 0, every `minted_targets` entry `backed: true` on both sides (only line numbers shift), and the only `obligations` differences are the three edited statements (FR-038-AC-2, AC-27, AC-37). No AC lost its backing.

Each deleted test was checked against what it covered:

- `refused(&base, CheckedPackageEvidence::new())` -> `stale_dependency` at `/lock/sources/0`, `stale_catalog`, `byte_evidence`: removed behaviour (lock vs evidence).
- `evidence_for`'s artifact loop, `locator`, `locked_artifacts`: helpers for removed behaviour.
- "A version that differs from the entry's" (`revision-mismatch`): removed behaviour (`dependency_references.rs:111`).
- `raw_only` block: asserted that a domain package digest attested only as a raw artifact refuses `missing_import`/`missing-selection` at `/lock/model_selections/0/digest`. The raw-artifact half is gone with the API, but the refusal it asserted is kept behaviour (FND-001).

Kept checks were mutated one by one (see SR-634): seven of eight go red; the `admit_document` digest-domain check does not (SR-634 FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-038-AC-2's edited clause "a domain package selection ... one with no supplied document as `missing_import` with cause `missing-selection`" has no AC-2-tagged test any more. The deleted `raw_only` block was the only reader-level assertion of that refusal and pointer (`/lock/model_selections/0/digest`). The behaviour is still guarded at unit level by `tc_048_a_selection_admits_only_the_document_it_names` (tagged FR-038-AC-27, asserts `SelectionRefusal` member `digest`), so a regression in `admit_document` still fails a test; what is lost is the end-to-end `read` assertion that AC-2 names. Replace the block with a no-document read (evidence without the selected document) rather than deleting it. | tests/it/checked_package_v2_reader.rs:1201 |

## Verdict

Coverage is preserved row for row (17 unbacked before and after, identical). One low gap: AC-2's rewritten no-document clause lost its only reader-level test. Mergeable; fold FND-001 into the fix round for SR-634 FND-001.
