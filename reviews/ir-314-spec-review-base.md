---
id: SR-978
title: "base review of PR 203 (ADR-0056, FR-345)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/decisions/ADR-0056-spec-layout-convention.md, spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md, spec/test-matrix.md, spec/index.md"
review_set: subset
---
# SR-978: base review of PR 203

Former id: SR-591 (cited by the marker on IR-314).

## Summary

Ticket: IR-314. Base checklist over ADR-0056, FR-345, the two TM-001 rows and the index entry: ID formats, duplicates, link integrity and the coverage rules. FR-345 and TC-224 are new, unduplicated IDs (next free after FR-344 and TC-223); every relative link in the diff resolves; FR-345's six ACs are each cited by TM-001 and each verified by TC-224.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Finding Detail

None.

## Scope

- `ADR-0056-layout` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Directory layout rules 1-8: spec/spec.md is the only master-requirements document; subsystem directory holds six kind directories; subsystems derived from AD-001; core; STD placement; AD/assurance in spec/assurance/; ADRs in spec/decisions/; reviews in root reviews/, spec/reviews/ does not exist.
- `ADR-0056-registry` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: spec/spec.md carries a ### Subsystem Registry table with exactly these columns: Subsystem | Path | Role | Owning crates/modules | ADs | Owner, one row per subsystem directory, core first.
- `ADR-0056-traceability` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: spec/tests.md is typed TestMatrixIndex. It carries a ## Requirements Traceability table with the columns Subsystem | Requirements | Local Matrix | Status, one row per subsystem.
- `ADR-0056-identifiers` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: IDs are flat and global per repository per family ... never carries a subsystem prefix ... never renumbered by a move ... An unprefixed ID names an artifact in the same repository ... ix://agent-ix/<repo>/<ID> ... next free ID ... A spec artifact is a document under spec/ outside spec/reviews/.
- `ADR-0056-idblocks` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: When two or more authors write specifications in one repository at the same time, each writes only IDs from a block issued to it. ... recorded in the ## ID Blocks table of spec/spec.md in one commit on the default branch, before dispatch.
- `ADR-0056-collision` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: A collision is two spec artifacts declaring the same frontmatter id, or two matrix rows declaring the same TC in a Test Case Summary. make spec fails on either, naming both files.
- `ADR-0056-matrices` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Each subsystem has exactly one matrix ... The root spec/tests.md is typed TestMatrixIndex ... Every TC is declared by exactly one matrix ... A new root index or a new subsystem matrix takes the next free TM ID.
- `ADR-0056-gate` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Restructure gate 1-8: structural first (adds no ID), one writer, relocation map TSV old_path/new_path/old_id/new_id, ID-set equality, no meaning change, link check, embedded-path scan, gates unchanged.
- `ADR-0056-tools` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: quire validate ... does not report two files declaring the same id. quire coverage --scope <DIR> takes the repository root ... a subsystem directory passed as --scope ... is refused. The TestMatrixIndex archetype validates the root index's Requirements Traceability columns and mints no test case.
- `ADR-0056-adoption` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: quire-contract-codegen and quire-contract-runtime adopt this layout the same way as this repository, each in its own structural pull request: 1-6.
- `ADR-0056-context` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Three existing trees already show the target shape: filament-ide-rs ..., quire-specification ..., ecaz ...
- `FR-345` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: make spec SHALL run a repository-local check that fails when two artifacts claim one identifier, when a test case is declared by more than one matrix row, when recorded ID blocks overlap, or when a relocation map disagrees with the tree, and that, given a base revision, fails a structural change whose identifier set or renames differ from its relocation map.
- `FR-345-AC-1` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: Two spec artifacts declaring one frontmatter id fail the check with the identifier and both paths; the same identifier in body prose alone, or declared twice only by files under plan/, reviews/ or spec/reviews/, does not.
- `FR-345-AC-2` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: A TC ID leading two Test Case Summary rows fails the check with the identifier and each matrix:line, both when the rows are in one matrix and when they are in two; citing the ID in another table does not.
- `FR-345-AC-3` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: Two ## ID Blocks rows of one family with a common ID fail the check naming both rows; adjacent non-overlapping ranges and ranges of different families pass.
- `FR-345-AC-4` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: A relocation map with a wrong header, unsorted rows, a repeated old_path, a new_path absent from the tree, or a new_path whose file declares a different id than new_id fails the check with path:line for each defect.
- `FR-345-AC-5` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: With SPEC_BASE set, a structural change that drops, adds or renumbers an ID, or renames a file with no row in an added relocation map, fails the check; the same moves with a complete, ID-preserving relocation map pass.
- `FR-345-AC-6` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: A tree with none of these defects exits 0 with one summary line, and make spec fails whenever the check fails.
- `TM-001-FR-345` (spec/test-matrix.md), examined: | FR-345 | FR-345-AC-1 through FR-345-AC-6 | TC-224 | planned; the repository-local ID and relocation-map check is not built and make spec does not run it. |
- `TM-001-TC-224` (spec/test-matrix.md), examined: | TC-224 | The repository-local ID and relocation-map check fails on each seeded defect, names the file and line, and passes a clean tree | Integration | P0 | FR-345 | planned |
- `index-ADR-0056` (spec/index.md), examined: ADR-0056 fixes the subsystem specification layout, registry format and ID-block allocation that this repository, Contract Codegen and Contract Runtime follow. FR-345 is this repository's local artifact-ID and relocation-map check that ADR-0056 names; TM-001 covers it.

## Verdict

Clean. Measured on a detached worktree at the PR head: no spec-artifact id (85 documents under spec/ outside spec/reviews/) is declared twice; no TC leads two Test Case Summary rows across TM-001 and TM-002 (45 rows). `make spec` exits 2 at both origin/main and the PR head, failing only on MP-001 and MP-002 MeasurementPlan frontmatter. `quire coverage --scope . --strict --json` exits 1 at both; unbacked rows go 37 -> 45, the eight added being FR-345, FR-345-AC-1..6 and TC-224, none removed; backed stays 224 (total 267 -> 274). `scripts/validate_matrix_status.py` exits 0 at the PR head.
