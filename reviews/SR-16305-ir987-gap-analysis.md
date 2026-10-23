---
id: SR-16305
title: "IR-987 PR 529 acceptance evidence gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@fd8d746e456f6b1aa7afaafae3a501ebec04a382; IR-987 AC-1..4; FR-019-AC-5, FR-023-AC-7, FR-038-AC-139/144/162/163, FR-049-AC-5, FR-053-AC-2; five owner paths and tagged tests in 17 changed Rust files"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-023", type: references }
---
# SR-16305: IR-987 PR 529 acceptance evidence gap analysis

## Summary

Planless, diff-scoped reconciliation of the ticket's four acceptance checks with changed production code and tagged tests. The index design removes the five repeated whole-vector searches by source inspection, and public reader tests exercise normal repeated requests. Two evidence gaps prevent the new tests from establishing the claimed measured and before/after parity properties.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The four performance probes count lookup calls rather than actual graph/source candidates or index builds, so a whole-vector scan reintroduced after the probe would still pass the K-by-R assertions | crates/quire-contract-model/src/checked_package/v2/model_fields.rs:837; crates/quire-contract-model/src/checked_package/v2/scalar_operands.rs:122,274; crates/quire-contract-model/src/checked_package/v2/contract_package/result_context.rs:47; crates/quire-contract-model/src/checked_package/selected_population.rs:41; IR-987 AC-1/2 |
| FND-002 | medium | Repeated public tests take expected values from the changed implementation and omit recorded pre-change work-meter and canonical-byte results for the four paths, so before/after result parity is not independently asserted | tests/it/checked_package_v2_model_fields.rs:549-571; tests/it/admitted_result_context.rs:350-372; tests/it/checked_package_v2_model_members/reaches_population.rs:284-306; IR-987 AC-3 |

## Finding Detail

- FND-001: Each `visited()` or `node_visited()` call is placed once before the keyed lookup, not in a candidate-walk loop or index-construction loop. Replacing the following `BTreeMap::get`/`self.node` with the former `.iter().find` would still produce exactly K (or 3K) counts, including at R=16. Thus the printed `graph_candidates` and `candidates` values are operation counts, not candidate visits. Source inspection shows the current code is indexed, but the new tests cannot detect the stated regression, and no retained one-build count appears for the new source indexes. The ticket's prior red/green table is a claim in a comment; the committed assertions do not encode its before/after measurement.
- FND-002: `expected` is evaluated from the candidate implementation immediately before the repeat loops, then checked against repeats of that implementation. The tests do not include recorded pre-change typed outputs, canonical package bytes, or `consumed` results for the four accessors. Identity/source-map equality catches mutation but cannot establish the ticket's explicit before/after charge and byte comparison. Existing focused error tests remain useful; this finding is limited to the new parity acceptance claim.

## Coverage

- Plan completion: not assessed.
- `quire matrix --scope . --format json` computed all eight examined criteria (FR-019-AC-5, FR-023-AC-7, FR-038-AC-139/144/162/163, FR-049-AC-5, FR-053-AC-2) as `tagged`. Trace tags are present on the changed tests for TC-035, TC-048, TC-227, TC-447 and TC-453. No spec or matrix file changed; the analysis is confined to acceptance evidence for this PR.
- The checked code has owning criteria in FR-023, FR-038, FR-049 and FR-053. FR-019's implemented item inventory includes both enclosing public types, with inherent methods covered by the inventory rule.
- Focused tests and full aggregate gate were not run by this independent review. The author retains the final-head aggregate gate.

## Verdict

CONDITIONAL while FND-001 and FND-002 remain open. Production code inspection found no remaining whole-graph/source scan in the five changed owner paths; the gap is that the tests do not prove the stated measurement and before/after parity.
