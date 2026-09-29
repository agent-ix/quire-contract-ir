---
id: SR-598
title: "scope-boundary review of PR 203 (ADR-0056, FR-345)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir@11c6b013a0c94ddbf76040cbe81142858935eae3; spec/decisions/ADR-0056-spec-layout-convention.md, spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md, spec/test-matrix.md, spec/index.md"
review_set: subset
---
# SR-598: scope-boundary review of PR 203

## Summary

Ticket: IR-314. Boundary between this repository's local check, Quire, and the Codegen/Runtime repositories.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-345 text is conditioned on Quire's future state (a schedule, not a requirement). | spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md:21-23 |

## Finding Detail

**FND-001** (low, confidence low, other; spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md:21-23): The requirement text is conditioned on an external tool's future state ('while quire validate does not ...'; ADR-0056:306-307 'when quire validate reports these defects, the local check is deleted'). That is a schedule, not a requirement; state what the check does and leave retirement to a later change.

## Scope

- `FR-345` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: make spec SHALL run a repository-local check that fails when two artifacts claim one identifier, when a test case is declared by more than one matrix row, when recorded ID blocks overlap, or when a relocation map disagrees with the tree, and that, given a base revision, fails a structural change whose identifier set or renames differ from its relocation map.
- `ADR-0056-tools` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: quire validate ... does not report two files declaring the same id. quire coverage --scope <DIR> takes the repository root ... a subsystem directory passed as --scope ... is refused. The TestMatrixIndex archetype validates the root index's Requirements Traceability columns and mints no test case.
- `ADR-0056-adoption` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: quire-contract-codegen and quire-contract-runtime adopt this layout the same way as this repository, each in its own structural pull request: 1-6.

## Verdict

The boundary is clear: the checks belong in quire validate, each repository runs its own local check, nothing is copied between repositories.
