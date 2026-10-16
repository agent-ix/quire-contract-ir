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

Mixed. The neutral outcome and synthetic census checks for FR-052-AC-1
through AC-6, AC-12 through AC-15, and AC-17 through AC-23 are Implemented
with traced tests in `tests/it/source_obligation.rs` and the default-feature
model-crate consumer. FR-052-AC-8 is retired (ADR-0056). Owner-native evidence
is still required where a criterion says so. FR-052-AC-24, split from AC-3, is
Planned until Linear IR-820 settles the `SemanticNode` fact. Owner-native completion, full-output comparison, and the
full matched-source census remain Planned/UNRUN under AC-7, AC-9 through
AC-11, and AC-16. Quire's selected-scope producer awaits PLAT-1165. Synthetic
outcomes do not establish owner-native exhaustion.

## Description

Verify that the neutral model port preserves complete owner inventories and
explicit source-author mappings, and returns typed errors rather than a
complete-looking census when any owner input is absent or incomplete.

## Test Procedure

Compile an external default-feature consumer that constructs
`SourceObligationInput`, calls `build_source_obligation_census`, and borrows
the resulting `SourceObligationCensus` and rows through
`quire_contract_model::source_obligation`. The test that does this is
specified under "Synthetic backing for FR-052-AC-1, AC-2, AC-3, AC-5 and
AC-6". Inspect the public constructor
signature and public-item inventory for the absence of result, method, profile,
artifact, attempt, and run inputs.

For the owner-native criteria (FR-052-AC-7, AC-9 through AC-11, and AC-16),
build four owner inputs with their native APIs or typed adapters. For Quire,
select a validated spec corpus and its declared obligation sources. Enumerate
every authored acceptance criterion as the tuple of obligation declaration,
scope-relative document and authored criterion ID, including a criterion with
no correspondence assertion or test binder. Compare with the complete
selected Quire output, not a caller-authored row list. `ClauseRef` names a
separate formal clause; if a clause relation is needed, supply it explicitly
and verify that it cannot change criterion membership. Bind the selected Quire
scope to the chosen IR `SourceIdentity` through an explicit source-author
binding and test missing, foreign and conflicting bindings separately. A
matching source label alone shall not count as identical source bytes.
For IR V2,
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
each `SourceInventoryOutcome` carry its matching owner, selected source, scope, observed count, and the
owner-emitted completeness state. Verify that `Complete` is emitted only after
normal exhaustion of that owner's full scope, and compare every complete
outcome's entries with the full owner output for that selection. An owner
item omitted from an otherwise populated selected source must prevent a
complete outcome.

Until PLAT-1165 provides a Quire API that issues terminal outcomes, record the
Quire owner portion of this procedure as UNRUN. `obligation::derive` returns
rows plus `SkippedRow` but no terminal state or scan diagnostics; it cannot
serve as an owner-issued `Complete`. Once that API exists, exercise valid
zero-row selection, unreadable or skipped selected documents, missing row ID
or statement, failed discovery, missing declared table or ID column, and an
early stop. Compare each typed cause and retained partial rows with Quire's
own selected output. Deleting or changing one authored criterion must change
the inventory before generation or proof results; deleting one emitted row
after full output, even with an adjusted count, must fail owner comparison.

