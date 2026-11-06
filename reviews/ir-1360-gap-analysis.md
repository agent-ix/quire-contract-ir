---
id: SR-2026110602354615
title: "IR-1360 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1360; FR-352, FR-354, TC-467 and changed native APIs/tests"
review_set: subset
---

## Summary

All eight new FR-352 criteria compute tagged. Real public/default-feature and strict-admission tests cover successful and refused witnesses, but the resource oracle misses unpaid candidate/text scans.

## Scope

- role: examined; id: FR-352-AC-22; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A default-feature external consumer constructs a complete admitted fixture's witness through `from_stage_rows`, all five source joins and the checked public `with_admitted_type_rows`, using actual selected model-tuple and source-unit producer rows and one shared `NativeJoinBudget`. `validate_and_rebind` succeeds with an ample allowance; all rows, ordered payloads, opaque identities and both privately retained channels remain equal. Repeating against an admitted package clone gives an equal result and budget deltas. A genuinely obligation-free admitted package a
- role: examined; id: FR-352-AC-23; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). Omitting, in turn, a required declaration, unread effective field, source-family row, selected model-tuple acquisition row or source-unit acquisition row returns `IncompleteIdentityWitness` at the first missing native obligation during complete validation. The latter two cases exercise missing private-map entries through public acquisition, without a getter or unchecked mutation. Retaining only a valid subset or removing an entire family cannot shrink the package-derived domain. Each refusal returns no rebound witness and leaves the input unchanged.
- role: examined; id: FR-352-AC-24; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A witness acquired for a foreign admitted selection is refused even when package/display names and local indexes collide. Changing a retained owner, selected document/source or structural/member reference also refuses with the defined native cause. Foreign rows in an otherwise unread family and foreign private-map entries are checked. A package clone remains accepted; arbitrary opaque keys gain no producer authentication.
- role: examined; id: FR-352-AC-25; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A complete mixed-family witness succeeds at separately measured exact work and storage requirements. One unit below either requirement refuses at the exact first unpaid kind, amount and native locus with no rebound value. Prior acquisition charges remain consumed when the same budget is reused; two individually affordable operations whose combined cost exceeds its remaining allowance cannot reset the budget. Counters and first stops are deterministic and checked arithmetic cannot wrap.
- role: examined; id: FR-352-AC-26; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). Complete-witness validation detects stale forms, missing/reordered/extra members, positions or cases inside a supplied row, wrong tuple/source-unit domains and cross-family identity conflicts with the existing typed causes. A fixture with two independent defects demonstrates binding-before-content-before-completeness ordering and the defined family/native order; the later defect never masks the first. All refusal paths preserve the immutable input.
- role: examined; id: FR-352-AC-27; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A nonempty witness is checked using borrowed retained facts; success precharges its returned shared handles and any newly retained binding provenance. Actual acquisition copy-on-write map/key/entry/handle copies are charged before each copy/reserve/insert; one unit below an identified copy or returned-handle charge refuses without publishing any part of the output. Merely borrowing retained facts charges work but no storage; owned scratch state is precharged under FR-358. Success deltas and refused-call cumulative counters follow FR-358 without claiming IR-1359
- role: examined; id: FR-352-AC-28; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A default-feature external consumer calls the checked public `with_admitted_type_rows` and single `validate_and_rebind` signatures with `&mut NativeJoinBudget`. The selected-type declaration projection uses the same checked admitted-type join; the old unchecked copy path is absent. The public surface exposes no private-map getter, unchecked rebind, completeness flag, wrapper or second validator. Full finite package-local checks use native admitted facts, terminate iteratively, perform no source parse, and acquire no QSL/runtime dependency or pin. Missing emitte
- role: examined; id: FR-352-AC-29; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). The checked admitted-type acquisition joins a genuine selected model tuple and source-owned declared unit, preserving their complete private provenance and other witness families. Foreign nodes, wrong selected document/source, wrong tuple/unit form, changed owner/original declaration, compound `UnitId`, duplicate rows and cross-family handle conflicts refuse with the typed first native cause and no partial extension. Exact measured work and storage allowances succeed; one unit below either requirement refuses at the first unpaid kind, amount and locus before th
- role: examined; id: TC-467; path: spec/checked_package/matrix/TC-467-native-selected-identity-join-budget.md; excerpt: Run the FR-358 tagged tests in the `it` integration binary with `conformance-cli` enabled. For each family, obtain its successful work and storage charges from an ample budget, then repeat the same admitted value at each exact limit and one unit below each limit. Join two collection rows under one budget, including a nonempty prior map copied by `Arc::make_mut`, and repeat the ordered joins against a cloned admitted package.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The work-limit tests derive their allowance from an ample run of the same implementation, so omitted census/text charges lower both the measured requirement and its one-below oracle. There is no fixed-allowance growing-non-unit-census or long-owner-text control to detect the unpaid scans identified by code-review FND-001. Add independent logical-work/pre-inspection assertions for those cases. | tests/it/complete_identity_witness.rs:294 |

## Verdict

FAIL. Findings require a same-PR fix and disposition. No applicable AssuranceProfile was found. This reviewer ran no Cargo or final scoped gate. Author logs were inspected: active strict/public tests, workspace Clippy and scoped specification/format checks pass; these results do not prove the missing resource properties. Runtime model metadata is unavailable and is recorded as unknown.

## Coverage

Plan completion: not assessed

Computed matrix: FR-352-AC-22 through AC-29 are all tagged (28 total criterion bindings). Repository-wide strict matrix exits 1 with 83 untagged criteria and one criterion tagged only by ignored tests outside these eight new criteria; no repository-wide assurance PASS is claimed. The reviewed changed tests are active and traverse real native checking/projection, not mocks. Code-review findings also prevent a semantic acceptance claim.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: Independent fixed logical ledgers exercise growing non-unit candidates, even an empty selected model, and long source/model-unit owner text without deriving the allowance from an ample run. All five new controls fail for their intended assertions on the original source and pass on the fix. |

## Round 1 Verdict

PASS for the inspected changed behavior; all findings are fixed, with no new finding. The lead's final scoped gate remains pending. No Cargo or final gate was run by this reviewer. Original repository-wide matrix baseline limitations remain outside the changed criteria; no repository-wide assurance PASS is asserted. Plan completion: not assessed.

Evidence inspected: the original-source red log records five intended assertion failures; the fixed log records 16 complete-witness root tests and two strict model controls passing, plus the affected tuple/unit/runtime controls. Workspace Clippy with warnings denied and scoped Quire validation pass.
