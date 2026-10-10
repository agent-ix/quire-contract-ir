---
id: SR-5020
title: "IR-347 PR 331 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@4c749b2d2360f3ac05d3b712045e84ab6af3922b; model/root public API, TC-058, import migration, TC-041 deletion"
review_set: subset
---
# SR-5020: IR-347 PR 331 code review

## Summary

Ticket: IR-347. Independent diff-scoped code review with the Rust-review lane folded in. Reviewed the explicit model-root named exports against FR-019's public table, the root crate's removal of model re-exports, all changed import paths, TC-058, the deleted TC-041 bridge test, and the documented boundary. QSL main's `quire_contract_ir` name aliases the `quire-contract-model` package in its manifest; codegen main imports actual root items only from `kani`. No production panic, unsafe, new copy, or gate weakening was introduced. The partial PR does not complete FR-039's full Kani interface inventory.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | TC-058 claims FR-039-AC-1 while only inspecting `src/lib.rs`; excluded public Kani items still remain reachable and the test passes. Remove the FR-039-AC-1 trace tag until TC-055 inventories the full `src/` surface. | tests/it/public_interface.rs:143-150; src/kani/mod.rs:19-27; spec/kani/functional/FR-039-root-crate-public-interface.md:108 | correct-requirement-no-evidence |

## Coverage

- FR-019-AC-5 and TC-058, examined: named model-root exports, default public table, feature exception, and one model/root compile-fail path are represented. The source parser is intentionally constrained by the root's export form; it rejects glob or unexpected named-export syntax.
- FR-028-AC-2 and AC-5, examined: the deleted TC-041 bridge test asserted an API path now prohibited. Its trace tags are removed, leaving these criteria untagged as already documented.
- FR-039-AC-1, examined: the root `src/lib.rs` no longer re-exports model items, but `src/kani/mod.rs` still exports lowerings and provider items excluded from FR-039's table. The code PR is a partial slice; the finding concerns its trace claim.
- Import migration, examined: changed IR imports compile through the model crate; current QSL main aliases its dependency name to the model package, and current codegen main uses `quire_contract_ir::kani` for root imports.
- CI/workflow integrity, examined: no Makefile, Cargo manifest, or CI workflow changes. Full gates are owned by the team leader.

## Verdict

CONDITIONAL. The model-interface change is coherent; correct the FR-039-AC-1 trace claim before merge. The remaining FR-039 Kani-surface work stays open separately.

## Dispositions

Reviewed the same PR at agent-ix/quire-contract-ir@794bf6ef3a6a0abaa5333d5bbc39e6372a26bd3c.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 794bf6e — `tests/it/public_interface.rs:143` now tags only TC-058 and FR-019-AC-5. FR-039-AC-1 remains unbacked for the future Kani work. |
