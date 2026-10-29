---
id: SR-558005
title: "IR-1014 PR 562 acceptance coverage review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-059, FR-060, FR-062, TC-060, TC-061, TC-063 and PR #562 tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-059
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-060
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-062
    type: reviews
---

# SR-558005: IR-1014 PR 562 acceptance coverage review

## Summary

Ticket: IR-1014. `quire matrix --scope. --format json` tags all fourteen reviewed criteria, but several tags cover only a subset of the named vectors. This is a diff-scoped gap analysis, not a plan audit. No source or test stub was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The single FR-059-AC-4 test compares `possible` with `always-possible` and adds a second occurrence, but never varies source spelling, the request subject, or an `over` instance while asserting the admitted node and writer bytes are retained. FR-059-AC-5 claims IR-owned positive vectors for SN-01 through SN-17; SN-07 and SN-16's retention distinctions have no assertions. Add IR-owned public reader/writer controls for the representable node/source-map portions, with the request identity left to its owning layer as the spec says. | tests/it/checked_package_v2_claim_terminal.rs:736 |
| FND-002 | medium | FR-060-AC-4 tags the identity test, but the test never changes only the written terminal declaration name and asserts unchanged node and package identities (MT-04). The other identity pairs are checked. Add the missing name-only control through the public read/write path. | tests/it/checked_package_v2_claim_terminal.rs:839 |
| FND-003 | medium | FR-059-AC-3 and FR-060-AC-3 tag one ordering test that proves dependency before state-graph and state-graph before terminal, but does not exercise a temporal or abstraction body defect before terminal, nor a terminal defect before a later body-step defect. The criterion explicitly requires those orders. Add mixed-defect vectors at the public reader. | tests/it/checked_package_v2_claim_terminal.rs:1092 |

## Coverage

Examined FR-059-AC-1 through AC-5, FR-060-AC-1 through AC-5, and FR-062-AC-1 through AC-4. All are mechanically tagged in the computed matrix; the findings identify partial behavioral evidence behind four of those tags. The new code is owned by FR-059, FR-060 or FR-062; no reverse ownership gap was found. Plan completion: not assessed.

## Verdict

CONDITIONAL. Complete the named vector and reader-order assertions before treating the tagged criteria as backed.

## Dispositions

Fix round 1, PR:

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `state_graph_identity_and_shared_occurrences` now varies the source digest and source map while retaining the graph node and package identity, and round trips the changed source map through the public writer and reader. It also compares two instance-locus variations; request identities remain the owning layer's concern. After excerpt: `assert_eq!(possible["package_id"], respelled["package_id"]); assert_ne!(possible["source_map"], respelled["source_map"]);` |
| FND-002 | fixed | `terminal_identity_and_uniqueness_per_model` now varies only the source spelling between `Done` and `Stop`, asserts equal terminal node and package identity with distinct source maps, and round trips both packages through the public writer and reader. After excerpt: `assert_eq!(done["package_id"], stop["package_id"]); assert_ne!(done["source_map"], stop["source_map"]);` |
| FND-003 | still-open | The new vectors prove temporal and abstraction defects run before terminal, and a terminal defect runs before the operation step. The last control mutates a function operation identity (`terminal_before_operations`). An operation is a later operation step, not a later body step. FR-060-AC-3 also requires terminal before step-relation and later body-step defects; exercise at least one admitted later body form (hybrid, timed-model, protocol or schedulability) with its own defect. |

The gap-analysis verdict remains CONDITIONAL until FND-003 is resolved.

Fix round 2, PR:

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-003 | still-open | The new mixed vector uses `protocol`/`protocol_reference`, whose target join QSpec FR-322 assigns to the earlier declaration step. It is not a later protocol body-step defect. The test currently expects terminal to win, contrary to that order. Use a malformed later body form after the carrier ordering is corrected. |

The current gap-analysis verdict remains CONDITIONAL.

Fix round 3, PR:

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-003 | fixed | The revised test first proves the malformed `hybrid`/`hybrid_model` alone reaches `hybrid::validate_models` and refuses `invalid_model_binding`/`malformed-declaration` at its variable target. The same malformed hybrid beside an invalid-mode terminal reports the terminal's `invalid_package`/`invalid-value` at `body/members/1`, verifying terminal-before-hybrid body-step precedence. The carrier-before-terminal control separately preserves FR-322's earlier declaration-step order. After excerpt: `let bad_hybrid_only = package_with(malformed_hybrid_model);... let terminal_before_hybrid = package_with_nodes(vec![bad_terminal.clone, malformed_hybrid_model]);` |

## Final reviewed-head verdict

PASS. Every gap-analysis finding has a verified fixed disposition. Final aggregate gates remain with the lead.

## Final gate regression re-review

, the intake-only loop change leaves the FR-059, FR-060 and FR-062 vectors and their trace tags unchanged. The existing FR-038-AC-117 flat-wire test covers the exact 256 KiB stack regression, and the coder's focused log reports it passes with all 16 flat-wire tests. An isolated depth-61 diagnostic-details control also passed on the 256 KiB stack. No new coverage finding was established. PASS; the final aggregate gate belongs to the lead.

## Public API inventory re-review

, the only new diff is the FR-019 inventory row for the exported `CheckedTerminalDuplicate` type, with FR-062 ownership. This aligns the public inventory with FR-062-AC-4 and the existing public-consumer test; it changes no FR-059, FR-060 or FR-062 acceptance text or vectors. The focused public-interface inventory test passed in the coder's report. No new gap finding was established. PASS remains the gap-analysis disposition.
