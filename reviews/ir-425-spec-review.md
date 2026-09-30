---
id: SR-609
title: "spec review of PR 224 (residual tracking sweep)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; reviews/**, plan/**, spec/reviews/**, CLAUDE.md, AGENTS.md (git diff origin/main...HEAD, 81 files)"
review_set: subset
---
# SR-609: spec review of PR 224

## Summary

Ticket: IR-425. This review checks that each edit in PR 224 removes only tracking ceremony (recorded SHAs, reviewed-at revisions, "fixed <sha>" cells, pin sentences, status snapshots) and keeps what the sentence means. The owner's rule applies: tracking ceremony is deleted, including in review and plan records. Canonical content-identity digests, Cargo.toml and Cargo.lock, the unsafe audit and the corpus stay. The whole diff was read (1988 lines, 81 files), not a sample. Most edits are clean. Five problems remain: before/after measurements that lost their anchors, one disposition history now out of order, one dangling "the commit above" with a tag link still called immutable, one dangling "module roots above", and a few broken sentences.

`quire validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md'` exits 0. `make spec` reports 212/212 docs grammar-clean, then 22 unbacked rows and 0 contradicted statuses. These are the known 22 rows: FR-036, FR-037, FR-039 and FR-344 with their ACs, FR-019-AC-5, TC-045, TC-055, TC-058 and TC-222.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Before/after measurements lost the revisions that told them apart, so "at both", "21 ... and 39" and "37 -> 45" no longer say which run is the base and which is the PR head | reviews/2026-09-28-ir-22-spec-review-evidence-analysis.md:24 |
| FND-002 | medium | ir-89 dispositions now list FND-005 as fixed in Round 1, but FND-005 was found in round 2 and fixed in round 3 | reviews/ir-89-code-review.md:93 |
| FND-003 | medium | REV-005 FND-099 still cites "the commit above", which was deleted, and the replacement tag link is still called immutable | reviews/REV-005-canonicalization-implementation.md:31 |
| FND-004 | low | REV-014 still says "the same exact CLI/engine and three module roots above", but the checkpoint section no longer names them | reviews/REV-014-pr51-campaign-recovery.md:98 |
| FND-005 | low | Deleting the SHAs left broken sentences: a missing article, a verbless bullet, and "reviewed another" | reviews/2026-09-14-native-temporal-code-rust.md:19 |

## Finding Detail

- FND-001: reviews/2026-09-28-ir-22-spec-review-evidence-analysis.md:24 now reads "reports 21 unbacked rows and 0 contradicted statuses, and 39 unbacked and 0 contradicted", then "`make spec` exits 2 at both revisions". The revisions it compared (origin/main and the PR head) are gone, so the reader cannot tell which count is the baseline, and that is the whole finding. The same defect appears at reviews/ir-313-spec-review-base.md:31 ("exits 1 at both commits"), reviews/ir-314-spec-review-base.md:51 ("exits 1 at both; unbacked rows go 37 -> 45"), and reviews/ir-408-gap-analysis.md:13 ("was run, and on an export of origin/main"). Fix: name the roles instead of the SHAs, for example "21 unbacked at origin/main and 39 at the PR head", "at both origin/main and the PR head", and "was run at the PR head and on an export of origin/main".
- FND-002: the old cell read a commit id and "(round 3)". Stripping it to `resolved` under "Round 1." now claims FND-005 was fixed before the "New findings (disposition pass 2)" section that introduces it. This happens in three files: reviews/ir-89-code-review.md:93, reviews/ir-89-gap-analysis.md:73 and reviews/ir-89-spec-review-spec-integrity-analysis.md:63. Fix: move each FND-005 row into its own "Round 3." table after the round-2 section, or keep the words "(round 3)" in the cell.
- FND-003: FND-099's disposition reads "this review now retains an immutable official upstream source link resolving the v1.0.151 manifest at the commit above". The commit was deleted from FND-097, and the link at :70 now points at the tag `blob/v1.0.151/`. A tag can be moved, so the link is not immutable, yet :31, :59 and :70 still say it is. The substance of the finding holds: there is an official, independently checkable upstream source. Fix: at :31, write "retains an official upstream source link to the v1.0.151 manifest"; drop "immutable" before the manifest link at :70 and before "source manifest" at :29 and :59.
- FND-004: reviews/REV-014-pr51-campaign-recovery.md:98 reads "At recovery, the same exact CLI/engine and three module roots above report seven undeclared statuses". The PR deleted the process, ISO and engineering-assurance module roots from the checkpoint at :75. Fix: "At recovery, the same CLI, engine and module roots as the checkpoint above report seven undeclared statuses".
- FND-005: reviews/2026-09-14-native-temporal-code-rust.md:19 and reviews/2026-09-14-native-predicate-code-review.md:19 read "The review evaluated implementation candidate against". Fix: "evaluated the implementation candidate". spec/reviews/base.md:64 is now the bullet "The draft base plus the specification changes in this branch.", which has no verb. Fix: "Evaluated the draft base plus the specification changes in this branch." reviews/REV-006-conformance-implementation.md:99-100 reads "comment `5481528618` reviewed another". Fix: "reviewed a later revision".

