---
id: SR-14701
title: "IR-937 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-937; Makefile; crates/quire-contract-model/src/checked_package/v2/{derived_keys.rs,model_members.rs,model_members/tests.rs,operations.rs,owner.rs}; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/{TC-229-declared-population-reference-reader.md,tests.md}; tests/it/{checked_package_v2_abstraction_relation.rs,checked_package_v2_adr002_members.rs,checked_package_v2_catalogue_differential.rs,checked_package_v2_operand_families.rs,checked_package_v2_owners.rs,support/checked_package.rs}"
review_set: subset
---

## Summary

The computed matrix reports the new population criteria tagged, but the only new key-change tests vary owner.node. The required independent owner.identity mutation has no behavioral assertion.

## Verdict

**CONDITIONAL** — The computed matrix reports the new population criteria tagged, but the only new key-change tests vary owner.node. The required independent owner.identity mutation has no behavioral assertion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-294 requires changing either ModelOwner string to change the key, but TC-229 tests change only owner.node; an implementation ignoring owner.identity could pass. | tests/it/checked_package_v2_owners.rs:183; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4962 |

## Coverage

Examined FR-038-AC-294 through FR-038-AC-298, TC-229, QSpec FR-322-AC-82/83, and the changed code/test/Makefile diff. The computed Quire matrix was consulted for the changed criteria.

Plan completion: not assessed. Optional whole-repository semantic review: not requested. This review checked the changed acceptance-criterion bindings.

## New findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The new owner.identity mutation asserts the key change required by AC-303 but its trace tag cites only AC-304, so the computed AC-303 binder list omits the only independent identity-mutation test. | tests/it/checked_package_v2_owners.rs:214; spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4962 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | adds tc_229_population_owner_identity_is_joined_independently_of_node, which holds owner.node equal, asserts node_id differs when owner.identity changes, and checks missing-selection at the population node. |
| FND-002 | still-open | The new test is tagged only FR-038-AC-304; its key-change assertion also backs FR-038-AC-303 and needs that trace tag. |
| FND-002 | fixed | `tc_229_population_owner_identity_is_joined_independently_of_node` now carries both FR-038-AC-303 and FR-038-AC-304 trace tags; the computed Quire matrix binds it to both criteria at the reviewed PR head. |

## Final PR head inspection

PR agent-ix/quire-contract-ir#494 was rebased. The range-diff shows all 17 rebased commits patch-identical to the reviewed series. The final population criteria are FR-038-AC-303 through FR-038-AC-307; the computed matrix binds the owner-identity mutation to AC-303 and AC-304. Original finding wording and chronological dispositions above remain the review record.
