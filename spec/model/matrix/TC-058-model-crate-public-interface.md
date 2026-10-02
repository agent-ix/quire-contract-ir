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
item through `quire_contract_ir` and expect it to fail. Then compile member-set
probes for the FR-038 artifact references: a struct literal of `CheckedArtifactRef`
naming exactly `authority` and `identity`, one of `CheckedSourceRef` naming
exactly `authority`, `identity`, `digest_domain` and `digest`, and one of
`CheckedArtifactLocator` naming exactly `authority`, `identity` and `domain`,
each expected to build, and a probe adding any further member, such as
`revision`, to each, expected to fail; and a probe naming `CheckedRevision`,
expected to fail.

## Expected Results

`lib.rs` has no glob re-export. The default-feature inventory equals the table
with no extra or missing item, and the `fault-injection` inventory adds
`MappingAllocationPoint` alone. The `quire_contract_model` probe builds and
the `quire_contract_ir` probe fails to compile. Each member-set probe builds
with exactly its members and fails with an added one, and the `CheckedRevision`
probe fails to compile.

## Status

Planned.
