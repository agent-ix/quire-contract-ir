---
id: SR-5053
title: spec-review/integrity review of IR-347 PR 332
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir@5649b86f446d44fc54c558d7333bee32b0944eb9; spec/assurance/AD-004-checked-package-seam.md,
  spec/assurance/AD-005-qsl-consumption-seam.md, spec/assurance/AD-006-codegen-consumption-seam.md,
  spec/assurance/AD-007-cross-repo-dependency-graph.md, spec/core/functional/FR-044-typed-std001-code.md,
  spec/kani/functional/FR-031-bounded-kani-dispatch-and-terminal-map.md, spec/kani/matrix/TC-223-kani-outcome-fr331-result-map.md,
  spec/kani/matrix/tests.md
review_set: subset
---
# SR-5053: spec-review/integrity review of IR-347 PR 332

## Summary

Ticket: IR-347. Reviewed exact PR head `5649b86f446d44fc54c558d7333bee32b0944eb9` against base `efa4e37babce07afbf4829df0e641e2b124348e0`. Diff-scoped review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AD-005's Current state still says both crates use glob re-exports, the root compatibility bridge and its tc_041 bridge test exist, and codegen consumes that bridge. This conflicts with its updated D-4 and current IR/CG source. Rewrite this paragraph as historical or current fact so the architecture description has one coherent state. | spec/assurance/AD-005-qsl-consumption-seam.md:170-180 |

## Verdict

CONDITIONAL: the updated AD-005 decision contradicts its own Current state and the source.

## Coverage

Units examined (each excerpt is verbatim from the reviewed head):

- `AD-004` — `spec/assurance/AD-004-checked-package-seam.md` (examined): # QSpec to IR seam: the checked-package reader and lowering
- `AD-005` — `spec/assurance/AD-005-qsl-consumption-seam.md` (examined): # IR to QSL seam: what QSL takes from the model crate
- `AD-006` — `spec/assurance/AD-006-codegen-consumption-seam.md` (examined): # IR to codegen seam: what codegen consumes
- `AD-007` — `spec/assurance/AD-007-cross-repo-dependency-graph.md` (examined): # Cross-repo dependency graph
- `FR-044` — `spec/core/functional/FR-044-typed-std001-code.md` (examined): # FR-044: Export a typed STD-001 code from the model crate
- `FR-031` — `spec/kani/functional/FR-031-bounded-kani-dispatch-and-terminal-map.md` (examined): # FR-031: Dispatch bounded Kani modules and expose typed outcomes
- `TC-223` — `spec/kani/matrix/TC-223-kani-outcome-fr331-result-map.md` (examined): # TC-223: Kani outcomes are built only by validated constructors, with their check count and cause codes
- `TM-009` — `spec/kani/matrix/tests.md` (examined): # quire-contract-ir bounded Kani test matrix
- `AD-005` — `spec/assurance/AD-005-qsl-consumption-seam.md` (examined): - The model crate's own root uses glob re-exports for seven modules   (the `pub use` lines of `crates/quire-contract-model/src/lib.rs`), against AD-001 and FR-019 ("by name, no   glob").

## New findings (disposition pass 1)

Observed on interim fix head `3a571f83b204728e54d6f5e6ee0edc68b96c28d0` and fixed before this disposition was published.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | At the first fix head, AD-005:66-69 said QSL now keys the model dependency as `quire-contract-model`, but its revised Current state said the local name was `quire-contract-ir`; the latter matched QSL root and qsl-package manifests. Reconcile the identity paragraph, direction table and R3-Q1 status. | spec/assurance/AD-005-qsl-consumption-seam.md:66-69 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 3a571f83b204728e54d6f5e6ee0edc68b96c28d0 |
| FND-002 | fixed | 59f35103175d8d4fbdb2e20a8e8c09489eaf7295 |
