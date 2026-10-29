---
id: SR-8162
title: spec-review review of IR-743 ContractPackage
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir;;
  spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md,
  spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md,
  spec/assurance/AD-001-contract-ir-architecture.md, spec/assurance/AD-004-checked-package-seam.md
review_set: subset
---

## Summary

Ticket: IR-743. Independent spec-review pass on the exact frozen candidate.

## Verdict

**CONDITIONAL** — 1 finding(s).

## Examined units

- `FR-035` (`examined`): The FR-330 package emitter and its backend adoption remain planned in IR-743 and IR-744 respectively.
- `FR-035-AC-5` (`examined`): One call emits exactly one FR-330 `quire.contract-package/v1` value with the six required properties and one key-ordered `semantic_graph` node collection; repeated equivalent construction orders have identical canonical semantic identity. The digest is computed from exactly `contract_version`, `package`, `semantic_graph` and `feature_accounting`: changing an included member changes it, changing only valid `source_map` locations preserves it, and invalid or incomplete `source_map` correspondence refuses admission.
- `AD-001` (`examined`): Contract IR emits this published wire and reads it through the strict admitted-package boundary; IR-744 will adopt it in codegen.
- `AD-004` (`examined`): Contract IR emits and strictly reads this output boundary; codegen consumer adoption remains IR-744.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-035 still says the FR-330 emitter remains planned in IR-743, while this candidate and updated AD-001/AD-004 claim the emitter and strict reader are implemented. The requirement status contradicts its own implementation boundary. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:103 |

## Coverage

Review limited to the candidate diff and named acceptance units. Full gate is owned by the author; reviewer ran no Cargo suites.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |

## Disposition verdict

**PASS (round 1, scoped)** — Every original finding was fixed by the listed commits. No new finding arose from the fix diff. Full gates are owned by the author.
