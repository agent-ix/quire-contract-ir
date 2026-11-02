---
id: SR-202611010315
title: "Code review of carried quantity reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1256; crates/quire-contract-model/src/checked_package/v2/; tests/conformance_qspec/carried_quantity.rs"
review_set: subset
---
## Summary

Reviewed the frozen carried quantity reader and Rust test diff against FR-038-AC-401 through AC-405 and FR-019-AC-20. The source changes follow the specified admission and conversion boundaries. Later CI gates found missing string-edge annotations and an outdated exhaustive operand-family expectation; both fixes are recorded below.

## Verdict

PASS after disposition of both CI gate findings. The initial source review and `cargo fmt --check` passed; the lead owns the final aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Examined the changed Rust files in `crates/quire-contract-model/src/checked_package/v2/`, including vocabulary, shape, operand classification, operation checks, lowering, identity and composite traversal. Checked the strict reader's use of bounded carrier forms, mode pinning, same-dimension conversion, operand exclusion, typed refusal loci, public vocabulary and absence of evaluation outcomes or copied fixtures.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-002 | medium | The quantity operation identity comparisons lacked required string-edge annotations, causing the Rust Checks string-edge gate to fail. | crates/quire-contract-model/src/checked_package/v2/operations.rs:682 | implementation-bug-despite-evidence |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | The three functions containing the five reported comparisons now carry the repository's `#[ix_trace_rs::string_edge]` annotation. The fix changes no expressions or control flow; the lead reports the focused string-edge gate passed. |
| FND-003 | fixed | The exact full-vocabulary expectation now includes the carried quantity form in vocabulary order with family `quantity`; the test still compares the entire produced list, and no production code changed in this slice. |

## Disposition evidence

The CI fix slice adds only three function annotations in the checked-package operations module. They cover the conversion identity checks and the quantity operation prefix check. `git diff --check` passed. The lead reported passing focused string-edge, formatting, CQ conformance and scoped Clippy checks. This reviewer did not rerun aggregate gates.


## New findings (disposition pass 2)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-003 | medium | The full-vocabulary operand classification test omitted the new carried quantity form, causing the hosted model-library test to fail. | crates/quire-contract-model/src/checked_package/v2/operations.rs:6128 | implementation-bug-despite-evidence |

## Disposition pass 2 evidence

The fix slice adds exactly one expected tuple to `operand_classification_is_exactly_the_catalog_mapping`. The test still compares the complete `CheckedNodeKind::all()` family list in vocabulary order and separately checks type-shaped classification for every kind. Production classification and carried-operand refusal logic were unchanged. `git diff --check` passed. The lead reports passing focused model unit, CQ conformance, model Clippy, formatting and string-edge checks.
