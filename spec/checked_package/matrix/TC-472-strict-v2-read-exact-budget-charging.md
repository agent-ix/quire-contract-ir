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
allocation, at a `PackageRead` locus, with heap bytes charged to the budget's
separate read-storage counter; that an exact allowance admits and one below
refuses at the first unpaid charge with no partial package; that the charged
read storage equals the total bytes a counting allocator observes requested;
and that every sufficient budget returns the unlimited budget's outcome.

## Test Procedure

1. Build a dedicated integration test binary whose global allocator counts, on
   the reading thread only and only while a flag is set around the
   `CheckedPackageV2::read` call, the size of every allocation request and the
   new size of every reallocation request, ignoring frees. Exercise both an
   explicitly initialized catalog process and a fresh process without
   `initialize_operation_catalog`; serialize the reads and compare the first
   read's budget deltas with a later read's (FR-038-AC-515).
   The first package read is measured, not used as an unmeasured warm-up;
   repeated public reads must charge their owned catalog storage again.
   Preserve an original driver's enclosing allocation oracle: run proof
   probes serially on scoped workers, with their actual allocator counters
   armed, and keep the one original public read on its caller thread. Exclude
   only instrumentation setup, logging and probe-result destruction from the
   caller's capture; prove the original allocation totals, including result
   drop, unchanged and independently prove full worker realloc requests.
   Measure dropping a returned outcome in a separate counter interval, so
   destructor requests cannot be mistaken for during-call accounting.
2. Corpus: every positive TC-048 fixture, the QSpec checked-package
   conformance packages this repository reads by reference, one refused input
   per refusal code, and the adverse shapes of FR-038-AC-511 generated in the
   test from the repository's public fixture builders. Select original tests
   and read-bearing fixture closures from their actual trace attributes and
   compiled test listing. Interpose at their real public read boundaries;
   retain original builders and semantic oracles. Do not equate a handpicked
   registry or every row of a referenced JSON file with the complete corpus.
   Compiler-observed reader-entry counts must account for every actual
   measured boundary, including aliases, function items and shared helpers;
   reject zero-test, omitted-test, ignored-test and unexplained-entry runs.
   Log original test/callsite provenance and mode in temporary evidence.
3. Read each input through `CheckedPackageV2::read_with_budget` under
   `NativeJoinBudget::unlimited()`, and through the unbudgeted
   `CheckedPackageV2::read` and `read_checked_package` at ample, exact and
   one-below `limits.work`, preserving each original driver's structural
   limits, inputs, evidence and semantic oracle. Record the outcome, work,
   logical-storage and read-storage deltas, and observed bytes. Discover
   boundaries with the passive test-support native-budget observer: first
   count events, then reserve the recording buffer outside the active
   allocation counter and record the same read's Copy events without growing
   that buffer. Overflow fails the proof. Compare observer-on and observer-off
   outcomes and deltas. The observer is scoped to the worker/read and changes
   neither the actual budget layout nor its accounting. Each event supplies
   its real resource, amount, consumed-before value, admission result, locus
   and source-derived allocation class; do not infer classes from amounts.
   Record boundaries in runtime order; stage labels may decrease and must
   not be treated as a monotonic search key.
4. Re-read each input with the exact measured work and read-storage limits;
   then, for each stage S the input reaches, with the read-storage limit one
   below the cumulative read storage at S's last charge (work ample), and, for
   each stage that charges work, with the work limit one below its cumulative
   work (read storage ample). Include walker stack growths and quire-exact
   limb requests as denied requests (FR-038-AC-516). Repeat with the denied allocation class chosen as each case of
   FR-038-AC-497 through FR-038-AC-507 lists. Select stage-last requests and
   the first/last request of each reached stage/form/allocation class from
   the recorded events, including walker and exact-number classes. Verify
   those exact and one-below boundaries and repeated first faults without
   replaying every allocation prefix of the 100000-node chain. An existing
   earlier work denial remains the original first fault when the original
   finite work cap stops a storage probe first; no storage-first outcome is
   invented. Every probe still asserts requested bytes equal charged bytes.
   Keep the all-input ample/exact/last-short core tier distinct from the full
   normative boundary tier in commands and result records.
5. Repeat the one-below reads with a read-storage limit that leaves too little
   for the outcome's own diagnostic, and with zero read storage, including an
   input over the byte limit.
