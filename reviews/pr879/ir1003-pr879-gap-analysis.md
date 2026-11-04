---
id: SR-20261103150665
title: "IR-1003 PR 879 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 879; tests/conformance_qspec/timed_model.rs; FR-038-AC-319 and FR-038-AC-321"
review_set: subset
relationships:
  - type: references
    target: ix://agent-ix/quire-contract-ir/FR-038
---

## Summary

Scoped planless acceptance-criteria-to-tagged-test-to-reader audit of PR #879. Quire computes FR-038-AC-319 and AC-321 as tagged. The published positive fixture has source, place, reset, guard, invariant, urgency, constraint, and root timed forms. The PR adds no production behavior or reverse code ownership gap.

## Verdict

**CONDITIONAL.** A medium AC-319 gap remains despite the tagged matrix row. No repository-wide PASS is claimed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Graph-shape adverse controls cover only the clock-place node, not the other named timed node forms. | tests/conformance_qspec/timed_model.rs:1985 |

The published positive package is admitted and so checks its current dependencies, but every new stale-dependency and dangling-reference mutation targets node 45 (`clock_place`). A reader change that skips exact dependency or reference checking on `time_source`, `clock_reset`, `clock_guard`, `time_invariant`, `urgency`, or `clock_constraint` could pass all new tests. Add representative adverse mutations for those named forms, with exact graph-shape refusal code and locus before timed interpretation, while retaining the admitted positive control.

## Coverage

- FR-038-AC-319 (role: examined; excerpt): For a timed model with source, place, reset, guard, invariant, urgency and constraint nodes, graph shape checks exact dependencies and reference resolution before timed-model interpretation. A meaning-bearing body or owner edit is validated on its own terms; a stale dependency list refuses at graph shape. Reordering source occurrences does not alter structural ids.
- FR-038-AC-321 (role: examined; excerpt): A package with both a timed-model defect and a protocol or operation defect reports the timed defect; one with a hybrid defect and a timed defect reports the hybrid defect. Strict-wire and graph-shape defects continue to precede either body step.
- Computed Quire matrix: AC-319 tagged by `timed_source_occurrence_reordering_retains_structural_node_ids` and `timed_graph_shape_and_body_edits_refuse_at_their_own_stage`; AC-321 tagged by `timed_model_stage_precedence_has_independent_defect_controls`. Semantic inspection found the adverse control gap above and the code-review gaps in the companion artifact.
- Reverse gap: no production code changed. Stub and vendoring checks: no new stub or copied fixture; tests consume the QSpec checkout.
- Plan completion: not assessed

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The fix adds stale-dependency controls for every published timed form, resolving that part. Independent dangling-reference controls for those forms remain absent, and source-reference behavior still reaches timed validation. The coder removed both AC-319 trace tags; Quire computes AC-319 as untagged, so this PR no longer claims its completion. AC-319 remains planned for a later reader fix and evidence. |

## Disposition verdict

Round 1, PR head 408b3629b4ea1af366f70bd76ca1b9e6f76aacd8: **PASS for the remaining claimed FR-038-AC-321**. The deferred AC-319 gap is explicit and no longer concealed by a tagged matrix row. Plan completion: not assessed.
