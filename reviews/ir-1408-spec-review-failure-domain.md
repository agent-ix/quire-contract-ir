---
id: SR-2026110705000819
title: "failure-domain review of inherited dispatch admission"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; IR-1408; spec/checked_package/functional/FR-354-project-admitted-runtime-context.md; spec/checked_package/matrix/TC-472-inherited-operation-dispatch.md"
review_set: subset
---

## Summary

Reviewed identity joins, untrusted redefinition targets, ancestry, cycles, bounded work and preservation of complete producer candidates. No additional failure-domain omission found in the proposed contract.

Reviewed frozen spec head e44728e16a60b7d596d9b478dfd85be865a034b6 in PR 950. Direct review; no enforced workflow or human acknowledgement claimed. No applicable AssuranceProfile exists under the repository spec tree.

## Examined Scope

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
    }
  ]
}
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

- Read actual retained DomainModel::resolve, conforms_to, OperationDecl::redefines and existing dispatch validator. No alternate owner/API is needed.
- Changed-document `quire validate --scope .` exits zero, two grammar-clean documents and 12 extracted criteria.
- `quire properties --scope . spec/checked_package/functional/FR-354-project-admitted-runtime-context.md --json` exposes exactly AC-1 through AC-12; the new detached rows are absent.
- Full spec validation exits zero with eight unchanged grammar warnings.
- No Cargo builds, aggregate gate, source changes or review worktree created.

## Verdict

The contract distinguishes static context from selected family, rejects name-only authority, requires typed ancestral redefinition targets and rejects cycles. Work exhaustion preserves the existing meter and owner locus; strict typed denial returns no partial view.

## Spec Disposition Round 1

Inspected fix head 2553c78625c5c86a3229c9c6e3c5f8875ea4635f. The only source diff removes the two blank Acceptance Criteria table separators. Real `quire properties` now extracts AC-1 through AC-16 exactly once and in order; both changed documents validate with zero grammar findings. No requirement semantics changed and no new findings were introduced. SPEC PASS; implementation and its review/gate remain pending.

## Mixed Implementation Review

Reviewed frozen head 2382ea0fcdb05e86ed8143275f87226940e593b7 across the full PR diff. Rust review is folded into code-review; the gap method has its separate artifact. No new findings were found. The original integrity finding and its fixed disposition remain unchanged above.

- The selected family join binds both structural contexts to one retained model, requires static member == table context, resolves unique effective operation identity and validates each redefinition target as an operation of a strict ancestor. The bounded iterative walk refuses absent/wrong-kind/unrelated/same-owner/cyclic links and propagates Budget exhaustion without a reset.
- Every candidate is checked against the family declaring owner, not the narrower static receiver. Existing duplicate-key, exact witness, body, precondition, application and occurrence checks remain. No public API, wire form, lockfile, CI gate or dependency changed.
- All four new criteria are parsed and have computed live test binders; the seven trace attributes have exactly TC-first plus owning AC. Real tests inspect retained values and exact typed code/cause/locus; no mock substitutes production admission.
- The committed D10 package bytes compare exactly with the genuine QSL-1599 emitted package. The fixture is the ticket-requested producer input, not a vendored schema or alternate checker. Existing local fixture helpers are reused for independently constructed adverse cases.
- Recorded executions: corrected old-reader D10 regression RED at the required operation locus, same input GREEN after fix; seven TC-472 tests and three existing dispatch tests pass. Final strict-ancestry round reruns seven tests and production lib/root IT Clippy successfully. Rustfmt, diff check and changed docs pass.
- Workspace all-target Clippy reports three duplicate_mod findings on unchanged monitor-plan test module edges. The exact failure reproduces on unchanged main c838196c in the same worktree; it is a baseline failure, not an introduced defect. No full-gate PASS is claimed. Parent owns the single final combined gate.

### Mixed Scope

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

### Mixed Verdict

PASS in PR scope at the reviewed head; no open review findings. Final aggregate remains pending.
