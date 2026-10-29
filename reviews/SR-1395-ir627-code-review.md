---
id: SR-1395
title: "PR 627 refinement exports: code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/lib.rs; FR-019-AC-5; tests/it/public_interface.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
---
# SR-1395: PR 627 refinement exports: code and Rust review

## Summary

PR agent-ix/quire-contract-ir#627 exports fifteen existing, documented refinement body types by name from the model crate root. The types are the key, population, field, object, step, open, history, assumption, ensure, fairness, and granularity members needed to construct a typed `CheckedTerm::Refinement`; no helper or second implementation is introduced. The review covers the one changed Rust file and its exact public API contract. No workflow file changes.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The fifteen new root exports are missing from FR-019's exact Public items table. FR-019-AC-5 and TC-058 require the table and exported set to be equal; hosted Rust Checks at this head fail `public_interface::tc_058_model_root_exports_exactly_the_documented_public_items`. Add the types to the table in this PR without weakening the inventory test. | `crates/quire-contract-model/src/lib.rs:187-194`; `spec/model/functional/FR-019-rust-library-interface.md:150-166`; `tests/it/public_interface.rs` | missing-requirement |

## Verdict

**FAIL.** FND-001 blocks merge. The hosted test result is direct evidence; focused `cargo fmt --all --check` passes. The hosted test compilation also establishes that the crate compiles at this head. No additional Cargo build was run while the shared build lock is occupied.

## Dispositions

- **FND-001 — fixed:** The same PR now lists all fifteen refinement body exports in FR-019's `checked_package` Public items table. The original finding and gate result above describe the reviewed head; the final PR head requires a fresh TC-058 gate result.
