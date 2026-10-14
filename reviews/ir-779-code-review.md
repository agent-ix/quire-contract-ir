---
id: SR-10852
title: "Code review — IR-779 admitted guard context"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/contract_package.rs, crates/quire-contract-model/src/checked_package/v2/contract_package/guard_context.rs, crates/quire-contract-model/src/checked_package/v2/contract_package/result_context.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/lib.rs, tests/it/admitted_guard_context.rs, tests/it/main.rs; FR-050 and TC-450"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-050", type: references }
---
## Summary

Reviewed the frozen IR-779 API and TC-450 integration test diff against FR-050, including Rust idioms, test traceability and test-to-criterion alignment. The focused tests and lint checks passed; two medium findings remain for query resource bounds and the test's missing QSL-backed input.

## Verdict

**CONDITIONAL** — the API needs a bound or reuse strategy for repeated guard validation, and TC-450 does not exercise the QSL checked path condition named by AC-1.

## Coverage

- FR-050-AC-1 through FR-050-AC-5 and TC-450 examined; FR-035 and FR-049 read as context.
- `cargo fmt --all --check`: passed.
- `cargo test --locked --test it tc_450_ -- --nocapture`: 4 passed, 0 failed (isolated target under `/tmp/ir779-review/target`).
- `cargo clippy --locked -p quire-contract-model --lib -- -D warnings`: passed.
- `cargo clippy --locked -p quire-contract-ir --test it -- -D warnings`: passed.
- `cargo deny check`: unavailable; the shared advisory DB lock at `/home/peter/.cargo/advisory-dbs/db.lock` is read-only in this sandbox.
- Aggregate CI and Kani were not run, per the review assignment.
- No applicable AssuranceProfile was found under `spec/`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Repeated guards revalidate the entire package without a query work bound | crates/quire-contract-model/src/checked_package/v2/contract_package/guard_context.rs:159-160,225-239 |
| FND-002 | medium | TC-450 supplies synthetic guards instead of the QSL-checked path condition required by AC-1 | tests/it/admitted_guard_context.rs:70-108,298-315 |

## New findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The new 64-guard ceiling rejects QSL-valid checked paths with more than 64 guards | crates/quire-contract-model/src/checked_package/v2/contract_package/guard_context.rs:17-19,172-176 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The query rejects more than 64 guards before entering the per-guard loop, bounding repeated reached-view validation. |
| FND-002 | fixed | TC-450 now reads checked FR-062 RR-13 RequirementItems from QSL and maps each item's exact path-condition occurrence and polarity into admitted IR. |
| FND-003 | still-open | FR-062 RR-13 states path conditions are whatever extents and does not impose a 64-guard maximum. The public query therefore refuses valid checked paths above 64, and the code comment's claimed matching QSL depth bound is unsupported. |
