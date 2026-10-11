---
id: SR-8163
title: spec-review/integrity review of IR-743 ContractPackage
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir@3d97f7250257e7d5f83ebe8855428ef9321b5e44; base b2008d3f50127bf033b5bae8fb302a482e536dbc;
  spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md,
  spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/assurance/AD-001-contract-ir-architecture.md,
  spec/assurance/AD-004-checked-package-seam.md
review_set: subset
---

## Summary

Ticket: IR-743. Independent spec-review/integrity pass on the exact frozen candidate.

## Verdict

**CONDITIONAL** — 1 finding(s).

## Examined units

- `FR-035` (`examined`): The FR-330 package emitter and its backend adoption remain planned in IR-743 and IR-744 respectively.
- `TC-047` (`examined`): Planned / unrun for the FR-330 package emitter. The existing `tests/it/complete_v1_contract_package.rs` asserts the earlier IR package shape and cannot discharge this test until IR-743 updates it.
- `AD-001` (`examined`): Contract IR emits this published wire and reads it through the strict admitted-package boundary; IR-744 will adopt it in codegen.
- `AD-004` (`examined`): Contract IR emits and strictly reads this output boundary; codegen consumer adoption remains IR-744.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-047 still says the existing test asserts only the earlier IR wire and cannot discharge the case until IR-743 updates it, although this candidate rewrites that test to assert the six-field FR-330 wire. The test-case status is stale and conflicts with the new matrix claim of focused TC-047 coverage. | spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md:53 |

## Coverage

Review limited to the candidate diff and named acceptance units. Full gate is owned by the author; reviewer ran no Cargo suites.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 42c030299b5e735a7389339be030f1b2f357f545 |

## Disposition verdict

**PASS (round 1, scoped)** — Every original finding was fixed by the listed commits. No new finding arose from the fix diff. Full gates are owned by the author.
