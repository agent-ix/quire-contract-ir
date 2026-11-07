---
id: SR-2026110705000820
title: "Gap analysis of inherited operation dispatch"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1408; FR-354-AC-13 through FR-354-AC-16 and changed production/tests"
review_set: subset
---

## Summary

PR-scoped matrix, reverse ownership, stub and test-binding review at 2382ea0fcdb05e86ed8143275f87226940e593b7. The four new criteria are live and tagged; changed production behavior is owned by FR-354, and real native-reader tests exercise it. This is not a whole-repository assurance claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

The computed Quire matrix reports AC-13 (three binders), AC-14 (one), AC-15 (one), AC-16 (two) as tagged. All seven bindings were read against actual assertions. Exact admitted D10 rows, both declaration identities, context, candidate order and witness values are checked. Adversarial controls check exact typed refusal code/cause/pointer and no admitted result. Multi-hop finite work is tested at the measured exact admission boundary and one below with consumed work and original model-root locus. No ignored/stubbed test is used. No reverse gap exists for the two changed private production methods; FR-354 owns their semantics. No plan or extra audit scope was introduced.

```yaml
{
  "scope": [
    {
      "id": "FR-354-AC-13",
      "path": "spec/checked_package/functional/FR-354-project-admitted-runtime-context.md",
      "role": "examined",
      "excerpt": "A genuine inherited call whose operation member names static receiver `Sub` while its captured table names the selected `ConfigVersion` operation-family witness admits through the strict reader. Its public dispatch view preserves both distinct identities, receiver context `Sub`, the complete producer candidate order and each candidate's exact selected-operation witness. A genuine `ConfigVersion` family candidate remains admitted although it is not a subtype of the narrower static receiver `Sub`. A direct call with equal declaration identities continues to admit."
    },
    {
      "id": "FR-354-AC-14",
      "path": "spec/checked_package/functional/FR-354-project-admitted-runtime-context.md",
      "role": "examined",
      "excerpt": "Changing only that captured table's declaring witness to an unrelated selected declaration with a same-named operation refuses `InvalidPackage` with cause `InvalidValue` at `/checked_term_data/dispatch_tables/0/operation`; it returns no admitted package or dispatch view. Names alone cannot satisfy the family join."
    },
    {
      "id": "FR-354-AC-15",
      "path": "spec/checked_package/functional/FR-354-project-admitted-runtime-context.md",
      "role": "examined",
      "excerpt": "A table witness that differs from the call member's static declaration without an admitted inherited operation-family relation refuses `InvalidPackage` with cause `InvalidValue` at `/checked_term_data/dispatch_tables/0/operation`. In particular, merely making its declaration an ancestor with a same-named independent operation cannot replace the declared operation/redefinition-family join."
    },
    {
      "id": "FR-354-AC-16",
      "path": "spec/checked_package/functional/FR-354-project-admitted-runtime-context.md",
      "role": "examined",
      "excerpt": "A retained redefinition link to a field, to an operation outside the redefiner owner's ancestry, or into a cycle cannot authorize a mismatched table witness and refuses `InvalidPackage` with cause `InvalidValue` at the row's `operation` pointer. A valid multi-hop operation-family join succeeds with sufficient reader work; one unit below its measured successful work allowance returns the existing typed reader exhaustion with its selected-model owner locus and no admitted package."
    },
    {
      "id": "FR-354",
      "path": "spec/checked_package/functional/FR-354-project-admitted-runtime-context.md",
      "role": "examined",
      "excerpt": "For a captured dispatch row, Contract IR SHALL distinguish the call member's static receiver declaration from the selected operation family's declaring witness."
    },
    {
      "id": "TC-472",
      "path": "spec/checked_package/matrix/TC-472-inherited-operation-dispatch.md",
      "role": "examined",
      "excerpt": "Verify that strict dispatch admission preserves a call's static receiver identity while validating its distinct selected operation-family witness, and refuses mismatched witnesses outside that family."
    },
    {
      "id": "FR-354-AC-4",
      "path": "spec/checked_package/functional/FR-354-project-admitted-runtime-context.md",
      "role": "context_only",
      "excerpt": "The dispatch projection preserves every admitted candidate in producer order with its effective precondition term reference; a missing or ambiguous candidate/precondition join refuses without choosing a candidate or evaluating a predicate."
    },
    {
      "id": "crates/quire-contract-model/src/checked_package/v2/dispatch_projection.rs",
      "path": "crates/quire-contract-model/src/checked_package/v2/dispatch_projection.rs",
      "role": "examined",
      "excerpt": "Static context and selected family are joined through the same retained selected document; candidate order and exact producer witnesses remain unchanged."
    },
    {
      "id": "crates/quire-contract-model/src/checked_package/v2/model_members.rs",
      "path": "crates/quire-contract-model/src/checked_package/v2/model_members.rs",
      "role": "examined",
      "excerpt": "operation_in_family validates unique effective operations and exact retained anchors through typed strictly ancestral redefinition links under the existing Budget."
    },
    {
      "id": "tests/it/checked_package_v2_model_members/inherited_dispatch.rs",
      "path": "tests/it/checked_package_v2_model_members/inherited_dispatch.rs",
      "role": "examined",
      "excerpt": "Seven real strict-reader tests cover exact D10 bytes, inherited/direct families, unrelated/independent witnesses, invalid edges, candidates and exact work exhaustion."
    }
  ],
  "bindings": [
    {
      "test_id": "genuine_qsl_d10_retains_static_context_and_complete_selected_family",
      "ac_id": "FR-354-AC-13",
      "trace": "correct"
    },
    {
      "test_id": "direct_and_inherited_operations_retain_complete_producer_rows",
      "ac_id": "FR-354-AC-13",
      "trace": "correct"
    },
    {
      "test_id": "static_context_and_each_candidate_stay_bound_to_the_selected_family",
      "ac_id": "FR-354-AC-13",
      "trace": "correct"
    },
    {
      "test_id": "unrelated_same_named_selected_witness_refuses",
      "ac_id": "FR-354-AC-14",
      "trace": "correct"
    },
    {
      "test_id": "ancestry_and_same_spelling_do_not_replace_the_operation_family",
      "ac_id": "FR-354-AC-15",
      "trace": "correct"
    },
    {
      "test_id": "wrong_kind_unrelated_and_cyclic_redefinition_links_refuse",
      "ac_id": "FR-354-AC-16",
      "trace": "correct"
    },
    {
      "test_id": "multi_hop_family_charges_exact_reader_work_and_preserves_owner_locus",
      "ac_id": "FR-354-AC-16",
      "trace": "correct"
    }
  ]
}
```

## Evidence

Actual computed matrix: /tmp/ir1408-review-mixed-matrix.json. Seven TC-472 tests and three existing dispatch tests pass in the recorded focused executions. The exact original QSL D10 package bytes were compared, and the red/green reader receipts were inspected. No independent repeated Cargo or aggregate gate was run by the reviewer, per the user's one-final-gate instruction. No applicable AssuranceProfile exists.

## Verdict

PASS in reviewed PR scope. No gaps found in the new four-criterion behavior; final aggregate remains the lead's pending stage.
