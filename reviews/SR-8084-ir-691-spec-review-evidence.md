---
id: SR-8084
title: "Evidence-method review of IR-691 bounded record projection"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; FR-038-AC-232 through FR-038-AC-235; IR-691"
review_set: subset
---

## Summary

The installed `quoin advise --repo . --json` returns no method mismatch for AC-232 through AC-235. Authored Test for all four and Inspection for AC-235 are appropriate to the planned public and private controls. The concrete control gaps are recorded in the base and failure-domain reviews.

## Verdict

**PASS** — No separate verification-method mismatch found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined requirement text

- FR-038-AC-232 (examined): PLANNED/UNRUN (IR-691). A fresh QSL FR-093 checked and emitted `record Point { x: Int[0,9]; }` value `Point { x: 3 }` is admitted by the public V2 reader; its `record_value` binding references a `numeric.narrow` conversion whose sole argument references the canonical Integer literal `3`, and whose member declaration, result type and semantic type name the same `Int[0,9]` range. In an admitted structural.eq or structural.ne over that value, `composite_application_operands` returns the original record GraphChild and semantic type with Literal domain and no introduced positions. At the private post-admission seam, an equivalent graph with canonical integer literal and range endpoints beyond i128 has the same result; no copied or narrowed value is returned.
- FR-038-AC-233 (examined): PLANNED/UNRUN (IR-691). In crate-internal admitted-then-mutated controls, altering only the narrow node's operator, laws, leaves, mode, member kind/declaration, result type, semantic type or argument count/term; the range's presence, Integer base, endpoint grammar/order; or the literal's value kind, declared/semantic type or canonical spelling returns `MalformedDomain` with the enclosing ordinal and first valid type locus specified above. A canonical literal one below min or one above max also returns `MalformedDomain` at the actual range node, with no rounded, clamped or partial Literal result. A correctly shaped boundary literal equal to min or max succeeds, including decimal magnitudes beyond i128.
- FR-038-AC-234 (examined): PLANNED/UNRUN (IR-691). A narrow whose argument reaches a parameter, another application including a second narrow, or a value cycle returns `UnsupportedOperand`/`NonliteralGraphValue` at the first offending node with the enclosing ordinal; an unrelated application remains refused. A direct parameter operand retains its Parameter projection. Existing closed tuple, Option, collection and enum-value descendants remain accepted by their value-form paths, and a narrow integer field inside a supported composite receives only the stated narrow exception. Reader admission and first-refusal outcomes for the unchanged packages are identical before and after the accessor change.
- FR-038-AC-235 (examined): PLANNED/UNRUN (IR-691). Independently count the exact logical graph, term/member, copied-endpoint and examined-literal-byte charges for a small admitted narrow-record fixture. Its measured successful work limit succeeds; one less and zero return `WorkLimit` with the first attempted cumulative charge and no partial operands. A budget exhausted immediately before a malformed wrapper, out-of-range literal or later operand defect returns `WorkLimit` first. Reordering admitted graph nodes leaves the result and consumed work equal. No public accessor type, scalar eligibility, reader admission rule, second domain owner or fixed-width decimal conversion is introduced.
