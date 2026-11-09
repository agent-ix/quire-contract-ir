---
id: TC-471
title: "Monitor authority receipt, layout context and map collection closure"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-361
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-357
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: verifies
---
# TC-471: Monitor authority receipt, layout context and map collection closure

## Description

Verify that the selected monitor closure binds a same-emission authority row,
exposes the exact runtime layout context, joins map collection rows by their
key node, and charges every external and native step to one budget.

## Test Procedure

1. Build an IR-owned strict admitted Timed monitor package whose clause has
   two claim occurrences, once with a dense and once with a tick model-time
   source. Strictly read it, take its `admission_binding()`, and build an
   authority row for each occurrence with distinct nonempty order and
   association bytes. Read the closure with ample limits (FR-361-AC-1).
2. Independently replace the admission with an independent read of the same
   bytes and with another package, the clause with another clause, the
   ordinal with an absent one, the source kind and the tick period; give two
   defects at once for each adjacent pair of checks, and an authority defect
   beside a model-row, equality and source-row defect. Call `try_new` with a
   non-Claim role and with each byte value empty (FR-361-AC-2, FR-357-AC-17).
3. Compare `layout_context()` keys with the plan's structural keys, the lock
   and occurrence bytes with the member byte ranges of the admitted input,
   and `profile()` and the clock-binding value with independently written
   expected RFC 8785 values for each source; change model, tick operation,
   unit, period, kind and profile (FR-361-AC-3).
4. Measure the read's work and storage; repeat at exact limits and one below
   the cumulative charge before each authority, layout-context and
   clock-binding charge, including an `OccurrenceMismatch` input; repeat on a
   package clone (FR-361-AC-4).
5. Inspect the public API and the model crate's dependency graph
   (FR-361-AC-5).
6. Reach a map collection from an entry body; join it with a correct row,
   then with an absent, wrong and misplaced `key_node`; add an unreached map
   row; measure and repeat at exact limits and one below each (FR-357-AC-16).
7. Charge denied external charges with `AdapterRow(i)`, `AuthorityRow`,
   `PreparationOutput(k)` and `PreparationSelection(k)`, then admitted ones
   followed by a native read (FR-358-AC-10).
8. Read `tick_unit_dimension()` for a source and a model tick unit; repeat at
   the exact allowance and one below the first seed charge (FR-357-AC-18).

Tag the tests `#[trace("TC-471", "<criterion id>")]`. The explicit IR lane
`make conformance-qsl-ir1384` exercises genuine published QSL plan roots and
admitted authority for the receipt slice of steps 1, 2 and 4. It supplies local
handoff evidence; FR-361-AC-6's final `quire-integration` qualification remains
with that owner. The layout context and clock binding are implemented (steps 3 and 4, with
`LayoutIdentityUnavailable` for a missing, ambiguous or unsupported subject or
unit identity); the genuine `monitor-base.json` tick source supplies the
tick-source model, operation and period evidence.

## Expected Results

- Step 1 returns each occurrence's own receipt, clock and exact bytes.
- Step 2 returns each named code at its stated locus in the stated order,
  with charged work and no closure; the model-row defect wins over the
  authority defect, which wins over equality and source-row defects;
  `try_new` returns components `Occurrence`, `OrderValue`, `ProgressAssociation`.
- Step 3 values are byte-equal; each change changes the clock-binding value.
- Step 4 succeeds exactly and refuses one below at each charge with locus
  `MonitorAuthority` before resolution and `MonitorSeed(k)` with its form
  after; clones agree.
- Step 8 returns each unit's own admitted dimension; one below refuses at the
  first unpaid seed charge.
- Step 5 finds a required field, private row fields, no QSL, observation or
  runtime edge, and no decode, digest or second identity.
- Step 6 retains the map's key, value and `ValueType::Map`; each defective row
  refuses `CollectionKey` at the collection node; the unreached row does not
  refuse; exact limits succeed and one below refuses at the first unpaid
  key-join charge.
- Step 7 admits each charge at the exact allowance, keeps the exact source,
  form, kind and amount as first unpaid one below, and admitted external
  charges are cumulative with the native read.
