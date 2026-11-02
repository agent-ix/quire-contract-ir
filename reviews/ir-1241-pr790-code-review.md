---
id: SR-202610310148
title: "Code review of IR-1241 QSpec conformance harness PR 790"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 790 frozen head c7acd764; tests/conformance_qspec/{dimension_equivalence,extension,hybrid,main,model_tuple_union,node_index,protocol_vectors,reaches_population,selection_vectors,timed_model}.rs and tests/it/checked_package_v2_owners.rs; IR-1241"
review_set: subset
---

## Summary

Ticket: IR-1241. Reviewed all nine changed Rust conformance-harness files against the repository's Rust conventions, QSpec FR-322 graph-local index and source-key rules, and the recorded failures from the current QSpec vectors. No applicable AssuranceProfile was found under `spec/`. The helpers resolve structural node identities for ordering and rewrite graph-local indexes when constructing a smaller graph; the tests retain exact admission and refusal assertions.

## Verdict

PASS on frozen head c7acd764. No defect was found in the changed harness code. The final focused run has three reaches failures assigned to IR-1249; the separate TC-229 node-57 refusal remains with IR-1251. This verdict does not claim those external reader obligations are complete. The lead owns the final aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope and evidence

Examined FR-035-AC-9 and FR-038-AC-1/AC-387..389, with QSpec FR-322-AC-73, AC-95, AC-96, AC-268, AC-299 and AC-270 as external contract context. The changed tests call the public reader and compare its retained wire, structural identities, source-map occurrences, error codes, and loci; the independent fixture helper computes expected structural order from the QSpec graph's index segments. The changed files add no copied QSpec artifact, dependency, unsafe code, or CI workflow change. Focused evidence recorded by the coder: format and test Clippy pass; QSpec conformance at c6784e45 has 162 passed/3 failed. The remaining three failures and their owning ticket are stated in PR 790.

## Recheck at 844ad5d1

The added owner test now reports the actual refusal. After constructing each TC-229 population case, it sorts dependencies by `resolved_fixture_id`, which resolves graph-local node-index segments against the final owner-projected structural identities. It then refreshes application identity and source-map rows. The focused TC-229 log shows both formerly failing paths pass the node-45 dependency check and reach the same node-57 `MissingDeclaration/MissingSelection` refusal, already present on the exact base and assigned to IR-1251. `git diff --check` passed. No new finding was identified.

## Recheck at ae89204a

The typed XP fixture builder copies each published `pair` operand verbatim into `dimension_node_ids`; it adds graph-local dependencies only for local plain node refs and selects foreign packages through `dependency_selections`. XP-11..15 assert exact missing-selection, missing-name, self-minted and wrong-kind refusal codes and pointer members. XP-05's operand index is read from QSpec c6784e45 rather than a case-name branch. The imported-package evidence is filtered to direct selections; closure-only packages do not become direct evidence. The `hybrid.rs` reference path uses a graph-local `{node:i}` segment. QSpec FR-322 permits the producer to assign an identity label to an equivalence node, so the fixture's `equivalence-{index}` path is legal. The focused QSpec run reports both XP tests passing and three reaches failures outside this PR. The TC-229 node-45 sort fix is retained. `git diff --check` passed. No new finding was identified.

## Recheck at c7acd764

QSpec #580 (`2f8b9c8b`) explicitly confirms a producer-assigned string equivalence label and graph-local index segments; it rules out embedding a foreign operand node id in `node_id.path`. In the added reaches helper edit, `append_population` sorts dependencies with the existing graph-resolving helper, and `candidate` captures graph IDs before borrowing its application mutably, then sorts the appended population dependency by the same structural key. The changes replace both remaining raw `CheckedNodeId` comparisons, preserving FR-322 dependency ordering. Titan's focused conformance on exact c7acd764/QSpec 2f8b9c8b reports 162 passed/3 failures, all three the previously attributed IR-1249 reaches TC-048/233/453 at `OperatorIneligible` population locus; XP-01 and the adverse-pairs test pass. No new finding was identified.
