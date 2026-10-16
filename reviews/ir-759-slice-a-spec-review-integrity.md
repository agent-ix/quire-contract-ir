---
id: SR-11741
title: "IR-759 Slice A integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@7e0e95ba8bc513a5bb0d6b799f8b70412f06a8c8 (PR #411); spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-759 (Slice A). This review checks the edited statements, acceptance criteria and TC-452 procedure and expected results for consistency, completeness and one rule per condition. Each "envelope" sentence was read against `SourceInventoryOutcome::from_enumerator` (crates/quire-contract-model/src/source_obligation.rs:562-610).

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-052 envelope sentences | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | :41 the census does not accept raw `SourceInventoryEnvelope` values; :80-84 provisional port; :182 provisional envelope; :245-251 provisional port; AC-8 and AC-10 provisional envelope. |
| FR-052 census refusal paragraph | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | If the Quire binding or any owner outcome is absent, incomplete, or has a mismatched scope or source, `build_source_obligation_census` shall return a typed error naming that owner and shall return no census. An item-count, row-owner, row-source or stop-owner inconsistency is refused earlier by `SourceInventoryOutcomeError` and never reaches the census. |
| FR-052 SourceCorrespondenceError | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | `SourceCorrespondenceError` shall expose distinct variants `MissingOwnerInventory`, `IncompleteOwnerInventory`, `InventorySourceMismatch`, and `InventoryScopeMismatch` ... It has no item-count variant. |
| FR-052 foreign-row paragraph | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | A foreign-source inventory row is refused at the outcome boundary with `SourceInventoryOutcomeError::RowSourceMismatch` and never reaches the census; an outcome whose own source differs from the selected source returns `InventorySourceMismatch`. |
| FR-052-AC-3 | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | A foreign source, each dangling endpoint, an exact duplicate, conflicting inventory facts, conflicting assertion relation facts, and conflicting assertion provenance each receive their named deterministic disposition ... A foreign-source inventory row is refused at the outcome boundary, not by the census (FR-052-AC-13). |
| FR-052-AC-6 | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | Omitting each of the four owner outcomes in turn returns `MissingOwnerInventory` naming that owner and no census. Marking each outcome `Incomplete` returns `IncompleteOwnerInventory` with the supplied typed cause and no census. |
| FR-052-AC-7 | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | ... or scope/source mismatch produce a typed incomplete or error result and no census; an observed-count mismatch is refused as `SourceInventoryOutcomeError::ObservedCountMismatch` and issues no outcome. |
| FR-052-AC-2 | examined | spec/model/functional/FR-052-source-obligation-correspondence.md | ... removing one occurrence from a complete IR V2 outcome changes it. |
| TC-452 Test Procedure (census errors) | examined | spec/model/matrix/TC-452-source-obligation-correspondence.md | Supply an outcome source mismatch and a wrong owner/scope position to the census. Separately supply an observed count that differs from the retained rows, and a foreign-source row, to the outcome boundary. |
| TC-452 Expected Results (census errors) | examined | spec/model/matrix/TC-452-source-obligation-correspondence.md | A source mismatch returns `InventorySourceMismatch`; a wrong owner/scope returns `InventoryScopeMismatch`, each with no census. A count mismatch is refused by `SourceInventoryOutcomeError::ObservedCountMismatch` and a foreign row by `RowSourceMismatch`, each with no outcome. |
| FR-052-AC-13 | context_only | spec/model/functional/FR-052-source-obligation-correspondence.md | Mismatched observed count, owner/scope pair, row owner, row source, or stop-cause owner returns the corresponding typed structural error and no outcome. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | "Owner/scope" now names two different conditions with two different errors, and the text does not keep them apart. AC-13 and TC-452:115 send a mismatched owner/scope *pair* inside one enumerator to `SourceInventoryOutcomeError::OwnerScopeMismatch`. TC-452 Expected :162-163 says "a wrong owner/scope returns `InventoryScopeMismatch`". FR-052's list of conditions "refused earlier" (:260-262) names item-count, row-owner, row-source and stop-owner but leaves out the owner/scope pair. A reader can therefore expect the census to see a self-inconsistent owner/scope outcome, which `from_enumerator` (:567-568) makes unconstructible. The census condition is really "a well-formed outcome supplied in the wrong owner position" (TC-452:135-136 says "position"). Expected Results and FR :258-262 should say that. | spec/model/matrix/TC-452-source-obligation-correspondence.md:162-163; spec/model/functional/FR-052-source-obligation-correspondence.md:258-262 |
| FND-002 | low | AC-6 says "Marking each outcome `Incomplete`", and TC-452 says "a partial outcome marked `Incomplete`" (:130) and "an outcome marked `Complete`" (:133). This is caller-flag wording. FR-052:157-158 says completeness "is evidence from enumeration, not a caller-authored boolean", and the private fields plus `from_enumerator` make an outcome impossible to mark. The criterion should say the outcome is issued `Incomplete` by an enumerator's `Err(cause)` terminal event. | spec/model/functional/FR-052-source-obligation-correspondence.md:327 |

## Verdict

ACCEPTABLE WITH LOW FINDINGS. The core change is internally consistent:
- Every remaining "envelope" sentence describes the provisional port only: :41, :80-84, :182, :245-251, AC-8, AC-10, and TC-452:85, :104-105 and :179.
- The two rules that moved now have exactly one owner each. `InventoryItemCountMismatch` is gone from FR-052, TC-452 and the error list, and only `ObservedCountMismatch` (source_obligation.rs:596-600) applies. `ForeignInventory` is gone, and only `RowSourceMismatch` (:578-579) applies. AC-3 and AC-7 point to the outcome boundary for both.
- The remaining dispositions (Mapped, ZeroMappings, ConflictingIdentity) and rejection reasons agree with AC-2, 3, 5, 6 and 7 and with TC-452 Expected Results.
- Whether the duplicate and conflict cases are reachable is covered separately in SR-11742.

## Dispositions

Round 1. Reviewed head 6f3115000fac784783339a2140794883583e1baf, diff `git diff 7e0e95b..6f31150`.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-052:264-268 now adds the owner/scope pair to the conditions refused earlier by `SourceInventoryOutcomeError`, and states that the census sees only a well-formed outcome in the wrong owner position (`InventoryScopeMismatch`). TC-452 Procedure (:135-138) and Expected Results (:163-165) separate the two cases: wrong position gives `InventoryScopeMismatch` with no census, and a pair mismatch gives `OwnerScopeMismatch` with no outcome. This matches `from_enumerator` (source_obligation.rs:567-568). |
| FND-002 | fixed | AC-6 now reads "an outcome whose enumerator ended with a typed `Err` (issued `Incomplete`)". TC-452:130-133 says "issued `Incomplete`" and "issued `Complete` after the enumerator's `Ok(None)`". No caller-flag wording remains. |
