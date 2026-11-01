---
id: SR-202611010114
title: "IR-1240 PR 795 structural member mapper gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #795; spec/model/functional/FR-048-derive-structural-and-group-candidates.md; crates/quire-contract-model/src/checked_package/v2/typed_terms.rs; tests/it/checked_package_v2_node_map.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-048", type: references }
---

## Summary

Reviewed the IR-1240 spec, tagged tests, and changed mapper code without selecting a plan. The external test proves the nested element slot, while the remaining operation-member census is exercised only through constructed unit records. Plan completion: not assessed.

## Verdict

**CONDITIONAL** — FR-048-AC-7's external-consumer census remains incomplete.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The external admitted-operation test covers only element; the other named operation-member slots are exercised only in constructed unit records, leaving the required external-consumer census unverified. | tests/it/checked_package_v2_node_map.rs:178 |

## Coverage

- Plan completion: not assessed
- FR-048-AC-6, examined: the external test is tagged, admitted, discriminating, and old-code failing for the nested element declaration.
- FR-048-AC-7, examined: source and unit tests cover five declaration variants, three relationship-end slots, and six fairness targets; external admission covers element only.
- FR-048-AC-8, context_only: deep graph performance is outside this PR's focused coverage.
- Matrix: `quoin matrix --repo . --json` computed FR-048-AC-6 as tagged by two external tests and AC-7 as tagged by five unit tests plus five external tests. Both have `evidence_status: no run evidence`; that status is not a code failure.
- Optional semantic audit: not run as a separate repository-wide pass; targeted AC-to-test inspection was performed for this PR diff.

## Evidence

The new external test obtains a real admitted `quire.op.collection.index` operation and matches only `CheckedOperationMember::Element`; its second map of the nested ID would fail under the previous flat-only implementation. The four other declaration variants, relationship-end member, and six fairness targets are tested in `typed_terms.rs` by directly constructing `CheckedApplicationOperation` with populated `views`. These unit tests do not prove that each named variant can be obtained from `CheckedPackageV2::read` and remapped through the public API with its borrowed view intact. Quoin's computed matrix confirms tags but cannot distinguish this semantic gap. An external consumer should cover the complete named census with admitted operations or explicitly establish why a variant cannot be admitted.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Commit `b1be4b10` adds one external `CheckedPackageV2::read` census test for every currently admissible named member-ID site: the five declaration variants, both relationship-end population shapes, and operation/protocol-root/attempt-node/parallel-branch/compensation-template fairness targets. Each maps flat to nested to another whole ID and checks callback count/order plus raw and borrowed typed mirrors. HybridTransition remains in the nested unit test; this IR reader has no `temporal_bindings` support for the QSpec hybrid fixture, so no currently admitted hybrid control exists. |
