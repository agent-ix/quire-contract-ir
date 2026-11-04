---
id: SR-20261104020009
title: "IR-721 PR 883 spec and code alignment review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 883; spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md"
review_set: subset
---

## Summary

Reviewed the status-only FR-357 diff against merged PR #882 public selected-fact API, the traced integration test, and the retained complete-closure boundary. No applicable AssuranceProfile exists under spec/ or plan/.

## Verdict

PASS. AC-5 through AC-8 describe the merged selected-fact read; the diff changes no code, fixture, pin, or copied artifact. No status claim extends to monitor-root closure or QSL-to-IR integration.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

- role: context_only; id: FR-357-AC-1; excerpt: PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A strictly admitted generated monitor root with four entries yields one complete selected type/declaration closure; records, tuples, collections, unions, enum cases, object fields and source units retain original identities, producer order, presence, shape and member/payload types sufficient to build the canonical `TypeEnvironment`. Shared references do not duplicate declarations.
- role: context_only; id: FR-357-AC-2; excerpt: PLANNED/UNRUN (IR-721 and QSL same-acquisition witness retention). A missing, foreign, duplicate, conflicting or stale-after-rewrite producer row, unresolved reachable declaration, unrepresentable type or native nominal/Exact refusal returns its typed cause and first locus, with no partial view or invented identity.
- role: context_only; id: FR-357-AC-3; excerpt: PLANNED/UNRUN (IR-721). A large selected closure amid unrelated graph nodes and unsupported declarations visits only its reachable nodes/edges and charged identity/value material using an iterative worklist. Equal inputs and allowances give equal complete results or first work stops; one below the required work refuses without a partial view.
- role: context_only; id: FR-357-AC-4; excerpt: PLANNED/UNRUN (IR-721). Public API and dependency inspection finds a bounded IR projection over admitted package facts and original QSL witnesses; it finds no IR-owned `TypeEnvironment`, second plan/evaluator, RT dependency, source reparse or witness fabrication.
- role: examined; id: FR-357-AC-5; excerpt: Implemented (IR-721 selected facts; IR-owned fixture). Given an admitted package, explicit IDs for one selected model declaration, its selected record field and one equality application/occurrence, and matching IR-owned same-emission rows, the public bounded read returns exactly those complete facts. It preserves selected document, final structural node, original identities, field order/presence/type, exact equality occurrence and opaque FR-353 decision. No monitor-entry traversal or generated-source closure is claimed.
- role: examined; id: FR-357-AC-6; excerpt: Implemented (IR-721 selected facts). A missing, foreign, duplicate, conflicting or pre-rewrite selected declaration, field, equality application or fact row; wrong document, form, owner, role, occurrence or source-map correspondence; selected unsupported form or unrepresentable type; and native FR-353 refusal each produce a named typed outcome with first native locus and no partial view or invented identity. An unrelated unsupported graph form or absent witness outside the explicit selection leaves the same successful result. Duplicate selected IDs produce one item.
- role: examined; id: FR-357-AC-7; excerpt: Implemented (IR-721 selected facts). The bounded read inspects the explicitly selected admitted facts without traversing monitor entries or scanning unrelated graph nodes. Locating selected witnesses in full-emission row slices charges every inspected row and index entry, including unrelated rows; adding unrelated rows either raises required work or causes a charged first-row-locus stop at the former allowance. The exact allowance succeeds and one less refuses with attempted work and first uncharged locus. Equal inputs and allowance, including a package clone, yield equal results or first stops.
- role: examined; id: FR-357-AC-8; excerpt: Implemented (IR-721 selected facts; API and dependency inspection). Public API and dependency inspection shows the read takes an admitted package, explicit selected model declaration/field IDs and equality application/occurrence pairs, IR-owned row slices and a work allowance, performs no QSL/RT dependency or source recheck, and returns neither an IR environment nor `ProgramParts`. A caller adapts QSL-owned equality emission rows without changing their values; IR checks admitted joins but does not claim to authenticate arbitrary caller-created Exact payloads.
- role: examined; id: FR-357-AC-9; excerpt: PLANNED/UNRUN (cross-repo `quire-integration` test using the available IR selected-fact API). A `quire-integration` test obtains a real final `PlanRoot.emission` through QSL, selects admitted IDs from its available model declaration/field rows and one equality application/occurrence pair, translates QSL equality rows without changing their values, and exercises the public IR selected-fact read on those explicit IDs. It verifies successful joins for representable selected model facts and an opaque FR-353 equality decision or its native typed missing-witness result for the selected equality. It does not traverse four monitor entries or claim a complete generated-source closure, `TypeEnvironment` or RT `ProgramParts`, and it adds no QSL dependency to IR.

## Verification and Limits

Checked the exact one-file PR diff at 69e48b431bb82bf7124228adc594beba28dde926. Merged code PR #882 is 9604dd15459413c06e009f47a3e678e689da5d45; its traced integration test exercises AC-5 through AC-7, and the public model API/dependency graph supports AC-8 inspection. Its final gate had 1,214 integration and 366 model tests pass; strict coverage improved 108 to 105 unbacked rows with zero contradicted statuses. AC-9 real PlanRoot.emission integration and AC-1 through AC-4 complete closure remain open. No Cargo gate was run for this Markdown-only review.
