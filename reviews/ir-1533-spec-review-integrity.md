---
id: SR-2026111317273300000003
title: "spec-review (integrity) of IR-1533 FR-038 field-edge population link"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #1020 commit 3ab92eb0; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Typed population of a reaches application: preamble, field-edge link, accessor errors, FR-038-AC-542..544, consumer bullet); spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md (reaches_field population section); spec/checked_package/matrix/tests.md (AC-542..544 row)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Spec review, integrity sub-analysis, for the spec commit of quire-contract-ir PR #1020. Ticket: IR-1533. The new section was read against the existing "The reader check" and "Dependencies" paragraphs of the same section, FR-038-AC-275 and FR-038-AC-276, and PR #1018's FR-038/TC-048/tests.md edits (AC-541). No ceremony was added: no pins, digests or tracking records.

## Examined scope

- FR-038 "The field-edge link" paragraph (examined).
- FR-038 IR-1533 preamble ("every sentence that says reaches application without qualification reads both operations") (examined).
- FR-038 accessor error sentence for `NotReachesApplication` (examined): consistent with the code.
- FR-038-AC-542, FR-038-AC-543, FR-038-AC-544 (examined): each atomic, testable and tagged.
- FR-038 "The reader check", last paragraph, and "Dependencies" (examined): see FND-001 and FND-002.
- FR-038-AC-275, FR-038-AC-276 (examined).
- TC-048 "Population of a reaches_field application" (examined).
- tests.md row for AC-542..544 (examined), and PR #1018's AC-541 edits (context_only).

## Verdict

Changes requested. The new link is well specified for type and refusal, and the AC ids do not collide with PR #1018's AC-541. However, the section now contradicts itself: the Dependencies paragraph and AC-276 exclude `reaches_field` from the join, while TC-048 tells tests to list the population (FND-001). AC-275 and the reader-check paragraph still say a `field` member admits the key unread (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | "The field-edge link" says nothing about dependencies. "Dependencies" names `quire.op.model.reaches` and `quire.op.model.navigate` explicitly and says "The join shall add nothing for an application of any other operation". AC-276 refuses a package that "lists a population id no reaches or navigate application names". The new TC-048 section says to "list it in `dependencies`". These cannot all hold, and the preamble does not reconcile them, because the Dependencies paragraph names the operation identity. Amend "Dependencies" and AC-276, or add an AC, so that a `reaches_field` population is added to the join (matching code-review SR FND-002). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4076-4089, 6317; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:2510-2522 |
| FND-002 | medium | The last paragraph of "The reader check" says every member kind other than navigate/reaches, `field` included, "is opaque today and admits it unread". AC-275 says "`field`, `position` and `element` members admit it unread". The field-edge link now reads and refuses the key on a `reaches_field` `field` member. Under the preamble, "other than navigate and reaches" is ambiguous. AC-275 is Implemented and its `field` clause is now false for `reaches_field`. Except `reaches_field` explicitly in both. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4066-4073, 6316 |
| FND-003 | low | "The reader checks the key after every other check of the application" is broader than the code. The check runs last in `check_reference_edge`, inside the constraint loop, before `check_mode_type`/`check_leaf_count` and before any later constraint. State it as the reaches link does: "as the last step of `check_reference_edge`". The paragraph's normative verbs are also indicative ("checks", "compares", "refuses") where the section uses `shall`. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3941-3957 |
| FND-004 | low | The tests.md row is inserted at the same anchor (after the AC-524..529 row) as PR #1018's AC-541 row, so `git merge-tree` reports a textual conflict there. FR-038 and TC-048 merge cleanly, and the ids do not collide (541 against 542..544). After #1018 merges, rebase and keep both rows. | spec/checked_package/matrix/tests.md:167 |
