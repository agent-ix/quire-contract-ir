---
id: SR-8692
title: "QSL-737 quire-contract-ir PR 376 spec review (base): FR-347 and FR-038 hunks"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@d0b83ff93fdea368f23de95da2b781fd9f7b9e6c; spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-448-checked-package-v2-operation-argument-conformance.md; spec/checked_package/matrix/tests.md; spec/spec.md; spec/tests.md"
review_set: base
---

## Summary

Ticket: QSL-737. Base checklist review of the spec hunks in quire-contract-ir
PR 376. The analysis lenses ran on the merged FR-347 in spec PR 375, so this
review covers the hunks only. The hunks are:

- FR-347: a new section, "Record-typed binders and payloads".
- FR-038 line 1719: a union payload type node may be a
  `model`/`record_value_type` declaration node.
- FR-038 line 2157: a `value`/`parameter` node's `semantic_type` may be a
  `model`/`record_value_type` declaration node.
- TC-448 status and the matrix rows change from PLANNED to implemented.

Checked:

- Consistency with the upstream rule. Merged QSpec FR-322 "Record
  conformance" (quire-specification main 65609b9) lets a parameter type be a
  record type node P ("Calls") and lists the union member payload position as
  a slot ("Slots"). That requires both FR-038 sentences. Before them, FR-038
  restricted the binder type to `scalar_type`, `composite_type` and
  `bounded_domain`, and the union payload rule required "a type". That made
  FR-347-AC-10, AC-11 and AC-13 unreachable. The change closes a gap in this
  repo's FR-038 and is not a new design decision.
- No conflict with an existing criterion. No FR-038 AC refuses a
  record-typed binder or a record payload. FR-038-AC-99's "a payload
  reference to a node that is not a type refuses" still holds for non-type
  nodes.
- Placement. The widening is the minimum FR-347 needs and is stated in the
  FR (FR-038) whose grammar it amends, with a back-link from FR-347. It
  belongs in this PR. A separate spec PR would only repeat the same two
  sentences.
- Statuses. TC-448 and both matrix rows now say implemented. That matches
  the 14 tagged tests and the coverage log (FR-347 14/14). The spec.md
  sentence changes from "plans its tests" to "tests it".
- IDs and links: no new IDs. Both FR-038 links resolve to the FR-347 file.
- Hash / digest / pin antipattern: none in the hunks. The selected-document
  digest FR-347 refers to is the existing canonical content identity of the
  lock selection, not a tracking record.

## Verdict

PASS WITH FINDINGS (one low). The FR-038 widening is correct, minimal and
consistent with merged QSpec FR-322. No separate spec review or spec PR is
needed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new FR-347 paragraph "Record-typed binders and payloads" records change history ("before this requirement it admitted neither") instead of stating the rule. It also gives no criterion for the limit of the widening: only `record_value_type`, so not an object type, a relationship or an event type. The Scope section says this only for conformance. | spec/checked_package/functional/FR-347-admit-conforming-records-as-operation-call-arguments.md:225-230 |
