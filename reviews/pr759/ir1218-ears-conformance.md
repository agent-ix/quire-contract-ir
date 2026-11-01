---
id: SR-202610310123
title: "IR-1218 ears-conformance review"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; IR-1218 stack; protocol Location projection and eight changed paths"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: reviews
---

## Summary

Independent ears-conformance pass on the captured protocol Location reader and its owner-backed conformance evidence.

## Verdict

PASS for this scoped method; all review findings are fixed. The final PR #768 head CI gate remains outstanding before merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope and evidence

Ticket: IR-1218, PR #759, application version 0.1.0. Scope is the stack delta against the IR-1214 sidecar branch, not its entire delta against main. Examined all eight changed paths: protocol_locations.rs, dispatch_projection.rs, V2 mod.rs, model lib.rs, FR-019 inventory row, FR-354, conformance main.rs and conformance protocol_locations.rs. The canonical checked_term_data types and shared envelope belong to the base owner and are not duplicated here. No applicable AssuranceProfile or required enforced review policy was found under spec/. No workflow or dependency changes, shims, evaluator, QSL dependency, copied fixture, unsafe operation or mirror semantic type are introduced.

Owner vectors were read directly from the QSpec checkout. Initial parent-run Location conformance log contains nine passing tests; review found that this was false-green for PL-R2 because its expected refusal was discarded. Parent has corrected that test to assert the recorded code, cause and locus; no current green result is claimed after that correction. The same known base Clippy failure is E0063 at model_fields/tests.rs:1049, missing checked_term_data in a shared envelope initializer, already reported to IR-1214. Initial format drift was corrected by the parent; final format and final merged-base checks remain pending. This reviewer ran no Cargo or Kani command. Final base integration and disposition remain mandatory before merge.

## Analysis

The changed projection statement names Contract IR and gives the trigger (a consumer requests a protocol semantic location) and concrete response (borrow the canonical captured row). Validation, distinct missing-data outcomes, bounded complete output and forbidden inference are observable assertions. The scoped deterministic quire validation completed without EARS warnings; existing module duplicate-registration diagnostics are contextual tool warnings. No changed requirement has an ambiguous EARS trigger, vague response or missing actor. The upstream vector contradiction is recorded by base/integrity, not a grammar finding.

## Same-review final validation

The parent added `reversed_distinct_term_keys_refuse_at_the_first_out_of_order_row`; the inspected focused log reports one passed, zero failed or ignored. It reverses distinct term keys and asserts `invalid_package`, `invalid-value` and the second row's exact pointer. The corrected PL-R2 test was also executed separately: zero passed, one failed because the production reader returned Admitted rather than the owner-recorded refusal. That red is retained honestly; it is not a passing conformance result or a completed feature. The parent confirmed that positive PL-01-root, after removing its id, equals the entire sole PL-R2 row. No additional owner census distinguishes them.

The draft remains unmerged and blocked pending an owner witness or clarified contract; no inference, shim, skipped oracle or acceptance narrowing was introduced. Shared-base E0063 remains separately tracked on IR-1214. This same-review disposition ran no Cargo or Kani command and made no source edit.

## Same-review main rebase and fix audit

This is a continuation of the original independent review, not a second independent pass. The canonical sidecar landed on main through IR-1214. The previous stacked PR #759 was closed when its base was deleted and the host refused reopening/retargeting, so replacement draft PR #768 carries the same feature and review artifacts against main. The audit compares the feature commits with their actual merged main base; newer unrelated main commits are not deletions made by this feature. The common checked_term_data types are unchanged by the feature. No source edit, Cargo or Kani command was run by this reviewer.

The base initializer Clippy E0063 reported earlier is resolved by the merged sidecar. The parent supplied an actual whole-IT run: 1073 passed, eight failed, one ignored. One failure was the retired unsupported-protocol-row expectation and is repaired by a canonical borrowed-row assertion plus a bad-occurrence refusal while equality negatives remain intact. Five failures required the authoritative scanner executable; its task-local installation now produces a direct zero-occurrence scan after marker/typed-target edits. The remaining two wire-pointer assertions reproduce on clean main and were not narrowed. Seven focused tests passed before the final target-only parsing refinement; final focused, conformance, Clippy and format reruns are pending. No whole-IT green or current all-checks-green claim is made. The PL-R2 owner contradiction remains unresolved on current QSpec, and the replacement remains draft and unmerged.

## Same-review source fix closure

The subsequent fix removes the broad marker from validate_rows, passes the existing CheckedNodeKind slice into Location validation, and uses typed Expression matching for the graph-family decision. Ordering, graph lookup and occurrence/source-map joins remain unmarked. Separate narrow marked intakes now check only still-untrusted row namespace, safe numeric term coordinate, and captured origin/ordinal grammar, in the same refusal order and with the same row/work charges. Locality is retained explicitly because main may hold imported graph nodes. No public runtime read or broad semantic validator is exempted.

Hybrid validation now uses the private canonical CheckedFairnessTarget::hybrid_for_join intake. It reads exactly the existing pre-shape join coordinates and retains the previous empty-name fallback, without prematurely enforcing extra-field shape. The ordinary later operation-member shape step still rejects unknown members, so this grants no new accepted wire, public authority, mirror type or compatibility API. The added hybrid_join_preserves_extra_target_member_refusal_order control asserts both extra-only OperationMemberMismatch at member and extra-plus-wrong-declaration MissingName at target/name. This source fix addresses FND-003; its final execution is pending.

