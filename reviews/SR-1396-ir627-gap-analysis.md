---
id: SR-1396
title: "PR 627 refinement export gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@796609b0ce70e2c2a9367d3df8d4e20e43166ba6; FR-019-AC-5, TC-058, crates/quire-contract-model/src/lib.rs, tests/it/public_interface.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
---
# SR-1396: PR 627 refinement export gap analysis

## Summary

Plan completion: not assessed. The changed public exports have an owning requirement, FR-019-AC-5, and a live inventory test, TC-058. There is no new behavior or test trace to add. The exact inventory check catches the current code/spec mismatch. This method records the same single blocker as the code review, without adding a second independent defect.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The FR-019 Public items table omits all fifteen new root exports; TC-058 fails on the checked PR head. The new public surface is therefore outside its stated exact inventory until the table is updated. | `spec/model/functional/FR-019-rust-library-interface.md:150-166`; `crates/quire-contract-model/src/lib.rs:187-194`; `tests/it/public_interface.rs` | missing-requirement |

## Verdict

**FAIL.** FND-001 requires a same-PR spec update and a passing TC-058 before merge. The broader repository's existing unbacked criteria are outside this one-file PR's change; no coverage claim is inferred from them.

## Dispositions

- **FND-001 — fixed:** FR-019's Public items table now includes the fifteen exported refinement body types in this PR. The exact-inventory TC-058 result must be taken from the final head before merge.
