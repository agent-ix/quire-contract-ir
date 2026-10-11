---
id: SR-8081
title: "Base specification review of IR-691 bounded record projection"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@d2055f10f84cfa7a27b7d5fe0fdfe8c79e710657; FR-038-AC-232 through FR-038-AC-235, TC-048 bounded integer record section; IR-691"
review_set: subset
---

## Summary

The exact numeric.narrow wrapper, typed refusal and public/private admission split are grounded in the current QSL FR-093 lowering and IR accessor contract. One acceptance gap remains in the defensive numeric controls.

## Verdict

**CONDITIONAL** — AC-233 does not force signed, arbitrary-precision comparison or full endpoint-literal validation to fail when implemented incorrectly.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-233 leaves negative beyond-i128 ranges and malformed endpoint literal type/shape outside its required controls | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4246 |

## Finding detail

The FR requires canonical signed decimal range comparison and canonical Integer literal endpoints. AC-233 asks for beyond-i128 min/max and one-below/one-above values, but those may all be positive. A comparator that handles positive magnitudes yet misorders negative values can pass. The specified endpoint mutations cover grammar and order, while a canonical-looking endpoint literal with the wrong type or malformed binding shape is not explicitly tested. Add a negative range beyond i128 with min/max and outside values, and a wrong-typed or malformed endpoint literal control. Derive expected outcomes from the authored decimal strings, not the accessor.

## Scope

- FR-038-AC-232, AC-233, AC-234 and AC-235: examined.
- TC-048 bounded integer record steps 1 through 4: examined.
- QSL FR-093 Coerce lowering and local source assessment: context only; assessment claims were checked against the local FR-093 operation table and IR accessor source, not treated as instructions.

## Examined requirement text

- FR-038-AC-232 (examined): PLANNED/UNRUN (IR-691). A fresh QSL FR-093 checked and emitted `record Point { x: Int[0,9]; }` value `Point { x: 3 }` is admitted by the public V2 reader; its `record_value` binding references a `numeric.narrow` conversion whose sole argument references the canonical Integer literal `3`, and whose member declaration, result type and semantic type name the same `Int[0,9]` range. In an admitted structural.eq or structural.ne over that value, `composite_application_operands` returns the original record GraphChild and semantic type with Literal domain and no introduced positions. At the private post-admission seam, an equivalent graph with canonical integer literal and range endpoints beyond i128 has the same result; no copied or narrowed value is returned.
- FR-038-AC-233 (examined): PLANNED/UNRUN (IR-691). In crate-internal admitted-then-mutated controls, altering only the narrow node's operator, laws, leaves, mode, member kind/declaration, result type, semantic type or argument count/term; the range's presence, Integer base, endpoint grammar/order; or the literal's value kind, declared/semantic type or canonical spelling returns `MalformedDomain` with the enclosing ordinal and first valid type locus specified above. A canonical literal one below min or one above max also returns `MalformedDomain` at the actual range node, with no rounded, clamped or partial Literal result. A correctly shaped boundary literal equal to min or max succeeds, including decimal magnitudes beyond i128.
- FR-038-AC-234 (examined): PLANNED/UNRUN (IR-691). A narrow whose argument reaches a parameter, another application including a second narrow, or a value cycle returns `UnsupportedOperand`/`NonliteralGraphValue` at the first offending node with the enclosing ordinal; an unrelated application remains refused. A direct parameter operand retains its Parameter projection. Existing closed tuple, Option, collection and enum-value descendants remain accepted by their value-form paths, and a narrow integer field inside a supported composite receives only the stated narrow exception. Reader admission and first-refusal outcomes for the unchanged packages are identical before and after the accessor change.
- FR-038-AC-235 (examined): PLANNED/UNRUN (IR-691). Independently count the exact logical graph, term/member, copied-endpoint and examined-literal-byte charges for a small admitted narrow-record fixture. Its measured successful work limit succeeds; one less and zero return `WorkLimit` with the first attempted cumulative charge and no partial operands. A budget exhausted immediately before a malformed wrapper, out-of-range literal or later operand defect returns `WorkLimit` first. Reordering admitted graph nodes leaves the result and consumed work equal. No public accessor type, scalar eligibility, reader admission rule, second domain owner or fixed-width decimal conversion is introduced.

## Dispositions — round 1

Reviewed fix revision `380bccef51abff270d9f960bcdfadfab2bfdbda6` against the original finding at `d2055f10f84cfa7a27b7d5fe0fdfe8c79e710657`. This was a focused two-document specification diff review; no code test or full gate was run.

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed 380bccef51abff270d9f960bcdfadfab2bfdbda6 | AC-233 now requires malformed endpoint binding and literal shape, wrong endpoint literal type, positive and negative ranges beyond i128, signed one-below/one-above refusals, and min/max admissions. TC-048 supplies independent negative decimal strings and Boolean-typed endpoint mutation, with expected order derived from authored inputs. |

No new finding arose from this fix diff. The original finding text and reviewed scope above remain the record of the first pass.
