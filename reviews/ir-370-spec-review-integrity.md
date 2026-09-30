---
id: SR-626
title: "spec integrity review of PR 229 (FR-040 reaches_field statement)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@65d29c5a3bf803c340987fb21eaadb6bd173013b; spec/contract/FR-040-admit-frame-entries-and-state-clauses.md"
review_set: subset
---
# SR-626: spec integrity review of PR 229

## Summary

Ticket: IR-370. The only spec change is FR-040's `reference_edge` paragraph
(lines 203-212). It was compared against QSpec FR-322 "Reaches over a field"
on quire-specification `origin/main`, and against the code in
`check_reference_edge`.

The four rules match FR-322 in order and wording. The phrase "conforms to
`T`" has the directional meaning discussed in SR-624 FND-001. The missing AC
is recorded in SR-625 FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-040 says an ineligible application refuses "at the operand or member name at fault". The code also refuses `ill_typed`/`operator-ineligible` at `/body/operation/member/declaration` when the declaration names no node, or a node that is not a model declaration node. The unit test `operation_defect_refuses_a_reference_edge_with_no_declaring_node` pins that location. The statement does not name that location. | spec/contract/FR-040-admit-frame-entries-and-state-clauses.md:210-212 |

## Finding Detail

- FND-001: Change the sentence to "at the operand, member declaration or
  member name at fault". The other option is to move the code's
  declaration-level refusals to a location the statement already names.

## Verdict

One low wording defect. Otherwise the statement is consistent with FR-322.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 467ac07: FR-040 now reads "at the operand, the member declaration or the member name at fault", and names missing-selection at the member declaration and ambiguous-name at the member name. |
