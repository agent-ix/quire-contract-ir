---
id: SR-20261104133706
title: "IR-1324 PR 891 base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 891; FR-352 AC-13 and AC-15"
review_set: subset
---

## Summary

Independent scoped review of PR #891 at b786cf1e03bab94732b716200a372c0624633657 against QSpec FR-154 at b840f0985a8078ce6e3d8b9fee3ea874ec1c5ef5.

## Verdict

PASS. AC-13 names valid required option and optional collection cases; AC-15 restricts deep cases to admitted nested sequences and alternating sequence/option layers.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

- role: examined; id: FR-352-AC-13; excerpt: PLANNED/UNRUN (IR-1324). An admitted selected record with a required collection field whose element type is an option alias returns an Exact `Collection<Option<Integer>>` with its admitted kind and bound and the inner Exact `Option<Integer>` intact. An optional collection field of native Integer reports `Presence::Optional` after consuming its outer presence layer and retains Exact `Collection<Integer>` with its kind and bound. A required `[1,1]` field typed by an admitted option alias retains Exact `Option<Integer>`. The already supported optional field of native String reports `Presence::Optional` and Exact `String`. No positive case uses an optional `[1,1]` field typed by an option value type or an `Option<Option<T>>` alias chain.
- role: examined; id: FR-352-AC-15; excerpt: PLANNED/UNRUN (IR-1324). A selected record with a deep admitted chain of sequence member types, including an alternating sequence and option chain in which each option's inner target is a sequence rather than an option directly or through aliases, projects using bounded iterative work and a small host stack. At the exact work allowance it returns the complete view; one less refuses with typed work exhaustion, charged work and first uncharged member locus. Equal admitted inputs and allowances, including a package clone, return equal views or equal first stops.
- role: examined; id: FR-352; excerpt: This projection consumes only QSpec FR-154 admitted member types. It SHALL NOT admit `Option<Option<T>>`, including an alias path to the inner `Option`, or an optional `[1,1]` field typed by an option value type; those shapes are refused before projection.

## Verification and Limits

Reviewed the frozen one-file diff, FR-352 context, QSpec FR-154 option and presence rules, and IR admission guards. Quire validation applies to this spec-only review. Implementation and QSL parity are not claimed.
