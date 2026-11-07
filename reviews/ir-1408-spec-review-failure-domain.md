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
