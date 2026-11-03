---
id: SR-20261103150649
title: "Base spec review of IR-1279 occurrence-keyed temporal bindings PR 860"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 860 head 9168940ba852d81957001d5d34a73a9987d27d68; FR-038 prose, AC-412..416, AC-437..441"
review_set: base
---

## Summary

Reviewed the FR-038 temporal binding prose and all ten changed acceptance criteria against merged QSpec FR-322 and its published temporal clock occurrence and compensation vectors. IR-1279 is the implementation ticket; the review examines the specification at PR #860 head.

## Verdict

PASS. The required occurrence member, exact pair key, clause claim and compensation declaration coverage, shared-node twins, row-wise refusal order, precise pointers, read/write preservation, bounded work, and by-reference conformance are specified consistently. Planned obligations are marked in the first criterion text cell; the no-configuration case alone retains its implemented claim. No new hash, pin, copied fixture, or compatibility path appears in this diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

QSpec origin/main FR-322 temporal binding prose and FR-322-AC-294, 359..370; QSpec temporal-clock-occurrence-vectors.json BO-P1/P2, BO-S1..S3, BO-R1..R9; temporal-clock-binding-vectors.json CB-01..08. Compared against the only changed file at PR head 9168940ba852d81957001d5d34a73a9987d27d68. `git diff --check` passes. No applicable AssuranceProfile file was found in this repository.

## Scope

- **FR-038** (examined): The V2 reader SHALL use QSpec FR-322's closed `TemporalBinding` row `{declaration, occurrence, clock, clock_name}` and its borrowed `CheckedPackageV2::temporal_bindings()` read.
- **FR-038-AC-412** (examined): planned until IR-1279 lands. A clock-configured package with one temporal clause and two `protocol`/`compensation` nodes admits three `temporal_bindings` rows (`CB-01`), and a package with two compensations and no clause admits two (`CB-02`). Each compensation row is keyed by its own node id and `declaration` occurrence ordinal; equal model, profile and clock names do not merge rows. The one array is in ascending `(NodeRef, occurrence)` order. The borrowed reader returns the exact admitted pairs, names and closed configurations without a per-compensation profile law.
- **FR-038-AC-413** (examined): Implemented for the no-configuration case. With a lock selection that has no clock configuration, two compensation nodes and no `temporal_bindings` member admit (`CB-08`); the reader exposes absence rather than synthesizing rows. The row schema changes under IR-1279 as specified in AC-437.
- **FR-038-AC-414** (examined): planned until IR-1279 lands. A missing required compensation row (`CB-03`) refuses `invalid_package`/`timed-node-reference` at that node's `/semantic_graph/nodes/{N}/body`, choosing the lowest graph index among missing clauses and compensations. A repeated `(declaration, occurrence)` pair (`CB-04`) refuses `invalid_package`/`conflicting-definition` at its second `/temporal_bindings/{i}` entry; an `await` or other non-clause, non-compensation declaration (`CB-07`) refuses `invalid_package`/`timed-node-reference` at `/temporal_bindings/{i}/declaration`. Each refusal reports the relevant node as 
- **FR-038-AC-415** (examined): planned until IR-1279 lands. For a resolved compensation row, a changed `clock_name` (`CB-05`) refuses `invalid_package`/`timed-node-reference` at `/temporal_bindings/{i}/clock_name` before configuration checking; a configuration kind differing from the lock's `temporal_profile` selection (`CB-06`) refuses that pair at `/temporal_bindings/{i}/clock/kind`. The reader uses the node's fifth text-valued `clock` argument and the lock selection, with no per-node temporal-profile law.
- **FR-038-AC-416** (examined): planned until IR-1279 lands. TC-048 reads QSpec `proposals/checked-package-v2/temporal-clock-binding-vectors.json` by reference through the selected checkout and checks `CB-01` through `CB-08` against exact admission or code, cause, primary RFC 6901 pointer and node locus, including the mandatory occurrence in each row. Because the compensation cases are abstract, the test constructs full compensation packages around their published declarations and rows; it keeps no local copy of the vectors, schema or fixture.
- **FR-038-AC-437** (examined): planned until IR-1279 lands. The public strict reader admits only the closed `TemporalBinding` members `{declaration, occurrence, clock, clock_name}` and the canonical writer emits the mandatory non-negative integer `occurrence` without a declaration-only fallback. It refuses BO-S1's omitted occurrence as `malformed_wire` at `/temporal_bindings/{i}` and BO-S2/BO-S3's negative or noninteger occurrence as `malformed_wire` at `/temporal_bindings/{i}/occurrence`; no package or partial row view is admitted.
- **FR-038-AC-438** (examined): planned until IR-1279 lands. BO-P1 admits two `claim` occurrences of one structural temporal-clause node, each with its own row and distinct valid clock configuration; BO-P2 admits the same twin shape with equal configurations without collapsing the rows. The borrowed read and canonical write/read-back retain each exact `(declaration NodeRef, occurrence ordinal)` key and configuration. Compensation rows use their node's `declaration` occurrence; clause and compensation rows share ascending pair order and are never selected by clock name, formula, array index or inferred path.
- **FR-038-AC-439** (examined): planned until IR-1279 lands. After earlier aggregate-order checks, each temporal binding row completes declaration join, occurrence-role/ordinal join, repeated-pair check and pair-order check before the next row. An absent or wrong-kind declaration refuses `invalid_package`/`timed-node-reference` at `/temporal_bindings/{i}/declaration`; an unowned or wrong-role occurrence (BO-R2) refuses the same pair at `/temporal_bindings/{i}/occurrence`; a repeated pair (BO-R3) refuses `invalid_package`/`conflicting-definition` at the later row. A descent (BO-R4) refuses `invalid_package`/`timed-node-refere
- **FR-038-AC-440** (examined): planned until IR-1279 lands. After all row joins, duplicates and ordering, every eligible clock-configured `claim` or compensation `declaration` occurrence has one row. An uncovered occurrence (BO-R1) refuses `invalid_package`/`timed-node-reference` at the lowest-index missing node's `/semantic_graph/nodes/{N}/body`; a row-order descent wins over that gap (BO-R7). After coverage, rows are checked in array order, comparing the second occurrence's `clock_name` before `clock.kind`: BO-R8 refuses the same code/cause at `/temporal_bindings/{i}/clock_name`, and BO-R9 at `/temporal_bindings/{i}/clock
- **FR-038-AC-441** (examined): planned until IR-1279 lands. TC-048 reads QSpec `proposals/checked-package-v2/temporal-clock-occurrence-vectors.json` from the selected checkout by reference, verifies BO-P1/P2, BO-S1..S3 and BO-R1..R9, and checks each complete package against exact admission or refusal code, cause, primary RFC 6901 pointer and structural node locus. It also reads the updated binding vectors by reference for CB-01..08 and the single- and multi-clause positives. Source-map entries remain charged by the existing occurrence limit; the reader charges binding rows before inspection and coverage work to the validati
