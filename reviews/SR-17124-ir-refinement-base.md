---
id: SR-17124
title: "Base review of IR FR-061 and TC-062"
type: SpecReview
analysis: base
scope: "quire-contract-ir spec/checked_package/functional/FR-061-admit-refinement-nodes.md and spec/checked_package/matrix/TC-062-refinement-reader-writer.md; QSpec FR-375/FR-322 "
review_set: base
---

## Summary

The draft covers the closed refinement row vocabulary, local and external keys,
identity, dependencies, occurrences, and read/write refusal vectors. No active
AssuranceProfile was found in the IR specification or project configuration.
The two QSpec baseline conflicts were corrected. The IR-local
occurrence pointer and deterministic body traversal findings were fixed in the reviewed PR. The resulting reader-order conflict was aligned to QSpec's four
stages without narrowing acceptance criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | **Resolved upstream.** FR-322 now excludes refinement from both general key stages and its Structural node preimage, and directs the dedicated refinement step to recompute `quire.refinement-node/v1`; FR-375 states the exact preimage and stale-key refusal. | QSpec FR-322 lines 428-430, 475-493, 916-920; QSpec FR-375 lines 474-495; IR FR-061-AC-7, AC-9 |
| FND-002 | high | **Resolved upstream.** FR-322 places the refinement body step after abstraction and before terminal; FR-375 specifies ascending node-id visits and identity, canonical order, header join and row stages. | QSpec FR-322 lines 465-474; QSpec FR-375 lines 474-525; IR FR-061-AC-9 |
| FND-003 | medium | **Resolved.** Behavior and AC-8 now distinguish a present bad entry's pointer from `/source_map` for a missing occurrence after all present entries pass; TC-062 tests both. | IR FR-061-AC-8; TC-062; QSpec FR-322 lines 829-839 |
| FND-004 | medium | **Resolved and aligned.** Behavior and AC-9 now use QSpec's deterministic stages, array and nested-array order, ascending index, and schema-declared entry-member order where QSpec leaves a tie-break; TC-062 tests simultaneous defects. | IR FR-061 Behavior, FR-061-AC-9; TC-062 |
| FND-005 | high | **Resolved.** IR Behavior and AC-9 now follow QSpec's dedicated identity, canonical array-order, `semantic_type`/`abstract`/optional `protocol`/`profile` header, and row-join stages. TC-062 combines defects across stages and repairs each earlier defect to assert the next refusal, preserving every adverse vector. | IR FR-061 Behavior, FR-061-AC-9, TC-062; QSpec FR-375 lines 474-525 |

The canonical node and package identities use required content-addressed
preimages to bind admitted meaning; this review found no unrelated tracking
hash, pin, or copied QSpec fixture. TC-062 addresses each FR-061 criterion
by procedure, and the PR is a planned spec with no implementation trace yet.
