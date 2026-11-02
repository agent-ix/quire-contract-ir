---
id: SR-16623
title: "IR-641 PR 553 code review with Rust lane"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@beb1975f6b297edc8c8eabf3044b551aa3d55e34; PR #553 Makefile, removed scripts/check_one_copy.awk, tests/it/lock_coherence.rs and tests/it/main.rs; FR-055-AC-1 through AC-5"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-055", type: references }
---
# SR-16623: IR-641 PR 553 code review with Rust lane

## Summary

Independent diff-scoped code and Rust review of draft PR #553. The four-file diff changes the normal deny recipe to call the Quire owner checker, removes the local AWK checker, and adds real-command Make integration controls. Ticket: IR-641.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-055-AC-1 examined: Makefile:155-157 invokes `$(QUIRE) check-lock Cargo.lock` after cargo-deny. The tests run this recipe via `make -C <scratch> -f <repository Makefile> deny`; the aligned and committed-lock cases must succeed and observe the command echo.
- FR-055-AC-2 examined: tests/it/lock_coherence.rs:64-114 generates valid 40-hex Git source fragments. Same-name/different-revision and different-name/same-repository/different-revision controls both require the owner diagnostic `coherence refused`; the aligned two-crate control succeeds. The old AWK script cannot satisfy both the command echo and split-revision assertion.
- FR-055-AC-3 examined: tests/it/lock_coherence.rs:116-128 checks malformed input and a missing executable fail the Make recipe. The preceding aligned control establishes that the checker is present for the malformed case; it cannot pass only because the command is missing.
- FR-055-AC-4 examined: the local AWK file is deleted; no replacement checker, fallback, or revision allowance appears in the four-file diff. The normal recipe has no AWK call.
- FR-055-AC-5 examined: the diff changes no Cargo manifest or lock. The committed-lock test reads the manifest and lock bytes before and after running the owner checker, and requires Make success. Its cargo-deny stage is deliberately isolated by `CARGO=true`; the actual final deny gate remains pending.
- Rust lane: the new test uses ordinary subprocess and scratch-file ownership, cleans only its own scratch path, propagates subprocess errors, and binds the test functions with `#[trace]`. No unchecked caller-controlled indexing, production API, runtime dependency, or CI workflow change was found.
- Evidence boundary: the installed released Quire CLI in the review environment does not yet expose `check-lock` (`quire check-lock --help` exits 3). The author's focused 2/2 result used an owner-built checker. I did not run a full gate or claim released-command qualification.

## Verdict

PASS for the diff-scoped code and Rust review of draft head beb1975f6b297edc8c8eabf3044b551aa3d55e34. Final FR-055-AC-1/5 qualification and merge remain dependent on the released installed CLI and a final-head gate.

## Continuation at 35d44745fbc157b8f8426ca9bcc0c19f23bec808

The new `.github/workflows/ci.yml` install uses the exact quire-cli source revision `374d50853331bf7bc86a991a2caa107a9a49442f` with `cargo install --locked`. The Rust Checks job supplies its binary only to `make test` through `QUIRE_LOCK_CHECKER`; the Supply-chain Checks job supplies its binary only to `make deny` through `QUIRE`. Both use a separate `${{ runner.temp }}/quire-check-lock` root, so the fleet `quire` on `PATH` remains available to the spec recipe. The supply-chain job still runs cargo-deny and cargo-audit. No source or binary copy was introduced. The hosted jobs were still in progress at this continuation review, so no hosted result is claimed here. The released-binary mismatch is recorded by the separate gap-analysis artifact SR-16624.
