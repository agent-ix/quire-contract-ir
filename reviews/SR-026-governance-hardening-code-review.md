---
id: SR-026
title: "Code review — post-merge governance reconciliation hardening"
type: SpecReview
analysis: code-review
scope: "76ed240..HEAD; tests/governance_reconciliation.rs; tests/fixtures/campaign-issue-dispositions-v1.json; tests/fixtures/campaign-bodies/; CONTRIBUTING.md; STD-002; FR-021; TC-025/TC-026/TC-028"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/42
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-021
    type: references
---

# SR-026: Code review — post-merge governance reconciliation hardening

## Summary

Reviews the change closing the four residuals retained by the PR #41 re-review
(quire-contract-ir #42): the dotted-key dependency-guard bypass (R-1), TC-026's
overstated scope (R-2), unverified receipt marker fields (R-3), and the
campaign-document guard's false positive on quoted policy in `reviews/**` (R-4).
All four are closed, the guards are mutation-probed, and every local gate passes.

## Verdict

**CONDITIONAL** — no high findings; two low findings are recorded as accepted
residual limits of the new quoting rule, both bounded to `reviews/**`.

## Gates

Run on `issue/42-governance-hardening`, all green:

- `make fmt-check` — clean.
- `make lint` (`cargo clippy --all-targets -- -D warnings`) — clean.
- `make ci` (fmt-check, lint, governance + `cargo test --include-ignored`,
  corpus, corpus-repro, `cargo deny check licenses`, unsafe-comment audit) — clean.
- `make spec` (`quire validate`, `quire coverage --strict`, matrix status
  census) — clean; FR-021 remains 5/5 and every ✅ row still resolves.
- `tests/governance_reconciliation.rs` — 6 tests, all passing.

## Findings

| ID      | Severity | Summary                                                                 | Refs                                    |
| ------- | -------- | ----------------------------------------------------------------------- | --------------------------------------- |
| FND-001 | low      | A prescription split across a blockquote boundary in a review is not reported | tests/governance_reconciliation.rs:214   |
| FND-002 | low      | Retained bodies are not covered by the historical byte lock              | tests/fixtures/campaign-bodies/           |

## Finding detail

### FND-001 — partially quoted prescription

`campaign_prescription_violations` drops whole blockquote lines before
normalizing, so a review that quotes the first half of an obsolete prescription
and states the second half as live text no longer matches the full string.

Failure scenario: `reviews/SR-999.md` contains `> The common evidence envelope
identity is` followed by an unquoted line `` `quire.derivation-evidence/v1`. ``.
The guard reports nothing.

Accepted: the residue is a fragment, not a stated prescription, and the exposure
is confined to `reviews/**`, which is non-normative by construction. The
unterminated-fence variant of the same class *was* closed — a stray opening
fence restores its lines rather than exempting the remainder of the document —
and is regression-probed in TC-028.

### FND-002 — retained bodies outside the historical lock

`tests/fixtures/historical-pgm01-files.sha256` locks `evidence/**` and the
PGM-01 schemas. The newly retained bodies under `tests/fixtures/campaign-bodies/`
are locked instead by the digests inside
`tests/fixtures/campaign-issue-dispositions-v1.json`, which TC-026 verifies on
every run.

Failure scenario: editing a retained body fails TC-026 immediately, so the bytes
are not unprotected; they are simply protected by a second, receipt-local lock
rather than by the historical lock file. Recorded so the two mechanisms are not
later mistaken for one.

## Residual checks

- **R-1 seam.** `production_dependency_violations` now resolves a dotted key to
  its first segment. The exact issue examples (`quoin.version = "1"`,
  `quoin.git = "..."`) are positive probes, alongside a workspace-inherited
  (`quire-rs.workspace = true`) and a quoted-key
  (`'quire'.version`) probe, plus a non-Quoin negative (`serde.version`,
  `serde.features`). The pre-existing comment-injection negative is retained.
  Each positive probe fails against the pre-change guard, so the probes gate.
- **R-2 honesty.** TC-026 is renamed
  `tc_026_binds_the_retained_campaign_disposition_bytes`, carries a scope doc
  comment, and asserts the receipt's own `proves` / `doesNotProve` /
  `limitations` statements. STD-002 states the same split in prose. Nothing
  claims the duplicated constants observe live GitHub.
- **R-3 enforcement.** Bodies for issues #1 and #7 and for comment 5497534831
  are retained verbatim; each hashes to the digest already recorded in the
  receipt (verified against live `gh api` output before retention). TC-026
  checks digest ↔ bytes, then every `requiredMarkers` entry present and every
  `absentMarkers` entry absent in those bytes. The marker fields are now
  enforced, not decorative.
- **R-4 rule.** The quoting rule is normative in CONTRIBUTING.md and mirrored in
  FR-021 behavior. TC-028 probes blockquote-quoted and fence-quoted review text
  as clean, and unquoted review text, an unterminated fence, a blockquote in
  `spec/`, and unquoted README text as violations. Governed campaign content
  gains no exemption from quoting.

## Seam and idiom notes

- The file's established idiom is source inspection over `include_str!`
  constants; the new assertions follow it rather than introducing a second
  style. Guard logic is extracted into pure functions
  (`without_quotations`, `campaign_prescription_violations`, `retained_body`,
  `assert_markers`) so it can be probed with synthetic inputs instead of
  requiring fixture files on disk.
- No `unsafe`, no new `#[allow]`, no `#[ignore]`, no clock-based assertion, no
  suppressed warning, and no lowered gate.
- Panics are confined to test code, where they are the failure mechanism.
