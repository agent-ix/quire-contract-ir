---
id: SR-720
title: "code review of PR 247 (IR-476 test trace hygiene)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@f512aac4330ad47cf474d23af9f2407a32c45f19; tests/it/executable_binding.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---
# SR-720: code review of PR 247

## Summary

Ticket: IR-476 (partial; the PR body says "Part of IR-476" and has no Closes). Reviewed
head f512aac against origin/main 4233b56 (merge base equals main). Rust-review lane folded
in. The diff touches test annotations only: 13 doc blocks in `tests/it/executable_binding.rs`
move from bare `/// TC-035` + `/// FR-023-AC-n` lines to `use ix_trace_rs::trace;` plus
`#[trace("TC-035", "FR-023-AC-n", ...)]` and a `/// Tracing:` line, and four unit tests in
`operations.rs` gain a `/// Tracing: TC-048` doc line. No Cargo.toml or Cargo.lock change:
`ix-trace-rs` is already a root dev-dependency (Cargo.toml:28) and 19 other `tests/it` files
already import it. `make lint` (clippy -D warnings, both lanes) passes; the 13
`executable_binding` tests and the 48 `operations` unit tests pass; `make deny` passes
(advisories, bans, licenses, sources ok; one-copy check ok).

Each of the 13 attribute rewrites carries exactly the TC and AC ids the bare block carried
(checked block by block in the diff).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The four added `/// Tracing: TC-048` lines bind nothing. `Tracing:` matches no declared trace form (the legacy `rust-trace-line` form is `Trace:`), and quire reports all four as `unmatched_tags` on evidence symbols at head; bound count is 239 before and after, and the "tagged 243 -> 245" rise is two of these four tests (the other two already carried an unmatched id). The tests do not carry the `tc_048_` name prefix that binds TC-048 for the other 25 TC-048 tests in this file. The annotation therefore reads as coverage it does not provide. Either bind them (rename with the `tc_048_` prefix, the form this file already uses and which binds in this crate with no new dependency) or do not add them | crates/quire-contract-model/src/checked_package/v2/operations.rs:2594,2641,4776,4806 |

## Verdict

The `executable_binding.rs` rewrite is correct and binding-neutral, and it is justified: the
module declares the bare `///` id form as legacy (`rust-doc-comment-id`, `rewrite_to:
rust-trace-attribute`), so the attribute is the canonical form, as the ticket said. Not
mergeable as is because of FND-001: the operations.rs half adds inert tags.

## Dispositions

Round 1, reviewed agent-ix/quire-contract-ir@f6cc2e9a7f9a387a073211e653fddfac5f4dd54c (delta
from f512aac4330ad47cf474d23af9f2407a32c45f19; merge base is current origin/main
4233b569a4723f1cf60c0103a3dc1f3e5fd9e798). The four application-key tests are renamed with the
`tc_048_` prefix and their bare `Tracing: TC-048` lines removed; `quire coverage --json` at
head binds all four (bound 239 -> 243) and reports no unmatched TC-048 or FR-038 tag in
operations.rs. The delta also adds `#[trace(...)]` to 24 tests and `ix-trace-rs` as a
dev-dependency of quire-contract-model (Cargo.lock delta: that one edge). Rust lane on the
delta: the import sits in the `#[cfg(test)] mod tests`, attribute order matches
`tests/it`, and the clean gate log shows fmt-check, both clippy lanes and every test target
passing, with the four renamed tests run under their new names.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 743407e13fdfc7a3fc3c4f8a865834d4ae81326f |
