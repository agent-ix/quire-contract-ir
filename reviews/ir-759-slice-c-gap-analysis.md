---
id: SR-11817
title: "IR-759 Slice C source-obligation census gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #419 diff origin/main...: FR-052, FR-019, TC-452, spec/model/matrix/tests.md, crates/quire-contract-model/src/source_obligation.rs, tests/it/source_obligation.rs"
review_set: subset
---

# IR-759 Slice C source-obligation census gap analysis

## Summary

Ticket: IR-759. PR: quire-contract-ir#419. Planless gap analysis of the census builder diff. It compares FR-052's construction rules and AC-15 and AC-17 to AC-23 with their tagged tests and the code. Plan completion: not assessed.

Measured with `quire coverage --scope. --strict` (quire 0.36.2, engine 0.50.2), each run in its own checkout:

- merge-: 55 unbacked rows, 0 contradicted.
- origin/main: 55 unbacked rows, 0 contradicted.
-: 47 unbacked rows, 0 contradicted.
- head merged with origin/main: 47 unbacked rows, 0 contradicted.

The author's claim of 55 -> 47 with 0 contradicted is confirmed. The eight rows that became backed are FR-052-AC-15 and AC-17 through AC-23. AC-8 was already backed and is now deleted. FR-052-AC-1, 2, 3, 5, 6, 7, 9, 10, 11 and 16 are still unbacked and still Planned/UNRUN. `quire validate` is clean over `spec/**` and `reviews/**`.

## Verdict

**NOT MERGEABLE** until FND-001 is fixed (shared with SR-11816 FND-001). The other findings are low.

What holds:

- **AC-8 retirement (ADR-0056).** The FR-052-AC-8 row is deleted and its ID is not reused. The status text reads "FR-052-AC-8 is retired and its ID is not reused (ADR-0056)" and names AC-17 as the successor. TC-452's criteria in `spec/model/matrix/tests.md` drop AC-8. No AC-8 trace tag remains, and no orphan FR-052-AC-8 reference remains outside the retirement notes.
- **Status truthfulness.** Only AC-15 and AC-17 to AC-23 change state. AC-4 and AC-12 to AC-14 were already Implemented. The status text, TC-452's Execution Status and the TC-452 matrix row all keep AC-1, 2, 3, 5, 6, 7, 9, 10, 11 and 16 as Planned/UNRUN, and claim no full-owner or matched-source census.
- **IR readings stay labelled.** The conflicting-binding definition, the QSL-row disposition and the AC-17 and AC-18 expectations are still marked as IR readings pending owner confirmation, and the Open questions are unchanged.
- **No leakage.** The diff contains no IR V2 owner adapter, no QSL or `quire-code-rs` adapter, and no Slice D material. The model gains no new dependency (only `std::collections`).
- **Code with no owner.** Every new public item and function is owned by FR-052 and listed in FR-019:191.
- **Test oracle.** AC-17 compares against literal expected rows. The helpers only build literal values and do not re-run the census rules. The QSL outcome holds one `RequirementRecord` and one `PendingObligation` row, both expected as `ZeroMappings(NoAuthoredMapping)`. AC-18 tests all five adjacent pairs named in the criterion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-17 is marked Implemented, but its fixture does not have the "AC-2 fixture shape" the criterion requires: it has no many-to-many assertions. The evidence is therefore narrower than the criterion. Distinct relations that share criterion and occurrence but differ in function are never built, and a mutant that collapses them passes (SR-11816 FND-001). Either extend the fixture or narrow the status claim. | spec/model/functional/FR-052-source-obligation-correspondence.md:470, 478-484; tests/it/source_obligation.rs:523-618, 651-798 |
| FND-002 | low | Three construction rules have no backing test in any criterion this PR flips: step 1 (exact duplicate assertions collapse), the provenance-source branch of step 3's `ForeignSource`, and step 3's `DanglingCriterion`-before-`DanglingOccurrence` order (plus the rejection ordering rule). Each rule's mutant survives. These rules are FR-052 Behavior, and the only criterion that names them is AC-3, which is Planned. Add tests, or say in the status that AC-3 still owns them. | spec/model/functional/FR-052-source-obligation-correspondence.md:379-397, 423-424, 457 |
| FND-003 | low | Editorial problems from the rewrite. FR-019:191's `source_obligation` type list is no longer alphabetical: `SourceCensusInventoryRow` through `SourceObligationRow` are inserted between `SourceInventoryCompleteness` and `SourceInventoryIdentity`. The FR-052 status paragraph runs into `## Dependencies` with no blank line. The rewritten status also drops the explicit guard that passing AC-12, AC-13 and AC-17 to AC-23 "shall not change" the status of AC-1 to 3, 5 to 7 and 9 to 11. The Planned list still says this, but the guard sentence no longer does. | spec/model/functional/FR-019-rust-library-interface.md:191; spec/model/functional/FR-052-source-obligation-correspondence.md:478-485 |

## Examined scope

```yaml
scope:
- {id: FR-052-AC-8, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "FR-052-AC-8 is retired and its ID is not reused (ADR-0056). FR-052-AC-17 succeeds it."}
- {id: FR-052-AC-15, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "A missing, foreign or conflicting binding refuses a complete census with distinct typed causes"}
- {id: FR-052-AC-17, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "where the outcomes hold the AC-2 fixture shape plus a QSL outcome with at least one `RequirementRecord` row and one `PendingObligation` row... a census equal to a hand-written expected census"}
- {id: FR-052-AC-18, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "The tested pairs are a foreign binding with an incomplete Quire outcome... and an incomplete QSL outcome with a missing `quire-code-rs` outcome"}
- {id: FR-052-AC-19, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "an assertion that is both foreign and dangling is rejected only with `ForeignSource`"}
- {id: FR-052-AC-20, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "permuting the assertion order and the row order of each outcome yields a census equal to the unpermuted one."}
- {id: FR-052-AC-21, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "both in the wrong position and incomplete yields `InventoryScopeMismatch`"}
- {id: FR-052-AC-22, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "two assertion IDs stating one relation with equal provenance yield exactly one accepted row"}
- {id: FR-052-AC-23, role: examined, path: spec/model/functional/FR-052-source-obligation-correspondence.md, excerpt: "each orphan or conflicting row stays in the census."}
- {id: TC-452, role: examined, path: spec/model/matrix/TC-452-source-obligation-correspondence.md, excerpt: "Mixed. The neutral outcome and synthetic census checks for FR-052-AC-4, AC-12 through AC-15, and AC-17 through AC-23 are Implemented and traced"}
```

## Dispositions

Round 1, checked against fix. `quire coverage --scope. --strict` still reports 47 unbacked rows and 0 contradicted. `quire validate` is clean over `spec/**` and `reviews/**`. No new findings.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The AC-17 fixture now holds the many-to-many shape: occurrence 0 maps to two functions, and `verify` is reached from occurrences 0 and 1. The hand-written census is extended to match, and the relation-key mutant is killed. |
| FND-002 | fixed | Both options were taken. A new tagged test covers step-1 duplicate collapse, provenance-only `ForeignSource`, the dangling precedence and the rejection ordering, and kills all four mutants. The FR-052 status now names that evidence and says AC-3 stays Planned/UNRUN. |
| FND-003 | fixed | FR-019:191 is alphabetical again. The FR-052 status has a blank line before `## Dependencies` and restores the guard that the synthetic AC-17 through AC-23 checks do not change the status of AC-1 through AC-3, AC-5 through AC-7 or AC-9 through AC-11. |
