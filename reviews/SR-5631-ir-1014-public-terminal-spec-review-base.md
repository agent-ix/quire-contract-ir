---
id: SR-5631
title: "IR-1014 PR 563 public terminal export spec review"
type: SpecReview
analysis: base
scope: "IR PR 563: FR-019 Public items checked_package row against FR-062 and the model crate root"
review_set: base
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-062
    type: references
---

# IR-1014 PR 563 public terminal export spec review

## Summary

The one-line table correction adds `CheckedTerminalDuplicate` to the
`checked_package` public-items row and names FR-062 as owner. FR-062 defines
that enum and requires its crate-root export in AC-4. The concurrent IR
code PR 562 defines the public enum in `checked_package/shared.rs` and
explicitly re-exports it from `quire_contract_model`'s root. PR 563 changes
no acceptance criterion and introduces no compatibility layer or copied
artifact. `quire validate --scope .` on FR-019 and `git diff --check` pass.
No applicable `AssuranceProfile` was found in this repository's spec tree.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | At PR 563 head 0b871acc and its current base 23051f55, the model crate root does not export `CheckedTerminalDuplicate`; that export exists only on concurrent unmerged code PR 562. FR-019-AC-5 requires the default-feature public items to equal every unmarked implemented table entry. Integrate this exact table correction into PR 562's code head and close PR 563, or merge PR 562 first and then rebase PR 563; either route keeps the public table and export in one admitted main revision. | FR-019 Public items and AC-5; FR-062 Outputs and AC-4; `crates/quire-contract-model/src/lib.rs` |

## Verdict

PASS for the table text and FR-062 ownership. This spec correction must land
with or after PR 562's exported code; it cannot merge alone against the
current base without violating FR-019-AC-5. Carrying the correction into PR
562 and closing this draft satisfies the finding. Existing strict-coverage
gaps are outside this one-line correction.
