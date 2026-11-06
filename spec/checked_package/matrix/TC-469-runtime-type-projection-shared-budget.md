---
id: TC-469
title: "Shared native budget for runtime type projection"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-360
    type: verifies
---
# TC-469: Shared native budget for runtime type projection

## Description

Verify that runtime type projection, monitor-closure, nominal, and selected
unit-definition reads charge one caller-owned native budget before bounded
work or any reachable IR-owned storage, and preserve admitted facts within the
measured allowance.

## Test Procedure

Using strict reader-admitted fixtures with selected source and model type
declarations, selected members, nominal rows, and a selected unit conversion
closure, invoke each public runtime projection, monitor-closure, nominal-read,
and unit-definition path with ample limits and record the successful value and
cumulative work and storage charges. Repeat each operation with exactly the
measured work and storage limits, then with each limit reduced by one in turn.
Include runtime-reachable member types that exercise `element_type` and
`slot_type`, including a unit-bearing member that requires unit-string
formatting. Exercise a refusal that has no remaining storage for an owned
diagnostic locus. Inspect these entry-point signatures and the call graph:
their budget-aware helper variants must charge the caller's native budget
directly, without routing storage through `model_members::Budget` /
`WorkMeter`. Confirm that `model_fields`, `model_operations`, semantic
operation, state, `step_relation`, `comparison_operands`,
`model_tuple_position_type`, `model_variant_payload_member_type`, and
`read_enum_index` remain
unchanged and outside the audited runtime/closure call path.

## Expected Results

Every exact-limit operation returns the same complete runtime view and typed
semantic result as the ample-budget operation. Each one-below operation refuses
before the first unpaid inspection or allocation, reports the first unpaid
resource kind and amount, preserves earlier cumulative charges, reports no
success delta, and exposes no partial view. A zero-storage diagnostic refusal
does not allocate an owned locus. Tests demonstrate that every member-type
helper allocation and formatting operation reachable from runtime projection
or monitor closure is charged directly to the same caller-owned native budget.
The excluded model, operation, state, comparison, tuple-position, and
variant-payload APIs remain unchanged. Repeating the same reads over a clone
of the admitted package produces
equal values, charge totals, and refusal loci.
