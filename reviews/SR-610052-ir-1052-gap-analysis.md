---
id: SR-610052
title: "gap-analysis — IR-1052 Boolean transition V2 reader"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/, tests/conformance_qspec/transition.rs; IR-1052"
review_set: subset
---

## Summary

Independent gap-analysis review of PR #607 at e52bde10a26c5a059b0814e14ff05e8acb13a4ed against QSpec 60383a35. Examined FR-067-AC-1 through AC-7.

## Verdict

**FAIL** — 3 finding(s); QSL-authored AC-7 remains open on QSL-1163.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The duplicate (anchor, subject) test asserts code and cause but not the required locus at the later transition in ascending node-id digest order. An implementation returning the wrong node path passes this test. | tests/conformance_qspec/transition.rs:838-849 |
| FND-002 | medium | The two-defect evidence covers schema, stale key, frame, clause and within-transition order, but does not exercise graph/dependency or selected-declaration before frame, anchor before transition, or operation-catalog after state. These are explicit precedence boundaries in AC-4 and TC-064; a reordering of those stages can pass every added test. | tests/conformance_qspec/transition.rs:512-535 |
| FND-003 | high | The computed Quire matrix marks AC-7 untagged. The IR-owned QSpec fixture/mutation half executes under other AC tags, but no test binds AC-7. The QSL-authored source/claim half remains genuinely blocked by QSL-1163. Add the AC-7 trace to an IR-owned test that asserts direct QSpec fixture use; keep the ticket AC open for the QSL-authored evidence. | tests/conformance_qspec/transition.rs:1-1225 |

## Coverage

Plan completion: not assessed. The computed Quire matrix marks FR-067-AC-1 through AC-6 tagged and AC-7 untagged. No specification file changed in this PR; spec-review was not applicable. Formal aggregate gates are reserved for the final reviewed head by the code lead.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Duplicate test derives later node by digest and checks both its path and typed locus. |
| FND-002 | fixed | New two-defect tests cover graph/dependency and selected owner ahead of frame, anchor ahead of transition, and state ahead of operation law; earlier tests cover remaining boundaries. |
| FND-003 | fixed | The public-reader QSpec fixture test now carries AC-7, so the IR-owned half is tagged. QSL-authored source/claim evidence remains open on QSL-1163 and cannot be claimed by this fix. |
