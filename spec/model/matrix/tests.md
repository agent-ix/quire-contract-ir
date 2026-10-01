---
id: TM-004
title: "quire-contract-ir model test matrix"
type: TestMatrix
---
# quire-contract-ir model test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-013 | FR-013-AC-1 through FR-013-AC-4 | TC-016 | ✅ implemented |
| FR-014 | FR-014-AC-1 through FR-014-AC-7 | TC-016 | ✅ implemented |
| FR-015 | FR-015-AC-1 through FR-015-AC-7 | TC-016 | ✅ implemented |
| FR-016 | FR-016-AC-1 through FR-016-AC-4 | TC-017 | ✅ implemented, including the caller byte budget on all five closed object kinds |
| FR-017 | FR-017-AC-1, FR-017-AC-2 | TC-017 | ✅ implemented |
| FR-019 | FR-019-AC-1 through FR-019-AC-5 | TC-018, TC-058 | 🚧 AC-1 through AC-4 implemented and verified by TC-018, including `expected_inventory` as declared stable surface; AC-5, the model crate's by-name re-export of exactly the Public items table with no glob and no path through `quire_contract_ir`, is planned (TC-058): the model crate root has seven glob re-exports and the root crate re-exports the whole model |
| FR-023 | FR-023-AC-1 through FR-023-AC-5 | TC-035 | ✅ implemented |
| FR-028 | FR-028-AC-1 through FR-028-AC-5 | TC-041 | 🚧 AC-1, AC-3 and AC-4 implemented and backed by `tc_041_model_dependency_graph_is_cycle_free_and_owner_free` (`tests/it/cycle_free_model.rs`; AC-3 checks the root manifest names no crate from the quire-spec-language repository, by name and by git source; 🚧 planned, not implemented: QSL's production graph has no quire-contract-ir and the composition build, in agent-ix/quire-integration, Linear IR-358); AC-5 is tagged on `tc_041_bridge_reexports_the_exact_model_api_and_keeps_model_sources_single`, which checks only that model and root types coincide, so the copied-type, callback, trust-flag and local-parser prohibitions are not tested; AC-2, byte/result identity through `quire_contract_model` paths with no root re-export, is planned. `tc_041_bridge_reexports_the_exact_model_api_and_keeps_model_sources_single` (`tests/it/cycle_free_model.rs`) is tagged FR-028-AC-2 but asserts that the root crate's glob re-export is present, the opposite of AC-2; the tag is stale and does not back AC-2 |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-016 | Types, expressions, short-circuiting, and definedness conform | Integration | P0 | FR-012..FR-015, NFR-002, STD-001 | ✅ implemented |
| TC-017 | Canonical bytes, digests, version preflight, and orphan classes conform | Property | P0 | FR-016, FR-017, NFR-001, NFR-003 | ✅ implemented |
| TC-035 | Derived executable projections bind complete typed clause populations through the public IR boundary | Integration | P0 | FR-023 | ✅ implemented |
| TC-041 | Reach the model through `quire_contract_model` while proving the model/owner/root Cargo graph is acyclic | Integration | P0 | FR-028-AC-1, FR-028-AC-2, FR-028-AC-3, FR-028-AC-4, FR-028-AC-5 | 🚧 AC-1, AC-3 and AC-4 implemented and backed by `tc_041_model_dependency_graph_is_cycle_free_and_owner_free` (`tests/it/cycle_free_model.rs`; AC-3 checks the root manifest names no crate from the quire-spec-language repository, by name and by git source; 🚧 planned, not implemented: QSL's production graph has no quire-contract-ir and the composition build, in agent-ix/quire-integration, Linear IR-358); AC-5 is tagged on `tc_041_bridge_reexports_the_exact_model_api_and_keeps_model_sources_single`, which checks only that model and root types coincide, so the copied-type, callback, trust-flag and local-parser prohibitions are not tested; AC-2 planned: the test reaches the model through the root crate's re-export |
| TC-058 | The model crate's public interface is exactly FR-019's list, re-exported by name | Integration | P0 | FR-019-AC-5 | 🚧 planned |

## Coverage Design

| Test | Coverage rule | Required cases |
|---|---|---|
| TC-041 cases | Coverage, dependency, feature, edge | Cargo metadata cycle check for default/all/minimum features; model package owner/TL dependency absence; no model item reachable through a `quire_contract_ir` path; unchanged schema, canonical-output, diagnostic and corpus results; root package manifest names no `quire-spec-language` or `qsl-*` dependency (the QSL-side graph check and the locked root-package-and-QSL composition build live in agent-ix/quire-integration); compile-fail probes for public owner constructors, callbacks, trait validators, trust flags and copied owner wire types |
| TC-058 cases | Coverage, compile-fail, closure | no glob `pub use` in the model crate root; default-feature public-item inventory equal to FR-019's table both ways; `fault-injection` inventory adds `MappingAllocationPoint` alone; one table item builds through `quire_contract_model` and fails through `quire_contract_ir` |