For the IR-794 boundary, drive each owner-native enumerator or typed adapter
from its selected owner object to normal exhaustion and capture its full
output independently of the model outcome. Compare source, typed owner,
scope, every row and observed count field by field. Repeat with a complete
zero-row scope and a fully parsed production module with zero functions.
Delete one emitted row after the owner produces its full output; even if the
candidate count is altered to equal the shortened row list, the full-output
comparison must fail. Check the adapter's completion branch cannot be reached
by passing an arbitrary caller-authored row vector, count or flag in place of
the selected owner object. Exercise early stop, parse/request failure and
missing selected module; verify each returns `Incomplete` with partial rows
and the cause actually reported by that owner path. For QSL, separately
exercise type-work exhaustion with dimension, limit, prior, requested charge
and site, and unfinished admission with its declaration handle. Exercise an
IR V2 reader limit and compare its `CheckedPackageIncomplete` limit kind,
limit, consumed count and optional path field by field through the distinct
neutral cause. On an owner path that accepts `quire_exact::Cancel`, trigger
`Requested` and `Deadline` separately and compare the corresponding neutral
`SourceInventoryCancelCause` variants; do not attribute this behavior to the
current QSL inventory or `quire-code-rs` adapter. Confirm that an owner stop
type with no typed neutral projection cannot be silently converted to an
`EarlyStop`, `RequestFailed` string or `Complete`. Verify that no API
promotes an absent stop cause to `Complete`; `Complete` follows only from the
enumerator's `Ok(None)`. Pass mismatched source, owner,
scope and row count to the neutral retention boundary and verify typed refusal.
These checks do not substitute for matched-source four-owner census tests.

Separately, drive a synthetic `SourceInventoryEnumerator` with zero and
multiple rows. Observe that only `Ok(None)` produces a neutral `Complete`
outcome, while replacing it with `Err(typed_cause)` produces
`Incomplete(typed_cause)` retaining exactly the preceding rows. Compare the
returned count with the retained row count for zero and nonzero streams.
Mutate the synthetic enumerator's reported count, owner/scope pair, row owner,
row source and stop-cause owner one at a time; each shall return its named
typed structural error and no outcome. This procedure tests only the neutral
terminal and structural boundary, not owner-native full-output issuance.
Compile external default-feature consumer fixtures against
`quire_contract_model::source_obligation`: one positive fixture borrows the
retained rows and reads the observed count through public accessors; separate
negative fixtures directly read the outcome's retained-rows field, directly
read its count field, and construct an outcome with a struct literal. Require
each negative fixture to fail specifically because the outcome fields are
private, not because of an unrelated import, type or syntax error. A change
that makes either field public or permits literal construction must turn its
corresponding negative fixture into an unexpected compile success.

The census checks below use synthetic outcomes. The FR-052-AC-6 exercise
(each owner outcome omitted in turn, a typed `Err` stop in each position, and
the complete empty module) is specified under "Synthetic backing for
FR-052-AC-1, AC-2, AC-3, AC-5 and AC-6". Its `quire-code-rs` stop is
`ModuleUnavailable`. Supply to the census an outcome whose source differs from the selected
source, and a well-formed outcome in the wrong owner position. Separately
supply an owner/scope pair mismatch to the outcome boundary. Separately supply an observed count that
differs from the retained rows, and a foreign-source row, to the outcome
boundary.
The FR-052-AC-2 fixture and the FR-052-AC-3 dispositions are specified in
that same section. Conflicting inventory facts (a `SemanticNode` repeated
with one `CheckedNodeId` and differing `CheckedSemanticId`) belong to
FR-052-AC-24, which waits on Linear IR-820.

Separately, build the same fixture from four synthetic `Complete` outcomes
and a valid binding. Its QSL outcome holds at least one `RequirementRecord` row
and one `PendingObligation` row, and one occurrence is asserted only without a
production function. Hand-write the expected census for this fixture: every
inventory row with its disposition and missing-mapping reason, every accepted
row with its provenance, every rejection, and their order. Compare the built
census with it for equality. Do not derive the expectation by re-running the
census construction rules in a helper. The QSL rows' expected
`ZeroMappings(NoAuthoredMapping)` is an IR reading pending owner confirmation.
Permute assertion order and the row order of each outcome.
Inject two faults in each adjacent pair of positions in the order missing or
foreign binding, Quire outcome, conflicting binding, IR V2, QSL,
`quire-code-rs`:
- a foreign binding with an incomplete Quire outcome;
- an incomplete Quire outcome with a conflicting binding;
- a conflicting binding with a missing IR V2 outcome;
- missing IR V2 and QSL outcomes;
- an incomplete QSL outcome with a missing `quire-code-rs` outcome.
Fault one position twice: wrong position with incomplete, and wrong source with
incomplete. Supply a missing binding, a binding naming another source, and a
binding whose obligation scope differs from a Quire criterion row's obligation
declaration. That last case uses the conflicting-binding definition, which is
an IR reading pending owner confirmation.
Supply an assertion that is both foreign and dangling, and one assertion ID
whose members differ in both relation and provenance. Supply two assertion IDs
stating one relation with equal provenance, and two with differing provenance.
Supply three assertions whose endpoints do not resolve: one whose occurrence
lies under a conflicting `SemanticNode`, one whose occurrence has no
`SemanticNode` row, and one whose production function has no
`ProductionModule` row. These checks use synthetic outcomes only.