5a. Exercise the budget API of FR-358-AC-14: the default zero read-storage
   limit of `NativeJoinBudget::new`, `with_read_storage_limit`, the read-storage
   getters and a `ReadStorage` denial, and a native join after a read showing
   its logical storage requirement unchanged.
6. Compose: two reads on one budget whose limits are below their sum; a read
   followed by an FR-358 native join on the same budget; the same read twice
   and over an admitted package clone.
7. Read the strict-grammar inputs of FR-038-AC-509 (each byte-stream fault, the
   precedence pairs, the 300-level body) under the unlimited and exact budgets.
8. Exercise `CheckedJson` (FR-038-AC-514): constructors, `node()`, equality,
   `Debug`, serde round trips of `CheckedTerm` and `CheckedDiagnosticV2`.
9. Enumerate the QSpec adverse corpus for vectors whose expected outcome
   depends on byte-stream fault order (FR-038-AC-509).
10. Inspect the read path for the absence of `serde_json` parsing, `Value`,
   `RawValue`, `num-bigint` arithmetic, stable sorts, `BTreeMap`, `BTreeSet`,
   `HashMap`, a private work meter, an unmetered entry or a second read
   implementation; confirm the public read entries take the budget and the
   public outcome types of FR-038-AC-517, with the existing result, limit and
   charge types unchanged and the unbudgeted entries delegating to the one
   budgeted implementation. Ground all reached functions, including generic,
   Drop, operator, closure and finite-vtable edges, in the actual compiled
   production entry with opaque borrowed inputs. Map compiler identities and
   spans back to source syntax; unresolved indirect edges fail closed. Do not
   replace this with whole-file scanning of mixed V1/postread code or a name
   allowlist. Executed-corpus entry counts are separate evidence and do not
   establish the static AC-513 property.

## Expected Results

- Every read's admitted read-storage delta equals its observed requested
  bytes, under unlimited, exact and one-below budgets, for admitted and refused
  inputs; its logical storage delta is zero; no request is observed for a
  denied charge, and every request after a denial belongs to the charged
  outcome diagnostic.
- The unbudgeted entries return today's outcomes: under their
  `limits.work` budget, a work denial is the existing `CheckedPackageIncomplete`
  `Work` with the same limit, consumed and pointer, and no read-storage denial
  or `Exhausted` occurs.
- Exact limits admit an outcome equal to the unlimited outcome. Each one-below
  read refuses at the last charge of its stage, with first unpaid kind and
  amount equal to that charge, `first_unpaid_locus` naming
  `PackageRead(stage, index)` and its form, no admitted package and no success
  delta; earlier charges stay in cumulative consumption.
- A work or read-storage denial with an affordable diagnostic returns
  `BudgetIncomplete` with the configured limit, cumulative consumption plus the
  denied amount and the pointer FR-038 names, including a zero read-storage
  limit, whose denial owns no pointer and so costs nothing; an unaffordable
  diagnostic (a refusal or `incomplete` outcome that owns a pointer) returns
  the fixed-size `Exhausted` outcome with the original denial, its configured
  bound, its locus and the pending outcome class, and requests nothing more; an over-byte-limit input returns
  `incomplete` `Bytes` at any read-storage limit.
- Each strict read parses and owns its catalog, charging every catalog
  allocation before requesting it at `PackageRead(Envelope, 0)` with form
  `"operation_catalog"`. Any catalog charges already admitted remain spent
  when decoding returns `CatalogUnavailable`; a denied catalog reservation
  follows the ordinary budget-incomplete/exhausted rules. Calling
  `initialize_operation_catalog` does not pay for or cache a strict read's
  catalog storage (FR-038-AC-515).
- `CheckedJson` and the budget API behave as FR-038-AC-514 and FR-358-AC-14
  state.
- Composed reads and joins stop at the first unpaid charge of the later
  operation; repeated reads and reads over a clone charge equal amounts and stop
  at the same locus.
- Each strict-grammar input refuses with the code, pointer and precedence
  FR-038-AC-509 states, independent of the budget once it is sufficient.
- The work delta of every TC-048 fixture that states a work requirement equals
  that requirement.
- Inspection finds one budgeted read implementation with no `serde_json`
  parse, `Value`, map or set construction, and no private work meter.
