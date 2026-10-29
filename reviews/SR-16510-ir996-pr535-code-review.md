---
id: SR-16510
title: "IR-996 PR 535 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/{scalar_operands.rs,contract_package/scalar_operands.rs}; tests/it/admitted_scalar_operands.rs"
review_set: subset
---
# SR-16510: IR-996 PR 535 code and Rust review

## Summary

Independent diff-scoped review of the scalar argument kind projection in both admitted package views and the changed tests. No public API, wire schema, dependency, lock, spec, marker, or scanner rule changes appear in the PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope examined

- `ScalarArguments::project` resolves a referenced child through the existing `node_index` and reads the retained `CheckedNodeKind` at the same position. A `Value(Literal)` uses its retained term for the singleton range; other child kinds keep the existing type-range path.
- `CheckedPackageV2::scalar_application_operands` passes its admitted `kinds`; `ContractPackageV1::scalar_operands` passes `node_kinds`, which the package producer and strict reader both build in graph-node order. Missing private state becomes `MissingRange` without indexing panic.
- The private parameter/literal metadata mutations distinguish retained-kind dispatch from the old raw-spelling branch and assert candidate counts. The reader-admitted graph literal integration test exercises both public package views, exact singleton range, graph child identity, sources, argument order and unchanged wire.
- Existing arbitrary-integer, `i128`, unbounded-range, occurrence, refusal, lazy first-error, work-count and golden controls remain present. No new vendored content, unchecked narrowing, unsafe operation, public item or gate weakening was found.
- `git diff --check` passed. The author's focused test, Clippy and formatting logs are on IR-996; the final aggregate gate is not part of this review.

## Verdict

PASS for the exact PR diff. Repository-wide scanner cleanliness and the final gate are separate claims.
