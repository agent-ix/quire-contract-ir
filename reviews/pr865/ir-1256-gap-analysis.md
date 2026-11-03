---
id: SR-20261103150655
title: "Gap analysis of IR-1256 rational carried quantity evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-038-AC-450..452; tests/conformance_qspec/carried_quantity.rs; QSpec FR-322-AC-382 and FR-142-AC-31"
review_set: subset
---

## Summary

Compared the changed FR-038 criteria with the computed test bindings, the QSpec vector harness and the reader. Plan completion: not assessed.

## Verdict

**CONDITIONAL** — the vector test proves admission, refusal codes and pointers, and typed carried bindings, but does not inspect retained rational bounds in the admitted graph.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-450's retained rational numerator and denominator bounds have no assertion on the admitted carrier node | tests/conformance_qspec/carried_quantity.rs:43 |

## Coverage

The test reads every published QSpec wire and evaluation case by reference, requires CQ-W16..W19 and CQ-E21..E26 without freezing the complete inventory, and checks admitted carried bindings and exact refusal code, cause and primary pointer. The AC-450..452 trace tags are present on the vector test. The missing bound assertion means the test could pass if a future reader changed the admitted carrier's bound terms after validation.

Plan completion: not assessed

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The revised reviewed diff adds W16/W18 assertions on the admitted `RationalRange` node's typed aggregate. The test compares all four bound names, literal kinds and values with the referenced QSpec node; the focused QSpec test passes on this head. |
