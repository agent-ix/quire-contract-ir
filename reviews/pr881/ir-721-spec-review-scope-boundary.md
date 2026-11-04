---
id: SR-20261104020004
title: "IR-721 PR 881 spec-review-scope-boundary review"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir; PR 881; FR-357 witnessed-subset amendment"
review_set: subset
---

## Summary

Independent scoped spec-review-scope-boundary review of FR-357 at PR #881 head 327026b2. PASS; no additional findings in this lens.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

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

QSL retains final-emission producer rows. The caller translates QSL-owned equality rows into IR-owned row shapes without changing values; IR checks admitted joins but cannot authenticate arbitrary caller-created Exact values. The driver owns environment and ProgramParts assembly. No copied carrier, compatibility layer, hash or pin appears in the diff.

## Verification and Limits

Reviewed the exact one-file PR diff and QSL main emission API by source inspection. Scoped Quire validate and properties completed; no Rust build, Cargo gate or full suite was run. No applicable AssuranceProfile was found under spec/. Review artifacts are outside the PR worktree.
