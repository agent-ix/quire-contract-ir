---
id: SR-612069
title: "IR-1071 PR 639 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1071; PR #639; Cargo.toml, Makefile, tests/conformance_qspec/main.rs"
review_set: subset
---

## Summary

Reviewed the three-file diff at PR head a0ce4e18 against IR-1071 AC-1 through AC-3 and the repository's Rust conventions. The manifest retains `test = false`; the new CI prerequisite explicitly compiles the test executable with `--no-run`, and the named fixture lane retains its executing command and checkout-dependent assertions.

## Verdict

**PASS** for the scoped code and Rust review. The `cause_word` match has no wildcard and includes `InvalidDerivedShape`, so omitting an enum variant produces a compiler error when this target builds. The code lead owns the final aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Examined IR-1071 AC-1: "IR's `make ci` or hosted Rust workflow runs a compile-only check of the `conformance_qspec` target without needing `QUIRE_SPECIFICATION_DIR` or a QSpec checkout."

Examined IR-1071 AC-2: "The check fails if a future `CheckedPackageRefusalCause` variant is omitted from the conformance target's exhaustive match."

Examined IR-1071 AC-3: "QSpec fixture execution remains in the existing named `make conformance-qspec` lane, which reads the authoritative QSpec checkout; no fixtures are copied into IR."

Read repository `AGENTS.md` and `CLAUDE.md`, the Rust review and style rules, the changed manifest/Makefile/test comment, `cause_word`, `fixtures_dir`, and the existing named fixture target. `make -n ci` includes `cargo test --locked --workspace --no-run --test conformance_qspec`. The coder's focused compilation without `QUIRE_SPECIFICATION_DIR`, negative E0004 probe, formatting and Clippy results are reported in the PR; this review did not repeat a heavyweight build or execute fixtures. No applicable AssuranceProfile was found under `spec/`.
