---
id: TC-058
title: "The model crate's public interface is exactly FR-019's list, re-exported by name"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-046
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-048
    type: verifies
---
# TC-058: The model crate's public interface is exactly FR-019's list, re-exported by name

## Description

Verify FR-019-AC-5: the `quire_contract_model` crate root re-exports its
public items by name with no glob re-export, its default-feature public items
equal FR-019's implemented Public items table entries at the tested revision,
and none of those items is reachable
through a `quire_contract_ir` path.

## Test Procedure

Read `crates/quire-contract-model/src/lib.rs` and fail on any `pub use` whose
path ends in `*`. Inventory every public item the `quire_contract_model` crate
root exports in a default-feature build, grouped by defining module, and
compare the set with FR-019's implemented Public items table entries, excluding
entries explicitly marked planned until their owning code lands. Build the same inventory
with the `fault-injection` feature and confirm the only added item is
`MappingAllocationPoint`. Compile one probe that names a table item through
`quire_contract_model` and expect it to build, and one that names the same
item through `quire_contract_ir` and expect it to fail. The member sets of the
FR-038 artifact references are FR-019-AC-6's, verified under TC-018 (struct
literals naming exactly each reference's members, and a `CheckedRevision`
probe), not here.

## Expected Results

`lib.rs` has no glob re-export. The default-feature inventory equals the table's
implemented entries
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

## Compiled candidate application API consumer

Implemented (IR-738). The compiled public consumer controls and the three
source/spec inventory controls have passed. Verify
[FR-046](../functional/FR-046-derive-candidate-application-node-identity.md),
AC-1, and the candidate application identity row under Public items in
[FR-019](../functional/FR-019-rust-library-interface.md).
The source/spec inventory includes the five named candidate API exports;
its source inspection remains separate from the compiled consumer checks.
Compile and execute an external consumer probe that creates a borrowed `ApplicationNodeDraft`,
constructs `ApplicationIdentityContext`, calls `derive_application_node_id`,
obtains `CandidateApplicationNodeId::candidate_node_id()` and exhaustively
matches `ApplicationIdentityError`. The consumer assembles a candidate before
reading any package; no helper result constructs or substitutes for an admitted
`CheckedPackageV2`. Preserve the existing model-only/root-crate path checks.
Assert the candidate node key and each specified typed error outcome through
the compiled public interface. Source-text inventory matching alone cannot
satisfy this compiled consumer procedure.

## Compiled candidate structural API consumer

Planned for IR-763; no compiled or tagged result is claimed. Verify
[FR-048](../functional/FR-048-derive-structural-and-group-candidates.md),
AC-1, and the candidate structural identity row under Public items in
[FR-019](../functional/FR-019-rust-library-interface.md).
Compile and run an external default-feature `quire_contract_model` consumer
that builds ordered `CheckedSemanticNodeV2` drafts, calls
`derive_structural_candidates`, obtains each typed candidate node ID and group
label through accessors, and exhaustively matches `StructuralCandidateError`.
Compile-fail probes shall reject direct construction of candidate results and
imports through `quire_contract_ir`. Compare the default-feature root export
set against the updated FR-019 inventory, retaining the existing no-glob and
fault-injection-only-delta checks. The consumer shall obtain candidates before
any package read and cannot turn a candidate into an admitted package.
TC-048 owns the subsequent reader admission, stale-key and resource controls.
