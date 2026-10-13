---
id: SR-8684
title: QSL-737 failure-domain review of operation-call argument record conformance
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-contract-ir@70f0c54b97ab22981edf8bb0447e0fc59156a162; spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md,
  spec/checked_package/matrix/TC-448-checked-package-v2-operation-argument-conformance.md
review_set: subset
---

## Summary

Ticket: QSL-737. Edge cases of the FR-347 conformance walk were checked against the existing walk it reuses (`DomainModel::ancestors`, model_members.rs:826-847, at origin/main 6e45fd57): multi-parent and diamond ancestry, early stop, cycles, the work meter row, a foreign document, and source-owned lookalikes.

## Verdict

**CONDITIONAL**: the work-accounting rule does not pin a number the reused walk would make different.

What holds. Cycles terminate, because the reused walk keeps a `reached` set. A foreign document and a source-owned lookalike fall through to the exact comparison and refuse at the argument. An unrelated record or an ancestor-typed argument walks only its own (shorter) chain. The order of exact match, then walk, then refusal is total. The "first offending argument in position order" rule is deterministic.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | "One `work` unit per `supertypes` edge followed" does not say whether the walk stops on reaching the parameter's record. FR-347 also says the walk is "the ancestors the reader already walks for FR-038-AC-29". That walk (`ancestors`, reached through `conforms_to`) gathers every proper ancestor and charges each edge it pops, a repeated edge in a diamond included. So `settle(t: TaxedMoney)` with a `GiftTaxedMoney` argument costs 2 under the reused walk and 1 under an early-stop walk, and a diamond costs more than its distinct edges. AC-8's one vector (grandchild at the root `Money`) costs 2 either way, so it cannot tell the two apart. State which walk applies (the full `ancestors` walk, if it is reused), and add a vector that differs between them: `GiftTaxedMoney` at `settle` with its exact charge, and one diamond. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:109-111,131 |

## Coverage

Examined the FR-347 behavior sections ("What the reader consults", "Exact kinds stay exact", "Refusals", "Work accounting", "Scope") and AC-3, AC-5, AC-6 and AC-8 for failure modes. The meter row is the declaring owner's selection (`Budget::new(meter, owner.selection, ...)`). The argument's record is in that same document, so FR-038-AC-30's `/lock/model_selections/<i>` pointer is fixed. No finding there. Review-only, no edits or builds.

## Examined units

```yaml
scope:
- id: FR-347-AC-8
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: Resolving the conformance of a GiftTaxedMoney argument at a Money parameter charges one work unit per supertypes edge followed (two), an exact Money argument charges none, and a work limit one below the measured total returns incomplete for work; the exact total admits.
- id: FR-347-AC-3
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A call to settle(t TaxedMoney) with an argument typed at Money (the ancestor) refuses ill_typed/operator-ineligible at that argument, and so does a call to grandsettle(g GiftTaxedMoney) with an argument typed at TaxedMoney (an intermediate ancestor) or at Money. settle with a GiftTaxedMoney argument admits.
- id: FR-347-AC-5
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: An argument typed at a composite_type/record node with a source owner, with the same name and fields as TaxedMoney, refuses ill_typed/operator-ineligible at charge(m Money); ancestry is read from the selected document only. A TaxedMoney record of a second selected document does not conform to the first document's Money and refuses the same way.
- id: FR-347-AC-6
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: examined
  excerpt: A call with two record-typed parameters, a conforming first and a non-conforming second argument, refuses at argument 2 (the second parameter); with both non-conforming it refuses at argument 1. A call with the wrong argument count refuses at arguments before any argument is judged, as before.
- id: FR-347
  path: spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md
  role: context_only
  excerpt: A walk charges one work unit per supertypes edge followed, to the same meter as the member resolution of FR-038-AC-30. An exact match charges none. A limit one below the work a read used returns incomplete for work.
bindings: []
```

## Dispositions

Round 1; reviewed 66a261051c487bebfe2637e155d815f4d308f68d.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 66a261051c487bebfe2637e155d815f4d308f68d |

## Disposition round 1 verdict

**PASS**: the work rule now names the whole reused walk. Traced against `DomainModel::ancestors` (model_members.rs:826-847): `GiftTaxedMoney` costs 2 at both `Money` and `TaxedMoney`. The diamond pops C (1, extend A), A (2), B (3, extend A), A (4, already reached), so it costs 4, which matches "each ancestor's edges followed once". An exact match short-circuits in `conforms_to` and costs 0.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The function-call rule reads a function node's body `parameters` binding, but the reader validates nothing in a function node's body today: there is no structural shape rule for `function` nodes, and no code reads a named binding of one. FR-347 does not say what happens when the binding is malformed: `parameters` is not the first binding, or its value is not an `aggregate`, or an entry is not a `reference`, or names a node absent from the graph, or names a node that is not a `value`/`parameter` node (whose `semantic_type` is then not a parameter type). It also does not say whether "first binding" or "a binding named `parameters`" decides. Two implementers would refuse, skip or admit these differently. Each needs a stated outcome (most likely `ill_typed`/`operator-ineligible` at the call's `arguments` or at the callee) and an AC-13 vector. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:181-195,227 |
| FND-003 | low | The work-accounting rule charges the walk "to the same meter as the member resolution of FR-038-AC-30", whose `incomplete` pointer is the declaring owner's `/lock/model_selections/<i>` row. A case arm, a union payload and a `function.call` argument have no declaring owner, so the row the walk charges and reports for those positions is unstated. Name it, for example the expected record's selection row. AC-8 covers only the operation-call position. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:162-172 |

## Dispositions round 2

Round 2; reviewed 8a8cf551e52968fff3631ca733181ad1a89a6c49. No open findings from round 1: FND-001 was fixed in round 1.

## Disposition round 2 verdict

**CONDITIONAL**: the walk-cost text is unchanged and still correct. New FND-002 (medium) and FND-003 (low).
