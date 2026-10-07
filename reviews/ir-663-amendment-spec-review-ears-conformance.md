---
id: SR-2941
title: "EARS conformance review of the IR-663 private intake retention amendment"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir PR 321 (IR-663 amendment, specification only); spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Ticket: IR-663. Engine check: `quire validate --summary` reports both changed
documents grammar-clean (0 grammar findings) at base and at head. Semantic
judgment over the amended normative statements (retention section, the step 1
declaration paragraph) and AC-167/173/174/175 finds no vague responses and no
public-API promise for the future consumer, but AC-174 and AC-175 each bundle
several independently failing obligations, and several normative sentences
name a data value, the document itself or an external owner as the subject
rather than the reader.

## Examined units

- FR-038 "When FR-154 model intake refuses a located declaration, the private `SelectionRefusal` ... SHALL retain ..." (examined)
- FR-038 "The private intake refusal SHALL retain the selected declaration's authentic identity and valid origin" (examined)
- FR-038 "this amendment SHALL NOT extend `CheckedPackageRefusal` ..." (examined)
- FR-038 "Open item: ... its owner shall allocate a separate consumer contract ..." (examined)
- FR-038-AC-167, FR-038-AC-173, FR-038-AC-174, FR-038-AC-175 (examined)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-174 packs at least eight separately failing obligations (identity, Source origin, Generated origin, nested context, coordinate boundary, missing/malformed None, the new grouped-duplicate origin rule, unchanged first refusal and accounting) into one criterion; one failing sub-check fails the whole AC and the matrix cannot show which part is covered. The grouped-duplicate rule added here is the clearest candidate to stand alone. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3806 |
| FND-002 | low | AC-175 mixes a runtime Test obligation (constructor and conversion preservation with mutation oracles) with a negative Inspection (public conversion unchanged, no new public field); the two have different verification methods and fail independently. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3807 |
| FND-003 | low | Normative subjects are not the system: "the private `SelectionRefusal` ... SHALL retain", "The private intake refusal SHALL retain", "this amendment SHALL NOT extend" and the open item's "its owner shall allocate". EARS needs the reader as the actor ("the reader SHALL retain ... in the private refusal"); the open item should be non-normative so it carries no obligation on an external future owner. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1127,1150-1152,1207-1215 |

## Verdict

Conditional: no high-severity grammar defect; atomicity of AC-174 is the main
item to fix while the amendment is open.

## Dispositions

Round 1, fix commit "spec: close IR663 private intake review findings".

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
| FND-002 | fixed | fixed by commit "spec: close IR663 private intake review findings" |
| FND-003 | fixed | fixed by commit "spec: close IR663 private intake review findings" |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | AC-188 combines two obligations that fail independently: nested field/operation/parameter/relationship-end context retention, and the separate sequential repeated-relationship rule (later offending relationship's metadata). The sequential rule could stand alone or move beside AC-190's contrasting type-group rule. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3848 |
