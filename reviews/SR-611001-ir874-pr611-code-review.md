---
id: SR-611001
title: "IR-874 PR 611 code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/model_fields.rs; crates/quire-contract-model/src/checked_package/v2/model_fields/tests.rs; crates/quire-contract-model/src/checked_package/v2/model_members.rs; crates/quire-contract-model/src/checked_package/v2/refinement.rs; tests/it/checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

Ticket: IR-874. Diff-scoped code and Rust review of PR #611. Examined FR-038-AC-325 and TC-048. The activation catalog joins and normal inherited order are implemented, but a diamond with the redefining branch second has the wrong binder order.

## Examined units

- **FR-038-AC-325** (`examined`): "An `on each (x: T)` clause admits `activate <protocol>` associated with that clause node and binder types `[T]` for non-record `T`, or the effective fields of record `T` in source supertype order then own declaration order, retaining a diamond field once and placing a redefining field where the hidden field stood; reversing source supertype order changes that binder order."
- **TC-048** (`examined`): "For record activation binders, require authored field and supertype order, diamond deduplication, and a redefinition through L at Top::t's position."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A diamond whose redefining branch is second moves the replacement after the first branch's own fields instead of keeping the hidden field's position. With supertypes [Right, Left], where Left redefines Top/t, the derived binder order is [Right/r, Left/t_prime, Left/l, Bottom/z]; required is [Left/t_prime, Right/r, Left/l, Bottom/z]. | crates/quire-contract-model/src/checked_package/v2/model_fields.rs:423 |

## Verdict

FAIL. FND-001 produces a wrong checked binder signature for a valid source-order variation required by FR-038-AC-325. The public writer/read positive and adverse checks for non-record activation and ordinary inherited order are sound in the reviewed diff. The new operation-parameter reorder assertion was compiled but not executed at this review head; the final gate must exercise it.

## Checks

`git diff --check origin/main...HEAD` passed. Coder evidence at this head: activate-focused 4/4, field-table 19/19, and model-lib Clippy passed. No review-owned Cargo build was started while the shared build lock was occupied. No CI workflow or dependency-file diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | In the reviewed PR, each effective field carries origin identities. `build_component` sorts exposed slots by the first position of any origin, so Left/t_prime takes Top/t's slot even when Right is the first listed supertype. The new public-reader reverse-diamond test passed 1/1; the model-field unit test asserts the identities directly. |

## Current verdict

PASS for the reviewed fix. The previous finding is fixed. `git diff --check` passed. The separate operation-parameter reorder assertion remains for the final aggregate gate.

## Integration check

PASS at PR #611 merge. The merge with IR #603 changes only the shared test import in its conflict resolution; `structural_key` and `rekey_application_node` are both retained. The #603 owner-kind guards in `validate_catalog` and `protocol_classes` remain intact and precede activation class reconstruction. Its `refinement_step_defect_precedes_wrong_protocol_clause_owner` and `refinement_catalog_joins_owned_finish_without_a_step_row` public-reader tests remain present and unchanged; the activation and reversed-diamond tests remain present. `git diff --check` passed. No new finding from this integration delta. The final aggregate gate remains pending.