## Scope

Spot-checked against the base text, sentence by sentence:

- `plan/PLAN-002-contract-ir-v01/plan.md`, examined: "Post-merge `main` passes the complete isolated local CI lane" (clean).
- `plan/PLAN-007-temporal-ecosystem-closure/log.md`, examined: PR #79 rebase-merge note (clean).
- `plan/PLAN-009-complete-v1-backend-delivery/plan.md`, examined: "The accepted QSpec is the semantic source for AD-004..." and the E01 row (clean).
- `reviews/2026-09-13-cycle-free-contract-model-code.md`, examined: Summary and AP-001 paragraph (clean).
- `reviews/2026-09-14-native-temporal-code-rust.md`, examined: Summary (FND-005).
- `reviews/2026-09-14-native-predicate-code-review.md`, examined: Summary (FND-005).
- `reviews/2026-09-14-temporal-ecosystem-model-code-rust.md`, examined: FND-6512 row and the deleted "Promotion Reconciliation" section, which only recorded SHA rewrites (clean).
- `reviews/2026-09-28-ir-22-spec-review-evidence-analysis.md`, examined: coverage paragraph, FND-005 row and dispositions (FND-001).
- `reviews/2026-09-28-ir-22-spec-review-integrity-analysis.md`, examined: Summary and dispositions (clean).
- `reviews/2026-09-28-ir-22-spec-review-scope-boundary-analysis.md`, examined: ruling bullets and dispositions (clean).
- `reviews/REV-002-contract-ir-foundation.md`, examined: FND-R03, whose `pgm-01` record no longer exists in the repo (clean).
- `reviews/REV-005-canonicalization-implementation.md`, examined: FND-097, FND-099 and the closing-gate link (FND-003).
- `reviews/REV-006-conformance-implementation.md`, examined: closing gate (FND-005).
- `reviews/REV-007-contract-ir-v01-epic.md`, examined: EPC-004, the TASK-009 row and the verification snapshot (clean).
- `reviews/REV-014-pr51-campaign-recovery.md`, examined: FND-1412, FND-1414 and the checkpoint sections (FND-004).
- `reviews/REV-015-executable-binding-recovery.md`, examined: integration checkpoint (clean).
- `reviews/SR-052-issue-64-native-temporal-base.md`, examined: Summary, FND-6401 and the independent audit (clean).
- `reviews/SR-055-issue-64-native-temporal-dependency.md`, examined: FND-6433 (clean).
- `reviews/SR-060-issue-64-observation-closure-follow-up.md`, examined: scope and `git diff --check` bullet (clean).
- `reviews/SR-583-pr-108-checked-package-deletion.md`, examined: Summary and vendored-bytes bullet (clean).
- `reviews/ir-313-spec-review-base.md`, examined: measurements and dispositions (FND-001).
- `reviews/ir-313-spec-review-spec-integrity-analysis.md`, examined: cross-repo reads, FND-003, FND-004 and the round tables (clean).
- `reviews/ir-314-spec-review-base.md`, examined: Verdict (FND-001).
- `reviews/ir-314-spec-review-failure-domain.md`, examined: FND-004 detail (clean).
- `reviews/ir-408-gap-analysis.md`, examined: Summary (FND-001).
- `reviews/ir-418-code-review.md`, examined: FND-001 detail and the model-owner alias bullet (clean; the alias stays in Cargo.toml).
- `reviews/ir-89-code-review.md`, `reviews/ir-89-gap-analysis.md` and `reviews/ir-89-spec-review-spec-integrity-analysis.md`, examined: dispositions and round-2 notes (FND-002).
- `spec/reviews/base.md`, examined: intake bullets (FND-005).
- `spec/reviews/kani-status-reconciliation/base.md`, examined: evidence bullets (clean).
- `spec/reviews/output-mapping-foundation/base.md`, examined: authority bullets (clean).
- All other files in the diff, examined: `scope:` and `evaluated_revision` strips and `fixed | resolved` cells (clean).

## Verdict

Changes requested, for three medium findings. The sweep is otherwise sound. It removes SHAs, reviewed-at lines, fix-sha cells and the SHA-only "Promotion Reconciliation" section without dropping any real finding, and it keeps the Cargo alias, the unsafe audit and the canonical digests. Validation passes (212/212), and coverage matches the known 22 unbacked rows. FND-001 to FND-003 are one-line wording fixes that restore meaning without adding any SHAs back.
