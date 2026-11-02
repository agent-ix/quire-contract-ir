---
id: SR-202611010315
title: "Code review of carried quantity reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1256; crates/quire-contract-model/src/checked_package/v2/; tests/conformance_qspec/carried_quantity.rs"
review_set: subset
---
## Summary

Reviewed the frozen carried quantity reader and Rust test diff against FR-038-AC-401 through AC-405 and FR-019-AC-20. The source changes follow the specified admission and conversion boundaries. A later CI gate found missing string-edge annotations; the fix is recorded below.

## Verdict

PASS after disposition of the string-edge gate finding. The initial source review and `cargo fmt --check` passed; the lead owns the final aggregate gate.

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

## Disposition evidence

The CI fix slice adds only three function annotations in the checked-package operations module. They cover the conversion identity checks and the quantity operation prefix check. `git diff --check` passed. The lead reported passing focused string-edge, formatting, CQ conformance and scoped Clippy checks. This reviewer did not rerun aggregate gates.
