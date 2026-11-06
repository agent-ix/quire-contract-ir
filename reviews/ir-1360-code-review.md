---
id: SR-2026110602354614
title: "IR-1360 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1360; FR-352, FR-354, TC-467 and changed native APIs/tests"
review_set: subset
---

## Summary

Reviewed the changed native APIs, seven-buffer admission association, immutable ownership, all-family census and accounting. Three actionable defects require fixes; no unsafe code, source parsing, dependency change, copied upstream artifact or gate weakening was found.

## Scope

- role: examined; id: FR-352-AC-22; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A default-feature external consumer constructs a complete admitted fixture's witness through `from_stage_rows`, all five source joins and the checked public `with_admitted_type_rows`, using actual selected model-tuple and source-unit producer rows and one shared `NativeJoinBudget`. `validate_and_rebind` succeeds with an ample allowance; all rows, ordered payloads, opaque identities and both privately retained channels remain equal. Repeating against an admitted package clone gives an equal result and budget deltas. A genuinely obligation-free admitted package a
- role: examined; id: FR-352-AC-23; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). Omitting, in turn, a required declaration, unread effective field, source-family row, selected model-tuple acquisition row or source-unit acquisition row returns `IncompleteIdentityWitness` at the first missing native obligation during complete validation. The latter two cases exercise missing private-map entries through public acquisition, without a getter or unchecked mutation. Retaining only a valid subset or removing an entire family cannot shrink the package-derived domain. Each refusal returns no rebound witness and leaves the input unchanged.
- role: examined; id: FR-352-AC-24; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A witness acquired for a foreign admitted selection is refused even when package/display names and local indexes collide. Changing a retained owner, selected document/source or structural/member reference also refuses with the defined native cause. Foreign rows in an otherwise unread family and foreign private-map entries are checked. A package clone remains accepted; arbitrary opaque keys gain no producer authentication.
- role: examined; id: FR-352-AC-25; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A complete mixed-family witness succeeds at separately measured exact work and storage requirements. One unit below either requirement refuses at the exact first unpaid kind, amount and native locus with no rebound value. Prior acquisition charges remain consumed when the same budget is reused; two individually affordable operations whose combined cost exceeds its remaining allowance cannot reset the budget. Counters and first stops are deterministic and checked arithmetic cannot wrap.
- role: examined; id: FR-352-AC-26; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). Complete-witness validation detects stale forms, missing/reordered/extra members, positions or cases inside a supplied row, wrong tuple/source-unit domains and cross-family identity conflicts with the existing typed causes. A fixture with two independent defects demonstrates binding-before-content-before-completeness ordering and the defined family/native order; the later defect never masks the first. All refusal paths preserve the immutable input.
- role: examined; id: FR-352-AC-27; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A nonempty witness is checked using borrowed retained facts; success precharges its returned shared handles and any newly retained binding provenance. Actual acquisition copy-on-write map/key/entry/handle copies are charged before each copy/reserve/insert; one unit below an identified copy or returned-handle charge refuses without publishing any part of the output. Merely borrowing retained facts charges work but no storage; owned scratch state is precharged under FR-358. Success deltas and refused-call cumulative counters follow FR-358 without claiming IR-1359
- role: examined; id: FR-352-AC-28; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A default-feature external consumer calls the checked public `with_admitted_type_rows` and single `validate_and_rebind` signatures with `&mut NativeJoinBudget`. The selected-type declaration projection uses the same checked admitted-type join; the old unchecked copy path is absent. The public surface exposes no private-map getter, unchecked rebind, completeness flag, wrapper or second validator. Full finite package-local checks use native admitted facts, terminate iteratively, perform no source parse, and acquire no QSL/runtime dependency or pin. Missing emitte
- role: examined; id: FR-352-AC-29; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). The checked admitted-type acquisition joins a genuine selected model tuple and source-owned declared unit, preserving their complete private provenance and other witness families. Foreign nodes, wrong selected document/source, wrong tuple/unit form, changed owner/original declaration, compound `UnitId`, duplicate rows and cross-family handle conflicts refuse with the typed first native cause and no partial extension. Exact measured work and storage allowances succeed; one unit below either requirement refuses at the first unpaid kind, amount and locus before th
- role: examined; id: FR-354; path: spec/checked_package/functional/FR-354-project-admitted-runtime-context.md; excerpt: This mandatory parameter is additional to the existing projection work limit; the latter continues to bound projection work, rather than silently creating a native join allowance.
- role: context_only; id: FR-358; path: spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md; excerpt: A scan over caller-controlled text or a sequence charges each examined byte or element before advancing.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Some census candidates and text are inspected before their work charge: the unit census calls node_kind for every non-unit node without charging it, filtered declaration/model scans likewise skip charges, and new owner/model-unit string comparisons lack byte precharges. A small remaining allowance can therefore scan an arbitrarily large admitted candidate set before the next denied charge. Charge each candidate before filtering and every compared text pair before inspection. | crates/quire-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs:611 |
| FND-002 | high | The new tuple/source-unit validators duplicate the live checks in runtime_type_projection.rs instead of sharing one native authority. They already disagree for a tuple row naming an existing primitive node: the public join returns UnresolvedSelectedDeclaration, while projection rejects it earlier as MismatchedIdentityWitness. Reuse one validator and leave only projection-specific census/index construction in the projection. | crates/quire-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs:829 |
| FND-003 | medium | Model-owned unit rows belong to the declaration channel, but completeness checks them in the later private-unit scan, after effective fields and model tuples. A witness missing both a model-unit declaration and an effective field therefore reports the field first, violating the specified declaration-before-field order. Include model-unit obligations in the ordered declaration phase and add a mixed-defect control. | crates/quire-contract-model/src/checked_package/v2/exact_projection/complete_witness.rs:598 |