### Synthetic backing for FR-052-AC-1, AC-2, AC-3, AC-5 and AC-6

These tests use synthetic outcomes. Each criterion has the traced tests
below; none grants owner-issued completeness, owner full-output comparison,
or a complete four-owner census.

- FR-052-AC-1 and AC-5: add a `#[test]` external consumer in a new
  model-crate integration target,
  `crates/quire-contract-model/tests/source_obligation_public.rs`, traced with
  `#[trace("TC-452", "FR-052-AC-1", "FR-052-AC-5")]`. This follows the
  FR-053-AC-1 precedent in `tests/selected_population_public.rs`; `quire
  matrix` binds a trace only to a `#[test]` function, not to a doctest. A
  `tests/it` test does not satisfy "default-feature": the root package's
  dev-dependency enables `fault-injection`, and a `--workspace` build unifies
  that feature into the model crate. So add a `make test` line that runs
  `cargo test -p quire-contract-model --test source_obligation_public` on its
  own, which builds the model crate with only its default features, next to
  the existing `-p quire-contract-model --doc` line. The test imports the
  input, constructor, census and row types from
  `quire_contract_model::source_obligation`. It builds four synthetic
  `Complete` outcomes through a `SourceInventoryEnumerator` and a valid
  binding, then calls `SourceObligationInput::new` and
  `build_source_obligation_census`. It borrows the `SourceObligationCensus`
  and, through `inventory_rows()`, `correspondence_rows()` and
  `rejected_assertions()`, a `SourceCensusInventoryRow`, a
  `SourceObligationRow` and a `SourceCorrespondenceRejection`, each bound to an
  explicitly typed reference. A doctest on `build_source_obligation_census`
  may repeat this as extra evidence, but it is not the traced test. For the
  inventory half of AC-1, also trace
  `tc_058_model_root_exports_exactly_the_documented_public_items` to
  FR-052-AC-1: it compares the `source_obligation` module's top-level public
  items with FR-019's table in both directions. When AC-1 and AC-5 are
  backed, add them to the FR-052 list in FR-019's `source_obligation`
  public-items row; that list must not name them before then.
- FR-052-AC-6: add one test in `tests/it/source_obligation.rs` that keeps the
  other three positions complete and, for each of the Quire, IR V2, QSL and
  `quire-code-rs` positions in turn:
  - omits that outcome and observes `MissingOwnerInventory` naming the owner
    and no census;
  - supplies an outcome whose enumerator ended with `Err(cause)` and observes
    `IncompleteOwnerInventory` for that owner, with a cause equal to the
    supplied one, and no census. The causes are `EarlyStop` for Quire; both
    `EarlyStop` and `IrV2ReadLimit` with its `CheckedPackageIncomplete` fields
    for IR V2; both `TypeWorkExhausted` and `TypeAdmissionUnfinished` for QSL;
    and `ModuleUnavailable` for `quire-code-rs`.

  The same test builds the census with a `Complete` `quire-code-rs` outcome
  holding a module row with no function facts. It observes a census, that the
  module row stays with `ZeroMappings(NoAuthoredMapping)`, and that this
  result differs from both the missing and the incomplete results. The
  synthetic Quire `Complete` outcome is not owner evidence.
