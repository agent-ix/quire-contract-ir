---
id: SR-11662
title: IR-803 EARS conformance review of the typed composite operator requirement text
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-contract-ir@9c2a2fe0550f18dad6e8313f9588c0cea52af6ef (PR 407, diff vs origin/main 055ad1f); spec/checked_package/functional/FR-038-consume-checked-package-v2.md
review_set: subset
---

## Summary

Ticket: IR-803. EARS requirement grammar of the new "shall" sentences at FR-038:3123-3132 and the AC-246 statement. `quire validate` reports 876/876 docs grammar-clean.

## Verdict

**PASS with one low finding.** The new sentences are ubiquitous "shall" statements with one subject each, and the Eq/Ne mapping is a biconditional ("exactly when"). The refusal-scope wording is recorded in SR-11660 FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-246 says changing the identity "changes the returned operator and no other operator-dependent projection field". "Operator-dependent" is circular. Read literally, it allows any field to change as long as that field is called "not operator-dependent". The text after the colon gives the real list. State it directly: "changes only the returned operator; `consumed_work` and the operands' order, ordinals, child identities and domains are equal, and each result keeps its own application identity and occurrence." | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4353 |

## Coverage

Examined the six new prose sentences at 3123-3132 and the AC-246 statement.

## Examined units

```yaml
scope:
- id: FR-038-AC-246
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Changing the operation identity between those two admitted cases changes the returned operator and no other operator-dependent projection field: each result retains its own authentic application identity and occurrence, while operand order, ordinals, child identities, domains and `consumed_work` remain equal."
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "The result's `operator` shall be `CheckedCompositeOperator::Eq` exactly when the selected application's admitted operation identity is `quire.op.structural.eq`, and `CheckedCompositeOperator::Ne` exactly when it is `quire.op.structural.ne`."
bindings: []
```

## Dispositions

Round 1; reviewed 81ddbb6046a6b5f063e80624ab874366b08f053c (diff 9c2a2fe..81ddbb6).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |

## Disposition round 1 verdict

**PASS**. The circular phrase is gone. AC-246 now says "only the returned operator and the selected application's identity and occurrence may differ", and lists the fields that are equal. The new scoped `IneligibleOperator` sentence is a single-condition "shall" statement. No new findings.
