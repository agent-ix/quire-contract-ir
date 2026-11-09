---
id: SR-2026110909552102
title: "IR-1404 PR 1010 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 1010 diff at a12ad8b91, FR-019-AC-16, FR-019-AC-27, FR-028-AC-5"
---

# IR-1404 PR 1010 gap analysis

## Summary

Ticket: IR-1404. Scoped to PR 1010's diff. Plan completion: not assessed. The computed matrix
was not run; the criteria touched by the diff were checked by hand against their tagged tests.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The new AC-16 clause on the budget observer has no test. The inventory now exempts the four names by name without regard to cfg, tc_058 asserts the feature gate only for the fixture export, and neither AC-16 test names the observer. | tests/it/public_interface.rs:944-951, tests/it/public_interface.rs:1597, tests/it/public_interface.rs:1654-1667 | correct-requirement-no-evidence |
| FND-002 | medium | FR-028-AC-5 still forbids any callback or trait object, while the scan tracing it now allows plumbing callbacks and dyn ReadStorage. Spec and test disagree on what is enforced. | spec/model/functional/FR-028-separate-cycle-free-contract-model.md:69, tests/it/cycle_free_model.rs:745-782 | wrong-requirement |

## Verdict

Two gaps to close, both small. FR-019-AC-27's seven types match the lib.rs exports and FR-361
exactly, so that inventory addition is intended. The observer exports are feature-gated today
at lib.rs:342 and v2/mod.rs:179-182; the exemption hides nothing at this head, but nothing would
fail if both gates were removed.
