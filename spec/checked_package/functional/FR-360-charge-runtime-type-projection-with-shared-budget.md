---
id: FR-360
title: "Charge runtime type projection through one shared native budget"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
---
# FR-360: Charge runtime type projection through one shared native budget

## Description

When a consumer reads selected runtime type declarations from an admitted
checked package, Contract IR SHALL account for runtime projection work and
logical storage through the caller's existing finite native budget. This
requirement covers allocations reachable from runtime type projection,
monitor-closure reads, their nominal reads, and the selected unit-definition
read. It leaves clock admission and Exact projection to the separate IR-1359
requirement; it does not widen or redefine those paths. Separate general model,
operation, state, and expression-checking APIs are outside this runtime/closure
call path and are not widened by this requirement.

## Inputs

An admitted `CheckedPackageV2`, the selected runtime declarations and any
producer-supplied nominal rows required by the requested read, and the same
caller-owned `NativeJoinBudget` used by the surrounding native operation.

## Outputs

A complete borrowed or owned runtime type view and the budget's cumulative
charges on success; or a typed refusal identifying the first refused native
locus and the existing first-unpaid budget charge when a finite limit prevents
the read. A refusal exposes no partial runtime type view.

## Behavior

The public runtime type projection, monitor-closure, runtime-reachable nominal-read, and
selected unit-definition entry points SHALL require the caller's mutable
`NativeJoinBudget`. These paths SHALL use budget-aware helper variants that
charge allocations against that caller budget. They SHALL NOT adapt storage
charges through `model_members::Budget` / `WorkMeter`, create a fresh local
allowance, reset cumulative charges, or provide an unbudgeted runtime entry
point or compatibility wrapper.

The runtime type path SHALL charge each bounded inspection and validation
before performing it. It SHALL charge the logical storage of every IR-owned
allocation, clone, copy, or retained value before allocating or copying it,
using the storage units defined by
[FR-358](./FR-358-compose-native-identity-join-budget.md). This includes
storage created while selected `DomainModel::element_type` and
`DomainModel::slot_type` helpers resolve runtime or closure member types,
temporary sets and vectors, cloned names and member types, formatted unit
strings, and every other temporary or retained allocation reachable from
those runtime/closure entry points. This includes the reached-model prerequisite's
selected graph walk, followed tuple/variant members and object operation slots,
model enum stage and shape joins, shape attachment, and reached model-unit and
dimension reconstruction. The same caller budget covers these steps and the
subsequent selected type projection; selection and source/model ownership rules
remain those of FR-357. Budget-aware helper variants SHALL charge
the caller's `NativeJoinBudget` before allocation or copy. Borrowed reads SHALL
not be charged as new retained storage. The charge order SHALL follow selected
declaration and authored member order.

The runtime read SHALL charge the bounded logical storage of every owned
diagnostic component before allocation. If the caller's storage limit cannot
admit a diagnostic charge, then the runtime read SHALL return a bounded budget
refusal using the budget's fixed-size first-unpaid charge metadata and a
non-allocating native locus representation; it SHALL NOT allocate an uncharged
owned locus. The first refusal SHALL retain the first unpaid resource kind and
amount, and the read SHALL not return a partial view.

The public `NativeJoinLocus` and `NativeJoinLocusSource` types expose that
fixed-size location: a static native form and the original package, graph,
row or unresolved declaration-request coordinate. The location does not own
a copied node or key. A known position resolves directly using the original
caller context; an unresolved request coordinate remains explicit rather than
triggering an unpaid identity search. These metadata types belong to the
closed model-root export inventory in [FR-019](../../model/functional/FR-019-rust-library-interface.md).

The budgeted read SHALL preserve the existing runtime type result, selected
identities, authored order, and typed semantic-refusal decision whenever the
caller limits admit every measured charge. Equal admitted input and rows SHALL
produce equal results and charge
totals, including when the admitted package is cloned. A refused charge SHALL
not increase the corresponding consumed counter; prior admitted charges remain
visible and no success delta is reported for the refused operation.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-360-AC-1 | Each public runtime type projection, monitor-closure, runtime-reachable nominal-read, and selected unit-definition entry point requires the caller's mutable `NativeJoinBudget`; budget-aware helper variants charge allocations to that same budget. They do not route storage through `model_members::Budget` / `WorkMeter`, create a local allowance, or offer an unbudgeted runtime entry point or compatibility wrapper. | Inspection, Test |
| FR-360-AC-2 | Every inspected runtime declaration, closure node or edge, nominal row, identity component, and selected unit edge charges work before inspection. Every IR-owned temporary or retained allocation, clone, copied name/type, formatted unit string, and runtime-reachable `element_type` / `slot_type` result charges FR-358 logical storage to the caller's budget before allocation or copying. Exact-limit work and storage succeed; one unit below either measured requirement refuses before the first unpaid operation. | Test, Inspection |
| FR-360-AC-3 | A storage-limited refusal occurs before the denied allocation and returns a bounded typed budget refusal with the first unpaid resource and amount, no partial runtime view, and no uncharged owned diagnostic payload. A work-limited refusal reports the first unpaid work locus without advancing the denied charge. Any earlier admitted charges remain visible and the refused operation has no success delta. | Test |
| FR-360-AC-4 | At sufficient limits, representative source and model runtime type reads, selected nominal reads, and selected unit-definition reads return the same complete values, original identities, order, and semantic refusal causes as the existing behavior. Equal input, rows, and limits produce equal values, cumulative charges, deltas, or first refusals for the original and a clone of the admitted package. | Test |
| FR-360-AC-5 | Runtime projection and monitor-closure paths use budget-aware variants for `DomainModel::element_type` / `slot_type` and other helpers, charging every reachable `BTreeSet` / `Vec`, cloned name/member type, formatted unit string, and other allocation to the caller's `NativeJoinBudget` before allocation or copy. This ticket does not add storage accounting to `WorkMeter` / `model_members::Budget` or widen `model_fields`, `model_operations`, semantic-operation, state, `step_relation`, or `comparison_operands` APIs; these serve separate paths and are not called by the audited runtime/closure path. It excludes the independent numeric `read_enum_index` API. It also does not widen public `model_tuple_position_type` or `model_variant_payload_member_type`; these independent model-read APIs are not called by that path. Value-asserting tests deny each relevant runtime/closure allocation before it occurs. Clock admission and Exact projection remain IR-1359 scope. | Test, Inspection |

## Dependencies

- [FR-358](./FR-358-compose-native-identity-join-budget.md) defines the
  caller-owned cumulative work and logical-storage budget and its charge
  semantics.
- [FR-354](./FR-354-project-admitted-runtime-context.md) owns selected runtime
  type projection and its admitted source facts.
- [FR-019](./FR-019-rust-library-interface.md) owns the exported library API
  inventory.
- IR-1359 owns clock admission and Exact projection refusal-payload changes;
  those code paths are outside this requirement. IR-1366's final integration
  waits for that separate work where shared files or interfaces require it.
