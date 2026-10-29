---
id: SR-12371
title: "IR-864 gap-analysis of IR PR #444"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/canonical.rs, crates/quire-contract-model/src/decimal.rs, crates/quire-contract-model/src/expression.rs, crates/quire-contract-model/src/output_mapping.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: references
---

# IR-864 gap-analysis of IR PR #444

## Summary

Ticket: IR-864. The diff removes only redundant doc trace lines beside existing attributes. Quire base and head normalized coverage JSON is identical: 515/584 backed, 39 unbacked, 0 contradicted. Normalized matrix JSON is identical across 550 criterion rows, including statuses and binders. Strict coverage exits 1 on the same 39 pre-existing unbacked rows at the base and PR head.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| crates/quire-contract-model/src/canonical.rs::tc_017_a_limit_and_an_encoder_fault_share_the_code_but_not_the_message | examined | `#[trace("TC-017")]` |
| crates/quire-contract-model/src/decimal.rs::tc_016_the_grammar_is_minimal_base_ten | examined | `#[trace("TC-016")]` |
| crates/quire-contract-model/src/decimal.rs::tc_016_range_is_decided_after_the_grammar | examined | `#[trace("TC-016", "FR-013-AC-5")]` |
| crates/quire-contract-model/src/expression.rs::exact_rational_products_reduce_before_i128_range_check | examined | `#[trace("TC-016", "FR-015-AC-8")]` |
| crates/quire-contract-model/src/expression.rs::saturated_integer_intermediates_clamp_at_declared_bounds | examined | `#[trace("TC-016", "FR-015-AC-8")]` |
| crates/quire-contract-model/src/expression.rs::tc_016_merged_integer_ranges_cover_exactly_the_unmerged_pairwise_results | examined | `#[trace("TC-016", "FR-014-AC-7")]` |
| crates/quire-contract-model/src/expression.rs::tc_016_repeated_products_of_a_split_range_stay_two_intervals | examined | `#[trace("TC-016", "FR-014-AC-7")]` |
| crates/quire-contract-model/src/expression.rs::tc_016_widened_ranges_are_a_superset_with_the_same_extremes_closing_the_narrowest_gaps | examined | `#[trace("TC-016", "FR-014-AC-7")]` |
| crates/quire-contract-model/src/expression.rs::tc_016_range_sets_at_and_past_the_size_limit | examined | `#[trace("TC-016", "FR-014-AC-7")]` |
| crates/quire-contract-model/src/expression.rs::tc_016_rational_ranges_merge_and_widen_like_integer_ranges | examined | `#[trace("TC-016", "FR-014-AC-7")]` |
| crates/quire-contract-model/src/expression.rs::tc_016_the_limit_is_64_and_widening_keeps_a_hole_at_zero | examined | `#[trace("TC-016", "FR-014-AC-7")]` |
| crates/quire-contract-model/src/output_mapping.rs::tc_043_absolute_region_arithmetic_overflow_refuses | examined | `#[trace("TC-043")]` |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS — the diff creates no new coverage or trace gap. No runtime tests were rerun for this deletion-only review; the PR author reports its full CI receipt separately.

## Acceptance criteria examined

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-013-AC-5 | examined | Integer bounds and literals at `i128::MIN` and `i128::MAX`, an unsigned type `[0, u64::MAX]` with its maximum literal, rational numerator bounds and literals at both i128 endpoints, and positive maximum denominator and literal denominator at `i128::MAX` decode and serialize back to the same decimal strings. Each of the eight members given as a JSON number (`0`, `1`, `1.0`, `9223372036854775807` and `100000000000000000001`) or a string outside the minimal-decimal grammar (`""`, `"+1"`, `"01"`, `"-0"`, `"1.0"`, `"1e3"`, `" 1"`) refuses `invalid_wire_format`. Each of the eight given `"17014118346 |
| FR-014-AC-7 | examined | A guarded product of 120 leaves checks successfully for integer and rational operands because range intervals merge; a sum of more than 64 guarded leaves checks successfully because the range set is widened to a superset with the same minimum and maximum; a divisor whose resulting set contains zero reports `non_zero_divisor`; and every case returns a typed result or diagnostic without panic. |
| FR-015-AC-8 | examined | At an `i128::MIN` endpoint, integer `reject` negation and division by `-1` report `potentially_undefined` with the checked-range obligation at the operator span; `saturate` negation returns the declared maximum when its mathematical result exceeds that bound. Rational `i128::MIN/1` is admitted and normalizes without panic. In the rational type with numerator bounds `[0, i128::MAX]` and maximum denominator `i128::MAX`, the singleton product `i128::MAX/(i128::MAX - 1) * (i128::MAX - 1)/i128::MAX` is admitted as normalized `1/1` despite both raw products exceeding i128. In that type, `i128::MAX/1 |
