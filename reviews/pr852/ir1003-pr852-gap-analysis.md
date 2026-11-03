---
id: SR-2026110283836
title: "PR 852 model root identity gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1003; PR 852; tests/it/checked_package_v2_reader.rs; FR-038-AC-20"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-contract-ir/FR-038", type: references}
---

## Summary

Independent scoped review of PR 852's single native reader test against FR-038-AC-20. No substantive defects found. Production behavior and normative specification are unchanged.

## Verdict

PASS within this one criterion and changed test. Ready for the lead's final aggregate integration gate; no repository-wide assurance, proof or merge verdict is claimed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-20, role: examined, path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md. Excerpt: Two `model_root` nodes naming one `identity` refuse `invalid_semantic_graph` at the later root's `owner` (`/semantic_graph/nodes/<i>/owner`), whichever order the graph holds them in; two roots naming different identities admit, each other check passing. A root carries no `version` and no `digest`, so the identity is the whole locator (FR-038-AC-62).
- The entire new test, its existing local fixture/evidence/refusal helpers and selected_model_roots implementation were inspected. No copied owner fixture, duplicate helper, synthetic whole-family tag, production change, gate narrowing or unowned behavior was introduced.
- Both distinct-identity root orders pass through real public CheckedPackageV2::read and must admit with exactly two roots. This establishes that the selected documents, source map and all other checks are valid before each mutation. The fixture is locally authored synthetic reader input, not producer-origin qualification, and claims no producer-origin credit.
- Each adverse case changes only the later root's owner identity to equal the earlier root's identity. Root positions come from the original complete node array, not a filtered-root index. Swapping the complete roots preserves the positions while reversing their identities. Both refusals assert InvalidSemanticGraph and the later root's owner pointer, with no cause, locus, contract-version or document-pointer payload, through the existing canonical expected-refusal comparison. A first-root pointer or admission regression fails these assertions.
- selected_model_roots scans wire nodes in authored order, rejecting a repeated selected identity at its current node position before selected-document admission. The positive controls prevent an unrelated fixture refusal from standing in for the targeted duplicate check.
- Computed matrix independently inspected with Quire CLI 0.38.1 / engine 0.51.0 and the process provider plus installed ISO module. FR-038-AC-20 is tagged to exactly tc_048_model_root_identities_are_unique_in_either_graph_order at tests/it/checked_package_v2_reader.rs:1349. Existing candidate coverage reports 98 unbacked, 22 retired, zero status contradictions, 988 total / 966 active criteria. This is one binder added; remaining pre-existing gaps are outside this scoped verdict.
- Plan completion: not assessed
- Semantic alignment explicitly requested for this criterion; no whole-repository semantic audit performed.
- No applicable AssuranceProfile found under spec.

## Validation evidence

Inspected existing focused-test.log: the new integration test passes once, none ignored. Inspected existing workspace Clippy receipt with locked dependencies, conformance-cli and target it: success with warnings denied. Existing fmt receipt is clean. No repeated Cargo build, full aggregate gate, Kani, IEEE/CBMC or proof execution was performed by this reviewer. Final native integration target it belongs to the lead.
