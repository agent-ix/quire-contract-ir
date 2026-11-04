---
id: SR-2026110413592001
title: "IR-1325 PR 890 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 890 at ee68bad80; FR-038-AC-460; crates/quire-contract-model/src/checked_package/{evidence.rs,v2/mod.rs}; tests/it/checked_package_v2_dependency_selections.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1325. Scoped Rust code review of PR #890 against FR-038-AC-460 at frozen head ee68bad80. The repository's Rust conventions and rust-review checklist were applied. No aggregate gate was run.

## Verdict

PASS. The public accessor yields immutable borrows of the retained admitted package values in lock selection order. The returned iterator is exact-size, stays at one dependency edge, and adds no carrier, copy, hash, digest, or compatibility path. Removing the obsolete package_id digest claim matches the current schema and reader behavior.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-460 (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5762`): borrowed direct package values in lock order; distinct non-name content, empty and child/grandchild cases; no copy or surrogate; owner-bound lifetime.
- Implementation (examined, `crates/quire-contract-model/src/checked_package/v2/mod.rs:1234` and `:1327`): admission collects retained `Arc<CheckedPackageV2>` values by iterating lock selections; the accessor maps their immutable slice iterator through `Arc::as_ref`. Rust ties returned references to `&self`.
- Tests (examined, `tests/it/checked_package_v2_dependency_selections.rs:90` and `:136`): source-map end differs between geometry and units; value equality checks ordered output; zero-selection and one-edge traversal cases are asserted.
- Digest-comment correction (examined, `crates/quire-contract-model/src/checked_package/evidence.rs:188`): removes a package_id digest assertion absent from the current wire schema; retains the caller-claim and shared-Arc description.

No independent Cargo run was needed for this static PR review. Focused test and aggregate gate ownership remain with the coding lead.
