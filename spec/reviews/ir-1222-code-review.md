---
id: SR-202610310120
title: "Code review of IR-1222 model enum admitted join PR 766"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 766 diff at 387681d0; FR-354, FR-354-AC-6, FR-019-AC-5"
review_set: subset
---

## Summary

Reviewed all ten changed paths: the new model enum witness join, selected variant retention, dependent readers, public exports, FR-019 inventory, and public tests. Rust review, API, safety, boundary, and test checks were applied. No CI workflow changed.

## Verdict

FAIL. A declaration and a member can claim the same Exact NodeKey and both enter the validated index.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Separate declaration and member key sets accept one Exact NodeKey for both selected nodes. | crates/quire-contract-model/src/checked_package/v2/model_enum_witness.rs:226 |

## Evidence

The constructor uses `declaration_keys` for declaration rows and a fresh `member_keys` for member rows. A member whose `member_key` equals its declaration row’s `node_key` passes both insertions at lines 215 and 290. Both rows then read successfully with the same Exact identity, contrary to FR-354’s explicit cross-node uniqueness rule. Use one key set across both row kinds and add an adverse public test. The rest of the join validates model owner/form, selected document, original member identity, checked enum body, and package instance.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head ead9d58d: one `exact_keys` set now receives declaration `node_key` and member `member_key`; a new public adverse test supplies the declaration key as a member key and asserts `ConflictingWitness`. After excerpt: `|| !exact_keys.insert(row.member_key.clone())`. |
