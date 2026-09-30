---
id: SR-610
title: "gap analysis of PR 224 (residual tracking sweep)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; whole repository at the PR head (git grep), CLAUDE.md and AGENTS.md banner claim"
review_set: subset
---
# SR-610: gap analysis of PR 224

## Summary

Ticket: IR-425. Planless gap analysis. Plan completion: not assessed. PR 224 changes the CLAUDE.md and AGENTS.md banner to say that hashes, digests, SHAs, pins, checksum catalogs and tracking records "have been removed from this repository". This analysis tests that claim against the whole tree at the PR head. It uses `git grep -n -E '\b[0-9a-f]{7,40}\b'` and a 41+ hex-character search, then greps for pin, checksum, provenance and revision wording. Each hit is judged by the value test: what breaks if it is deleted?

Hits that stay: Cargo.toml and crates/quire-contract-model/Cargo.toml git `rev`s; the corpus; canonical and golden test digests and `0123456789abcdef` fixtures in tests/ and src/; numeric bounds in schemas and spec; and GitHub comment and crates.io ids, which are identifiers, not SHAs. `.github/workflows/ci.yml` has 9 SHA-pinned `uses:` lines. A separate PR handles them, so they are not counted here, but they also make the banner's "have been removed" untrue until that PR merges.

Gates: `quire validate` over spec, plan and reviews exits 0. `make spec` reports 212/212 grammar-clean and 22 unbacked rows with 0 contradicted statuses, the same known set as main. No new unbacked row.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Banner claim is false at the PR head: two review files that came in with the #223 merge still carry recorded SHAs | reviews/ir-362-code-review-223.md:6 |
| FND-002 | medium | Checksum and digest records that only track files remain in review and plan prose | reviews/REV-005-canonicalization-implementation.md:51 |
| FND-003 | low | A pin sentence remains in a plan | plan/PLAN-006-native-temporal-correspondence/plan.md:85 |
| FND-004 | low | `evaluated_revision` still records a branch name in six spec/reviews files, while SR-530 to SR-533 dropped the field | spec/reviews/output-mapping-foundation/base.md:8 |

## Finding Detail

- FND-001: the merge of origin/main into this branch brought in reviews/ir-362-code-review-223.md and reviews/ir-362-gap-analysis-223.md. They carry `scope: "agent-ix/quire-contract-ir@<sha>"`, "diff <base>...<head>", "origin/main <sha>", "ran every gate on <head sha>", "at base <sha>" and "Grep on <head sha>" (code-review :6, :13, :15, :34; gap-analysis :6, :13, :46, :50). The PR text says it "avoids every file changed by #223". That was true before the merge, but the banner describes the tree as it is now. Fix: strip these eight references the same way as the rest of the sweep (for example `scope: "agent-ix/quire-contract-ir; diff against the PR 218 head: ..."`, "ran every gate on the PR head").
- FND-002: reviews/REV-005-canonicalization-implementation.md:50-51 and :55-56 restate the `serde_json` and `zmij` crate checksums, which Cargo.lock already holds. Deleting them breaks nothing, because the sentence already says "exactly matching `Cargo.lock`". plan/issue-53-formal-profile-qualification.md:37-40 records SHA-256 digests of two research input files at a local path. These are provenance digests of files outside the repo, not canonical identities that bind a proof to its content. Fix: delete the four hex values and keep "exactly matching `Cargo.lock`". Delete the two input digests. Separately, plan/issue-53:37 and reviews/2026-09-15-plan-009-complete-v1-backend-gap-analysis.md:41 expose absolute local user paths in a public repo. That predates this PR and is out of its scope, but it is worth cleaning in the same pass.
- FND-003: plan/PLAN-006-native-temporal-correspondence/plan.md:85 says "Owner APIs and schema bytes remain pinned by merge revision." This is a pin sentence of the kind the owner rule deletes. Fix: delete the sentence. The next sentence ("Contract IR never mirrors owner wire types...") carries the real rule.
- FND-004: spec/reviews/checked-package-v2/base.md:8 and ears-conformance.md:8, spec/reviews/kani-status-reconciliation/base.md:8 and ears-conformance.md:8, and spec/reviews/output-mapping-foundation/base.md:8 and ears-conformance.md:8 keep `evaluated_revision: "<branch>"` after the "based on <sha>" suffix was removed. A deleted branch name is a revision record with no remaining use, and SR-530, SR-531, SR-532 and SR-533 removed the field entirely in this same PR. Fix: remove the `evaluated_revision` line from those six files. `quire validate` does not require it.

## Scope

- `CLAUDE.md` banner, examined: "Hashes, digests, SHAs, pins, checksum catalogs and records that track files, versions or tools are an antipattern and have been removed from this repository." (FND-001 to FND-004)
- `AGENTS.md` banner, examined: same text (FND-001 to FND-004).
- `git grep '\b[0-9a-f]{7,40}\b'` over the whole tree except Cargo.lock and corpus, examined: every hit judged (FND-001).
- `git grep '[0-9a-f]{41,}'`, examined: hits in src/, tests/ and scripts/ are canonical or golden digests and stay. The hits in docs are FND-002.
- Pin, checksum, provenance and evaluated_revision wording under spec/, plan/ and reviews/, examined: most hits are the subject of a real historical finding or domain prose ("source provenance", "pinned by TC-015") and stay (FND-003, FND-004).
- `Cargo.toml` and `crates/quire-contract-model/Cargo.toml` git revs, context_only: kept by the owner rule.
- `.github/workflows/ci.yml` SHA-pinned actions, context_only: handled in a separate PR.

## Verdict

Changes requested. FND-001 makes the PR's own headline claim false at its head. The fix is eight mechanical edits in two files that the #223 merge brought in. FND-002 to FND-004 are small residual ceremony that the banner also covers. The spec gates are unchanged from main: validate exits 0, and coverage has the known 22 unbacked rows and no new one.
