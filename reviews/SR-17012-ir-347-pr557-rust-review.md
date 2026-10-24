---
id: SR-17012
title: "IR-347 PR 557 Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir#557@59d855c0418db8f707c2084d5644dad2b8427b86 vs origin/main 7075edc; src/kani/mod.rs, tests/it/kani_shared.rs, tests/it/public_interface.rs, tests/it/main.rs"
review_set: subset
---
# SR-17012: IR-347 PR 557 Rust review

## Summary

Ticket: IR-347. This review applies `rust-review` and the `rust-style` baseline (testing, API
design and safety references) together with the repository's `CLAUDE.md`, `AGENTS.md` and
`rustfmt.toml` (stable channel, 100 characters). The PR adds no CI, Makefile or Cargo manifest
change, so no gate is narrowed.

Gates run at the frozen head: `make fmt-check` was clean, and `make lint` (workspace clippy with
`-D warnings`, plus the model-only lane) was clean. The 3 `tc_055` tests and 57 kani doctests
pass.

Examined units:

- `src/kani/mod.rs`, the module list and re-exports (examined). Only private `mod abi`,
  `dispatch`, `outcome` and `profile` remain, and there is no glob.
- `tests/it/public_interface.rs`, `tc_055_root_public_inventory_matches_fr_039` and
  `tc_055_no_foreign_runtime_or_replay_surface_in_source` (examined).
- `tests/it/kani_shared.rs`, the `tc_055_negative_kani_corpus_*` test (examined; its coverage
  is reported in SR-17011).
- `tests/it/main.rs` (examined).

Both new tests carry `#[trace]` tags.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new `#[macro_export]` guard walks only each file's top-level `syn` items, so an exported `macro_rules!` inside a nested inline `mod` (for example a `#[cfg(test)] mod tests`) escapes it. Such a macro is still exported at the crate root. The pre-existing model-crate scan at line 951 splits the whole file text on `#[macro_export]` and has no such blind spot | tests/it/public_interface.rs:1433-1450 |
| FND-002 | low | One 191-character `assert!(…matches!(…) if …)` line sits beyond the repository's 100-character `rustfmt.toml` width. rustfmt gave up on the nested macro, so `fmt-check` stays green without formatting it. Bind the `any(..)` result to a named local, or use a `let … else`, so it formats | tests/it/public_interface.rs:1403 |

## Verdict

**PASS WITH FINDINGS (low only).**

- The tests read source through compile-time `include_str!` or `CARGO_MANIFEST_DIR` paths, not
  through the runtime working directory.
- The public-surface parser fails closed on globs, inline or `#[path]` public modules and
  unrecognised item kinds.
- Exports are cross-checked against their leaf declarations.
- There is no unsafe code, no new public API and no panic path in production code. The diff
  removes production code only.
