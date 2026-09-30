---
id: SR-596
title: "evidence review of PR 203 (ADR-0056, FR-345)"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; spec/decisions/ADR-0056-spec-layout-convention.md, spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md, spec/test-matrix.md, spec/index.md"
review_set: subset
---
# SR-596: evidence review of PR 203

## Summary

Ticket: IR-314. Verification method and evidence for FR-345's criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Finding Detail

None.

## Scope

- `FR-345-AC-1` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: Two spec artifacts declaring one frontmatter id fail the check with the identifier and both paths; the same identifier in body prose alone, or declared twice only by files under plan/, reviews/ or spec/reviews/, does not.
- `FR-345-AC-2` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: A TC ID leading two Test Case Summary rows fails the check with the identifier and each matrix:line, both when the rows are in one matrix and when they are in two; citing the ID in another table does not.
- `FR-345-AC-3` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: Two ## ID Blocks rows of one family with a common ID fail the check naming both rows; adjacent non-overlapping ranges and ranges of different families pass.
- `FR-345-AC-4` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: A relocation map with a wrong header, unsorted rows, a repeated old_path, a new_path absent from the tree, or a new_path whose file declares a different id than new_id fails the check with path:line for each defect.
- `FR-345-AC-5` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: With SPEC_BASE set, a structural change that drops, adds or renumbers an ID, or renames a file with no row in an added relocation map, fails the check; the same moves with a complete, ID-preserving relocation map pass.
- `FR-345-AC-6` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: A tree with none of these defects exits 0 with one summary line, and make spec fails whenever the check fails.
- `TM-001-FR-345` (spec/test-matrix.md), examined: | FR-345 | FR-345-AC-1 through FR-345-AC-6 | TC-224 | planned; the repository-local ID and relocation-map check is not built and make spec does not run it. |
- `TM-001-TC-224` (spec/test-matrix.md), examined: | TC-224 | The repository-local ID and relocation-map check fails on each seeded defect, names the file and line, and passes a clean tree | Integration | P0 | FR-345 | planned |

## Verdict

Clean. Every AC names Test (TC-224); TC-224 is declared once in TM-001 with FR-345 as its trace and both rows are marked planned, which quire coverage reports as the eight new unbacked rows rather than a status lie (status_lies 0).
