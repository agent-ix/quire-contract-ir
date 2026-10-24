---
id: SR-16629
title: "Code and Rust review — IR-645 private numeric pairing"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-645; PR #556 expression.rs and expression integration test"
review_set: subset
---

## Summary

Diff-scoped code and Rust review of PR #556 at 6f8f2248. The private checked value has closed integer and rational variants carrying the same-kind concrete type and range. Typed binary operator operands preserve that pairing; unary negation matches a paired variant before using its range. The public checker signature and diagnostic paths remain unchanged.

## Verdict

**PASS** — no code or Rust finding in the reviewed diff. This is not a final-gate result or qualified Inspection credit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- FR-014-AC-9 (examined): the type and range pairing, private constructors, unary and binary dispatch, public signature, and invalid-operand diagnostic contract.
- TC-444 (examined): source guard and the required manual inspection procedure. The guard is supplemental and does not qualify Inspection evidence.
- FR-014-AC-7/8 (context only): existing bounded range behavior and wide literal behavior.

## Evidence

`CheckedValue::Integer` owns `IntegerType` and `Vec<(i128,i128)>`; `CheckedValue::Rational` owns `RationalType` and `RationalOperand`. `from_type`, literals, refinement, child constructors, binary result construction, and negation construct these variants directly. `CheckedInteger` and `CheckedRational` take/return same-kind fields, and the binary operator functions consume them. Negation matches the closed paired variants before reading or returning a range. The `Checked` fields no longer include independent `value_type` and `range`. The source guard tests the representation and both binary signatures; its limited scope is accurately disclosed. Focused tests 15/15 and scoped Clippy/fmt are author-reported, not rerun as an aggregate gate.
