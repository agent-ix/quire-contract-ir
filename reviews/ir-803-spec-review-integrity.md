---
id: SR-11661
title: IR-803 integrity review of the typed composite operator on the composite operand accessor
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir@9c2a2fe0550f18dad6e8313f9588c0cea52af6ef (PR 407, diff vs origin/main 055ad1f); spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-803. Integrity of the new type-table row, the new prose, AC-246, the TC-048 step and the tests.md rows against the unchanged criteria that cover the same accessor. Review-only.

## Verdict

**CONDITIONAL**: one low finding. AC-246 has a unique id, one TC, and the same planned status in all three files. The IR-owned closed `Eq`/`Ne` enum keeps the result free of QSL or CG types. AC-177 to AC-182 are byte-identical to the base. The internal contradiction on `IneligibleOperator` scope is recorded once, in the base review (SR-11660 FND-001), and is not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-182 stays "Implemented ... verified": "an external consumer ... exhaustively matches all named output ... types". The PR adds `CheckedCompositeOperator` to the named types, and this commit has no such type. Nothing in AC-246 or its TC-048 step says to extend the AC-182 fixture (`inspect_public_fields`, tests/it/checked_package_v2_composite_operands.rs:282-292). Failure: the code PR destructures `operator: _`, or ships the enum `#[non_exhaustive]` (a downstream crate cannot then match it exhaustively). The closed-variant promise at FR-038:3110 then goes unchecked while AC-182 still reads Implemented. Either have AC-246's TC step match `Eq`/`Ne` exhaustively in the external fixture, or note on AC-182 that the operator part is covered under AC-246 (IR-803). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3110,4292,4353 |

## Coverage

Checked that the `CheckedCompositeOperands` row change, the new `CheckedCompositeOperator` row, the prose at 3123-3132, AC-246, TC-048:764-784 and tests.md rows 21 and 32 agree on the field name, variant names, status and TC. Checked that `consumed_work`, the refusal set and operand order are unchanged, against AC-181's exact-budget rule and the existing exact-work test.

## Examined units

```yaml
scope:
- id: FR-038-AC-182
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. An external consumer calls the named public accessor and exhaustively matches all named output, descriptor, edge, key and error types using the specified fields without body JSON or a QSL model dependency."
- id: FR-038-AC-246
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "PLANNED/UNRUN (IR-803). For a reader-admitted `quire.op.structural.eq` application, `composite_application_operands` returns `CheckedCompositeOperator::Eq` together with its operands; for an otherwise equivalent reader-admitted `quire.op.structural.ne` application it returns `CheckedCompositeOperator::Ne` together with its operands."
- id: FR-038-AC-181
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Zero and one below measured successful work refuse WorkLimit; that exact budget succeeds."
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: "Mark this step PLANNED/UNRUN until the typed result and tagged assertions land."
bindings: []
```

## Dispositions

Round 1; reviewed 81ddbb6046a6b5f063e80624ab874366b08f053c (diff 9c2a2fe..81ddbb6).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |

## Disposition round 1 verdict

**PASS**. AC-246 and the TC-048 step now both require `inspect_public_fields` (the AC-182 fixture) to match `Eq` and `Ne` exhaustively, with no wildcard arm and no `#[non_exhaustive]`. A third variant, a wildcard, or a non-exhaustive enum would each break that external fixture's compile. AC-182 itself is byte-identical. No new findings.
