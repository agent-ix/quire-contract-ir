---
id: SR-035
title: "Rust review — exact 1.98.1 qualification"
type: SpecReview
analysis: code-review
scope: "agent-c/rust-1.98.1 implementation commit 32b9a73 against origin/main 690bde7; NFR-005 and TC-036/TC-037"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-005
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/ADR-0055
    type: references
---

# SR-035: Rust review — exact 1.98.1 qualification

## Summary

Applied `agent-skills/rust-review` to the exact-compiler migration at commit
`32b9a73`. The review covers repository Rust idioms, test trace markers, panic
and unsafe surface, dependency policy, compiler/tool declarations, real gate
execution, and whether failure classes were kept distinct.

The supported minimum and qualification compiler now both name exact Rust
1.98.1 across Cargo, rustup, Clippy, Make, hosted checks, normative requirements,
and live assurance inputs. TC-036 and TC-037 import `ix_trace_rs::trace` and use
bare `#[trace(...)]` attributes. TC-036 detects inherited 1.75, stale 1.85,
floating stable, absent, inconsistent, unaccepted, and unbounded-hold mutations.

## Verdict

**CONDITIONAL.** No Rust 1.98.1 compiler, target, code-quality, dependency, or
traceability finding remains open. The repository's full release gate remains
red on the separately disclosed `ix-flow 0.2.3` disagreement with the exact
`0.0.4` matrix pin and pending human acceptance; that failure is reproduced and
kept separate from compiler compatibility. External review and owner release
authority remain required.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1507 | high | **FIXED:** Cargo and Clippy claimed Rust 1.75 while rustup and both hosted jobs floated on `stable`. All live declarations now distinguish and pin supported minimum and qualification compiler to exact 1.98.1; mutation controls fail for stale, floating, absent, and inconsistent values. | `Cargo.toml:5`; `clippy.toml:1`; `rust-toolchain.toml:2`; `.github/workflows/ci.yml`; TC-036 | wrong-requirement |
| FND-1508 | medium | **FIXED:** stable rustfmt reported that `imports_granularity` and `group_imports` were nightly-only and ignored. The ineffective settings were removed and the 1.98.1 formatting result passes without a waiver. | `rustfmt.toml`; ADR-0055; NFR-005-AC-2 | correct-requirement-no-evidence |
| FND-1509 | medium | **FIXED:** warning-denied Clippy 1.98.1 found `chunks_exact_to_as_chunks` in digest parsing and three `manual_repeat_n` cases in adversarial fixtures. Each was migrated idiomatically; no allow attribute or warning downgrade was added. | `src/canonical.rs:88`; `tests/conformance.rs`; NFR-005-AC-2 | correct-requirement-no-evidence |
| FND-1510 | high | **FIXED:** the release composite ran license-only cargo-deny and omitted cargo-audit. Executing the real advisory gate reproduced vulnerable `idna 0.4.0` and `time 0.3.36`. Their transitive lock resolutions are now `idna 1.1.0` and `time 0.3.55`; full cargo-deny and cargo-audit pass and are wired into local and hosted gates. | `Makefile`; `.github/workflows/ci.yml`; `Cargo.lock`; NFR-005-AC-3 | correct-requirement-no-evidence |
| FND-1511 | medium | **FIXED:** marking TC-036/TC-037 complete exposed duplicate test-id rows in the coverage-design table; the matrix census treated the later prose rows as the canonical status and rejected both tests as incomplete. The case-design labels no longer impersonate test-summary rows, and the positive plus mutation-backed traced tests resolve. | `spec/contract-test-matrix.md`; `tests/toolchain_policy.rs`; `scripts/validate_matrix_status.py` | correct-requirement-no-evidence |
| FND-1512 | high | **CORRECTED:** the earlier 9-failure attribution was an under-provisioned-environment result, not a Quire compatibility defect. With the pinned CLIs present, `quire 0.31.0` and Quoin classify as compatible and the suite is 20 passed / 1 failed. The remaining failure is the already-disclosed `ix-flow 0.2.3` disagreement with the exact `0.0.4` matrix pin, while the matrix remains `pending_human_acceptance`. Ownership stays with the matrix entry and the required human decision; it is not assigned to Engineering Assurance #59 or quire-research #60. | `tests/test_shared_assurance.py`; `scripts/check_shared_pins.py`; NFR-005 known non-compatibility findings | correct-requirement-no-evidence |

## Exact implementation-commit gates

| Gate | Result |
|---|---|
| `rustc +1.98.1 --version --verbose` | `rustc 1.98.1 (48a229cea 2026-09-01)`, LLVM 22.1.8 |
| `cargo +1.98.1 fmt --all -- --check` | pass |
| `cargo +1.98.1 clippy --locked --all-targets -- -D warnings` | pass |
| `cargo +1.98.1 test --locked --all-targets` | 47 passed, 0 failed |
| `make supported-rust`; `make qualification-rust` | pass on exact 1.98.1 |
| release all-target build; warning-denied docs | pass |
| conformance run; corpus byte reproduction | pass |
| `cargo +1.98.1 audit` | pass after lockfile remediation |
| `cargo +1.98.1 deny check` | pass; 0 errors, 2 non-domain configuration warnings |
| `make audit-unsafe` | pass; no unsafe change |
| focused Python orchestration/matrix/ordering suites | 7 passed |
| `make spec` | pass; 119/119 grammar-clean, contract matrix 11/11 backed; inherited ambient diagnostics remain visible |
| full Python/shared-assurance suite | **20 passed, 1 failed** when fully provisioned; the remaining `ix-flow` matrix disagreement and pending human acceptance are disclosed by NFR-005 |
| hosted manual workflow | not dispatched; external review pending |

## Rust checklist result

- No public API, error type, ownership boundary, async/locking behavior, wire
  contract, resource bound, or unsafe surface changed.
- The only production expression change is equivalent two-byte digest chunking;
  the pre-existing exact 64-byte lowercase-hex guard makes the remainder empty.
- All other source changes are compiler/tool declarations, gate composition,
  traced inspection tests, or 1.98.1 lint migrations in existing tests.
- The `ix-trace-rs` dependency is dev-only, exact-version constrained, tag-pinned,
  and explicitly allow-listed as the repository's sole Git source.
- No lint allowance, advisory waiver, ignored test, network-dependent test,
  clock assertion, panic path in production, or unbounded input path was added.

## Release boundary

This review demonstrates compiler/tool compatibility and records the unrelated
red shared-assurance path. It does not approve the pull request, synthesize a
human decision, dispatch hosted qualification, publish the crate, or waive the
remaining `ix-flow` matrix disagreement.
