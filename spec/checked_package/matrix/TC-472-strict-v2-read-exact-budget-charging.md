---
id: TC-472
title: "Exact budget charging of the strict V2 checked-package read"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: verifies
---
# TC-472: Exact budget charging of the strict V2 checked-package read

## Description

Verify that the one strict V2 read charges every metered work step and every
heap allocation it makes to the caller's `NativeJoinBudget` before the step or
allocation, at a `PackageRead` locus; that an exact allowance admits and one
below refuses at the first unpaid charge with no partial package; that the
charged storage equals the bytes a counting allocator observes; and that every
sufficient budget returns the unlimited budget's outcome.

## Test Procedure

1. Build a dedicated integration test binary whose global allocator counts, on
   the reading thread only and only while a flag is set around the
   `CheckedPackageV2::read` call, the size of every allocation and the growth of
   every reallocation, ignoring frees and shrinks. The binary runs its reads on
   one thread.
2. Corpus: every positive TC-048 fixture, the QSpec checked-package
   conformance packages this repository reads by reference, one refused input
   per refusal code, and the adverse shapes of FR-038-AC-511 generated in the
   test from the repository's public fixture builders.
3. Read each input under `NativeJoinBudget::new(u64::MAX, u64::MAX)` and record
   the outcome, the work and storage delta, and the observed bytes. With a
   crate-private stage probe, record the cumulative work and storage admitted
   at the last charge of each `PackageReadStage`.
4. Re-read each input with the exact measured work and storage limits; then,
   for each stage S the input reaches, with the storage limit one below the
   cumulative storage at S's last charge (work ample), and, for each stage that
   charges work, with the work limit one below its cumulative work (storage
   ample). Repeat with the denied allocation class chosen as each case of
   FR-038-AC-497 through FR-038-AC-507 lists.
5. Repeat the one-below reads with a storage limit that leaves too little for
   the outcome's own diagnostic, and with zero storage.
6. Compose: two reads on one budget whose limits are below their sum; a read
   followed by an FR-358 native join on the same budget; the same read twice
   and over an admitted package clone.
7. Read the strict-grammar inputs of FR-038-AC-509 (each byte-stream fault, the
   precedence pairs, the 300-level body) under the unlimited and exact budgets.
8. Inspect the read path for the absence of `serde_json` parsing, `Value`,
   `RawValue`, `BTreeMap`, `BTreeSet`, `HashMap`, a private work meter, an
   unmetered entry or a second read implementation, and confirm the public
   read entries take the budget.

## Expected Results

- Every read's admitted storage delta equals its observed allocated bytes,
  under unlimited, exact and one-below budgets, for admitted and refused
  inputs; no allocation is observed after a denied charge.
- Exact limits admit an outcome equal to the unlimited outcome. Each one-below
  read refuses at the last charge of its stage, with first unpaid kind and
  amount equal to that charge, `first_unpaid_locus` naming
  `PackageRead(stage, index)` and its form, no admitted package and no success
  delta; earlier charges stay in cumulative consumption.
- A work or storage denial with an affordable diagnostic returns `incomplete`
  with the configured limit, cumulative consumption plus the denied amount and
  the pointer FR-038 names; an unaffordable diagnostic, or zero storage,
  returns the fixed-size `exhausted` outcome with the original denial, its
  configured bound, its locus and the pending outcome class, and allocates
  nothing.
- Composed reads and joins stop at the first unpaid charge of the later
  operation; repeated reads and reads over a clone charge equal amounts and stop
  at the same locus.
- Each strict-grammar input refuses with the code, pointer and precedence
  FR-038-AC-509 states, independent of the budget once it is sufficient.
- The work delta of every TC-048 fixture that states a work requirement equals
  that requirement.
- Inspection finds one budgeted read implementation with no `serde_json`
  parse, `Value`, map or set construction, and no private work meter.
