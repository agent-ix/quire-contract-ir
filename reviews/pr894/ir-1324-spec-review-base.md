---
id: SR-2026110414560002
title: "IR-1324 PR 894 spec-review/base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 894 head 57bc85a95c5cdd3d76537e586c9fedeef491931e; spec/checked_package/functional/FR-352-project-admitted-exact-value-types.md; FR-352-AC-12..15"
review_set: base
---

## Summary

Independent scoped spec-review/base of the four status cells in PR #894. Ticket: IR-1324.

## Examined Scope

- role: examined; id: FR-352-AC-12; excerpt: IMPLEMENTED (IR-1324). For an admitted selected record with a plain `Collection<Integer>` member, the public record view returns a witnessed Exact `MemberId`, the original IR field identity and `Presence::Required`, and an Exact collection whose kind, integer element and cardinality bound equal the admitted member. Distinct sequence, set, bag and ordered-set kinds remain distinct, as do unbounded and `[0,3]` cardinality. The complete view can be formed from checked-package bytes and selected producer rows without a source recheck.
- role: examined; id: FR-352-AC-13; excerpt: IMPLEMENTED (IR-1324). An admitted selected record with a required collection field whose element type is an option alias returns an Exact `Collection<Option<Integer>>` with its admitted kind and bound and the inner Exact `Option<Integer>` intact. An optional collection field of native Integer reports `Presence::Optional` after consuming its outer presence layer and retains Exact `Collection<Integer>` with its kind and bound. A required `[1,1]` field typed by an admitted option alias retains Exact `Option<Integer>`. The already supported optional field of native String reports `Presence::Optional` and Exact `String`. No positive case uses an optional `[1,1]` field typed by an option value type or an `Option<Option<T>>` alias chain.
- role: examined; id: FR-352-AC-14; excerpt: IMPLEMENTED (IR-1324). A nested member reference to a witnessed record in a selected cyclic nominal graph retains `Composite` with that record's original Exact `NodeKey` and terminates without expanding the cycle. A missing, foreign, conflicting or wrong-form nested identity refuses with its native typed cause and first referenced declaration; an unrepresentable nested leaf refuses as `UnsupportedExactType`. Neither refusal exposes a partial record view.
- role: examined; id: FR-352-AC-15; excerpt: IMPLEMENTED (IR-1324). A selected record with a deep admitted chain of sequence member types, including an alternating sequence and option chain in which each option's inner target is a sequence rather than an option directly or through aliases, projects using bounded iterative work and a small host stack. At the exact work allowance it returns the complete view; one less refuses with typed work exhaustion, charged work and first uncharged member locus. Equal admitted inputs and allowances, including a package clone, return equal views or equal first stops.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Analysis

The status claims match merged projector PR #892 and tagged tests at tests/it/checked_package_v2_model_fields.rs:36-269. AC-12 covers four collection kinds and bounds; AC-13 covers admitted nested option, outer optional presence and required option alias; AC-14 covers cyclic nominal identity and missing/foreign/conflicting/wrong-form and unsupported leaf refusals; AC-15 covers the 64-layer admitted alternating Sequence/Option chain on a 256 KiB stack and the exact versus one-less work allowance. The test admits a checked package through its reader before projection. The updated text preserves QSpec FR-154 lines 280-306 and AC-39/40: no direct or alias Option<Option<T>>, and no optional [1,1] Option-typed positive case.

The four criteria remain specific, testable and trace-linked. No new requirement, duplicated QSpec rule, or hash/digest/pin ceremony enters the diff.

## Verification and Limits

Released Quire 0.38.3 scoped validation of FR-352 passed. Scoped properties classified 15 criteria, 5 extractable, with zero candidates. Quire trace found a verifying tagged test for each of AC-12 through AC-15. The review is read-only; no Cargo or full gate was run. No applicable AssuranceProfile document was found under spec/.

## Verdict

PASS. The four Implemented status cells are backed by merged implementation and meaningful test assertions.
