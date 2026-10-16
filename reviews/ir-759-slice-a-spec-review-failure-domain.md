---
id: SR-11742
title: "IR-759 Slice A failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@7e0e95ba8bc513a5bb0d6b799f8b70412f06a8c8 (PR #411); spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md"
review_set: subset
---

## Summary

Ticket: IR-759 (Slice A). This review asks which census failure modes outcome-backed input can still produce now that the census consumes `SourceInventoryOutcome` rather than raw envelopes. In particular, can duplicate and conflicting inventory entries still reach the census, and does the spec say so. Checked against `SourceInventoryOutcome::from_enumerator` (crates/quire-contract-model/src/source_obligation.rs:562-610), `SourceInventoryRow::new` (:351-357) and `SourceInventoryIdentity` (:305-325).

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-052 census dedup paragraph | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | Exact repeated inventory entries shall collapse to one row. When one typed identity is repeated with differing inventory facts, each conflicting entry shall remain inspectable with `ConflictingIdentity`; none shall be selected by first-wins order. |
| FR-052 dispositions | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | Inventory disposition shall distinguish at least `Mapped`, `ZeroMappings`, and `ConflictingIdentity`. |
| FR-052-AC-3 | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | A foreign source, each dangling endpoint, an exact duplicate, conflicting inventory facts, conflicting assertion relation facts, and conflicting assertion provenance each receive their named deterministic disposition; rejected entries remain inspectable and never enter accepted relations. |
| FR-052 observed-count rule | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | The outcome's observed row count shall equal the number of retained rows; `SourceInventoryOutcomeError::ObservedCountMismatch` owns that rule and refuses outcome construction, so the census never receives a count mismatch. |
| TC-452 duplicate/conflict procedure | examined | spec/model/matrix/TC-452-source-obligation-correspondence.md | exercise unmapped entities, foreign-source and dangling assertion endpoints, exact duplicate inputs, conflicting inventory facts, conflicting assertion relation facts, and conflicting assertion provenance. |
| TC-452 duplicate/conflict expected | examined | spec/model/matrix/TC-452-source-obligation-correspondence.md | Exact duplicates collapse, conflicting inventory entries remain inspectable, equivalent input permutations produce equal ordered censuses. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The spec does not say where duplicate and conflicting inventory entries come from now that the census only consumes outcomes. It also leaves "typed identity" undefined, so AC-3's "conflicting inventory facts" case is reachable only through one variant. `from_enumerator` keeps every emitted row and does not deduplicate (source_obligation.rs:574-584). So one outcome can carry exact duplicate rows, and its `observed_count` counts them. The census then collapses them, so the census row count and the outcome's validated count differ, and FR-052 does not say that is expected. A row is (owner taken from identity, source forced equal to the outcome's source, identity). Two rows with equal `SourceInventoryIdentity` are therefore always identical. A "conflict" exists only if the census keys on part of the identity, and in practice that is `SemanticNode { identity, semantic }`: the same `CheckedNodeId` with a different `CheckedSemanticId`. Each owner position holds one outcome, and identities are owner-specific, so there is no cross-outcome conflict. FR-052 :287-291 should name the identity key per variant, state that duplicates and conflicts arrive inside one outcome, and state how collapse relates to `observed_count`. Otherwise AC-3's conflicting-inventory case may be untestable for every variant except `SemanticNode`. | spec/model/functional/FR-052-source-obligation-correspondence.md:287-291, 324 |

## Verdict

CHANGES REQUESTED for FND-001, an unstated failure-mode boundary that the move to outcome-only input creates.

The other failure modes now have exactly one owning rule each:
- A foreign row produces `RowSourceMismatch`.
- A foreign outcome produces `InventorySourceMismatch`.
- A count mismatch produces `ObservedCountMismatch`.
- A wrong-owner row produces `RowOwnerMismatch`.
- A wrong-owner stop produces `StopOwnerMismatch`.
- An absent or incomplete outcome produces `MissingOwnerInventory` or `IncompleteOwnerInventory`.

Assertion-side duplicates and conflicts (`ConflictingRelation`, `ConflictingProvenance`) do not depend on outcomes and remain producible as written.