- FR-052-AC-2: add one test that builds the AC-2 fixture: two criteria, one of
  them unmapped; three occurrences; one empty module; two production
  functions. A two-function fixture cannot both map every function
  many-to-many and leave one function unmapped, so the test builds the same
  inventory with two assertion sets:
  - The first set is many-to-many: one occurrence maps to both functions, and
    one function maps from two occurrences. Assert the exact accepted rows with
    their selected-source provenance and every inventory row.
  - The second set leaves one function, one occurrence and the second criterion
    with no accepted mapping. Assert each is visible with
    `ZeroMappings(NoAuthoredMapping)`, and the empty module likewise.

  For the first set, permute the assertion order and each outcome's row order
  and assert an equal census. Then rebuild it with one occurrence row removed
  from the `Complete` IR V2 outcome. Assert that the census differs and lacks
  that occurrence row.
- FR-052-AC-3: trace a test to AC-3 only where it asserts the clause:
  - foreign source and the dangling criterion and production symbol:
    `foreign_and_dangling_assertions_remain_separate_from_unmapped_inventory`;
  - an occurrence that does not resolve, and a function whose module row is
    absent: `orphan_or_conflicting_endpoints_do_not_resolve`;
  - exact duplicate assertions:
    `rejected_assertions_collapse_duplicates_and_sort_with_reason_precedence`,
    which observes one rejection for a repeated assertion. Do not use
    `assertion_conflicts_and_relation_dedup_are_deterministic` for this clause:
    its repeated assertion states the same relation and provenance as another
    ID, so relation deduplication (step 4) yields one row even without the
    exact-duplicate collapse (step 1);
  - exact duplicate inventory rows:
    `synthetic_census_has_hand_written_rows_and_dispositions`;
  - the foreign-source inventory row refused at the outcome boundary:
    `terminal_boundary_preserves_typed_stops_and_rejects_mismatched_rows`.

  Three clauses need new assertions before AC-3 is traced:
  - `assertion_reason_precedence_rejects_foreign_before_dangling_and_relation_before_provenance`
    observes `ConflictingRelation` but does not assert that no accepted row
    holds the conflicting relation; add that assertion.
  - No test asserts `DanglingOccurrence` for an assertion with a valid
    criterion whose occurrence is absent from the IR V2 inventory; add that
    case.
  - Conflicting assertion provenance means one `SourceAssertionId` whose
    members state one relation with differing provenance (step 2). The
    existing provenance cases use two assertion IDs, which is FR-052-AC-22's
    case (step 4). Add a case with one assertion ID, one relation and only
    the provenance differing; observe that every member is rejected with
    `ConflictingProvenance` and that no accepted row holds the relation.

  Do not trace `orphan_or_conflicting_endpoints_do_not_resolve` to
  FR-052-AC-24 before IR-820 is decided.

## Expected Results

The external consumer compiles and can borrow the census; the constructor has
no evidence-attempt inputs. Every complete fixture preserves all owner rows
and only its explicit valid relations, including visible zero-mapping entities.
The Quire owner row uses the authored acceptance-criterion identity, not a
formal `ClauseRef` or QSL requirement key. The selected-scope binding is
explicit and missing, foreign and conflicting bindings refuse completion.
Quire's `Complete` result and the full matched-source result remain UNRUN until
PLAT-1165 provides an owner-issued outcome and the four real owner selections
are shown to agree; synthetic criterion fixtures cannot discharge them.
For each owner, an absent outcome returns `MissingOwnerInventory` with that
owner and no census. An incomplete outcome returns
`IncompleteOwnerInventory` with the original typed cause and no census. A
fully parsed production module with no function facts is accepted as complete,
its module row remains visible, and it differs from both errors. A source
mismatch returns `InventorySourceMismatch`; a well-formed outcome in the wrong owner position returns
`InventoryScopeMismatch`, each with no census; an owner/scope pair mismatch is
refused earlier as `OwnerScopeMismatch` with no outcome. A count mismatch is
refused by `SourceInventoryOutcomeError::ObservedCountMismatch` and a foreign
row by `RowSourceMismatch`, each with no outcome. Rejected assertions retain their distinct typed
reasons; `ConflictingRelation` and `ConflictingProvenance` are each directly
observed and neither conflicting relation is accepted. Exact duplicates
collapse, conflicting inventory entries remain inspectable (FR-052-AC-24,
pending IR-820), equivalent input
permutations produce equal ordered censuses, and removing one authoritative
occurrence changes the census.

