---
id: SR-2026110602354617
title: "IR-1360 spec-review/failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; IR-1360; FR-352, FR-354, TC-467 and changed native APIs/tests"
review_set: subset
---

## Summary

The contract explicitly covers missing/foreign/conflicting identities, immutable atomic failure, first-stop order, budget exhaustion, deep/cyclic native structures and retained-producer trust. IR-1359 diagnostic allocation qualification remains explicit; implementation violations are recorded by code-review.

## Scope

- role: examined; id: FR-352-AC-23; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). Omitting, in turn, a required declaration, unread effective field, source-family row, selected model-tuple acquisition row or source-unit acquisition row returns `IncompleteIdentityWitness` at the first missing native obligation during complete validation. The latter two cases exercise missing private-map entries through public acquisition, without a getter or unchecked mutation. Retaining only a valid subset or removing an entire family cannot shrink the package-derived domain. Each refusal returns no rebound witness and leaves the input unchanged.
- role: examined; id: FR-352-AC-24; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A witness acquired for a foreign admitted selection is refused even when package/display names and local indexes collide. Changing a retained owner, selected document/source or structural/member reference also refuses with the defined native cause. Foreign rows in an otherwise unread family and foreign private-map entries are checked. A package clone remains accepted; arbitrary opaque keys gain no producer authentication.
- role: examined; id: FR-352-AC-25; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A complete mixed-family witness succeeds at separately measured exact work and storage requirements. One unit below either requirement refuses at the exact first unpaid kind, amount and native locus with no rebound value. Prior acquisition charges remain consumed when the same budget is reused; two individually affordable operations whose combined cost exceeds its remaining allowance cannot reset the budget. Counters and first stops are deterministic and checked arithmetic cannot wrap.
- role: examined; id: FR-352-AC-26; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). Complete-witness validation detects stale forms, missing/reordered/extra members, positions or cases inside a supplied row, wrong tuple/source-unit domains and cross-family identity conflicts with the existing typed causes. A fixture with two independent defects demonstrates binding-before-content-before-completeness ordering and the defined family/native order; the later defect never masks the first. All refusal paths preserve the immutable input.
- role: examined; id: FR-352-AC-27; path: spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; excerpt: Implemented (IR-1360). A nonempty witness is checked using borrowed retained facts; success precharges its returned shared handles and any newly retained binding provenance. Actual acquisition copy-on-write map/key/entry/handle copies are charged before each copy/reserve/insert; one unit below an identified copy or returned-handle charge refuses without publishing any part of the output. Merely borrowing retained facts charges work but no storage; owned scratch state is precharged under FR-358. Success deltas and refused-call cumulative counters follow FR-358 without claiming IR-1359
- role: context_only; id: FR-358; path: spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md; excerpt: A scan over caller-controlled text or a sequence charges each examined byte or element before advancing.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. No defect found by this method. Schema-required placeholder is not an actual finding. No applicable AssuranceProfile was found. This reviewer ran no Cargo or final scoped gate. Author logs were inspected: active strict/public tests, workspace Clippy and scoped specification/format checks pass; these results do not prove the missing resource properties. Runtime model metadata is unavailable and is recorded as unknown.

## Round 1 Recheck

PASS unchanged. The fix round preserves this method's clean contract conclusions; the changed dependency edge is checked in the dependency artifact. No new finding. No Cargo or final scoped gate was run by this reviewer.
