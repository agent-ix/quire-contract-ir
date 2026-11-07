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
   defects at once for each adjacent pair of checks. Call `try_new` with a
   non-Claim role and with each byte value empty (FR-361-AC-2, FR-357-AC-17).
3. Compare `layout_context()` keys with the plan's structural keys, the lock
   and occurrence bytes with the admitted wire's member byte ranges, and the
   clock-binding value with an independently written expected RFC 8785
   object for each source; change period, unit and kind (FR-361-AC-3).
4. Measure the read's work and storage; repeat at exact limits and one below
   each; repeat on a package clone (FR-361-AC-4).
5. Inspect the public API and the model crate's dependency graph
   (FR-361-AC-5).
6. Reach a map collection from an entry body; join it with a correct row,
   then with an absent, wrong and misplaced `key_node`; add an unreached map
   row (FR-357-AC-16).
7. Charge denied external charges with `AdapterRow(i)`, `AuthorityRow` and
   `PreparationOutput(k)`, then admitted ones followed by a native read
   (FR-358-AC-10).

Tag the tests `#[trace("TC-471", "<criterion id>")]`. FR-361-AC-6 runs in
`quire-integration` against a genuine QSL plan root.

## Expected Results

- Step 1 returns each occurrence's own receipt, clock and exact bytes.
- Step 2 returns each named code at its stated locus in the stated order,
  with charged work and no closure; `try_new` refuses its three inputs.
- Step 3 values are byte-equal and borrow the admitted buffer; each change
  changes the clock-binding value.
- Step 4 succeeds exactly and refuses one below at the first unpaid charge;
  clones agree.
- Step 5 finds a required field, private row fields, no QSL, observation or
  runtime edge, and no decode, digest or second identity.
- Step 6 retains the map's key, value and `ValueType::Map`; each defective row
  refuses at the collection node; the unreached row does not refuse.
- Step 7 keeps the exact source, form, kind and amount as first unpaid, and
  admitted external charges are cumulative with the native read.
