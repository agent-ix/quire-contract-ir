---
id: SR-12434
title: "IR-870 PR 455 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_recursive_leaves.rs; tests/it/checked_package_v2_require_bounds.rs; tests/it/checked_package_v2_structural_keys.rs; tests/it/checked_package_v2_temporal.rs; tests/it/checked_package_v2_union.rs; tests/it/complete_v1_checked_package.rs; tests/it/complete_v1_contract_package.rs; tests/it/executable_binding.rs; tests/it/kani_shared.rs; tests/it/output_mapping.rs; tests/it/public_interface.rs; tests/it/source_obligation.rs; tests/it/std001_code.rs"
review_set: subset
---

## Summary

Diff-scoped PR #455 review at the reviewed PR head. The diff deletes 199 Trace/Tracing doc lines and 42 empty doc lines across 13 integration files. Six deleted Trace lines in output_mapping.rs carry NFR-060 or NFR-061, but the same tests’ #[trace] attributes lack those IDs; 193 other deleted trace lines retain all IDs. Ticket: IR-870.

## Reviewed Scope

- `tests/it/checked_package_v2_recursive_leaves.rs` (examined)
- `tests/it/checked_package_v2_require_bounds.rs` (examined)
- `tests/it/checked_package_v2_structural_keys.rs` (examined)
- `tests/it/checked_package_v2_temporal.rs` (examined)
- `tests/it/checked_package_v2_union.rs` (examined)
- `tests/it/complete_v1_checked_package.rs` (examined)
- `tests/it/complete_v1_contract_package.rs` (examined)
- `tests/it/executable_binding.rs` (examined)
- `tests/it/kani_shared.rs` (examined)
- `tests/it/output_mapping.rs` (examined)
- `tests/it/public_interface.rs` (examined)
- `tests/it/source_obligation.rs` (examined)
- `tests/it/std001_code.rs` (examined)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Six removed `Tracing:` lines contain NFR-060 (five) or NFR-061 (one), absent from the corresponding test's `#[trace]` attribute. Add those IDs to the six existing attributes to preserve the claimed bindings. | `tests/it/output_mapping.rs`: base lines 296, 732, 1254, 1369, 1648, 1749; head test lines 296, 729, 1248, 1362, 1637, 1737 |

## Verdict

FINDING — Six cross-repository NFR bindings would be lost from source trace claims. No executable test-body changes were found.

## Coverage

Plan completion: not assessed
Before and after local IR coverage: 527/596 backed, strict 40 unbacked, 0 contradicted. The IR coverage report does not measure the external QSpec NFR-060/061 bindings implicated by FND-001.
Computed matrix: 562 criterion rows, normalized status and binder identity identical to base.
Diff and formatting: `git diff --check` and `make fmt-check` passed. Author reports full Cargo gates on pre-rebase doc-only tree.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | PR #455 adds NFR-060 to five same-test `#[trace]` attributes and NFR-061 to one. Independent all-prefix audit: 199 removed trace lines, 415 ID occurrences, zero missing from the same-function attributes. |
