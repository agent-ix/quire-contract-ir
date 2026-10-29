---
id: SR-10890
title: "Code review — IR-759 source-obligation census "
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/source_obligation.rs, crates/quire-contract-model/src/lib.rs, tests/it/source_obligation.rs, tests/it/public_interface.rs, tests/it/cycle_free_model.rs; FR-052-AC-1 through AC-7, FR-019-AC-5"
review_set: subset
---

## Summary

Reviewed the whole branch diff, including the Rust idiom and public API checks. The QSL occurrence-key, obligation-site/kind and exhaustion projections match the current native types. The source completeness claim remains unsupported by an owner-issued enumerator or adapter. Ticket: IR-759. Prior open findings: SR-10874/FND-001 and SR-10875/FND-001.

## Verdict

**FAIL** — FR-052-AC-7 can report a complete census after an owner fact was omitted. Preserve the API slice as partial; mark AC-7 and TC-452 UNRUN or Planned until a native owner oracle exists, or implement and test that oracle in this PR.

## Scope

Examined FR-052-AC-1 through FR-052-AC-7 and FR-019-AC-5; read FR-028, FR-045 and AD-008 for context. No applicable AssuranceProfile was found. The changed workflow files are empty.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A shortened owner inventory can publicly claim Complete and produce a complete-looking census | crates/quire-contract-model/src/source_obligation.rs:373 |

## Evidence

`SourceInventoryEnvelope::new` accepts any `Complete` value and a caller-supplied count. `build_source_obligation_census` checks the count only against that same supplied vector. The omission test at `tests/it/source_obligation.rs:793-802` removes an IR occurrence, reconstructs a `Complete` envelope, and expects `Ok`. There is no owner enumeration adapter in this PR or the inspected local QSL and quire-code-rs checkouts. This is the same still-open defect recorded in the private prior review; this replacement review does not dispose it.

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

Reviewed the uncommitted working-tree correction; no new commit head yet.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | SourceInventoryEnvelope::new still accepts Complete and a count computed from a shortened vector. No owner adapter or full-output oracle exists; FR-052-AC-7 is now explicitly Planned/UNRUN, so this implementation work remains in IR-759 Coding. |

## Dispositions — round 2

Reviewed frozen IR-784 input-only against the original finding.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The public Complete flag, caller count, SourceObligationCensus and build_source_obligation_census are absent. SourceInventoryEnvelope::new retains provisional rows and an optional stop_reason; None expressly does not prove exhaustion. A shortened vector can no longer produce a complete-looking census. |
