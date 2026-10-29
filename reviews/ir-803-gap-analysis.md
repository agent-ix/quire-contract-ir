---
id: SR-11665
title: "IR-803 typed composite operator gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #410 diff inspection; FR-038-AC-246 against TC-048 tests in tests/it/checked_package_v2_composite_operands.rs and crates/quire-contract-model/src/checked_package/v2/composite_operands.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-048
    type: references
---

# IR-803 typed composite operator gap analysis

## Summary

Ticket: IR-803. PR: quire-contract-ir#410. This analysis is planless, so "Plan completion: not assessed". It used the computed matrix (`quire matrix --scope . --format json`; quire 0.36.2, engine 0.50.2; quoin 0.28.3) and test-oracle strength, and ran a semantic intent-test-code review scoped to the diff.

- **Tagging.** FR-038-AC-246 is now tagged by `tc_048_structural_operator_is_typed_across_operand_shapes` (`#[trace("FR-038-AC-246")]`, tests/it/checked_package_v2_composite_operands.rs:402-451).
- **Strict coverage.** `quire coverage --strict` falls from 50 to 49 unbacked rows with 0 contradicted. FR-038-AC-246 is the only row that left the unbacked set.
- **Ownership.** Every changed production line maps to FR-038 (the accessor contract and AC-246), and every new public item maps to FR-019. No code lacks an owning requirement.

## Verdict

**PASS**, clean. The tagged test checks each part of AC-246:

- **Operator.** It asserts `Eq` for structural.eq and `Ne` for structural.ne over three operand shapes: parameter/parameter, parameter/closed literal and closed/closed.
- **Identity and occurrence.** Each result keeps its own application identity (`assert_ne!` between the two, and each one equal to its own typed id), and both keep occurrence 0.
- **Everything else equal.** It compares `eq.operands == ne.operands`. `CheckedCompositeOperand` derives `PartialEq` over ordinal, child, semantic type and domain, so this one comparison covers order, child identities, semantic types and domains. It also asserts ordinals `[0, 1]` and `eq.consumed_work == ne.consumed_work`.
- **Exhaustive public-fields match.** `inspect_public_fields` runs on every projection through `project`. It destructures `operator` and matches both variants with no wildcard arm. A scratch `#[non_exhaustive]` mutant and a scratch third-variant mutant each fail to compile there with E0004.
- **No body JSON read.** The test reads the operator only from the returned result and reads no body JSON for the operation identity.

The operator mutants (constant Eq, constant Ne, swapped, operator from another application, operator from the occurrence, ne-only extra charge) all fail the AC-246 test.

Two mutants pass the AC-246 test, because its `consumed_work` oracle is relational (eq equals ne): a symmetric extra charge and a second lookup. Both are killed by the pre-existing exact-work tests at :632-633 and :1087, so no mutant survives the suite.

Status edits are accurate:

- The FR-038-AC-246 row reads "Implemented (IR-803)". Apart from that status word, the criterion text is byte-identical to the merged spec.
- The FR-038 Outputs bullet, the TC-048 AC-246 step and the tests.md FR-038 row change only the status wording.
- No other AC text changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: FR-038-AC-246
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Between those cases, only the returned operator and the selected application's identity and occurrence may differ; operands (including order, child identities, semantic types and domains), ordinals and consumed_work are equal."
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: "Assert operator is CheckedCompositeOperator::Eq for eq and CheckedCompositeOperator::Ne for ne in the returned operand-bearing result."
- id: FR-038-AC-182
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: "Public-fields inspection extended by AC-246."
- id: FR-038-AC-181
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: "Exact consumed_work oracles and refusal variants that pin the unchanged work and refusal behaviour."
```

## Dispositions

Round 1, against fix: this review had no findings, so there are no outcome rows. The fix round changes only composite_operands.rs (operator selection and rustdoc); no test, tag or spec status changed. FR-038-AC-246 remains backed, strict coverage stays at 49 unbacked / 0 contradicted, and the re-run swapped, wildcard-default, extra-charge and second-lookup mutants are all killed.
