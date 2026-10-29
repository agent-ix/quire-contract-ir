---
id: SR-610050
title: "IR-1032 PR 612 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-063-AC-5, changed reader and nominal identity implementation and changed tests"
review_set: subset
relationships:
  - type: references
    target: ix://agent-ix/quire-contract-ir/FR-063
---
# SR-610050: IR-1032 PR 612 gap analysis

## Summary

Ticket: IR-1032. Initial planless code-to-test-to-criterion audit at PR #612 exact. Disposition is PASS: the graph and precedence cases are now AC5-tagged public-reader tests. A subsequent is a Clippy-only `&at` to `at` adjustment with no test or semantic change. FR-063-AC-5 is tagged on the two new dimension-equivalence tests and the changed model-owner test. The tests reach the public `CheckedPackageV2::read` path; they are not hollow. The new production branches are owned by FR-063-AC-5. The remaining graph and precedence clauses are not asserted by a test bound to AC5. The semantic intent review is part of the assigned AC check; no repository-wide optional semantic sweep was run.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-063-AC-5's graph half and check-order clause have no AC5-tagged regression: the new tests exercise only wire refusals, and the pre-existing equivalence tests are tagged AC2/AC4, not AC5. Add public-reader cases with a schema-valid unreduced `root_scale`, one missing dependency, non-self `semantic_type`, unselected source owner and retained stale key after pair/scale mutation, plus one independent earlier wire defect to prove precedence. Assert code, cause and contracted location. | tests/it/checked_package_v2_dimension_equivalence.rs:200-269 | correct-requirement-no-evidence |

## Verdict

PASS at reviewed. The initial review was CONDITIONAL because the graph and precedence clauses lacked AC5-tagged evidence; the Dispositions section records their added tests. Plan completion: not assessed.

## Coverage

Examined FR-063-AC-5 as the changed obligation, QSpec FR-322-AC-139 as its upstream contract, and all changed tests and production files. `quire coverage --scope . --strict --json` at this head reports 85 unbacked rows and zero status lies; FR-063-AC-5 is mechanically backed by the three tagged tests, but the semantic gap above remains. No plan supplied. Plan completion: not assessed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, `equivalence_graph_defects_refuse_at_the_node_after_wire_checks` covers unreduced ratio, one missing dependency, non-self semantic type, unselected owner, stale scale and stale pair through public `CheckedPackageV2::read`, asserting code, cause and node pointer. `equivalence_wire_defect_precedes_independent_graph_defect` combines zero numerator with non-self type, observes wire refusal first, repairs only the wire defect and observes graph refusal. Both carry the AC5 trace. Author reports 11/11 focused tests passed. |
