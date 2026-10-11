---
id: SR-8083
title: "Failure-domain review of IR-691 bounded record projection"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@d2055f10f84cfa7a27b7d5fe0fdfe8c79e710657; FR-038-AC-232 through FR-038-AC-235, TC-048 bounded integer record section; IR-691"
review_set: subset
---

## Summary

The new exception is bounded to one exact conversion wrapper and states typed errors, cycle handling and first-refusal precedence. Its byte accounting omits endpoint bytes when they are examined for comparison but not copied.

## Verdict

**CONDITIONAL** — The endpoint-comparison work cost is not bounded by the stated meter rule.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Charge range-endpoint bytes examined by narrow comparison even when no endpoint is copied into the result | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3147 |

## Finding detail

The narrow record returns only the original GraphChild and Literal disposition, so its range endpoints need not be copied. The added rule charges each byte of a *copied* endpoint and each byte of the referenced literal, while comparing the arbitrary-length endpoint text necessarily reads its bytes. A package with very long admitted endpoints can therefore consume comparison time under a tiny accessor work limit. The TC-048 step likewise counts copied-endpoint bytes only. Require a charge for each endpoint byte examined before comparison, once per logical visit, and test that charge with an authored long endpoint and exact/one-less limits.

## Scope

- FR-038-AC-232 through AC-235 and TC-048 steps 1 through 4: examined.
- Existing FR-038 work-meter and typed-error rules: context only.

## Examined requirement text

- FR-038-AC-232 (examined): PLANNED/UNRUN (IR-691). A fresh QSL FR-093 checked and emitted `record Point { x: Int[0,9]; }` value `Point { x: 3 }` is admitted by the public V2 reader; its `record_value` binding references a `numeric.narrow` conversion whose sole argument references the canonical Integer literal `3`, and whose member declaration, result type and semantic type name the same `Int[0,9]` range. In an admitted structural.eq or structural.ne over that value, `composite_application_operands` returns the original record GraphChild and semantic type with Literal domain and no introduced positions. At the private post-admission seam, an equivalent graph with canonical integer literal and range endpoints beyond i128 has the same result; no copied or narrowed value is returned.
- FR-038-AC-233 (examined): PLANNED/UNRUN (IR-691). In crate-internal admitted-then-mutated controls, altering only the narrow node's operator, laws, leaves, mode, member kind/declaration, result type, semantic type or argument count/term; the range's presence, Integer base, endpoint grammar/order; or the literal's value kind, declared/semantic type or canonical spelling returns `MalformedDomain` with the enclosing ordinal and first valid type locus specified above. A canonical literal one below min or one above max also returns `MalformedDomain` at the actual range node, with no rounded, clamped or partial Literal result. A correctly shaped boundary literal equal to min or max succeeds, including decimal magnitudes beyond i128.
- FR-038-AC-234 (examined): PLANNED/UNRUN (IR-691). A narrow whose argument reaches a parameter, another application including a second narrow, or a value cycle returns `UnsupportedOperand`/`NonliteralGraphValue` at the first offending node with the enclosing ordinal; an unrelated application remains refused. A direct parameter operand retains its Parameter projection. Existing closed tuple, Option, collection and enum-value descendants remain accepted by their value-form paths, and a narrow integer field inside a supported composite receives only the stated narrow exception. Reader admission and first-refusal outcomes for the unchanged packages are identical before and after the accessor change.
- FR-038-AC-235 (examined): PLANNED/UNRUN (IR-691). Independently count the exact logical graph, term/member, copied-endpoint and examined-literal-byte charges for a small admitted narrow-record fixture. Its measured successful work limit succeeds; one less and zero return `WorkLimit` with the first attempted cumulative charge and no partial operands. A budget exhausted immediately before a malformed wrapper, out-of-range literal or later operand defect returns `WorkLimit` first. Reordering admitted graph nodes leaves the result and consumed work equal. No public accessor type, scalar eligibility, reader admission rule, second domain owner or fixed-width decimal conversion is introduced.

## Dispositions — round 1

Reviewed fix revision `380bccef51abff270d9f960bcdfadfab2bfdbda6` against the original finding at `d2055f10f84cfa7a27b7d5fe0fdfe8c79e710657`. This was a focused two-document specification diff review; no code test or full gate was run.

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed 380bccef51abff270d9f960bcdfadfab2bfdbda6 | The FR now charges each present endpoint's full UTF-8 length once before validation or comparison, even when an early sign or digit decides order, and forbids a second charge on copying. AC-235 and TC-048 independently count examined endpoint bytes and require a private 4096-digit first-digit-decisive fixture at exact and one-less work limits. |

No new finding arose from this fix diff. The original finding text and reviewed scope above remain the record of the first pass.
