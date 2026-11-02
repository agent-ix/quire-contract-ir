---
id: SR-202611010315
title: "Code review of carried quantity reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1256; crates/quire-contract-model/src/checked_package/v2/; tests/conformance_qspec/carried_quantity.rs"
review_set: subset
---
## Summary

Reviewed the frozen carried quantity reader and Rust test diff against FR-038-AC-401 through AC-405 and FR-019-AC-20. The source changes follow the specified admission and conversion boundaries; the evidence gaps are recorded in the separate gap analysis.

## Verdict

PASS for source review. `cargo fmt --check` passed. Large build and aggregate gates were reserved for the lead's final reviewed head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Examined the changed Rust files in `crates/quire-contract-model/src/checked_package/v2/`, including vocabulary, shape, operand classification, operation checks, lowering, identity and composite traversal. Checked the strict reader's use of bounded carrier forms, mode pinning, same-dimension conversion, operand exclusion, typed refusal loci, public vocabulary and absence of evaluation outcomes or copied fixtures.
