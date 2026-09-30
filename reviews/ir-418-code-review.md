---
id: SR-603
title: "code review of PR 218 (ceremony sweep, code half)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@f21a1948936b1cdf5282ad5678bfac21748cdc8c; Cargo.toml, Makefile, deny.toml, README.md, CLAUDE.md, .agent/rules/writing_rust.md, crates/quire-contract-model/src/output_mapping.rs, scripts/generate_conformance_corpus.py, tests/it/{cycle_free_model,kani_replay,output_mapping,checked_package_v2_reader,main}.rs, tests/fixtures/{bridge-qsl-consumer,model-alias-consumer}"
review_set: base
---
# SR-603: code review of PR 218

## Summary

Ticket: IR-418. Code review with the rust-review lane folded in, scoped to `git diff origin/main...f21a194`. The review checked the dependency removals, the deleted tests and targets, the FR-034/STD-003/FR-019 identity change, the unsafe audit against main, and test quality in `tests/it/output_mapping.rs`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The PR deletes the unsafe-baseline and `--update-baseline` line from the agent Rust rules, though the owner ruled the unsafe audit stays exactly as on main. CLAUDE.md:34 still documents it, so the two rule files now disagree | .agent/rules/writing_rust.md:12 |
| FND-002 | medium | `invalid_generator` and `invalid_observer` are tested only with `""`. STD-003 names three conditions for each (empty, unbounded, outside visible ASCII), and no test checks the 256/257-byte boundary, a non-graphic owner, or the required location. The PR also deleted the only other `InvalidGenerator` cases (the SemVer loop) | tests/it/output_mapping.rs:1625-1630 |
| FND-003 | low | The refused observation is built with a different observer owner, but no test asserts `refused.observer()`. Wrong-observer retention on the refused path would pass | tests/it/output_mapping.rs:1754-1763 |

## Finding Detail

- FND-001: Main has `- Pre-existing baselines (legacy unsafe without comments) live in scripts/unsafe_comment_baseline.txt. Regenerate with bash scripts/check_unsafe_comments.sh --update-baseline.` The script, baseline file, `audit-unsafe` target and its `ci` membership are byte-identical to main. Only this line was lost, probably when commit f21a194 restored the script. Fix: restore the line exactly as on main.
- FND-002: `valid_selection_member` (output_mapping.rs:527) refuses empty, >256 bytes, and non-`is_ascii_graphic` owners. Fix: in one test, for both `OutputGeneratorIdentity::new` and `StructuralObserverIdentity::new`, assert 256 bytes accepted, 257 bytes refused, and `"agent ix"` and `"agent-ïx"` refused. Also assert the error location is `generator` / `observer`, as STD-003 requires.
- FND-003: Add `assert_eq!(refused.observer().owner(), "agent-ix/other-observer");`.

## Verdict

Needs one fix round (FND-001, FND-002; FND-003 is cheap, so do it in the same round).

What checks out:
- The removed root deps (quire-observation, quire-protocol with test-support, serde_stacker, stacker, sha2, tl-syntax, tl-mltl, quire-mltl, agent-ix-baseline-producer) have no use in `src/` or `tests/`. `sha2` stays a dev-dep, used by the tests, and the model crate declares its own serde_stacker/stacker/sha2.
- Features are unaffected. `fault-injection` is still enabled only through the root dev-dep, and `make lint` passes both lanes, including the lane that checks the model as a standalone consumer.
- `cargo deny check` passes. Every remaining `allow-git` entry resolves to a source in Cargo.lock, and dropping `AGPL-3.0-only` breaks no license check.
- The deleted items were all ceremony: the fixture crates and their own locked graphs, `corpus-repro` and `--check` (a byte-for-byte corpus comparison), `supported-rust` and `qualification-rust` (exact-toolchain re-runs), and the bridge production-owner-presence assertions (they asserted that unused deps exist).
- The FR-034, STD-003 and FR-019 change matches the merged spec. The generator identity is owner-only. The observer identity is owner-only and keeps `InvalidObserver`. `ObserverResultDigest` is gone, with no other reference outside `reviews/`. The package identity preimage still binds the generator owner (tested by `changed_owner`). No corpus or schema carries the removed fields.
- The unsafe audit (script, baseline, `--update-baseline`, `make audit-unsafe`) is identical to main.
- The `quire-contract-model-owner` alias at 7b1ec7a is load-bearing, not a tracking pin. QSL 9395be4's Cargo.lock entry depends on exactly that model rev. Its `qsl-package/src/emit.rs:455` builds `NominalOwner::Model { identity, node }`, and HEAD's `NominalOwner::Model` has an added `version` field (checked_package/v2/identity.rs). A `[patch]` to the workspace model therefore cannot compile. The tests need the alias for type identity with QSL, so it passes the value test.
