---
id: SR-10893
title: "Evidence review — IR-759 native census "
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; FR-052-AC-7, TC-452, tests/it/source_obligation.rs"
review_set: subset
---

## Summary

Reviewed the changed acceptance evidence and its Test designation. TC-452 names the correct full-owner comparison and typed error oracles; the checked-in test does not execute the full-owner comparison. Ticket: IR-759.

## Verdict

**FAIL** — the evidence claim for AC-7 exceeds what the tagged test establishes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The AC-7 Test evidence proves tuple representation but not owner-issued completeness | tests/it/source_obligation.rs:809 |

## Scope

Examined FR-052-AC-7 and TC-452; read FR-052-AC-1 through AC-6 for context. The tests distinguish QSL tuple fields and incomplete reason variants, but they do not enumerate a selected native owner response or refuse an omitted member. No applicable AssuranceProfile was found.

## Reviewed units

- FR-052-AC-1 (context only): A default-feature consumer imports and calls the input, constructor, census and row types from `quire_contract_model::source_obligation`; FR-019's public-items inventory lists every exported item.
- FR-052-AC-2 (context only): A typed fixture containing two criteria, three semantic occurrences, one empty module, two production functions and explicit many-to-many assertions preserves every inventory row and exactly the asserted valid relations with their selected-source provenance. Unmapped criteria, occurrences, modules and functions remain visible with typed missing reasons. Reordering equivalent inputs yields the same ordered census; removing one occurrence from a complete IR V2 inventory changes it.
- FR-052-AC-3 (context only): A foreign source, each dangling endpoint, a foreign inventory entry, an exact duplicate, conflicting inventory facts, conflicting assertion relation facts, and conflicting assertion provenance each receive their named deterministic disposition; rejected entries remain inspectable and never enter accepted relations.
- FR-052-AC-4 (context only): Inspection of the public constructor and `SourceObligationInput` signature confirms that the input schema has no result, method, profile, artifact, attempt or run field.
- FR-052-AC-5 (context only): An external default-feature consumer compiles a borrow of `SourceObligationCensus` and its rows from `quire_contract_model::source_obligation`.
- FR-052-AC-6 (context only): Omitting each of the three owner envelopes in turn returns `MissingOwnerInventory` naming that owner and no census. Marking each envelope incomplete returns `IncompleteOwnerInventory` with the supplied typed cause and no census. A fully parsed production module with no function facts is accepted as complete, retains the module row, and differs observably from both outcomes.
- FR-052-AC-7 (examined): Complete IR V2 scope covers every semantic node and source occurrence in the selected V2 graph; complete QSL scope covers every checked `RequirementRecord` and native pending obligation for the selected request, identified by its collision-free native occurrence key or site/kind tuple rather than an IR requirement reference or semantic digest; complete `quire-code-rs` scope covers every selected production module and every function fact, including empty modules. Normal exhaustion is required for `Complete`; QSL exhaustion and `Unfinished` remain distinct typed causes, and an early stop, failed parse/request, unavailable selected module, scope/source mismatch, or item-count mismatch produces a typed error and no census.
- TC-452 (examined): Verify that the neutral model port preserves complete owner inventories and explicit source-author mappings, and returns typed errors rather than a complete-looking census when any owner input is absent or incomplete.

## Dispositions

Reviewed the uncommitted working-tree correction; no new commit head yet.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The synthetic tests no longer claim FR-052-AC-7 or TC-452 trace; both artifacts explicitly state that the full owner-output comparison is unrun. The underlying AC remains planned. |

## Dispositions — round 2

Reviewed frozen IR-784 input-only against the original finding.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The only new trace bindings are AC-4 and AC-8. The public API test checks absence of Complete, caller count, census and builder; FR-052 and TC-452 retain the future full owner-output oracle as Planned/UNRUN. |
