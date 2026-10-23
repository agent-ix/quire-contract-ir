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

## Dispositions

At PR #529 head `5a486dced38383d8f299992938d7c285f0e5b0ba`, same-reviewer round 1 inspected the exact 11-file delta from the original reviewed head and the author's pre-change baseline evidence comment `08d58aac`. No author branch edits or full gates were made by this reviewer.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The new `BUILDS` and `BUILD_ENTRIES` probes do count the two source-index construction loops, but each read probe remains after `entries.get(position)` or `nodes.get(position)` (and the model probe after `self.node`). Replacing that read with `entries.iter().find(...).inspect(...)` or `nodes.iter().find(...).inspect(...)` still increments once for the returned element, so the K-by-R assertions continue to pass while the whole-vector candidate scan regresses. The probes count successful returned items, not all candidate visits. |
| FND-002 | still-open | The added base-run comment and fixed current-branch assertions provide provenance and admission work-boundary numbers, but the tests hash `format!("{result:?}")` and canonical bytes into new SHA256 snapshot constants instead of comparing complete typed results and canonical bytes directly. This is a new use of tracking digests prohibited by this repository's AGENTS.md and CLAUDE.md (only canonical proof identity digests are retained). A Debug digest also does not assert individual typed fields or source occurrence order, so the required complete result parity is still not established. |

## Round 1 verdict

CONDITIONAL; both findings remain open. The newly instrumented build counts and recorded base-run values are useful partial evidence, but the review cannot treat the candidate-visit and direct parity requirements as satisfied on this head.
