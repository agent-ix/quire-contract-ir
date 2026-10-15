---
id: TC-452
title: "Verify the neutral source-obligation correspondence input"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-052
    type: verifies
---
# TC-452: Verify the neutral source-obligation correspondence input

## Execution Status

Planned/UNRUN. The provisional input port retains typed rows and stop causes,
but exposes no complete census. No owner adapter yet compares a declared
complete inventory with the full selected IR V2, QSL, or `quire-code-rs`
output. FR-052-AC-8 tests cover only that provisional port; they do not run
this full-owner procedure. The narrower IR-794 owner-issued completion boundary
and its owner-output comparison (FR-052-AC-9 through AC-11) are also UNRUN;
they do not by themselves complete this three-owner census case.

## Description

Verify that the neutral model port preserves complete owner inventories and
explicit source-author mappings, and returns typed errors rather than a
complete-looking census when any owner input is absent or incomplete.

## Test Procedure

Compile an external default-feature consumer that constructs
`SourceObligationInput`, calls `build_source_obligation_census`, and borrows
the resulting `SourceObligationCensus` and rows through
`quire_contract_model::source_obligation`. Inspect the public constructor
signature and public-item inventory for the absence of result, method, profile,
artifact, attempt, and run inputs.

Build three owner inputs with their native APIs or typed adapters. For IR V2,
enumerate the complete selected semantic graph and all of its source
occurrences. For QSL, enumerate the complete selected checked
`RequirementRecord` set and native pending-obligation set. For
`quire-code-rs`, account for every selected production module and fully parse
its function facts, including a module with no functions. Have each input
project QSL checked requirement map keys as wire node bytes plus occurrence
role and ordinal, and pending obligations as their native site and kind,
including a predicate target. Distinguish changes to each projected field;
collapse only identical native tuples. Exercise distinct typed QSL exhaustion
and unfinished-disposition causes, retaining the exhaustion dimension, limit,
prior, requested charge, and site and the unfinished declaration handle. Have
each input carry its matching owner, selected source, scope, item count, and the
owner-emitted completeness state. Verify that `Complete` is emitted only after
normal exhaustion of that owner's full scope, and compare every complete
envelope's entries with the full owner output for that selection. An owner
item omitted from an otherwise populated selected source must prevent a
complete envelope.

For the IR-794 boundary, drive each owner-native enumerator or typed adapter
from its selected owner object to normal exhaustion and capture its full
output independently of the model envelope. Compare source, typed owner,
scope, every row and observed count field by field. Repeat with a complete
zero-row scope and a fully parsed production module with zero functions.
Delete one emitted row after the owner produces its full output; even if the
candidate count is altered to equal the shortened row list, the full-output
comparison must fail. Check the adapter's completion branch cannot be reached
by passing an arbitrary caller-authored row vector, count or flag in place of
the selected owner object. Exercise early stop, cancellation, parse/request
failure and missing selected module; verify each returns `Incomplete` with
partial rows and the original typed cause. For QSL, separately exercise
type-work exhaustion with all five native fields and unfinished admission with
its declaration handle. Pass a provisional envelope with `stop_reason: None`
and verify no API promotes it to `Complete`. Pass mismatched source, owner,
scope and row count to the neutral retention boundary and verify typed refusal.
These checks do not substitute for matched-source three-owner census tests.

Using a valid selection, omit the IR V2, QSL, and `quire-code-rs` envelopes one
at a time. For each owner, also supply a partial enumeration marked
`Incomplete` with an owner-typed cause; exercise early termination for IR V2,
QSL type-work exhaustion and unfinished typing, and a selected module parse failure
for `quire-code-rs`. Compare each with an owner response marked `Complete`
whose valid scope contains an empty subinventory, such as the function facts
for a fully parsed module with no functions. Supply a source mismatch, a wrong
owner/scope pairing, and an item count that differs from the retained items.
Then build the full two-criterion, three-occurrence, one-empty-module,
two-function fixture with explicit many-to-many assertions, and exercise
unmapped entities, foreign and dangling endpoints, exact duplicate inputs,
conflicting inventory facts, conflicting assertion relation facts, and
conflicting assertion provenance. Permute equivalent inputs and remove one
occurrence from a complete IR V2 inventory.

## Expected Results

The external consumer compiles and can borrow the census; the constructor has
no evidence-attempt inputs. Every complete fixture preserves all owner rows
and only its explicit valid relations, including visible zero-mapping entities.
For each owner, an absent envelope returns `MissingOwnerInventory` with that
owner and no census. An incomplete envelope returns
`IncompleteOwnerInventory` with the original typed cause and no census. A
fully parsed production module with no function facts is accepted as complete,
its module row remains visible, and it differs from both errors. A source
mismatch returns `InventorySourceMismatch`; a wrong owner/scope returns
`InventoryScopeMismatch`; and a count mismatch returns
`InventoryItemCountMismatch`, each with no census. Rejected assertions retain their distinct typed
reasons; `ConflictingRelation` and `ConflictingProvenance` are each directly
observed and neither conflicting relation is accepted. Exact duplicates
collapse, conflicting inventory entries remain inspectable, equivalent input
permutations produce equal ordered censuses, and removing one authoritative
occurrence changes the census.

For the IR-794 boundary, `Complete` is observed only after normal owner
enumeration ends, including valid empty output. The row-deletion mutation
fails the independent owner full-output comparison even with a self-consistent
shortened count. Every stopped path remains `Incomplete` with its original
typed cause and retained partial rows; QSL exhaustion and unfinished are
distinct. A missing stop cause on the provisional input remains provisional.
The neutral model rejects mismatched source, owner, scope or count. No result
from these checks alone is reported as a complete source-obligation census.