## Verdict

FAIL. Findings require a same-PR fix and disposition. No applicable AssuranceProfile was found. This reviewer ran no Cargo or final scoped gate. Author logs were inspected: active strict/public tests, workspace Clippy and scoped specification/format checks pass; these results do not prove the missing resource properties. Runtime model metadata is unavailable and is recorded as unknown.

## Concrete Cases

- FND-001: after the two fixed binding checks for an obligation-free witness, a tiny remaining work budget still traverses every non-unit graph node at lines 656-657 before charging the first source-family candidate. Other filtered loops at 611-614, 646-647 and the empty selected-model path have the same pre-inspection gap. Model-unit owner comparisons near 789 and source-unit matching_owner near 914 also need byte charges under FR-358.
- FND-002: searched the existing runtime tuple/source-unit validation before accepting the new helpers. The repeated checks are live at runtime_type_projection.rs:628-782 and complete_witness.rs:829-935; the primitive-node outcome divergence is visible directly in their error branches.
- FND-003: use a strictly admitted model record with a missing effective field and an emitted model-owned unit whose declaration row is also omitted. The current code reaches selected_effective_field before selected_model_unit.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: Every declaration/model/field/tuple/unit candidate is now charged before classification; model-unit and source-unit owner comparisons precharge both text operands. Fixed-work growing-census and long-text controls go red on the original source and green on the fix. |
| FND-002 | fixed | Round 1: The projection calls the checked acquisition once before building its projection-owned indexes; duplicated document/form/source/owner/declaration validation branches are removed. A primitive-node tuple row now has the same MismatchedIdentityWitness outcome through both public entry points. |
| FND-003 | fixed | Round 1: One ascending full structural declaration scan includes model-owned units before effective fields. The strict mixed-defect test now returns selected_model_unit at the unit node before the missing unread field. |

## Round 1 Verdict

PASS for the inspected changed behavior; all findings are fixed, with no new finding. The lead's final scoped gate remains pending. No Cargo or final gate was run by this reviewer.

Evidence inspected: the original-source red log records five intended assertion failures; the fixed log records 16 complete-witness root tests and two strict model controls passing, plus the affected tuple/unit/runtime controls. Workspace Clippy with warnings denied and scoped Quire validation pass.
