---
id: SR-202611010114
title: "IR-1247 PR 793 failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; PR #793; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Checked identity confusion and graph-boundary failures for local references, direct package references, self-minted foreign references, and structurally ordered equivalence pairs. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**PASS** — distinct carriers prevent a foreign declared node from masquerading as an importing-graph node, and exact typed refusals cover missing selection, missing declaration, wrong kind, and self-minted local references.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038 dimension operand paragraph, role: examined. The direct-dependency lookup and no-mint rule close the graph identity boundary.
- FR-038-AC-387, role: examined. Positive mixed-package admission keeps only local operands in local dependencies.
- FR-038-AC-388, role: examined. Missing direct selection, closure-only selection, self selection, and absent declared node are distinct.
- FR-038-AC-389, role: examined. Wrong-kind, self-minted foreign local reference, reversed, duplicate, and other pair defects have stated refusals.
- FR-019 planned API paragraph, inventory row, and AC-17, role: examined. Public types preserve local/package distinction without silent conversion.
- QSpec FR-322-AC-299, role: context_only. It supplies the exact carrier and refusal envelope.

## Evidence

The spec requires no foreign node to be minted in the importing graph. It separates package lookup failures at `/package` and `/node` from invalid operand shapes at the operand pointer.
