---
id: SR-2026111317273300000004
title: "code-review of IR-69 PR #1022"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-69 PR #1022 at ae3da78ded98cbe09dd7b793bb3c4668f0502e69; crates/quire-contract-model/src/checked_package/v2/lower.rs, tests/it/checked_package_v2_lowering.rs"
review_set: subset
---

## Summary

Ticket: IR-69. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD`. The change moves `ScalarTypeForm::Float32` and `ScalarTypeForm::Float64` from the false arm to the true arm of the exhaustive `requires_bound` match in `lower.rs`. Under `require_bounds`, a float is now refused unless it is typed (not at a position) and a reachable `bounded_domain` names it as its `semantic_type`. The default profile skips the whole block (`if profile.require_bounds`), so it is unchanged. The match stays exhaustive with no wildcard, matching repo idiom. Focused tests: `cargo test --all-features --test it` (whole_read_ skipped) were run on the head; see Verdict.

The coder's note that the predicate ties a type to a domain only through `semantic_type` equality is accepted for this ticket. This is the same predicate every other unbounded form already uses. Admission already rejects `integer_range`, `text_bounds` and `collection_bounds` over a non-matching base (`structural.rs:1013`, `structural.rs:1036`), so none of those can bound a float. The remaining gap is that admission does not check a `float_rounding` domain's base type. That gap predates this PR and affects how integers, rationals and decimals are bounded, not floats. FND-001 records it at low severity.

## Verdict

PASS on the code change. The one finding is low and predates this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The lowering bound predicate checks only `domain.semantic_type == type`, never that the domain's form fits the type, and structural admission does not check a `float_rounding` domain's base type (only `text_bounds`, `integer_range` and `collection_bounds` are checked). An admitted `float_rounding` domain typed at an integer, rational or decimal therefore satisfies `require_bounds` for that type. This predates IR-69 and does not make floats over-bounded, because integer_range, text_bounds and collection_bounds over a float are refused at admission. | crates/quire-contract-model/src/checked_package/v2/lower.rs:725-737, crates/quire-contract-model/src/checked_package/v2/structural.rs:1011-1040 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Deferred to a linked IR-69 subticket the lead is opening. FR-038 states no admission requirement tying a float_rounding domain to a float base type, so adding the check needs a spec ruling first. The gap predates PR #1022 and does not affect how floats are bounded. |

Current verdict after disposition pass 1 (head fe7ca84a): **PASS**. No new findings in the fix diff.
