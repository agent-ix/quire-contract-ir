---
id: SR-997
title: "criterion strength review of PR 263 (FR-038-AC-81 to AC-88)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@376269e0d67ee1451ee9629dbe897c06a8a730de; spec/checked_package/functional/FR-038-consume-checked-package-v2.md FR-038-AC-81 to FR-038-AC-88; tests read in crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---
# SR-997: criterion strength review of PR 263

## Summary

Ticket: IR-532. For each AC, a mutation of the reader that the AC (and its test)
would catch:

- AC-81: `catalog.entry` always `Some`, or the operator-class comparison dropped.
  A reader that refuses everything fails the admission control.
- AC-82: the length checks dropped, or the role check dropped. The test uses a
  catalogued `integer_division` definition, so a role-check removal cannot pass
  through the definition check.
- AC-83: mode presence skipped, or `type_pin` ignored. Both the reference and the
  literal operand fail.
- AC-84: member-kind comparison dropped, the field-name lookup dropped, or
  `check_reference_edge` admitting an unknown declaration.
- AC-85: the arity check dropped. `literal`/`application` unresolved in
  `argument_family`. The clause special case dropped. `same_type` comparing operand
  nodes rather than resolved types.
- AC-86: any form moved in `operand_family` or `is_type_shaped`.
- AC-87: a string `operation` admitted.
- AC-88: the group rewrite or ordinal changed, or the stale-key check dropped.
  The admission control catches a reader that refuses every node.

No AC is vacuous. Planned markers are honest: tests.md lists AC-81 to AC-88 as
planned and untagged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-86's oracle is the implementation. The pair list was transcribed from `operand_family`/`is_type_shaped`, so it pins private tables rather than a QSpec-stated family assignment, and it already disagrees with AC-68 (SR-994 FND-001). Keeping it is reasonable as the only owner of `operand_classification_is_exactly_the_catalog_mapping`, but only once amended for `temporal`/`formula`. AC-88's preimage text is a contract (QSpec FR-322 preimage), not white-box, and should stay with a real external vector (SR-994 FND-002) | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1408 |

## Verdict

The criteria are strong. Open question (f): keep AC-86 amended, or land it after
#253. Keep AC-88 with its provenance clause fixed. It is not redundant with merged
AC-74 to AC-80: those fix the wire types' encoder, while AC-88 fixes the
application-node preimage and key check.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@9d4379bea459d7a30bc277ecbdaa72bae08d9bac.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | AC-88's oracle is now the canonical text the unit test spells out literally, so the AC defers its expected value to its own test; the named members (recursion, group_reference, stays reference) carry it, but the full text is not in the spec. TC-048 still says compare with the pinned one and the preimage equals the pinned text (lines 310, 327), a leftover of the dropped QSL claim. State the text in TC-048 or reword pinned to the expected text. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1444 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 9d4379b |
| FND-002 | fixed | cfb815d |
