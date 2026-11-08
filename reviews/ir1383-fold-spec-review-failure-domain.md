---
id: SR-202611070951309502
title: "IR-1383 fold binders \u2014 spec-review/failure-domain"
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/operations.rs;
  tests/it/checked_package_v2_operand_families.rs; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
review_set: subset
---

## Summary

Ticket: IR-1383. Reviewed frozen PR quire-contract-ir#961 at 42002f2b587ce1a066821d301a666452a5d1476e against origin/main. No applicable AssuranceProfile exists under spec/.

## Verdict

PASS for the scoped frozen diff — no defects found. The code lead retains the aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The projection has fixed depth: one accumulator binding, one singleton aggregate, one element binding, then a type-bearing final step. It introduces no recursion and no cycle-following algorithm. Existing reference validation remains before operation operand checks. Empty, multiple, non-binding and Boolean-step cases have explicit application-local refusals; a further aggregate keeps its earlier malformed-wire location and no cause. Direct binders and non-fold operations remain bounded by the existing path. No new callback, execution side effect or producer-authority trust is introduced.

## Examined scope

```yaml
scope:
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'For `quire.op.collection.fold` and `quire.op.collection.reduce`, the reader

    SHALL resolve `bound_is_member` through [QSL FR-093](https://github.com/agent-ix/quire-spec-language/blob/main/spec/functional/FR-093-lower-checked-value-expressions-to-fr-322-terms.md)''s

    two-binder encoding in [QSpec FR-322](https://github.com/agent-ix/quire-specification/blob/main/spec/objects/interfaces/FR-322-checked-package-artifact.md) terms: an accumulator `binding` whose value is an `aggregate`

    containing exactly one element `binding`, whose value names the final step.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The final step's resolved type SHALL equal the `type_argument` declaration.
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'An empty or multiple-member aggregate, a non-binding member, or a final step

    of a different type SHALL refuse `ill_typed`/`operator-ineligible` at the

    application node.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'A further nested aggregate step retains the earlier

    `malformed_wire` refusal at that step''s value, without a cause.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Direct single-binder

    bounds retain their existing meaning.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'This rule does not assign a type to

    arbitrary aggregate operands or change other operations'' binder resolution.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'FR-038-AC-247 and FR-038-AC-251 include these positive and negative controls

    and strict admission of both genuine Integer-only QSL fold bytes and the

    complete original monitor-base document with its selected model evidence.

    '
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented (IR-225). Through `CheckedPackageV2::read`, a catalogued `result: member`/`member: type_argument` application whose member has a valid `declaration` admits only when `result_type` equals that declaration, regardless of whether its family resolves or the entry has a `member_family` constraint. A mismatch refuses `ill_typed`/`operator-ineligible` at `/semantic_graph/nodes/{i}/body/result_type`, with the application node as locus. `collection.fold` and `collection.reduce` admit equal results without a family constraint and refuse unequal results. `model.lookup` and `model.all_instance'
- id: FR-038-AC-251
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented (IR-225; amended from an expected-set shape guard because the reader selects from the current catalogue entry). Member-result checking is selected by the catalogue entry shape alone (`result: member` and `member: type_argument`). Every other catalogued operation, including `model.lookup`, `model.all_instances` and the other member kinds, keeps its pre-IR-225 admission outcome; `collection.fold` and `collection.reduce`, which have no `member_family`, are checked for result-type/declaration equality only. A catalogue-wide differential test reads every entry from the same published ca'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'For fold and reduce, also read the accumulator binding whose aggregate holds

    one element binding and resolve the final step''s type.'
```
