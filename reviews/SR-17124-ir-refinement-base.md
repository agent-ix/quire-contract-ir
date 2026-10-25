---
id: SR-17124
title: "Base review of IR FR-061 and TC-062"
type: SpecReview
analysis: base
scope: "quire-contract-ir spec/checked_package/functional/FR-061-admit-refinement-nodes.md and spec/checked_package/matrix/TC-062-refinement-reader-writer.md at de741312; QSpec FR-375/FR-322 at 4c1266d9"
review_set: base
---

## Summary

The draft covers the closed refinement row vocabulary, local and external keys,
identity, dependencies, occurrences, and read/write refusal vectors. No active
AssuranceProfile was found in the IR specification or project configuration.
Two QSpec baseline conflicts block a coherent reader implementation. The
IR-local occurrence pointer and deterministic body traversal findings were
fixed at de741312 without narrowing their acceptance criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | **Upstream QSpec blocker:** FR-375 assigns `relation`/`refinement` a `quire.refinement-node/v1` key, but FR-322's derived-shape key stage and Structural node preimage still select `quire.structural-node/v1` for every non-application, non-nominal node except `value`/`union_value` and `correspondence`/`abstraction_relation`. An admitted refinement would fail FR-322's earlier stale-key check. QSpec must exempt refinement from structural re-keying and explicitly check its special preimage. This is not a defect introduced by IR #560. | QSpec FR-375 lines 375-397; QSpec FR-322 lines 473-487, 908-927; IR FR-061-AC-7, AC-9 |
| FND-002 | high | **Upstream QSpec blocker:** FR-322's ordered Body steps omit a refinement step. FR-375 defines body and meaning joins but their place relative to other body steps and operation checks is unspecified. IR FR-061-AC-9 promises an earlier-stage and cross-node first-defect order that cannot be derived completely from the merged QSpec ordering. Add the refinement step in QSpec, retaining FR-061's full acceptance scope. This is not a defect introduced by IR #560. | QSpec FR-322 lines 458-472; IR FR-061-AC-9 |
| FND-003 | medium | **Resolved at de741312.** Behavior and AC-8 now distinguish a present bad entry's pointer from `/source_map` for a missing occurrence after all present entries pass; TC-062 tests both. | IR FR-061-AC-8; TC-062; QSpec FR-322 lines 829-839 |
| FND-004 | medium | **Resolved at de741312.** Behavior and AC-9 now fix `term`, `abstract`, `profile`, six arrays, `protocol`, `abstract_internal`, then array index and schema-declared entry-member order; TC-062 tests simultaneous defects at each level. The existing body/offending-entry locus applies to the selected first defect. | IR FR-061 Behavior, “Occurrences and reader order”; FR-061-AC-9; TC-062 |

The canonical node and package identities use required content-addressed
preimages to bind admitted meaning; this review found no unrelated tracking
hash, pin, or copied QSpec fixture. TC-062 addresses each FR-061 criterion
by procedure, and the PR is a planned spec with no implementation trace yet.
