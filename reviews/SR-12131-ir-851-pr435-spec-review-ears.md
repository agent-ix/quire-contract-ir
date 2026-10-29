---
id: SR-12131
title: "IR-851 EARS review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir (PR #435); spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Typed relationship end of a reaches claim, AC-262..267)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

# IR-851 EARS review

## Summary

Ticket: IR-851. Each new normative sentence in the FR-038 section and each of AC-262..267 was checked for a named subject, one obligation, a concrete response and a condition where one applies. `quire validate --scope. 'spec/**/*.md'` exits 0 (warnings only).

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| API statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | relationship_end... shall return the destination end the reader resolved for that claim, as typed Rust values. |
| Retention sentence | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | which the reader therefore retains for each selected relationship declaration... That retention is not part of package equality, charges no work beyond what admission already charges, and does not change the reader's refusal order. |
| UnknownNode statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | An unknown id shall refuse UnknownNode. |
| NotRelationshipEndClaim statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | A node whose body root is not a quire.op.model.reaches application shall refuse NotRelationshipEndClaim, whatever its tag |
| UnresolvedEnd statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md |... shall refuse UnresolvedEnd. |
| Refusal-properties sentences | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Each refusal shall return only the typed error and no partial end. The accessor shall not panic on any input, shall charge no work, shall not change package admission or the V2 reader's refusal order, and shall not mutate the package. |
| FR-038-AC-262 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | When relationship_end is called with the id of an admitted quire.op.model.reaches claim... it returns Ok |
| FR-038-AC-263 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | For admitted reaches claims over a destination end of {0,3} unique... returns exactly that lower, upper and unique |
| FR-038-AC-264 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | relationship_end over an id absent from the graph refuses UnknownNode |
| FR-038-AC-265 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | When a crate-internal post-admission mutation... refuses UnresolvedEnd for each |
| FR-038-AC-266 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | When relationship_end is called twice... the results are equal |
| FR-038-AC-267 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | An external Rust API fixture calls relationship_end... the model root exports none of the selected_population items |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The retention sentence adds a new obligation on the reader: retain each selected relationship record, outside equality, with no extra charge and the refusal order unchanged. It is written in indicative mood with no `shall`, inside a paragraph about the accessor, so a reader cannot tell it is normative for the reader. This is the same pattern IR-832's SR-12001 FND-001 fixed. Fix: state it as "The reader shall retain..." with its three properties as `shall` clauses. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3226-3233 |
| FND-002 | low | AC-267 joins four independent obligations: the public shape, `model_object_fields` unchanged, the field-read node refusing `NotRelationshipEndClaim`, and no `selected_population` re-export. One failing clause hides which property broke. The field-read refusal belongs with AC-264's NotRelationshipEndClaim set. Fix: move the field-read case to AC-264 and keep AC-267 to shape plus FR-053 non-interference. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4614 |

## Verdict

PASS WITH FINDINGS. The API, UnknownNode, NotRelationshipEndClaim and UnresolvedEnd statements and the refusal-properties sentences are ubiquitous or event-driven obligations with a named subject and an observable response. AC-262, AC-264, AC-265 and AC-266 are single, falsifiable verification statements, and AC-262's unequal ends make an end swap fail. AC-263's closing clauses about unbounded/ordered ends and subtype operands are context, not obligations, which is acceptable.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 : the Retention paragraph is now normative: "The reader shall retain... The reader shall keep this record outside package equality, shall charge no work beyond what admission already charges, and shall not change its refusal order." |
| FND-002 | fixed | Round 1 : the field-read case moved to AC-264. AC-267 is the public shape only, and the new AC-268 holds model_object_fields non-interference and the re-export rule. Nothing is renumbered (AC-268 is new). |
