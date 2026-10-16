---
id: SR-11671
title: IR-225 integrity review of catalogue-driven member-result admission
type: SpecReview
analysis: integrity
scope: agent-ix/quire-contract-ir@39627960464edcc4138c29d20c18dbbcbefc603a (PR 408, diff vs origin/main c2b2a8b, merge-base 055ad1f); spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-225. Integrity check of the new FR-038 bullet, AC-247, the TC-048 step and the tests.md row against the unchanged text that covers the same check. Review-only.

## Verdict

**CONDITIONAL**: one low finding. AC-247 has one TC, a unique id, and the same PLANNED/UNRUN status in all three files. The PR branch merges cleanly with main, where #407 added AC-246. AC-222 and AC-223 are unchanged. The contradiction with the deferred-family paragraph is recorded once, as SR-11670 FND-001, and is not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Two adjacent bullets now name the same wire member two ways. The unchanged IEEE bullet says the `result_type` names "the same type node as `operation.member`'s `type_argument`". The new bullet, FR-038:3152 and the QSpec FR-322 fixtures say `operation.member.declaration`. The IEEE bullet's to_float32/to_float64 result rule is now a strict subset of the new bullet but is not marked as such. Failure: a reader takes "`type_argument`" as the member key, which is exactly the non-canonical spelling eleven IR fixtures use (see SR-11670 FND-002). They then implement or test against `member.type_argument`, and the two bullets disagree about which key is authoritative. Reword the IEEE bullet to `operation.member.declaration` and say its result clause is the IEEE instance of "Catalogued member results". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3512-3525,3526-3536,3152 |

## Coverage

Checked that the bullet, AC-247, TC-048:349-381 and tests.md:65 agree on the identities, refusal code and cause, pointer, locus and status. Checked the TC entry lists against the locked catalogue: 21 type_argument entries and 5 others. Checked that AC-213, AC-222 and AC-223 are unchanged and that their implemented tests remain valid packages under the new rule once fixture spelling is resolved.

## Examined units

```yaml
scope:
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "For `quire.op.ieee.to_float32` and `quire.op.ieee.to_float64`, the catalog's `member` result form requires the `result_type` to name the same type node as `operation.member`'s `type_argument`; the catalogued `member_family` constraint requires that type to have the `float32` and `float64` family respectively."
- id: FR-038-AC-247
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "PLANNED/UNRUN (IR-225). Through `CheckedPackageV2::read`, every catalogued operation with result form `member` and member kind `type_argument` admits only when the application's `result_type` equals that member's declared type node ..."
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: "The catalogue's `result: member` entries with `member: type_argument` are `quire.op.quantity.convert`, `quire.op.ieee.to_float32`, ... and `quire.op.collection.convert`. The five other `result: member` entries are `quire.op.record.project` (`field`), `quire.op.tuple.project` (`position`), `quire.op.collection.index` (`element`), `quire.op.model.navigate` (`relationship_end`) and `quire.op.model.dispatch_call` (`operation`)."
bindings: []
```

## Dispositions

Round 1; reviewed 9e608222751efbe09fe78c8260e3c5bcec143292 (diff 3962796..9e60822).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |

## Disposition round 1 verdict

**PASS**. The IEEE bullet now reads "the `result_type` to name the same type node as `operation.member.declaration`; this is the IEEE instance of **Catalogued member results** below". Both bullets use one spelling, and the new bullet closes the shape with "There is no alternate spelling". The remaining uses of "`type_argument`" in AC-213 and AC-223 refer to the member kind, not a wire key, and are unchanged. On the merged tree, AC-222, AC-223 and AC-246 are byte-identical, and the five tests.md rows, the TC-048 steps and AC-247 to AC-251 agree on status, code, cause, pointer and locus. No new findings.

## Disposition round 2 verdict

Round 2 reviewed 5918af852bbdbe600181906bd4b71be371902762 (9e60822 plus 05014ba plus a merge of origin/main c2b2a8b). **PASS**: the round-1 `fixed` outcome still holds on the merged tree, no row was added and there are no new findings.
