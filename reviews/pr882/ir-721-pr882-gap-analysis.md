---
id: SR-20261104020007
title: "IR-721 PR 882 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 882; FR-357 AC-5..8; six Rust files"
review_set: subset
---

## Summary

Independent scoped gap-analysis review of PR #882 on the frozen PR branch. Ticket: IR-721.

## Verdict

CONDITIONAL. The findings below require a fix or explicit disposition before merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The only selected-field integration fixture has one required field at order zero. A regression that always returns Required and order zero would pass while AC-5 requires preservation of optional presence and producer order. Add one later optional field and assert its order and presence through project_selected_facts. | tests/it/checked_package_v2_model_fields.rs:1103-1213 |
| FND-002 | medium | The selected-fact test tags AC-6 but never selects an unsupported form or an unrepresentable admitted field type, and never checks that an unsupported unselected graph form leaves the result unchanged. Those named branches can regress without a failing selected-fact test. | tests/it/checked_package_v2_model_fields.rs:1374-1467 |

## Examined Scope

- role: examined; id: FR-357-AC-5; excerpt: PLANNED/UNRUN (IR-721 selected facts; IR-owned fixture). Given an admitted package, explicit IDs for one selected model declaration, its selected record field and one equality application/occurrence, and matching IR-owned same-emission rows, the public bounded read returns exactly those complete facts. It preserves selected document, final structural node, original identities, field order/presence/type, exact equality occurrence and opaque FR-353 decision. No monitor-entry traversal or generated-source closure is claimed.
- role: examined; id: FR-357-AC-6; excerpt: PLANNED/UNRUN (IR-721 selected facts). A missing, foreign, duplicate, conflicting or pre-rewrite selected declaration, field, equality application or fact row; wrong document, form, owner, role, occurrence or source-map correspondence; selected unsupported form or unrepresentable type; and native FR-353 refusal each produce a named typed outcome with first native locus and no partial view or invented identity. An unrelated unsupported graph form or absent witness outside the explicit selection leaves the same successful result. Duplicate selected IDs produce one item.
- role: examined; id: FR-357-AC-7; excerpt: PLANNED/UNRUN (IR-721 selected facts). The bounded read inspects the explicitly selected admitted facts without traversing monitor entries or scanning unrelated graph nodes. Locating selected witnesses in full-emission row slices charges every inspected row and index entry, including unrelated rows; adding unrelated rows either raises required work or causes a charged first-row-locus stop at the former allowance. The exact allowance succeeds and one less refuses with attempted work and first uncharged locus. Equal inputs and allowance, including a package clone, yield equal results or first st
- role: examined; id: FR-357-AC-8; excerpt: PLANNED/UNRUN (IR-721 selected facts). Public API and dependency inspection shows the read takes an admitted package, explicit selected model declaration/field IDs and equality application/occurrence pairs, IR-owned row slices and a work allowance, performs no QSL/RT dependency or source recheck, and returns neither an IR environment nor `ProgramParts`. A caller adapts QSL-owned equality emission rows without changing their values; IR checks admitted joins but does not claim to authenticate arbitrary caller-created Exact payloads.

## Coverage

Plan completion: not assessed. Computed Quire matrix reports AC-5..7 tagged and AC-8 method-without-symbol. The review also inspected public API, producer joins, budget behavior, and focused tests.

## Verification and Limits

Source review of the frozen diff and computed Quire matrix. Coder focused test log shows the named tests passed; this reviewer did not rebuild a cleaned target. Aggregate gate belongs to the code lead after disposition.
