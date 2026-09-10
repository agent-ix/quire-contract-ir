---
id: SR-037
title: "Independent re-review — corrected Rust 1.98.1 qualification evidence at fd59a30"
type: SpecReview
analysis: code-review
scope: "PR #62 at fd59a30 against the reviewed head a12ba79; SR-036 FND-001..003; reviews/SR-035; .github/workflows/ci.yml; tests/toolchain_policy.rs"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-contract-ir/NFR-005"
    type: reviews
---

## Summary

Re-review of the three SR-036 findings at `fd59a30`. All three are closed, and
the high one is closed on the substance, not by rewording: I re-ran the full
Python suite with the pinned CLIs present and measured **1 failed / 20 passed**
with the single failure reading `ix-flow 0.2.3 is not the pinned 0.0.4`, and
`scripts/check_shared_pins.py` classifying `quire-cli 0.31.0` and `quoin 0.23.1`
as **compatible**. That is exactly what the corrected FND-1512 now claims.

The fix also went further than the finding asked. Every workflow action is now
SHA-pinned, each SHA resolves to a real commit in the right repository, and a
new `hosted action pin` gate refuses a mutable tag with its own mutation
control. The exact-job-count assertion is gone, replaced by "every declared
toolchain equals 1.98.1".

Two items remain, both about how far the corrections reach.

## Verdict

**CONDITIONAL** — no high findings. The blocker misattribution is corrected in
the findings table but survives in the verdict paragraph of the same document,
and the two new policy gates read one workflow file by name.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | SR-035's Verdict paragraph still attributes the red gate to "its pre-existing shared Quoin/Quire assurance integration", contradicting the corrected FND-1512 four lines below, which says Quoin and Quire classify as compatible | reviews/SR-035-rust-1.98.1-code-review.md:34, reviews/SR-035-rust-1.98.1-code-review.md:47 | correct-requirement-no-evidence |
| FND-002 | low | Both new policy gates read `.github/workflows/ci.yml` by name, so a second workflow file escapes the toolchain check and the action-pin check | tests/toolchain_policy.rs:35 | correct-requirement-no-evidence |

## Finding detail

### FND-001 — the correction stopped at the findings row

The corrected FND-1512 is accurate. I verified every part of it independently:

| Claim | Measured |
| --- | --- |
| suite is 20 passed / 1 failed when provisioned | `1 failed, 20 passed, 19 subtests passed` |
| the remaining failure is the `ix-flow` matrix pin | `ix-flow is unknown: ['ix-flow 0.2.3 is not the pinned 0.0.4 …']` |
| `quire 0.31.0` classifies compatible | `compatible   quire-cli   quire-cli 0.31.0 is the pinned version` |
| the matrix remains pending human acceptance | `pending_human_acceptance (authority: agent-ix/engineering-assurance)` |

Four lines above that row, the Verdict paragraph is unchanged:

> The repository's full release gate remains red on its **pre-existing shared
> Quoin/Quire assurance integration**; that failure is reproduced and kept
> separate from compiler compatibility.

Quoin and Quire are the two components the corrected finding names as
**compatible**. The document now says both things. A reader who stops at the
verdict — which is what a verdict is for — carries away the attribution the
correction was written to remove.

The same sentence appears once more in the closing paragraph, which was updated
("or waive the remaining `ix-flow` matrix disagreement") and is now consistent.
Only the verdict was missed.

**Fix:** one sentence. Name the `ix-flow 0.2.3` vs `0.0.4` matrix disagreement
and the pending human acceptance, as the closing paragraph already does.

### FND-002 — the new gates read one workflow by name

`PolicySurface::current()` loads `workflow: read(".github/workflows/ci.yml")`.
Both new checks — every `toolchain:` equals `RUST_VERSION`, and every `uses:`
revision is a 40-character hex SHA — run against that one string.

Measured, at `fd59a30`, with `cargo test --test toolchain_policy`:

| Mutation | Result |
| --- | --- |
| a third Rust job, correctly pinned, appended to `ci.yml` | **pass** — SR-036 FND-003 is closed |
| one `toolchain: 1.98.1` → `toolchain: 1.90.0` | fail (`hosted compiler declaration`) |
| `taiki-e/install-action@37f7c57…` → `@v2` | fail (`hosted action pin`) |
| a **new** `.github/workflows/zz-mutant.yml` with `uses: actions/checkout@v4`, `toolchain: stable` and `run: cargo +stable test` | **pass — escapes both gates** |

The repository has exactly one workflow file today, so nothing is currently
wrong. The gate is new, though, and it encodes "ci.yml" rather than "the
workflows". `read_dir(".github/workflows")` over every `*.yml` and `*.yaml`,
concatenated, costs one line and removes the assumption.

## Verified and found sound

Recorded so the next reviewer does not repeat the work.

- **Every pinned action SHA is real and points where the comment says.**
  `actions/checkout@11d5960a…` → tags `v4`/`v4.4.0`; `actions/setup-python@a26af69b…`
  → `v5`/`v5.6.0`; `Swatinem/rust-cache@6323deb1…` → `v2.9.2`/`v2`;
  `taiki-e/install-action@37f7c578…` → `v2.87.0`. A SHA pin that resolves to an
  unknown commit would be worse than the tag it replaced; none of these do.
- **`dtolnay/rust-toolchain@4360b525…`** carries the message `toolchain: stable`
  and is not on `master` (1 ahead, 2 behind) — it is a commit from that action's
  per-toolchain branch layout. It is immutable, and both jobs pass
  `toolchain: 1.98.1` explicitly, which the new gate now enforces, so the
  action's built-in default is never consulted. Not a finding.
- **The Rust gate set is green at this head**: `cargo fmt --check`,
  `clippy --all-targets --all-features --locked -D warnings`,
  `cargo test --locked` (13 suites, 47 tests, 0 failed), `cargo deny check`
  (advisories, bans, licenses, sources all ok), `cargo audit` (121 crates, zero
  advisories), and `scripts/check_unsafe_comments.sh` all pass.
- **`make pins` exits 1 for one reason only**, and it is the disclosed one.

## What this re-review does not do

It does not approve the pull request, dispatch hosted CI, accept the
compatibility matrix on the owner's behalf, or resolve the `ix-flow`
disagreement. FND-001 is a one-sentence documentation correction in an artifact
this PR already edits; it is not a reason to hold the compiler change.
