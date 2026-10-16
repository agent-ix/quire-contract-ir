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

## Dispositions

Round 1. Reviewed head 6f3115000fac784783339a2140794883583e1baf, diff `git diff 7e0e95b..6f31150`.

The author observed that `SourceInventoryIdentity` derives Eq/Ord over the whole variant (source_obligation.rs:304), so two `SemanticNode` rows that share a node id but differ in semantic id compare unequal in code. I re-checked the per-variant key table in FR-052:295-308 against the identity types:
- `RequirementRecord`: `SourceRequirementOccurrenceId` is node, role and ordinal. That is QSL's own map key, so keying on the whole identity is right.
- `SemanticOccurrence`: `CheckedOccurrence` is role and ordinal, scoped to its node (checked_package/shared.rs:524-529). Keying on the whole node-plus-occurrence identity is right.
- `PendingObligation`: FR-052:58-61 already makes site and kind together the identity.
- `ProductionModule` and `ProductionFunction`: the function is module plus name.
- The row's owner is derived from its identity, and its source is forced equal to the outcome source (:574-579). So `CheckedSemanticId` is the only fact that is not part of the key.

The statement is sound as a requirement on the census, which does not exist yet. The derived Eq on the identity enum only governs `SourceInventoryRow` equality; it does not say what the future census groups on. FR-052 now requires the census to group `SemanticNode` on `CheckedNodeId`, which makes `ConflictingIdentity` reachable, and AC-3 and TC-452:143 name exactly that fixture. A caveat for later: the "criterion identity" key and the "only fact beyond the key" sentence describe today's enum. IR-795's `SourceAuthoredCriterionId`, and any Quire provenance a PLAT-1165 row carries, will have to restate them. This is forward work already owned by IR-795, not a defect now.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-052:295-308 now covers all four points. Duplicates and conflicts arise only within one outcome. The inventory key is named for each `SourceInventoryIdentity` variant. `CheckedSemanticId` on `SemanticNode` is the one conflicting fact. Census collapse leaves the outcome's rows and `observed_count` unchanged, so a census may hold fewer rows than the count. FR-052:162-165 states that outcomes keep duplicates and count them. AC-3 and TC-452:143 name the reachable conflict fixture. |
