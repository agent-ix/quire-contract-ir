---
id: TC-058
title: "The model crate's public interface is exactly FR-019's list, re-exported by name"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: verifies
---
# TC-058: The model crate's public interface is exactly FR-019's list, re-exported by name

## Description

Verify FR-019-AC-5: the `quire_contract_model` crate root re-exports its
public items by name with no glob re-export, its default-feature public items
equal FR-019's Public items table, and none of those items is reachable
through a `quire_contract_ir` path.

## Test Procedure

Read `crates/quire-contract-model/src/lib.rs` and fail on any `pub use` whose
path ends in `*`. Inventory every public item the `quire_contract_model` crate
root exports in a default-feature build, grouped by defining module, and
compare the set with FR-019's Public items table. Build the same inventory
with the `fault-injection` feature and confirm the only added item is
`MappingAllocationPoint`. Compile one probe that names a table item through
`quire_contract_model` and expect it to build, and one that names the same
item through `quire_contract_ir` and expect it to fail. The member sets of the
FR-038 artifact references are FR-019-AC-6's, verified under TC-018 (struct
literals naming exactly each reference's members, and a `CheckedRevision`
probe), not here.

## Expected Results

`lib.rs` has no glob re-export. The default-feature inventory equals the table
with no extra or missing item, and the `fault-injection` inventory adds
`MappingAllocationPoint` alone. The `quire_contract_model` probe builds and
the `quire_contract_ir` probe fails to compile.

## Status

Implemented: `tc_058_model_root_exports_exactly_the_fr_019_public_items`,
`tc_058_fault_injection_adds_only_its_named_export`, and
`tc_058_root_crate_exposes_only_its_kani_module` in
`tests/it/public_interface.rs` check the source inventories and root boundary.
The positive model-path and negative root-path doctests in `src/lib.rs` check
that one table item builds only through `quire_contract_model`.
