---
id: SR-20261104020002
title: "IR-721 PR 881 spec-review-failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; PR 881; FR-357 witnessed-subset amendment"
review_set: subset
---

## Summary

Independent scoped spec-review-failure-domain review of FR-357 at PR #881 head 327026b2. Findings below block an unqualified PASS.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-5 promises success for a genuine four-entry root while generated source records, sequences, tuple and union are reached and AC-6 requires reached unsupported forms to refuse; define a supported-subset fixture or explicit typed refusal on generated roots | FR-357-AC-5 |

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

QSL main 13c7edf `monitor_entries::render` generates record State, V, Cell and other records, Sequence aliases and operations, tuple Step, and union Outcome. The four entry bodies reference these forms through their signatures and helper calls. The first-slice rows are model declaration/field and equality rows, so these source-owned identities are absent pending QSL-1631. AC-5 must identify a genuinely reachable supported subset that can succeed, or say the real generated root stops with a native typed refusal. An invented source identity is not permitted.

## Verification and Limits

Reviewed the exact one-file PR diff and QSL main emission API by source inspection. Scoped Quire validate and properties completed; no Rust build, Cargo gate or full suite was run. No applicable AssuranceProfile was found under spec/. Review artifacts are outside the PR worktree.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-5 now uses an admitted supported-only four-entry fixture for success and explicitly requires a real PlanRoot final-emission read to refuse at the first reached source-owned form lacking a QSL-1631 witness. |
| FND-002 | fixed | The first read now takes explicit admitted model declaration/field and equality application/occurrence IDs, does not traverse a root or its signatures, and returns selected facts only. AC-9 composes a real emission through quire-integration; AC-1 through AC-4 retain the future complete closure. |

## Fix-round evidence

At PR #881 head 932c8f68, AC-5 states: PLANNED/UNRUN (IR-721 subset). An admitted four-entry fixture whose entire reachable graph uses only supported model declarations/fields and equality forms succeeds through the public bounded read, retaining reached selected document, final structural node, original identities, field order/presence/type, exact equality occurrence and opaque FR-353 decision; shared references appear once. A test using a real final `PlanRoot.emission` first proves its selected declaration/field rows join through IR `SelectedExactIdentities::from_stage_rows` and its translated equality rows reach IR `admitted_equality` with a complete decision or its native typed missing-witness result, then proves the subset read refuses at the first reached source-owned form lacking QSL-1631 witness. Neither result claims a complete generated-source closure or `TypeEnvironment`.

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | The earlier supported-only four-entry success case is impossible: monitor-plan admission requires generated composite State, Inputs, Scope and tuple result signatures for every four-entry root, so an AC-5 success must use explicit fact selection rather than entry closure | FR-357-AC-5 |

## Fix-round evidence (disposition pass 3)

At PR #881 head d9807abf, the first read is an explicitly selected bounded fact set. AC-5's positive case names one selected model declaration, its field and one equality application/occurrence in an admitted package. AC-8's API takes those explicit IDs; AC-9 owns the real PlanRoot.emission integration without monitor-entry traversal. The complete four-entry generated-source closure remains Planned/Unrun in AC-1 through AC-4. No source-owned identity is inferred and no environment or ProgramParts is constructed.

## Review status after disposition pass 3

PASS at PR #881 head d9807abf. The latest disposition for FND-001 and FND-002 is fixed.
