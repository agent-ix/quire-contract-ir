---
id: TC-493
title: "Protocol correlation provenance read from an admitted package"
type: TC
relationships:
  - target: ix://agent-ix/filament-contract-ir/FR-378
    type: verifies
---

# TC-493: Protocol correlation provenance read from an admitted package

## Description

Verify [FR-378](../functional/FR-378-read-protocol-correlation-provenance-of-a-checked-package.md):
the reader admits FSpec FR-831's `protocol_correlation_occurrences` rows,
refuses each defect with FR-831's code, cause and pointer, charges the one
native join budget as FR-831 states, and exposes the provenance through the
typed read. Scope: FR-378-AC-1 to FR-378-AC-5.

## Test Procedure

1. Read each of FSpec TC-2113's positive packages through the public reader
   and compare the typed read with the published table; give one group's
   entry a second region and read it.
2. Read each reader refusal and each schema decision and compare code, cause
   and pointer.
3. Read a control with a nonempty `related` argument with and without the
   member.
4. Read each budget vector at its least admitting limit and one unit less.
5. Write rows through the writer as `None`, `Some(&[])` and an unsorted,
   repeated list, and read the output back; read each invariance pair.

## Expected Results

- Steps 1 to 3: the tables, refusals and capability FR-378-AC-1 to AC-3 state.
- Step 4: admission at the exact limit and the first unpaid charge one unit
  less.
- Step 5: the member written as given, and a pair that differs in the member
  alone.
