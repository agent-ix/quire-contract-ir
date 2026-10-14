---
id: SR-10892
title: "Base spec review — IR-759 source census at 2cdac57"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/functional/FR-019-rust-library-interface.md, spec/model/matrix/tests.md"
review_set: subset
---

## Summary

Reviewed all edited spec text and links in the PR. The native QSL identity wording is source-backed, and FR-019 lists the new public module and items. The TC-452 matrix line claims full implementation despite its specified full-owner comparison remaining unrun. Ticket: IR-759.

## Verdict

**FAIL** — the evidence status is false for FR-052-AC-7. Keep its full-owner test procedure, but mark it Planned or UNRUN until the owner adapter and oracle exist.

## Scope

Examined FR-052-AC-1 through FR-052-AC-7, TC-452 and FR-019-AC-5. No applicable AssuranceProfile was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-452 is marked implemented although its full-owner enumeration oracle is absent | spec/model/matrix/tests.md:34 |

## Evidence

TC-452 requires each complete envelope to be compared with the full owner output, and says an omitted owner item must prevent a complete envelope. The changed test uses hand-built vectors and checks the opposite omission outcome. The status should reflect the partial API slice while preserving the acceptance criterion and future integration procedure.

## Reviewed units

- FR-052-AC-1 (examined): A default-feature consumer imports and calls the input, constructor, census and row types from `quire_contract_model::source_obligation`; FR-019's public-items inventory lists every exported item.
- FR-052-AC-2 (examined): A typed fixture containing two criteria, three semantic occurrences, one empty module, two production functions and explicit many-to-many assertions preserves every inventory row and exactly the asserted valid relations with their selected-source provenance. Unmapped criteria, occurrences, modules and functions remain visible with typed missing reasons. Reordering equivalent inputs yields the same ordered census; removing one occurrence from a complete IR V2 inventory changes it.
- FR-052-AC-3 (examined): A foreign source, each dangling endpoint, a foreign inventory entry, an exact duplicate, conflicting inventory facts, conflicting assertion relation facts, and conflicting assertion provenance each receive their named deterministic disposition; rejected entries remain inspectable and never enter accepted relations.
- FR-052-AC-4 (examined): Inspection of the public constructor and `SourceObligationInput` signature confirms that the input schema has no result, method, profile, artifact, attempt or run field.
- FR-052-AC-5 (examined): An external default-feature consumer compiles a borrow of `SourceObligationCensus` and its rows from `quire_contract_model::source_obligation`.
- FR-052-AC-6 (examined): Omitting each of the three owner envelopes in turn returns `MissingOwnerInventory` naming that owner and no census. Marking each envelope incomplete returns `IncompleteOwnerInventory` with the supplied typed cause and no census. A fully parsed production module with no function facts is accepted as complete, retains the module row, and differs observably from both outcomes.
- FR-052-AC-7 (examined): Complete IR V2 scope covers every semantic node and source occurrence in the selected V2 graph; complete QSL scope covers every checked `RequirementRecord` and native pending obligation for the selected request, identified by its collision-free native occurrence key or site/kind tuple rather than an IR requirement reference or semantic digest; complete `quire-code-rs` scope covers every selected production module and every function fact, including empty modules. Normal exhaustion is required for `Complete`; QSL exhaustion and `Unfinished` remain distinct typed causes, and an early stop, failed parse/request, unavailable selected module, scope/source mismatch, or item-count mismatch produces a typed error and no census.
- TC-452 (examined): Verify that the neutral model port preserves complete owner inventories and explicit source-author mappings, and returns typed errors rather than a complete-looking census when any owner input is absent or incomplete.
- FR-019-AC-5 (examined): The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists, checked by a public-item inventory that fails on an added or missing item; no item in the table is reachable through a `quire_contract_ir` path.

## Dispositions

Reviewed uncommitted working-tree correction on base 2cdac572; no new commit head yet.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The TC-452 matrix row now says Planned/UNRUN and explicitly states that no owner adapter compares complete envelopes with full owner output. |
