---
id: SR-590007
title: "IR-1024 PR 577 acceptance gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; tests/conformance_qspec/hybrid.rs; tests/conformance_qspec/timed_model.rs; FR-038-AC-316; FR-038-AC-318; IR-1024"
review_set: subset
---

## Summary

Reviewed the changed tests against the ticket acceptance criteria, FR-038-AC-316, the computed Test Matrix, and the selected QSpec fixtures. Two acceptance gaps remain.

## Verdict

**CONDITIONAL**: the test change keeps strict reader oracles, but it does not exercise a valid hybrid claim paired with the protocol defect, and the 17-case baseline classification requested by the ticket is absent.

## Scope

- IR-1024 AC 1 (examined): Capture and classify each of the 17 failing conformance cases against a pinned QSpec commit and current IR main, naming whether the current QSpec contract or the IR reader/test needs correction.
- IR-1024 AC 2 (examined): Repair owned mismatches in the authoritative repository, preserving strict IR conformance assertions and reading QSpec fixtures from the QSpec checkout only; no fixture copies or weaker expectations.
- IR-1024 AC 3 (examined): Run the focused QSpec conformance target against the corrected pinned pair and record all results; keep any remaining external dependency as an explicit linked blocker.
- FR-038-AC-316 (examined): A valid hybrid claim and a later protocol defect yield the protocol refusal; a hybrid defect and protocol defect yield the hybrid refusal.
- FR-038-AC-318 (context only): The timed-model fixture admits and the writer emits complete bodies with no wire identity projection.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new protocol precedence test contains no valid hybrid claim, leaving the valid-claim branch of FR-038-AC-316 unverified | tests/conformance_qspec/hybrid.rs:1249 |
| FND-002 | medium | The deliverable records only a 15/2 rollup, not a classification of each of the 17 original failures required by IR-1024 AC 1 | qualification-logs/p-ir-79-code-lead/ir1024-focused-evidence.md:10 |

## Coverage

Plan completion: not assessed

The computed matrix marks FR-038-AC-316 `tagged`; the changed test's trace tag is syntactically present, but it cannot establish the valid-claim branch because QSpec `positive-all-families.json` contains hybrid model, mode and transition nodes and no hybrid claim. The other half of the changed test is a model defect and does distinguish hybrid-stage precedence. FR-038-AC-318 was already untagged before this PR; this diff neither claims nor changes its full criterion coverage. The 56-node assertion and existing round-trip assertions remain meaningful evidence for this fixture. No production code changed, so no new reverse code-to-spec gap was introduced. Optional semantic review beyond these changed bindings was not requested.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `braking_package` supplies a valid QSpec hybrid claim and the test asserts baseline admission before the protocol defect; it then retains that defect while changing the claim clock to `wall-time` and asserting the exact hybrid refusal. The hybrid reader owns this clock check at `hybrid.rs:963`, after the temporal pass. |
| FND-002 | fixed | PR comment 6420800977 lists 17 unique original failing cases and assigns 15 to QSpec #436 and two to IR #577. Independent comparison with the saved old-QSpec and current-QSpec raw test lines confirmed all 17 old failures, exactly 15 passing on unchanged old IR with QSpec #436, and the remaining two failures. The final current-pair raw log reports 68 passed, 0 failed, 1 ignored. |

## Current disposition verdict

**PASS after disposition round 1.** The original Findings and historical conditional verdict remain as the review-pass record; both findings above are fixed at PR. The same independent Opus reviewer found no new Rust test defect in the fix delta. The review files retain the one placeholder required by Quire's Findings table schema; the original finding text is not rewritten.
