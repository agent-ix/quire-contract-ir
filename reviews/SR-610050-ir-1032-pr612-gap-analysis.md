---
id: SR-610050
title: "IR-1032 PR 612 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@90bbf5f1f317431813b44d1c248473bf2ef041fb; FR-063-AC-5, changed reader and nominal identity implementation and changed tests"
review_set: subset
relationships:
  - type: references
    target: ix://agent-ix/quire-contract-ir/FR-063
---
# SR-610050: IR-1032 PR 612 gap analysis

## Summary

Ticket: IR-1032. Planless code-to-test-to-criterion audit at PR #612 exact head. FR-063-AC-5 is tagged on the two new dimension-equivalence tests and the changed model-owner test. The tests reach the public `CheckedPackageV2::read` path; they are not hollow. The new production branches are owned by FR-063-AC-5. The remaining graph and precedence clauses are not asserted by a test bound to AC5. The semantic intent review is part of the assigned AC check; no repository-wide optional semantic sweep was run.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-063-AC-5's graph half and check-order clause have no AC5-tagged regression: the new tests exercise only wire refusals, and the pre-existing equivalence tests are tagged AC2/AC4, not AC5. Add public-reader cases with a schema-valid unreduced `root_scale`, one missing dependency, non-self `semantic_type`, unselected source owner and retained stale key after pair/scale mutation, plus one independent earlier wire defect to prove precedence. Assert code, cause and contracted location. | tests/it/checked_package_v2_dimension_equivalence.rs:200-269 | correct-requirement-no-evidence |

## Verdict

CONDITIONAL. The wire mutations have direct public-reader evidence; the stated graph and precedence cases are unbacked in AC5's changed test slice. Plan completion: not assessed.

## Coverage

Examined FR-063-AC-5 as the changed obligation, QSpec FR-322-AC-139 as its upstream contract, and all changed tests and production files. `quire coverage --scope . --strict --json` at this head reports 85 unbacked rows and zero status lies; FR-063-AC-5 is mechanically backed by the three tagged tests, but the semantic gap above remains. No plan supplied. Plan completion: not assessed.
