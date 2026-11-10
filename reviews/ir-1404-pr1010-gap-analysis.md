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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-003 | medium | FR-038-AC-511 and UNPAID_BOUND's doc call the unpaid refusal diagnostic constant and independent of the input, but no overage was probed at two sizes, and the diagnostic's pointer text and node clone grow with member-name and node-id length. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6530, tests/it/support/read_observation.rs:50-54, crates/quire-contract-model/src/checked_package/common.rs:296-304 | correct-requirement-no-evidence |
| FND-004 | medium | The FR-038-AC-511 row in tests.md still presents the whole-read test as the corpus-wide proof over the QSpec packages after the provider-completeness test was deleted with 43 complete-package recipes unregistered, and still says five tests need the checkout; four do. | spec/checked_package/matrix/tests.md:20, tests/it/checked_package_v2_whole_read_accounting.rs:1430 | wrong-requirement |
| FND-005 | low | FR-038-AC-511 has a missing sentence break: "the size of the input The bound is headroom". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6530 | wrong-requirement |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at 2ac175217: gate text asserted at both the root re-export and the defining module |
| FND-002 | fixed | Verified at 2ac175217: FR-028-AC-5 names the permitted shapes |
| FND-003 | still-open | New this round: the constant claim is unshown |
| FND-004 | still-open | New this round: the row overstates the corpus proof |
| FND-005 | still-open | New this round: typo in the amended criterion |
| FND-003 | still-open | Spec still generalizes from two population families, and long names can exceed the stated bound |
| FND-004 | fixed | Verified at 8f9659178 |
| FND-005 | fixed | Verified at 8f9659178 |
| FND-003 | fixed | Verified at 591c893d6: AC-511 is scoped to the shapes and names exercised |


## New findings (disposition pass 4)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-006 | medium | The new AC-511-tagged denial control has no assertion distinguishing a budget denial from an ordinary refusal. It can pass on a malformed-wire result before the destination-type read, so it supplies no evidence that this changed denial path reaches its typed budget outcome. | tests/it/checked_package_v2_model_members.rs:4354-4361 | correct-requirement-no-evidence |

## Continuation review (delta 591c893d..17dcb1808)

**Delta verdict: CONDITIONAL** — the changed allocations are tied to the existing strict-read contract and the abstraction-relation accounting test uses the real reader and allocation proof, but the added relationship-destination test does not establish its claimed typed-denial result.

This was a scoped gap analysis of the three-commit delta only. Examined units: FR-038-AC-510 (budget-denial outcome), FR-038-AC-511 (whole-read accounting); the new relationship-destination denial test; the new abstraction-relation whole-read accounting test; and the production functions changed in `flat_wire.rs`, `v2/mod.rs`, `term_view.rs`, and `typed_terms.rs`. The changed tests carry the FR-038-AC-511 trace tag; the allocation proof reaches the real strict reader. No repo-wide matrix or source inventory was run because it would exceed the requested delta. No claim is made that AC-511's whole corpus is qualified. Plan completion: not assessed. The coder's focused results were accepted as reported and not rerun. No spec or plan files changed; semantic review was not requested.
| FND-006 | still-open | New in continuation review pass 3; the denial control must distinguish the typed budget outcome from an ordinary refusal before it can evidence this changed path. |

| FND-006 | fixed | Verified on 487d43b4b: the test now requires `BudgetIncomplete` or `Exhausted` with `ReadStorage`, checks the denied amount and locus, and panics on every other result. |

### FND-006 after_excerpt (round 5)

```rust
            other => panic!("limit {limit} did not produce a typed budget denial: {other:?}"),
        };
        assert_eq!(
            budget.first_unpaid_read_storage(),
            Some(denied.amount),
            "limit {limit} preserves the original denied read-storage request"
        );
```
