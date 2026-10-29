---
id: SR-10891
title: "Gap analysis — IR-759 source census "
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-052, TC-452, tests/it/source_obligation.rs, source_obligation.rs, FR-019 public API"
review_set: subset
---

## Summary

Compared FR-052 criteria and TC-452 with tagged tests and the implementation. The FR-052-AC-7 tags bind to synthetic typed fixtures but do not verify full owner output or normal exhaustion. Ticket: IR-759. Prior open finding: SR-10875/FND-001.

## Verdict

**FAIL** — the AC-7 binding is hollow for owner completeness. The other new criteria have targeted assertions; AC-7 must remain unclaimed until a real owner adapter and full-output oracle are exercised.

## Coverage

Plan completion: not assessed. Examined FR-052-AC-1 through FR-052-AC-7 and FR-019-AC-5. `quire matrix --strict` ran and reported 53 unrelated untagged criteria plus one tagged only by an ignored test across the full repository; FR-052-AC-7 appears tagged, but the tag overclaims its test. Semantic comparison here is limited to this changed PR diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-7's tests self-certify Complete instead of comparing every owner inventory against native full output | tests/it/source_obligation.rs:101 |

## Evidence

`make_envelope` derives `item_count` from `rows.len()` and sets `Complete` without enumeration evidence. Its omission case removes a selected IR occurrence and still expects a successful census. No IR/QSL/quire-code-rs adapter or independent owner-output comparison appears in the branch diff. Remove AC-7 trace and the TC-452 implemented claim for this slice, or add native producer evidence and the omission refusal required by TC-452.

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
| FND-001 | deferred | The false AC-7 test binding was removed and FR-052-AC-7/TC-452 now say Planned/UNRUN. Full-owner comparison and omission refusal remain to be implemented under IR-759 before that criterion can pass. |

## Dispositions — round 2

Reviewed frozen IR-784 input-only against the original finding.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The self-certified Complete test and its AC-7 binding are gone. Only AC-4 and AC-8 have provisional tests; FR-052-AC-1 through AC-3 and AC-5 through AC-7, and TC-452, explicitly remain Planned/UNRUN. The full owner comparison is still required for IR-759. |
