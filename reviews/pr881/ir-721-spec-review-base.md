---
id: SR-20261104020001
title: "IR-721 PR 881 spec-review-base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 881; FR-357 witnessed-subset amendment"
review_set: subset
---

## Summary

Independent scoped spec-review-base review of FR-357 at PR #881 head 327026b2. Findings below block an unqualified PASS.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-7 does not pin how full-emission row slices are found or charged; require an unrelated-row control and first-locus accounting for indexing | FR-357-AC-7 |

## Examined Scope

- role: context_only; id: FR-357-AC-1; excerpt: PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A strictly admitted generated monitor root with four entries yields one complete selected type/declaration closure; records, tuples, collections, unions, enum cases, object fields and source units retain original identities, producer order, presence, shape and member/payload types sufficient to build the canonical `TypeEnvironment`. Shared references do not duplicate declarations.
- role: context_only; id: FR-357-AC-2; excerpt: PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A missing, foreign, duplicate, conflicting or stale-after-rewrite producer row, unresolved reachable declaration, unrepresentable type or native nominal/Exact refusal returns its typed cause and first locus, with no partial view or invented identity.
- role: context_only; id: FR-357-AC-3; excerpt: PLANNED/UNRUN (IR-721). A large selected closure amid unrelated graph nodes and unsupported declarations visits only its reachable nodes/edges and charged identity/value material using an iterative worklist. Equal inputs and allowances give equal complete results or first work stops; one below the required work refuses without a partial view.
- role: context_only; id: FR-357-AC-4; excerpt: PLANNED/UNRUN (IR-721). Public API and dependency inspection finds a bounded IR projection over admitted package facts and original QSL witnesses; it finds no IR-owned `TypeEnvironment`, second plan/evaluator, RT dependency, source reparse or witness fabrication.
- role: examined; id: FR-357-AC-5; excerpt: PLANNED/UNRUN (IR-721 subset). Given one admitted selected root and its final QSL emission's model declaration/field rows and translated equality rows, the public bounded IR read returns only reached model declarations/fields and opaque FR-353 equality decisions from the four entries, preserving selected document, final structural node, original identities, field order/presence/type, exact equality occurrence and producer facts. Shared references appear once. The result is a subset, not a complete generated-source closure or `TypeEnvironment`.
- role: examined; id: FR-357-AC-6; excerpt: PLANNED/UNRUN (IR-721 subset). A missing, foreign, duplicate, conflicting or pre-rewrite declaration, field, equality application or fact row; wrong document, form, owner, role, occurrence or source-map correspondence; unresolved reached dependency; reached unsupported form; and native FR-353 refusal each produce a named typed outcome with first native locus and no partial view or invented identity. An unrelated unsupported declaration or missing witness outside the reached subset leaves the same successful result.
- role: examined; id: FR-357-AC-7; excerpt: PLANNED/UNRUN (IR-721 subset). A selected root amid unrelated graph nodes visits only reached nodes, edges and required rows with an iterative worklist; the exact work allowance succeeds and one less refuses with attempted work and locus. Equal inputs and allowance, including a package clone, yield equal results or first stops.
- role: examined; id: FR-357-AC-8; excerpt: PLANNED/UNRUN (IR-721 subset). Public API and dependency inspection shows the read takes an admitted package, selected root, IR-owned row slices and a work allowance, performs no QSL/RT dependency or source recheck, and returns neither an IR environment nor `ProgramParts`. A caller adapts QSL-owned equality emission rows without changing their values; IR checks admitted joins but does not claim to authenticate arbitrary caller-created Exact payloads.

## Analysis

AC-7 says only reached nodes, edges and required rows are visited, but the API takes full-emission row slices. Locating a needed row in those slices may require inspecting unrelated rows or building a full index. Specify whether that preprocessing is charged and include an unrelated-row inflation test; preserve deterministic first stops. This is distinct from the whole-graph exclusion and cannot be assumed from it.

## Verification and Limits

Reviewed the exact one-file PR diff and QSL main emission API by source inspection. Scoped Quire validate and properties completed; no Rust build, Cargo gate or full suite was run. No applicable AssuranceProfile was found under spec/. Review artifacts are outside the PR worktree.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | AC-5 requires real PlanRoot.emission joins but does not assign cross-repo test ownership; IR model cannot import QSL, and the existing QSL PlanRoot test covers only selected declaration rows, not an admitted equality join or subset refusal | FR-357-AC-5 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-7 now charges every inspected full-emission row and index entry, including unrelated rows, and tests unrelated-row inflation plus first uncharged row or graph locus. |
| FND-002 | still-open | Name a QSL-owned or other cross-repo integration test for the real PlanRoot.emission join and typed subset refusal, distinct from the local supported-only IR fixture; current AC-5 does not assign a feasible verifier. |
| FND-002 | fixed | AC-5 is now explicitly an IR-owned supported-only fixture; new AC-9 assigns the real final PlanRoot.emission QSL-to-IR joins and first unsupported-source refusal to quire-integration, Planned/Unrun after the IR subset API lands. |

## Fix-round evidence

At PR #881 head 932c8f68, AC-7 states: PLANNED/UNRUN (IR-721 subset). A selected root amid unrelated graph nodes visits only reached graph nodes and edges with an iterative worklist. Locating reached witnesses in full-emission row slices charges every inspected row and index entry, including unrelated rows; adding unrelated rows either raises required work or causes a charged first-row-locus stop at the former allowance, without scanning unrelated graph nodes. The exact allowance succeeds and one less refuses with attempted work and first uncharged locus. Equal inputs and allowance, including a package clone, yield equal results or first stops.

## Fix-round evidence (disposition pass 2)

At PR #881 head acd77e73, AC-5 names an IR-owned fixture and no longer requires real QSL emission. AC-9 separately states: PLANNED/UNRUN (cross-repo `quire-integration` test after the IR subset API lands). A `quire-integration` test obtains a real final `PlanRoot.emission` through QSL, proves its selected declaration/field rows join through IR `SelectedExactIdentities::from_stage_rows`, translates its equality rows without changing their values and exercises IR `admitted_equality` to obtain a complete decision or its native typed missing-witness result, then proves the IR subset read refuses at the first reached source-owned form lacking QSL-1631 witness. It does not claim a complete generated-source closure, `TypeEnvironment` or RT `ProgramParts`, and it adds no QSL dependency to IR.

## Review status after disposition pass 2

PASS at PR #881 head acd77e73. The latest disposition for FND-001 and FND-002 is fixed. AC-5 and AC-9 remain Planned/Unrun until their respective implementation and integration evidence lands.
