---
id: TM-008
title: "quire-contract-ir root bridge test matrix"
type: TestMatrix
---
# quire-contract-ir root bridge test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-037 | FR-037-AC-6 | TC-055 | 🚧 planned: TC-055's public-surface and source checks also show that Contract IR defines no replay envelope or witness, has no `replay` or `witness` module and calls no executor; the replay and witness modules no longer exist, and the row stays planned until TC-055's checks are authored. |
| FR-039 | FR-039-AC-1 through FR-039-AC-4 | TC-055 | 🚧 planned; the root crate still re-exports the whole model with `pub use quire_contract_model::*`, exports the family lowerings codegen owns, and exports `KaniProviderRecord` and `KaniProviderResult`, which FR-039 lists among the items QSL owns |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-055 | The root crate's public interface is exactly the listed items and names no QSL-owned replay type | Integration | P0 | FR-039, FR-037-AC-6 | 🚧 planned |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-055 cases | Coverage, compile-fail, closure | public-item inventory equal to FR-039's table with no model item; no `pub use` of `quire_contract_model` in `src/lib.rs` and a failing model-item probe through `quire_contract_ir`; no `replay`, `witness`, `arithmetic`, `collections` or `objects` module under `src/kani/`; no `quire_spec_language::runtime` name under `src/`; one compile-fail probe per QSL-owned and codegen-owned item; a search of `src/` finding no `KaniOutcome` to `TerminalValue` map (owned by agent-ix/quire-contract-codegen, Linear IR-358); negative corpora under `catch_unwind` |
