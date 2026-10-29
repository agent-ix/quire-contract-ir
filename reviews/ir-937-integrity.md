---
id: SR-14703
title: "IR-937 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-937; Makefile; crates/quire-contract-model/src/checked_package/v2/{derived_keys.rs,model_members.rs,model_members/tests.rs,operations.rs,owner.rs}; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/{TC-229-declared-population-reference-reader.md,tests.md}; tests/it/{checked_package_v2_abstraction_relation.rs,checked_package_v2_adr002_members.rs,checked_package_v2_catalogue_differential.rs,checked_package_v2_operand_families.rs,checked_package_v2_owners.rs,support/checked_package.rs}"
review_set: subset
---

## Summary

The acceptance-criterion table is inconsistent: AC-294 and AC-295 each have two conflicting statements and TC-229 binds to the already occupied IDs.

## Verdict

**FAIL** — The acceptance-criterion table is inconsistent: AC-294 and AC-295 each have two conflicting statements and TC-229 binds to the already occupied IDs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Duplicate AC-294/295 statements make each criterion non-atomic and give one ID two incompatible verification obligations. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4914; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4962 |

## Coverage

Examined FR-038-AC-294 through FR-038-AC-298, TC-229, QSpec FR-322-AC-82/83, and the changed code/test/Makefile diff. The computed Quire matrix was consulted for the changed criteria.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | moves the five population criteria to unique FR-038-AC-303 through AC-307 IDs and updates TC-229, the handwritten matrix, and new test tags. FR-038 now has no duplicate acceptance-criterion table IDs. |

## Final PR head inspection

PR agent-ix/quire-contract-ir#494 was rebased. The range-diff shows all 17 rebased commits patch-identical to the reviewed series. The final population criteria are FR-038-AC-303 through FR-038-AC-307; the computed matrix binds the owner-identity mutation to AC-303 and AC-304. Original finding wording and chronological dispositions above remain the review record.
