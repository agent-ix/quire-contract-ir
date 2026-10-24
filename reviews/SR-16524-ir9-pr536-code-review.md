---
id: SR-16524
title: "IR-9 PR 536 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@ba543f3f2000eeef22a6b60e0deb1f616486eea0; crates/quire-contract-model/src/checked_package/v2/scalar_operands.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
---
# SR-16524: IR-9 PR 536 code and Rust review

## Summary

Independent diff-scoped review of the `type_range` retained-kind decision and three private tests. The code is confined to `scalar_operands.rs`; no public API, wire format, dependency, scanner rule, marker, copied artifact, or CI file changes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope examined

- `ScalarArguments::project` passes the already retained kind slice to `type_range`. `graph_node` verifies the indexed node identity; `kinds.get(position)` then selects the kind retained for that same graph position. Reader admission pushes a decoded kind in graph order, and both lowered-package constructors retain that order.
- `ScalarType(Integer)` still gives `UnboundedRange`; `BoundedDomain(IntegerRange)` still reads the retained bound term; another or absent kind gives `MissingRange`. The checked accessor retains first-error order; the lowered accessor still maps those three own-range states separately. There is no new panic, narrowing conversion, allocation, unbounded walk, or unsafe operation.
- The two inverse damaged-state controls each call the real checked-package accessor and distinguish this implementation from the previous raw tag/form branch. The third control exercises lazy per-position projection and distinct `UnboundedRange`, `MissingRange`, and `RangeOutOfI128` results. These tests do not claim reader admission for inconsistent private states.
- Existing public scalar tests cover admitted finite bounds, graph literal singleton, unbounded integers, out-of-`i128` endpoints, missing child, occurrence, and operator eligibility. `git diff --check` passed. The final aggregate gate belongs to the author after review.

## Verdict

PASS for this exact diff. The repository-wide string-edge scan remains unresolved, and the external scanner was unavailable on this review host.