For the IR-794 boundary, `Complete` is observed only after normal owner
enumeration ends, including valid empty output. The row-deletion mutation
fails the independent owner full-output comparison even with a self-consistent
shortened count. Every stopped path remains `Incomplete` with its original
typed cause and retained partial rows; QSL exhaustion and unfinished are
distinct, IR V2 reader limits retain their fields, and an actually reported
requested/deadline cancellation remains two typed causes. An absent stop cause
is never promoted to `Complete`.
The neutral model rejects mismatched source, owner, scope or count. No result
from these checks alone is reported as a complete source-obligation census.

The synthetic boundary yields `Complete` only for `Ok(None)`, preserves the
exact typed `Err` as `Incomplete`, and rejects each structural mismatch with
no outcome. Zero and nonzero retained row counts agree with the private
outcome count. The accessor fixture compiles; each direct-field or struct-
literal fixture fails for field privacy, and none fails for an unrelated
reason. None of these observations grants owner-native AC-9 through
AC-11 credit.

The synthetic census equals the hand-written expected census and is equal
under permutation. Each QSL row is `ZeroMappings(NoAuthoredMapping)`, which is
an IR reading pending owner confirmation. Each adjacent pair returns only the
earlier position's error, in the order missing or foreign binding, Quire
outcome, conflicting binding, IR V2, QSL, `quire-code-rs`:
`ForeignCriterionSourceBinding`, `IncompleteOwnerInventory` for Quire,
`ConflictingCriterionSourceBinding`, `MissingOwnerInventory` for IR V2, and
`IncompleteOwnerInventory` for QSL. Within one position, scope mismatch
precedes source mismatch, which precedes incompleteness. Missing, foreign and
conflicting bindings return `MissingCriterionSourceBinding`,
`ForeignCriterionSourceBinding` and `ConflictingCriterionSourceBinding`; the
conflicting case follows the IR reading of a conflicting binding, pending owner
confirmation. Each rejected assertion carries the single reason the precedence
selects. One relation stated by two IDs with equal provenance yields one
accepted row, and with differing provenance both assertions are rejected with
`ConflictingProvenance`. Each assertion whose occurrence lies under a
conflicting node or has no node row is rejected with `DanglingOccurrence`. The
assertion whose function has no module row is rejected with
`DanglingProductionSymbol`. The orphan and conflicting rows stay in the census.
None of these observations grants AC-7 or AC-9 through AC-11 credit.

For the synthetic backing of FR-052-AC-1, AC-2, AC-3, AC-5 and AC-6, the
`source_obligation_public` test, run with the model crate's default features
only, compiles and borrows the census and its rows, and tc_058 finds FR-019's table equal to the exported items. Each omitted owner
outcome returns `MissingOwnerInventory` for that owner, and each `Err` stop
returns `IncompleteOwnerInventory` with the supplied cause; no census is
returned in either case. The complete empty module yields a census that keeps
its module row. The AC-2 fixture keeps every inventory row and exactly the
asserted relations, and its unmapped criterion, occurrence, function and
module are visible with `ZeroMappings(NoAuthoredMapping)`. Its permuted
census is equal, and the census without one IR V2 occurrence differs. Each
AC-3 clause receives its named disposition, including `ConflictingProvenance`
for every member of one assertion ID that differs only in provenance, and no
rejected assertion is an accepted row. These observations grant no owner-issued completeness, Quire
owner evidence, or four-owner census credit.
