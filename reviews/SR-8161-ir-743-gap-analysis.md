---
id: SR-8161
title: gap-analysis review of IR-743 ContractPackage
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir@3d97f7250257e7d5f83ebe8855428ef9321b5e44; base b2008d3f50127bf033b5bae8fb302a482e536dbc;
  spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md,
  spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, tests/it/complete_v1_contract_package.rs
review_set: subset
---

## Summary

Ticket: IR-743. Independent gap-analysis pass on the exact frozen candidate.

## Verdict

**CONDITIONAL** — 1 finding(s).

## Examined units

- `FR-035-AC-5` (`examined`): One call emits exactly one FR-330 `quire.contract-package/v1` value with the six required properties and one key-ordered `semantic_graph` node collection; repeated equivalent construction orders have identical canonical semantic identity. The digest is computed from exactly `contract_version`, `package`, `semantic_graph` and `feature_accounting`: changing an included member changes it, changing only valid `source_map` locations preserves it, and invalid or incomplete `source_map` correspondence refuses admission.
- `TC-047` (`examined`): Planned / unrun for the FR-330 package emitter. The existing `tests/it/complete_v1_contract_package.rs` asserts the earlier IR package shape and cannot discharge this test until IR-743 updates it.
- `TC-047-test` (`examined`): let same = lower(&input, &request()); assert_eq!(bytes(&first), bytes(&same)); let mut reversed = request(); reversed.reverse(); assert_ne!(first.package.package_id(), reordered.package.package_id());

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The tagged identity test repeats the same input/order and separately reverses request order expecting a different identity. It never constructs equivalent graph inputs in different orders and checks equal identity, leaving the construction-order clause of FR-035-AC-5 and TC-047 unverified. | tests/it/complete_v1_contract_package.rs:199 |

## Coverage

Review limited to the candidate diff and named acceptance units. Full gate is owned by the author; reviewer ran no Cargo suites.
Plan completion: not assessed
Semantic review: skipped; no opt-in was given.
Computed matrix: FR-035-AC-1..5 are tagged; the finding concerns assertion strength, not a missing tag.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 42c030299b5e735a7389339be030f1b2f357f545 |

## Disposition verdict

**PASS (round 1, scoped)** — Every original finding was fixed by the listed commits. No new finding arose from the fix diff. Full gates are owned by the author.
