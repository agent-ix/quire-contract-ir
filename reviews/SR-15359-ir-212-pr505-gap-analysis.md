---
id: SR-15359
title: "IR-212 PR 505 Gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; contract_package.rs and two public integration-test files; FR-035-AC-19..21"
review_set: subset
---

## Summary

Ticket: IR-212. Independent nonauthor review of PR 505's three changed Rust files at exact. Scope is FR-035-AC-19 through AC-21.

## Verdict

**PASS**. The computed Test Matrix marks AC-19, AC-20, and AC-21 tagged. The new public tests independently compare producer and strict-reader kind values to admitted source kinds, exercise recursion-group order and selected-dependency exclusion, compare unsupported tags to record dispositions by accounting index, and check malformed kind/tag refusal and incomplete admission. The changed API is owned by FR-035 and no stub was found. Plan completion: not assessed.

## Coverage

- FR-035-AC-19: represented local kinds survive canonical ordering and are visible on root and reached nodes; absent/unrepresented and selected dependency keys have no local kind.
- FR-035-AC-20: unsupported typed tags are tied to feature-accounting index, including absent requested nodes; other dispositions and out-of-range return no tag.
- FR-035-AC-21: retained vectors are outside the six-field wire and admission barrier; malformed input preserves reader refusal; limit stops produce no admitted view.
- Code and tests inspected: `crates/quire-contract-model/src/checked_package/v2/contract_package.rs`, `tests/it/checked_package_v2_recursive_leaves.rs`, `tests/it/complete_v1_contract_package.rs`.
- Matrix: AC-19, AC-20, AC-21 all `tagged` in computed `quire matrix --scope . --format tsv`.
- Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
