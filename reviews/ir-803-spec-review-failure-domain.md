---
id: SR-11663
title: IR-803 failure-domain review of the typed composite operator on the composite operand accessor
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-contract-ir@9c2a2fe0550f18dad6e8313f9588c0cea52af6ef (PR 407, diff vs origin/main 055ad1f); spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
review_set: subset
---

## Summary

Ticket: IR-803. Failure modes of the operator projection, checked against `composite_application_operands` (crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:351-492): identity confusion between eq and ne, the work meter, refusal order, and operand identity across the two applications.

## Verdict

**PASS**. No failure-domain defects.

- Identity confusion. The operator is chosen from the same `/operation/identity` string the eligibility gate has already matched (382-397). So no third value and no eq/ne swap can reach success without failing AC-246's two-sided assertions.
- Work. The identity read happens after the one-unit application lookup and costs nothing. The existing exact-work assertions for both eq and ne (tests/it/checked_package_v2_composite_operands.rs:574-576) would catch any added charge.
- Refusal order. The operator is chosen after the operation check and before any operand charge, so the first-defect order (FR-038:3336) does not change.
- Operand identity. The TC builds both applications over the same ordered children, so children, semantic types, domains and work are equal and each result keeps its own application id. An existing test (518) already shows that eq and ne give equal operands and work on one package.

The unscoped `IneligibleOperator` sentence is a contradiction in the spec text, recorded in SR-11660 FND-001. It is not a runtime failure mode.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Examined the new FR-038 prose (3123-3132), AC-246 and the TC-048 step (775-784), against the accessor and its tests.

## Examined units

```yaml
scope:
- id: FR-038-AC-246
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "For a reader-admitted `quire.op.structural.eq` application, `composite_application_operands` returns `CheckedCompositeOperator::Eq` together with its operands; for an otherwise equivalent reader-admitted `quire.op.structural.ne` application it returns `CheckedCompositeOperator::Ne` together with its operands."
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "The accessor already resolves the application node under the graph lookup's one-unit work charge before reading its operation identity. Selecting the typed operator from that identity shall add no work charge and shall leave `consumed_work`, the existing refusal variants, and operand order unchanged."
bindings: []
```

## Disposition round 1 verdict

Reviewed 81ddbb6046a6b5f063e80624ab874366b08f053c (diff 9c2a2fe..81ddbb6). There were no findings, so there is nothing to disposition. Re-checked against the fix diff:
- The scoped refusal sentence keeps the node, application, occurrence, operation order.
- The work-charge sentence is unchanged.
- AC-246's allowed differences (operator, application identity and occurrence) match what the accessor returns.

**PASS**, no new findings.
