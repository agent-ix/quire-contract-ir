---
id: SR-16454
title: "gap-analysis of PR 531 (IR-992 structural timed model)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2; tests/conformance_qspec/timed_model.rs"
review_set: subset
---
# SR-16454: gap-analysis of PR 531

## Summary

Ticket: IR-992. Reviewed the PR against QSpec #425. This review covers only the PR diff. Full timed semantics remain IR-997. No Cargo aggregate or Kani gate was run by the reviewer.

## Scope examined

- FR-038-AC-318: The public reader admits exactly QSpec FR-322-AC-120's eight `timed_model` forms. Each admitted form retains its published semantic type, occurrence roles, owner, declaration and complete body; an unknown form or malformed outer/body member refuses at strict wire validation before any timed-model body check. The writer emits one complete body per node and no wire identity projection.
- FR-038-AC-319: For a timed model with source, place, reset, guard, invariant, urgency and constraint nodes, graph-shape and structural-key checks compare each node's exact dependencies and QSpec preimage before the timed-model step. Changing one meaning-bearing body or owner member and rebuilding dependent keys changes the affected node and package identities; keeping a stale key or dependency list refuses at the earlier stage. Reordering only source occurrences changes neither identity.
- FR-038-AC-320: The timed-model step reconstructs the selected model from exactly one root and its referenced nodes. A missing root, dangling or wrong-form target, repeated place, noncanonical path, and invalid or non-time quantity each refuse with FR-322-AC-120's paired code and cause at the failing member, visiting timed nodes in ascending node-id digest order. The valid QSpec TC-367 CK-25 model admits and retains its normalized constraints.
- FR-038-AC-321: A package with both a timed-model defect and a protocol or operation defect reports the timed defect; one with a hybrid defect and a timed defect reports the hybrid defect. Strict-wire, graph-shape and stale-key defects continue to precede either body step.
- FR-038-AC-322: The public V2 writer round-trips an admitted timed model through the reader with the same complete node order, semantic identities, selected model and exact normalized quantities, and emits no `identity_preimage.identity_projection` member.

## Verdict

CONDITIONAL. The finding requires an author fix or a documented scope decision before merge. The remaining diff was inspected without another finding from this method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The sole new FR-038-AC-318 trace is on a vocabulary decode test. It does not exercise retained semantic type, occurrences, owner, declaration, complete body, strict-wire refusal, or writer output. The five conformance tests use TC-048 only. This tags the whole criterion as backed while most of it remains outside this slice; keep the test suite-only until a complete AC-318 test lands. | crates/quire-contract-model/src/checked_package/v2/vocabulary.rs:784-804 |

## Coverage

Plan completion: not assessed. Incremental matrix inspection: the only new FR-038 AC trace is the vocabulary test for AC-318; five timed conformance tests carry suite-only TC-048. Full-repository matrix parity was not reclassified in this diff-scoped review.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At PR #531 exact, The vocabulary-only test now has suite-only `TC-048` trace and no FR-038-AC-318 tag. The conformance tests remain suite-only; the full criterion is not claimed by this PR. |
