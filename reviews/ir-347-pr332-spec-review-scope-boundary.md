---
id: SR-5054
title: spec-review/scope-boundary review of IR-347 PR 332
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-contract-ir@5649b86f446d44fc54c558d7333bee32b0944eb9; spec/assurance/AD-004-checked-package-seam.md,
  spec/assurance/AD-005-qsl-consumption-seam.md, spec/assurance/AD-006-codegen-consumption-seam.md,
  spec/assurance/AD-007-cross-repo-dependency-graph.md
review_set: subset
---
# SR-5054: spec-review/scope-boundary review of IR-347 PR 332

## Summary

Ticket: IR-347. Reviewed exact PR head `5649b86f446d44fc54c558d7333bee32b0944eb9` against base `efa4e37babce07afbf4829df0e641e2b124348e0`. Diff-scoped review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AD-004 still routes CG's checked-package reader through the removed IR-root model re-export. The same document now says that re-export was removed, and current CG imports quire_contract_model directly. Update the dependency-direction paragraph so the seam diagram does not send implementers to a nonexistent path. | spec/assurance/AD-004-checked-package-seam.md:84-87 |

## Verdict

CONDITIONAL: the updated AD-004 current-state note conflicts with its dependency-direction paragraph.

## Coverage

Units examined (each excerpt is verbatim from the reviewed head):

- `AD-004` — `spec/assurance/AD-004-checked-package-seam.md` (examined): # QSpec to IR seam: the checked-package reader and lowering
- `AD-005` — `spec/assurance/AD-005-qsl-consumption-seam.md` (examined): # IR to QSL seam: what QSL takes from the model crate
- `AD-006` — `spec/assurance/AD-006-codegen-consumption-seam.md` (examined): # IR to codegen seam: what codegen consumes
- `AD-007` — `spec/assurance/AD-007-cross-repo-dependency-graph.md` (examined): # Cross-repo dependency graph
- `AD-004` — `spec/assurance/AD-004-checked-package-seam.md` (examined): CG reaches the reader through IR's root crate, which re-exports the model (see Current state)

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 3a571f83b204728e54d6f5e6ee0edc68b96c28d0 |
