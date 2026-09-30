---
id: SR-597
title: "dependency review of PR 203 (ADR-0056, FR-345)"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; spec/decisions/ADR-0056-spec-layout-convention.md, spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md, spec/test-matrix.md, spec/index.md"
review_set: subset
---
# SR-597: dependency review of PR 203

## Summary

Ticket: IR-314. Relationship edges added by the diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Finding Detail

None.

## Scope

- `FR-345` (spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md), examined: make spec SHALL run a repository-local check that fails when two artifacts claim one identifier, when a test case is declared by more than one matrix row, when recorded ID blocks overlap, or when a relocation map disagrees with the tree, and that, given a base revision, fails a structural change whose identifier set or renames differ from its relocation map.
- `ADR-0056-context` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Three existing trees already show the target shape: filament-ide-rs ..., quire-specification ..., ecaz ...
- `index-ADR-0056` (spec/index.md), examined: ADR-0056 fixes the subsystem specification layout, registry format and ID-block allocation that this repository, Contract Codegen and Contract Runtime follow. FR-345 is this repository's local artifact-ID and relocation-map check that ADR-0056 names; TM-001 covers it.

## Verdict

Clean. FR-345 traces_to StR-003 and references ADR-0056; ADR-0056 relates_to AD-001, quire-rs FR-050 (exists at quire-rs origin/main) and FR-345. All targets exist; no cycle beyond the ADR<->FR relates_to/references pair, which is informational.
