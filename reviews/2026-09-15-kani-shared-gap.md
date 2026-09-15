---
id: SR-042
title: "Gap analysis of shared bounded Kani foundation"
type: SpecReview
analysis: gap-analysis
scope: "#82; src/kani; tests/kani_shared.rs; FR-029 through FR-031; TC-042"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/82
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TM-002
    type: references
---
# SR-042: Gap analysis of shared bounded Kani foundation

## Summary

The #82 shared foundation implements the K0 profile/matrix, finite ABI
validation, typed outcomes, provenance identity, and non-overlapping dispatch
index. `quire coverage` recognizes the five TC-042 tests and backs all
#82-owned criteria; no unowned implementation symbol or completion stub was
found. Optional semantic review was not run because it requires separate user
selection.

## Verdict

**CONDITIONAL.** The shared ticket is ready to merge, with replay deliberately
retained as the explicit downstream integration obligation in #86.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-421 | low | FR-031-AC-3 is intentionally unbacked in #82 because concrete serialization and native `runtime::execute` replay belong only to integration ticket #86; no #82 outcome is presented as replayed. | FR-031-AC-3, #86 |

## Coverage

- `cargo fmt --all -- --check`: pass.
- `cargo test --locked --target-dir target-codex-backends --test kani_shared`: five pass.
- `cargo clippy --offline --locked --target-dir target-codex-backends --all-targets --all-features -- -D warnings`: pass.
- `quire coverage --scope . --json`: no unbacked matrix row, status lie, or
  untracked symbol; FR-029-AC-1..3, FR-030-AC-1..3, and FR-031-AC-1..2 are
  backed by TC-042. FR-031-AC-3 remains the explicit #86 dependency.
- Semantic review: skipped; no user selection requested it.
