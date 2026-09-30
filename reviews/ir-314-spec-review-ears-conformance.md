---
id: SR-593
title: "ears-conformance review of PR 203 (ADR-0056, FR-345)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; spec/decisions/ADR-0056-spec-layout-convention.md, spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md, spec/test-matrix.md, spec/index.md"
review_set: subset
---
# SR-593: ears-conformance review of PR 203

## Summary

Ticket: IR-314. EARS form, atomicity and testability of FR-345's statement, behaviour bullets and six ACs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | SPEC_BASE trigger is vacuously true for a change with no relocation map, so every ordinary spec PR fails ID-set equality. | spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md:58-61 |
| FND-002 | low | A collision-renumbering map disables the ID-set comparison entirely. | spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md:58-61 |
| FND-003 | low | 'SHALL NOT use the network' has no AC. | spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md:45-46 |

## Finding Detail

**FND-001** (medium, confidence high, ambiguous; spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md:58-61, 73): The behaviour trigger is 'When SPEC_BASE is set and every row of the relocation maps the change adds has equal old_id and new_id'. For a change that adds no relocation map that is vacuously true, so any ordinary spec PR adding an FR (or renaming a file) fails the check whenever SPEC_BASE is set, e.g. if make spec sets it to origin/main. AC-5 says 'a structural change' but the check has no way to tell one apart. Fix: trigger only when the change adds at least one relocation map.

**FND-002** (low, confidence medium, untestable-ac; spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md:58-61): When an added map records a collision renumbering (old_id != new_id), no ID-set comparison runs at all, so a renumbering PR can also drop or add unrelated IDs undetected. Require the base/head set difference to equal exactly the renumbered pairs.

**FND-003** (low, confidence high, coverage; spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md:45-46): This SHALL has no acceptance criterion, so TC-224 is not required to verify it. Add an AC or make it a non-behavioural constraint.

## Scope

- `FR-345` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: make spec SHALL run a repository-local check that fails when two artifacts claim one identifier, when a test case is declared by more than one matrix row, when recorded ID blocks overlap, or when a relocation map disagrees with the tree, and that, given a base revision, fails a structural change whose identifier set or renames differ from its relocation map.
- `FR-345-AC-1` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: Two spec artifacts declaring one frontmatter id fail the check with the identifier and both paths; the same identifier in body prose alone, or declared twice only by files under plan/, reviews/ or spec/reviews/, does not.
- `FR-345-AC-2` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: A TC ID leading two Test Case Summary rows fails the check with the identifier and each matrix:line, both when the rows are in one matrix and when they are in two; citing the ID in another table does not.
- `FR-345-AC-3` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: Two ## ID Blocks rows of one family with a common ID fail the check naming both rows; adjacent non-overlapping ranges and ranges of different families pass.
- `FR-345-AC-4` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: A relocation map with a wrong header, unsorted rows, a repeated old_path, a new_path absent from the tree, or a new_path whose file declares a different id than new_id fails the check with path:line for each defect.
- `FR-345-AC-5` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: With SPEC_BASE set, a structural change that drops, adds or renumbers an ID, or renames a file with no row in an added relocation map, fails the check; the same moves with a complete, ID-preserving relocation map pass.
- `FR-345-AC-6` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: A tree with none of these defects exits 0 with one summary line, and make spec fails whenever the check fails.

## Verdict

The statement and behaviour bullets use SHALL with explicit When-triggers; AC-1..AC-4 and AC-6 are testable, name path:line outputs and include a negative case each. The SPEC_BASE trigger is defective and one behaviour has no AC.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | resolved |
| FND-002 | fixed | resolved |
| FND-003 | fixed | resolved |
