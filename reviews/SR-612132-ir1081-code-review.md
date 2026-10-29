---
id: SR-612132
title: "IR-1081 PR 648 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1081; PR #648; Cargo.toml, tests/tc019_determinism.rs; NFR-001-AC-2/3, TC-019"
review_set: subset
---

## Summary

Reviewed the opt-in TC-019 capture and comparator on PR #648. The new target executes the real corpus twice, compares captured canonical bytes and ordered diagnostic tuples, and has mutation controls. No CI workflow, copied dependency, public library API, or applicable AssuranceProfile changes in this diff. The three-OS evidence remains open.

## Verdict

CONDITIONAL. The capture's OS provenance can be mislabeled; the new Rust file also omits the repository's source header. Neither finding narrows NFR-001-AC-2/3 or credits TC-019.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `capture` accepts a caller-supplied OS label without comparing it to the executing host, so three Linux executions labeled linux, macos and windows would pass the comparator as cross-platform evidence. Require the actual host OS to match the capture label. | tests/tc019_determinism.rs:121 |
| FND-002 | low | The new Rust source lacks the SPDX and copyright header used by this repository's Rust files and required by the Rust style checklist. | tests/tc019_determinism.rs:1 |

## Coverage

- Examined `Cargo.toml` and all of `tests/tc019_determinism.rs`, with the corpus runner and NFR-001/TC-019 contracts as context.
- The target is explicit and excluded from the ordinary Cargo test lane. The comparator refuses changed canonical bytes, ordered diagnostics, OS identities, provenance, and fixture inventories on the production path; `self_test` exercises six mutations through `compare_values`.
- Focused test and aggregate gate execution were left to the author/code lead because the shared build lock was in use. No cross-platform run or Analysis evidence binding was observed on this PR.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed |, `capture` refuses `os != env::consts::OS` before running the corpus; a self-test supplies a different supported OS and checks that refusal. |
| FND-002 | fixed |, the file begins with the repository's SPDX and copyright header. |
