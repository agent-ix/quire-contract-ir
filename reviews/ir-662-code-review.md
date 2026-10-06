---
id: SR-2190
title: "Code and Rust review of IR-662 numeric width change"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@a352c4509f155c6de1b430501a3e9b211283a2d2; Cargo.lock; crates/quire-contract-model/Cargo.toml; crates/quire-contract-model/src/{canonical,decimal,expression,wire}.rs; scripts/generate_conformance_corpus.py; corpus/contract-v0.1/inputs/expression-invalid-numeric-range.json; tests/it/{canonical_v1_quire_canonical,expression}.rs"
review_set: subset
---

## Summary

Ticket: IR-662. Reviewed the frozen PR #311 diff with the Rust idiom checklist, including wire decoding, canonical strings, interval arithmetic, rational normalization, tests, the corpus, and the dependency change. No defect was found in the changed code.

## Verdict

**PASS** — the widened public fields preserve the stated i128 bounds, checked integer arithmetic uses mathematical endpoints, and exact rational arithmetic reduces before conversion back to i128. Focused checks passed; the full pre-merge gates belong to the merger.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope and evidence

- Examined FR-013-AC-5 and FR-013-AC-6: each of the eight decimal-string members, one-past-i128 refusal, and wide constructor/accessor payloads. The changed `decimal.rs`, `wire.rs`, and `expression.rs` cases cover these boundaries.
- Examined FR-014-AC-8 and FR-015-AC-8: typed literal retention, i128 minimum negation and division, saturation, exact rational cancellation, and out-of-bounds refusal. The changed integration and unit tests exercise these paths.
- Examined FR-016-AC-9 and FR-019-AC-7: decimal-string canonical spelling and request admission. The changed canonical test and wire test bind those criteria.
- Examined the corpus input and generator together: both place the invalid bound at one past `i128::MAX`; the expected diagnostic is unchanged.
- Examined the manifest and lockfile: the new `num-bigint`, `num-integer`, and `num-traits` dependencies are used by the changed arithmetic and are not exact-pinned.
- Examined the PR's downstream-consumer inventory as an integration risk. CG and QSL retain typed i64/u64 call sites and have separate adaptation work; no compatibility shim is part of this PR.

`cargo fmt --all -- --check` passed. `cargo clippy -p quire-contract-model --all-targets -- -D warnings` passed. `cargo test -p quire-contract-model` passed (176 unit tests, 17 doc tests; one existing ignored performance test). `cargo test --test it wide_` passed all three changed wide-value integration tests. CI workflow files have no diff. The repo's full gates were outside this independent diff review.
