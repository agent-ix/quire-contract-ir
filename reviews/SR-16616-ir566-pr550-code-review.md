---
id: SR-16616
title: "IR-566 PR 550 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; Cargo.toml, Makefile, tests/it/cycle_free_model.rs, spec/conformance/functional/FR-020-json-conformance-interface.md; FR-020-AC-18"
review_set: subset
---

## Summary

Ticket IR-566. Independent diff-scoped code and Rust review of PR 550. The optional feature and locked metadata closures behave as specified, but the existing default test target no longer builds the binary launched by conformance integration tests.

## Verdict

**FAIL** — the default `make test` lane has a reproducible process-test failure on a clean target.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The default test lane skips the feature-gated conformance binary while integration tests still execute it; a clean `cargo test --locked --offline --test it conformance::tc_018_shared_cli_preserves_version_and_closed_argv -- --exact` fails with ENOENT, and the same test passes with `--features conformance-cli`. Make the integration lane build the binary with its required feature. | Cargo.toml:29; Makefile:73; tests/it/support/corpus.rs:23-26 |

## Evidence

- Locked default `cargo metadata` excludes `ix-cli-kit` and the feature-enabled metadata includes it in the root production closure. The binary declares `required-features = ["conformance-cli"]`.
- A clean `cargo test --locked --offline --workspace --all-targets --no-run` finished and listed the integration test executable but no conformance binary executable.
- The focused default process test returned exit 101: `No such file or directory` at `tests/it/support/corpus.rs:26`.
- The same focused test with `--features conformance-cli` passed 1/1.
- Repository Rust guidance and the changed code were checked for API, ownership, safety, wire, and test concerns. No other finding was established.

## Dispositions

Same reviewer rechecked the fix at exact against the original failing command.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `make test` now runs `cargo test --workspace --all-targets --features conformance-cli`, building the required binary for the integration suite. The default-only `conformance` and `conformance_mismatch_independence` modules, shared corpus helper, and two direct runner TC-041 tests are gated; the feature lane retains them. The focused default command selects 0 tests and exits 0; the feature command runs the formerly failing test 1/1 PASS. A feature test listing includes 31 conformance or direct runner tests. Locked metadata still excludes `ix-cli-kit` by default and includes it with the feature. |

## Final verdict

**PASS** for the reviewed PR diff; final aggregate gates remain the author's responsibility.
