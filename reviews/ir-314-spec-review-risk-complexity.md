---
id: SR-595
title: "risk-complexity review of PR 203 (ADR-0056, FR-345)"
type: SpecReview
analysis: risk-complexity
scope: "agent-ix/quire-contract-ir@11c6b013a0c94ddbf76040cbe81142858935eae3; spec/decisions/ADR-0056-spec-layout-convention.md, spec/functional/FR-345-check-artifact-ids-and-relocation-maps.md, spec/test-matrix.md, spec/index.md"
review_set: subset
---
# SR-595: risk-complexity review of PR 203

## Summary

Ticket: IR-314. Volatility and verification risk in the restructure gate and the tool-support claims.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | 'ix:// resolves at least as well' names no measure. | spec/decisions/ADR-0056-spec-layout-convention.md:262-264 |

## Finding Detail

**FND-001** (low, confidence medium, untestable-ac; spec/decisions/ADR-0056-spec-layout-convention.md:262-264): No tool or measure is named for 'resolves at least as well', so the clause cannot be checked. Intra-repo ix:// targets resolve by ID, which gate rule 4 already fixes; either name the resolver and metric or drop the clause.

## Scope

- `ADR-0056-gate` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: Restructure gate 1-8: structural first (adds no ID), one writer, relocation map TSV old_path/new_path/old_id/new_id, ID-set equality, no meaning change, link check, embedded-path scan, gates unchanged.
- `ADR-0056-tools` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: quire validate ... does not report two files declaring the same id. quire coverage --scope <DIR> takes the repository root ... a subsystem directory passed as --scope ... is refused. The TestMatrixIndex archetype validates the root index's Requirements Traceability columns and mints no test case.
- `ADR-0056-adoption` (spec/decisions/ADR-0056-spec-layout-convention.md), examined: quire-contract-codegen and quire-contract-runtime adopt this layout the same way as this repository, each in its own structural pull request: 1-6.

## Verdict

The tool-support section is pinned to what was measured and reproduces; one gate clause has no measurement.
