---
id: SR-615
title: "code review of PR 225 (one copy of every first-party crate)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@dbc07f9fe556f25fec0e3b3851628dd8ca459385; .bak, .gitignore, CLAUDE.md, Cargo.toml, Cargo.lock, Makefile, deny.toml, crates/quire-contract-model/Cargo.toml, src/kani/{mod,replay}.rs, tests/it/{cycle_free_model,kani_replay,main,checked_package_v2_parameters}.rs, tests/it/support/composed_types, tests/fixtures/native-rule-model.json"
review_set: base
---
# SR-615: code review of PR 225

## Summary

Ticket: IR-358. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` at dbc07f9. The removal of `replay_with_native_runtime`, the `Native*` types, the QSL dev-deps and the owner alias is clean: no live reference remains in src, tests, schemas, corpus, README or CLAUDE.md (only historical `reviews/` files and one stale test comment). Gates run by the reviewer at dbc07f9 with CARGO_TARGET_DIR in the worktree: fmt-check 0, lint 0, `cargo test --locked --workspace --all-targets -- --include-ignored` 0 (177 + 48 passed), corpus 0, audit-unsafe 0, deny 0, spec 2 (214/214 grammar-clean; the known 22 unbacked rows, 0 contradicted). The defects are in the gate and the local-dev targets.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A stray `.bak` file (a byte copy of the new Cargo.toml, same blob fa73d50) is committed at the repo root of a public repo | .bak:1-34 |
| FND-002 | medium | The gate's premise is false. Makefile and deny.toml say `deny-multiple-versions` only sees copies with different versions. Measured: this PR's deny.toml over origin/main's Cargo.lock fails bans with `error[duplicate]` for quire-contract-model (path vs git 7b1ec7a, both 0.1.0) and quire-verification-contracts (two revs, both 0.1.0), while origin/main's deny.toml reports `bans ok`. The Cargo.lock count is redundant for every listed name, and it only runs after `cargo deny check` has already failed | Makefile:82-94 |
| FND-003 | medium | The gate is a hardcoded name list kept twice (FIRST_PARTY and 18 `[[bans.deny]]` entries). It misses first-party crates the plan names (FCD `agent-ix-*`, quire-contract-runtime, quire-rs crates, codegen) and any crate added later. Nothing fails when the list drifts | Makefile:86 |
| FND-004 | medium | `make use-local` breaks every `--locked` make target. The patch rewrites the tracked Cargo.lock by dropping the `source` lines of the patched crates, and `cargo metadata --locked` then exits 101 ("cannot update the lock file"). `make use-remote` does not restore Cargo.lock, so `--locked` still fails afterwards, and a lock with no git sources is one `git add -A` from being committed | Makefile:118-131 |
| FND-005 | low | A patch whose sibling crate version does not satisfy the `=0.1.0` requirement is silently unused: cargo only warns "patch was not used" and builds from GitHub, while use-local has already reported success | Makefile:126 |
| FND-006 | low | Stale comment names the removed `NativeReplayAgreement` | tests/it/kani_replay.rs:547-553 |
| FND-007 | low | The tc_041 QSL-dependency assertion matches on names only (`quire-spec-language`, `qsl-*`). A dependency on `quire-exact`, or on any other crate from the QSL repository, passes it. Asserting on `dependency["source"]` covers the whole repository | tests/it/cycle_free_model.rs:71-82 |

## Finding Detail

- FND-001: `git rm .bak`.
- FND-002 / FND-003: Replace both lists with one generic check on Cargo.lock, with no hardcoded names: fail if any `[[package]]` whose `source` starts with `git+https://github.com/agent-ix/` shares its `name` with any other `[[package]]` entry (git, path/workspace with no `source`, or registry). The reviewer tested this awk against both lockfiles. It fails origin/main (quire-contract-model 2, quire-verification-contracts 2) and passes dbc07f9:

      function flush() { if (name != "") { count[name]++; if (index(src, "git+https://github.com/agent-ix/") == 1) first[name] = 1 } name = ""; src = "" }
      /^\[\[package\]\]/ { flush(); next }
      /^name = /   { name = $2 }
      /^source = / { src = $2 }
      END { flush(); bad = 0; for (n in first) if (count[n] > 1) { printf "one-copy: %s has %d entries in Cargo.lock\n", n, count[n] > "/dev/stderr"; bad = 1 } exit bad }

  Run it with `awk -F'"'` from `make deny` (a `scripts/check_one_copy.awk` next to `check_unsafe_comments.sh`). Then delete FIRST_PARTY and all 18 `[[bans.deny]]` entries. They are redundant with the generic check for every agent-ix-sourced crate, and a name list is the drift trap this PR exists to remove. Correct the comments in Makefile:82-85 and deny.toml:21-26.
- FND-004: In `use-remote`, run `git checkout -- Cargo.lock` after deleting the config. In the make targets, replace the literal `--locked` with `LOCKED ?= $(if $(wildcard .cargo/config.toml),,--locked)` so that lint/test/corpus/build work under use-local. Alternatively, document that under use-local cargo runs without `--locked`.
- FND-005: After writing the config, run `cargo metadata --format-version 1 >/dev/null` and fail if stderr contains "was not used".
- FND-006: Drop the `NativeReplayAgreement` clause from the comment.
- FND-007: Also assert `!dependency["source"].as_str().unwrap_or("").starts_with("git+https://github.com/agent-ix/quire-spec-language")`.

## Verdict

Request changes. Checked and correct: SIBLINGS resolves to <siblings dir> both from the main checkout and from the linked worktree `.worktrees/one-copy` (`git rev-parse --git-common-dir` is <repo>/.git in both, and `/../..` gives the parent). The generated `[patch."https://github.com/agent-ix/<repo>"] <crate> = { path = "<siblings>/<repo>/." }` is valid: cargo resolved both patched crates from the sibling paths. A missing sibling fails with a clear message, removes the partial config and exits 2. The rename to `checked_package_v2_parameters.rs` changes function names only. Every `#[trace]` id is unchanged, and nothing outside historical `reviews/` names the old file or function. The removal leaves `src/kani/replay.rs` with no QSL import, and lint is clean. Composed_types was dead code (not declared in `tests/it/support/mod.rs`), so deleting it loses nothing.