The direct authoritative scanner log is clean after these changes. Focused rerun, Location conformance, workspace Clippy and formatting receipts remain pending, so source-fixed dispositions are not an all-gates-green or merge verdict. The known whole-IT run and two clean-main baseline reproductions remain recorded above. PL-R2 remains blocked on an owner witness or explicit clarified contract.

Additional fix scope examined: crates/quire-contract-model/src/checked_package/v2/hybrid.rs, typed_terms.rs and protocol_locations.rs; tests/it/checked_package_v2_model_members.rs; tests/conformance_qspec/hybrid.rs. The existing checked_term_data.rs canonical wire types are unchanged. The hybrid/target-visitor scanner fixes are independently extractable into a main follow-up once their required validation completes.

## Final execution verification

The same-review final receipts are now available. The final source delta only corrects the canonical CheckedNodeKind import path and formatting; the audited marker boundaries and private pre-shape hybrid join semantics are unchanged.

- Whole IT actually ran: 1079 passed, two failed and one ignored. The only failures are protocol_owner_edge_has_closed_wire_presence_and_local_shape and inline_path_operand_refuses_at_the_segment. Both reproduce on the clean main baseline; their expected pointers were not weakened. This is not a whole-IT green result or an exception to fix them.
- Selected QSpec conformance actually ran: 13 passed and one failed. The new hybrid_join_preserves_extra_target_member_refusal_order test passes both extra-only and extra-plus-wrong-declaration branches. Existing positive/unknown-mode/wrong-declaring-model hybrid controls, the canonical Location tests and the reversed distinct-key control pass. The sole red is PL-R2, which still returns Admitted instead of its owner-recorded missing-term refusal. No failing oracle was skipped or reclassified as success.
- Workspace all-target Clippy with warnings denied passed. The normal authoritative scanner controls in the whole IT run pass, and the direct scanner reports no unresolved occurrences.
- Final cargo fmt --all -- --check passed with exit 0. The empty formatting log is expected; the parent supplied its successful process receipt.

Code-review FND-003 and FND-004 are now execution-verified fixed, in addition to their inspected source fixes. Code-review FND-002 remains fixed because the owner refusal oracle is asserted honestly. The scoped sorted-key evidence finding remains fixed. The remaining owner-contract finding is still open: the positive PL-01-root row equals the entire sole negative PL-R2 row and there is no retained required-term census or explicit owner ruling resolving that contradiction. Replacement PR #768 remains draft and unmerged. No final all-checks-green statement, AC narrowing or merge approval is given. This reviewer ran no Cargo or Kani command and made no source edit.

## Merged owner correction and final same-review disposition

STD-622 has now landed through QSpec PR #562. The published source/spec/vector files were compared byte-for-byte with the reviewed proposal and match. PL-01-root is unchanged. PL-R2 now retains the existing ordinal-one child row and therefore witnesses an absent ordinal-zero predecessor, rather than duplicating the valid one-term root capture. The merged FR-322 prose and AC-291 require consecutive zero-based ordinals per application/occurrence, preserve all existing row shape/join/duplicate/order refusal precedence before the array-level gap check, and explicitly forbid inferring an absent tail. The unknown-origin precision amendment preserves its rejected payload and malformed-wire code and specifies the supplied kind member as its locus. No required-term census or source reconstruction is invented.

The frozen IR source implements that observable owner rule in one typed pass: the existing key checks run first, expected ordinal is zero for a new application/occurrence or the previous ordinal plus one within its group, and a witnessed-gap flag is refused at the array pointer only after all row validation. The admitted safe-ordinal bound makes the increment safe. A lone zero is admitted; a lone one or [0,2] refuses; a new occurrence starts at zero. No missing successor is inferred. The shared canonical sidecar types are unchanged.

The inspected final selected run passes 16/16 tests with no ignored case. Its dynamic vector test executes all six PL IDs and asserts each owner-recorded disposition, cause when specified, and exact pointer; it prints every outcome. PL-01 admits, the grouped PL-02 child preserves its complete path, PL-03 admits, PL-S1 refuses at origin/kind, PL-R1 retains its conflicting-definition row1 refusal, and PL-R2 now refuses invalid_package/invalid-value at the array pointer. The interior-gap, occurrence-reset, reversed-key, exact work-bound and large-child-index controls pass, as do the four hybrid controls including both extra-field refusal-order branches. Workspace all-target Clippy with denied warnings, formatting and the direct authoritative scanner pass. No Cargo or Kani command was run by this reviewer.

The original owner-contract finding is fixed by the merged correction and its verified IR enforcement. All original review findings, including the sorted-key evidence, false-green oracle and scanner-fix findings, are now fixed. This is the same review and unchanged method/id set. The previous whole-IT baseline failure counts remain historical evidence, not a current full-suite claim. Overall review is CONDITIONAL only on the final PR #768 head completing its required CI checks green; Rust Checks are still in progress at this audit. The parent must wait for that final-head result before merge. No unconditional merge approval, weakened oracle or new independent review is given.
