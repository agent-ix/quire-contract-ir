---
id: SR-2026110301257
title: "Code review of bounded Rational tuple and payload mapping"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #869; crates/quire-contract-model/src/checked_package/v2/{model_members,model_fields,structural,exact_projection}.rs; tests/{conformance_qspec/model_tuple_union,it/checked_package_v2_model_fields}.rs"
review_set: subset
---

## Summary

Reviewed the PR #869 diff at the initial PR head against FR-069-AC-8, QSpec FR-154 Rational value types, and FR-322-AC-380. The changed reader rejects malformed bound carriers and retains the four values in the internal member type; the public view has separate Rational and carried quantity variants. No CI workflow changed.

## Reviewed Units

- FR-069-AC-8, role: examined. Excerpt: “The published `TK-36` case admits through both production paths with its tuple position and variant payload resolving to `Rational[-3, 7; 1, 12]` and the selected `bounded_domain`/`rational_range` shape.”
- QSpec FR-154 Rational value types, role: examined. Excerpt: “A bounded Rational value type with a `unit` `u` has the carried quantity type `Quantity<u, Rational[n1, n2; d1, d2]>`.”
- QSpec FR-322-AC-380, role: examined. Excerpt: “`Ratio` as a tuple position `typeRef` and as a variant case `payloadType` is admitted (`TK-36`).”

## Verdict

**PASS** — The scoped Rust review found no code defect. The independent gap analysis records the missing public retention assertion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Gates

The lead owns the single final aggregate gate. This review inspected the Rust diff, public mapping, malformed intake path, structural join, and CI workflow diff; it did not repeat full gates.

## Targeted Disposition Check

The review fix adds public tuple and variant member-type accessors for the TK-36 assertion. Both are documented and return the mapped retained type or `None` for an unknown or unmapped member. The typed `SelectedTypeRef::from_admitted_name` unit conversion uses `strip_prefix` and excludes the empty suffix; `rational_value_type` reuses that conversion, so the string-edge rule is respected without accepting an empty unit name. No new Rust finding arose from this targeted check.
