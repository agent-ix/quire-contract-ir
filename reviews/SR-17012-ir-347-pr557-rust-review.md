---
id: SR-17012
title: "IR-347 PR 557 Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir#557 vs origin/main; src/kani/mod.rs, tests/it/kani_shared.rs, tests/it/public_interface.rs, tests/it/main.rs"
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

## New findings (disposition pass 2)

Found by the code lead's final `make ci` at custodial, in its `make spec` step. `quire coverage --scope . --strict` reported a diff-owned suspicion. The reviewer reproduced it  by running the same command in a temporary detached worktree, which has since been removed.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | `tc_055_no_foreign_runtime_or_replay_surface_in_source` had every assertion behind a narrowing guard. Quire reports `[vacuous-under-guard]` with 7 of 7 assertions guarded and 0 unguarded. The macro assert ran only for files under `src/`, and the retired-module assert only for files under `src/kani`. No unconditional assertion showed that either source root was scanned at all, so a walk that found no files would have passed | tests/it/public_interface.rs:1456 |

## Dispositions

Round 1. Checked against fix. `make fmt-check` and `make lint` are clean, and `tc_055_nested_exported_macro_is_rejected_by_inventory` passes.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `exports_macro` uses a `syn::visit::Visit` visitor, which descends into nested and inline modules. The source scan calls it, and a positive and negative control test exercises it (tests/it/public_interface.rs:1419-1450, 1474) |
| FND-002 | fixed | The predicate is bound to `leaf_has_item` and rustfmt formats it within 100 columns (tests/it/public_interface.rs:1403-1406) |

Round 2. Checked against fix (`git diff ..HEAD`, `tests/it/public_interface.rs` only). `cargo test --locked --test it tc_055` passes 5 of 5. `quire coverage --scope . --strict` no longer reports `vacuous-under-guard`: 60 inherited unbacked rows, 0 contradicted, FR-039 4/4, FR-037 1/1. `make fmt-check` and workspace clippy with `-D warnings` are clean.

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | fixed | Each scanned file now records its violations instead of asserting inside the guard. The three checks are unchanged and still apply on the same paths: the nested `#[macro_export]` check on files under `src/`, the four forbidden strings with the same comment filter on every file, and the five retired `src/kani` module names. After the walk come three unconditional asserts: the root source was scanned, the model source was scanned, and the collected failure list is empty. Any forbidden path, retired module or exported macro in any scanned file now fails the test. No check was removed or narrowed (tests/it/public_interface.rs:1456-1537) |
