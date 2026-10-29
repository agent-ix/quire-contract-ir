---
id: SR-8164
title: spec-review/ears-conformance review of IR-743 ContractPackage
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-contract-ir;;
  spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md,
  spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md
review_set: subset
---

## Summary

Ticket: IR-743. Independent spec-review/ears-conformance pass on the exact frozen candidate.

## Verdict

**PASS** — no findings in this scoped method.

## Examined units

- `FR-035` (`examined`): The FR-330 package emitter and its backend adoption remain planned in IR-743 and IR-744 respectively.
- `FR-035-AC-5` (`examined`): One call emits exactly one FR-330 `quire.contract-package/v1` value with the six required properties and one key-ordered `semantic_graph` node collection; repeated equivalent construction orders have identical canonical semantic identity. The digest is computed from exactly `contract_version`, `package`, `semantic_graph` and `feature_accounting`: changing an included member changes it, changing only valid `source_map` locations preserves it, and invalid or incomplete `source_map` correspondence refuses admission.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Review limited to the candidate diff and named acceptance units. Full gate is owned by the author; reviewer ran no Cargo suites.
