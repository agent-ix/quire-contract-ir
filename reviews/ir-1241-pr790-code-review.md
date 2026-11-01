---
id: SR-202610310148
title: "Code review of IR-1241 QSpec conformance harness PR 790"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 790 frozen head 844ad5d1; tests/conformance_qspec/{extension,hybrid,main,model_tuple_union,node_index,protocol_vectors,reaches_population,selection_vectors,timed_model}.rs and tests/it/checked_package_v2_owners.rs; IR-1241"
review_set: subset
---

## Summary

Ticket: IR-1241. Reviewed all nine changed Rust conformance-harness files against the repository's Rust conventions, QSpec FR-322 graph-local index and source-key rules, and the recorded failures from the current QSpec vectors. No applicable AssuranceProfile was found under `spec/`. The helpers resolve structural node identities for ordering and rewrite graph-local indexes when constructing a smaller graph; the tests retain exact admission and refusal assertions.

## Verdict

PASS. No defect was found in the PR diff at 844ad5d1. The five focused conformance failures are assigned to IR-1247 and IR-1249; the separate TC-229 node-57 refusal remains with IR-1251. This verdict does not claim those external reader obligations are complete. The lead owns the final aggregate gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope and evidence

Examined FR-035-AC-9 and FR-038-AC-1, with QSpec FR-322-AC-73, AC-95, AC-96, and AC-270 as external contract context. The changed tests call the public reader and compare its retained wire, structural identities, source-map occurrences, error codes, and loci; the independent fixture helper computes expected structural order from the QSpec graph's index segments. The changed files add no copied QSpec artifact, dependency, unsafe code, or CI workflow change. Focused evidence recorded by the coder: format and test Clippy pass; QSpec conformance at 6a021ef8 has 159 passed/5 failed. The remaining five failures and their owning tickets are stated in PR 790.

## Recheck at 844ad5d1

The added owner test now reports the actual refusal. After constructing each TC-229 population case, it sorts dependencies by `resolved_fixture_id`, which resolves graph-local node-index segments against the final owner-projected structural identities. It then refreshes application identity and source-map rows. The focused TC-229 log shows both formerly failing paths pass the node-45 dependency check and reach the same node-57 `MissingDeclaration/MissingSelection` refusal, already present on the exact base and assigned to IR-1251. `git diff --check` passed. No new finding was identified.
