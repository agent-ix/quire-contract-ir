---
id: SR-2026110414560020
title: "IR-1003 native projection gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1003 PR 901; tests/it/checked_package_v2_model_fields.rs; partial FR-352-AC-6/7"
review_set: subset
---

## Summary

Ticket: IR-1003. PR: quire-contract-ir#901. Head branch: titan/ir1003-residual-now. Reviewed only the native late-member partial regression, not whole IR-1003 closure. No applicable AssuranceProfile exists under spec/. No production, specification, dependency, CI, or existing assertion changes are present.

The real public admission/projection path receives two required fields in declaration order, Integer then Float32, with both native identity witnesses. Independent expected work is six: requested node, semantic type, two inspected field witnesses, then two members. Production charges the second member before inspecting its leaf. Exact error equality at limit five checks ExhaustedWork, requested/at record, record_field form and work five. Limits six and 100 check UnsupportedExactType, requested/at record, model/record_value_type form and work six. Returning Err precludes a partial authoritative projection. A separately admitted all-Integer record checks complete Composite/key, node/type_node, ordered original identities, MemberIds, Required presence, Integer types, work six and the same limit-five error. Repeated reads, package clone reads, dropped source document and dropped original package exercise retained state.

The existing one-field Float32 test does not reach a successful earlier member or budget precedence; the adjacent nominal-target test checks a distinct witness/target branch. Existing document, typed_object, field, admit and identity helpers are reused; no parallel implementation or copied upstream artifact is introduced. Expected values are independently authored contract oracles rather than returned-value-derived expectations. No skips, hidden assertions, warning suppression, weakened gates, or false trace tags are added. Per explicit task authority, this partial test deliberately carries no whole FR-352-AC-6/7 tag and no unrelated TC tag. Trusted IR fixtures do not establish actual QSL producer identity, closure or full witness-family qualification.

Recorded evidence read: /tmp/ir1003-native-test/fmt-check.log, clippy.log, new-test-restored.log, nearby-unsupported.log, nearby-nominal.log and mutation-integer.log. The restored new regression and both nearby tests each record one pass. Clippy records successful completion; fmt-check is empty, consistent with the supplied exit-zero receipt. The temporary Float32-to-Integer fixture change records the expected assertion failure (complete success versus UnsupportedExactType), followed by a restored passing run. This is test-oracle discrimination, not production mutation proof. No Rust command, test, lint, Kani, matrix, coverage, properties, or trace scan was rerun by this reviewer. The lead's one final scoped gate remains pending.

## Verdict

**CONDITIONAL** for the explicitly bounded review. No defects found. Manual narrowly scoped acceptance-criteria-to-test/source evidence review only. The full Quoin gap-analysis method was not executed: user authority forbids live matrix/coverage/trace recomputation. Therefore this is not a repository assurance PASS, a measured coverage delta, or a claim that all FR-352 forms are covered. Qualified Inspection remains Planned under IR-1006/Quoin; FR-352-AC-6 and AC-7 are examined only for this partial behavior. No criteria or verification methods are weakened. Plan completion: not assessed

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-352-AC-6; role: examined; excerpt: Every other admitted V2 type form either projects to an Exact variant with its full bounds, mode, members and identity, returns `UnsupportedExactType` when that complete shape cannot be faithfully represented, or returns `UnavailableIdentityWitness` when a required authoritative opaque identity witness is absent. Each refusal names the requested node, first failing type node and form, and charged work, before exposing any partial view. An admitted union without its authoritative declaration and payload identity witness takes the missing-witness outcome; a declaration key alone does not establi

- FR-352-AC-7; role: examined; excerpt: Repeated reads of one admitted package and its clone return equal complete values or equal typed outcomes. A nested or cyclic type either completes within the finite limit or reports typed exhaustion with the charged work and locus; no call reparses wire/source, grows the call stack with type depth, or grants admission to an unadmitted fixture.

- admitted_native_record_late_unsupported_field_respects_exact_work_boundary; role: examined; excerpt: fn admitted_native_record_late_unsupported_field_respects_exact_work_boundary() {

Spec/plan analyses are inapplicable to the unchanged requirements. No full criterion credit or completion is claimed. Plan completion: not assessed
