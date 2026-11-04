---
id: SR-20261103150664
title: "IR-1003 PR 879 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 879; tests/conformance_qspec/timed_model.rs; FR-038-AC-319 and FR-038-AC-321"
review_set: subset
relationships:
  - type: references
    target: ix://agent-ix/quire-contract-ir/FR-038
---

## Summary

Ticket: IR-1003. Scoped review of PR #879 at 73f7f17e0a847c8dda46935dcff95768b9659ec4. The Rust lane inspected the three new public-reader tests, their published QSpec fixture, reader stage order, and computed Quire bindings. No production, specification, CI, or dependency file changed. No applicable AssuranceProfile was found under spec/.

## Verdict

**CONDITIONAL.** The AC-321 hybrid and protocol controls and the strict-wire and graph-shape controls assert independent defects and exact first loci. The AC-319 source-occurrence permutation compares every structural node id after admission. Two medium evidence gaps remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Owner edit is refused by strict wire validation, leaving semantic owner validation untested. | tests/conformance_qspec/timed_model.rs:2053 |
| FND-002 | medium | Timed-versus-operation precedence is untested by the protocol-clause control. | tests/conformance_qspec/timed_model.rs:1864 |

The AC-319 owner control adds an `owner` object to published node 45, a normally ownerless `clock_place`; the expected `MalformedWire` at `/owner` shows strict wire refusal, not a valid meaning-bearing owner edit checked on its own semantic terms. Use a timed node with a valid owner shape and change the owner identity or selected owner so a later validation stage adjudicates the edit.

The AC-321 `protocol` control mutates node 9, whose tag/form is `protocol`/`protocol_clause`. Its `OperationLawMismatch` is reported from the protocol body path. The test never combines the timed defect with an independent operation-node defect, so the `or operation` precedence clause could regress while this test passes. Add a valid wire operation-node body defect and assert its independent refusal and timed-first paired refusal.

## Coverage

- FR-038-AC-319 (role: examined; excerpt): For a timed model with source, place, reset, guard, invariant, urgency and constraint nodes, graph shape checks exact dependencies and reference resolution before timed-model interpretation. A meaning-bearing body or owner edit is validated on its own terms; a stale dependency list refuses at graph shape. Reordering source occurrences does not alter structural ids.
- FR-038-AC-321 (role: examined; excerpt): A package with both a timed-model defect and a protocol or operation defect reports the timed defect; one with a hybrid defect and a timed defect reports the hybrid defect. Strict-wire and graph-shape defects continue to precede either body step.
- Computed `quire matrix --scope . --format json` marks both criteria tagged to the three added functions. The tags identify relevant tests but do not establish full semantic coverage.
- Focused review was read-only; aggregate gates and compilation are owned by the lead on the final reviewed head.
