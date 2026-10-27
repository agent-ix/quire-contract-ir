---
id: SR-610034
title: "IR-1053 PR #603 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir PR #603 at f50585f52177269bbeb7405b1846d02b6ba97c8d; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, refinement.rs; tests/it/checked_package_v2_protocol_body.rs, checked_package_v2_refinement.rs; FR-064-AC-2/4/5, FR-038-AC-325"
review_set: subset
---

## Summary

Audited the changed owner and class-catalog behavior against its criterion tags and public reader tests. The positive second-selected-object retarget vector was replaced by a negative interface-owner vector, leaving a required acceptance path without evidence.

## Verdict

**CONDITIONAL** — one medium evidence gap requires a test in this PR. The full memory/drain catalog remains open under IR-1034 and does not receive a passing claim.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The test still tags FR-064-AC-2 but retargets to a selected `systems_interface` and asserts refusal; no test retargets a clause to a second valid selected `object_type`, rebuilds keys, then asserts admission and the changed clause/package identities. This leaves AC-2's positive path and the ticket's two-declaration vector unverified. | FR-064-AC-2; `tests/it/checked_package_v2_protocol_body.rs:1538-1571` | correct-requirement-no-evidence |

## Coverage

- FR-064-AC-2 (examined): A clause's `quire.application-node/v1` preimage contains `owner_model` as its local node id; an otherwise identical non-clause application preimage omits it. Retargeting the edge to a second valid local model and recomputing dependencies, clause key, downstream keys and package id changes the clause key and package id; retaining the old clause key while recomputing the envelope identity refuses `invalid_package`/`stale-node-key` at that clause's `node_id`.
- FR-064-AC-4 (examined): The changed test asserts wrong-kind selected interface refusal; unselected-object typed refusal is not exercised.
- FR-064-AC-5 (examined): The no-step owned class test asserts admission, missing class, wrong node and foreign class refusal through the public reader; new typed writer readback asserts package and refinement node identities.
- FR-038-AC-325 (examined): This PR covers non-memory owner/class joins only. IR-1034 must provide source-default memory/drain implementation and evidence; the ticket remains open.
- The computed matrix command did not return criterion rows during this review; trace tags and test assertions were manually inspected. Plan completion: not assessed. Optional repository-wide semantic review was not requested.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The expanded `refinement_catalog_joins_owned_finish_without_a_step_row` uses original selected A and a distinct selected B object-type node, asserts the changed clause key and package id after retarget, admits through the public reader, and round-trips the typed writer with the B owner and identities retained. |
| FND-002 | still-open | The new `protocol_role_member_defect_precedes_wrong_clause_owner` checks the earlier protocol-member phase and exact pointer, but no test pairs an earlier refinement-step row defect with a wrong-kind owner edge as TC-048 step 10 requires. |
| FND-003 | deferred | IR-1062 is the structured prerequisite: selected state_machine, process and record_value_type declarations cannot complete the current FR-322 owner join. Their wrong-form edge vectors must land after that owner-environment implementation, while IR-1053 and FR-064-AC-4 remain open. |
| FND-002 | fixed | The new `refinement_step_defect_precedes_wrong_protocol_clause_owner` first admits the healthy selected-B package, verifies the wrong owner alone refuses `invalid_model_binding` at the clause `owner_model`, then pairs it with an invalid refinement `source_class` and asserts the earlier `missing_declaration`/`missing-name` at `/body/steps/0/source_class`. The existing role-member pair covers the other required earlier phase. |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-002 | medium | FR-064-AC-4 and TC-048 step 10 now require an earlier refinement-step or protocol-member defect to win when paired with a typed owner-edge defect. The added tests check graph/dependency precedence and that a wrong owner wins over a later same-clause capture defect, but no paired earlier refinement-step or step-4/5 protocol-member defect is exercised. | FR-064-AC-4; TC-048 step 10; `tests/it/checked_package_v2_protocol_body.rs:1575-1700` | correct-requirement-no-evidence |
| FND-003 | medium | FR-064-AC-4/TC-048 step 10 name selected `state_machine`, `process` and `record_value_type` wrong-form owners, but this PR exercises only a selected `systems_interface`, a relation, source declaration and model root. The selected model-form branches have no public-reader refusal vector at `owner_model`. | FR-064-AC-4; TC-048 step 10; `tests/it/checked_package_v2_protocol_body.rs:1541-1672` | correct-requirement-no-evidence |

## Spec delta and coverage

The added FR-064 Behavior/AC-4 and TC-048 step 10 agree with QSpec FR-464-AC-20 at 000cd943 on the earlier unselected-target owner join and the typed clause edge. The public reader now has a genuine unselected object target test that asserts `missing_declaration`/`missing-selection` at the target `node_id`, plus missing local and wrong-kind edge tests. The two precedence pairs and three selected model-form vectors above remain required. Source-default memory/drain catalog evidence remains open under IR-1034; no claim of complete IR-1053 acceptance is made.
