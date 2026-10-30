---
id: TC-058
title: "The model crate's public interface is exactly FR-019's list, re-exported by name"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-046
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-048
    type: references
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

## Typed operation fixture feature consumer

Implemented in IR-1088. Verify FR-019-AC-14 through AC-16 with an external
consumer compiled twice: default features and `test-support`. The default
build must reject imports of `CheckedApplicationOperationFixture` and
`CheckedApplicationOperationFixtureError` and calls to
`CheckedApplicationOperation::{from_test_fixture,is_test_fixture}`. The
feature build must import those two fixture types through
`quire_contract_model`, call the constructor and typed accessors, and fail an
import through `quire_contract_ir`. Inventory the feature-only exports and
confirm the default inventory remains exact, `fault-injection` alone still
adds only `MappingAllocationPoint`, and both features add the union of their
independent exports.

Construct one fixture for each control used by CG's four scalar derivation
tests: missing and foreign operation identity; conversion identity with exact
and non-exact rounding mode; scalar target's exact, nearest-even and
toward-zero mode precedence; absent, foreign, conflicting, and each valid
integer-division law definition; absent, text-profile kind, wrong rounding
value, and matching rounding mode; and present or absent member declaration.
For every fixture, assert the typed accessors expose exactly the authored
identity, mode kind and value, ordered definition identities, and member key,
including intentional absence. Assert `is_test_fixture` remains true when
all optional inputs are absent and false for direct-serde and actual
reader-admitted operations. Refuse empty present identity, empty present mode
value, a law definition with empty authority or identity, and present member
keys with an empty package, empty path, ordinal outside QSpec FR-322's range
or a nested path node ID with an out-of-range ordinal with a typed error and
no operation. Accept
foreign and conflicting nonempty law identities
as valid fixture shape. Serialize a fixture operation into complete package
wire and confirm the ordinary strict reader still refuses a case admission
does not permit; the fixture marker must not survive serialization and readback.
Compile-fail probes reject conversion of a fixture into an admitted
`ContractPackageV1` or `CheckedPackageV2` without the production reader.

## Status

The source-inventory and root-boundary tests exist in
`tests/it/public_interface.rs`; the model-path and root-path doctests exist in
`src/lib.rs`. PLANNED/UNRUN (IR-1141): the compiled negative probes and exact
FR-019 inventory must be updated and rerun after candidate helper exports
are removed. Existing helper-dependent tests and doctests cannot back the
structural boundary below.

## Structural V2 identity API boundary

PLANNED/UNRUN (IR-1141). The former FR-046 and FR-048 content-derived candidate
helper APIs are withdrawn. The external default-feature consumer compiles a
`CheckedNodeId { package, path, ordinal }` with a non-empty package and path,
uses the typed V2 encoder and reader, and reads the same id after admission.
Negative compile probes cannot import `derive_application_node_id`,
`derive_structural_candidates`, the candidate result/context/error types,
`CheckedSemanticId`, the retired identity-preimage types or `PACKAGE_DOMAIN_V2`.
The model crate's named public inventory and the absence of root-crate reexports
remain enforced. A raw structural id does not assert admission; only the
strict reader returns an admitted `CheckedPackageV2`.
