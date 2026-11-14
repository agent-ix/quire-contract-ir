---
id: SR-2026111317273300000005
title: "gap-analysis review of IR-69 PR #1022"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-69 PR #1022; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (FR-038-AC-8), spec/checked_package/matrix/TC-052-checked-package-v2-lowering-vocabulary.md, tests/it/checked_package_v2_lowering.rs, crates/quire-contract-model/src/checked_package/v2/lower.rs"
review_set: subset
---

## Summary

Ticket: IR-69. Planless gap check of FR-038-AC-8 against its traced tests and the code. Plan completion: not assessed. Two tests trace to `TC-052`/`FR-038-AC-8`. `tc_052_unbounded_forms_are_exactly_the_ten_declared_forms` requires `requires_bound` for both floats at a record-field position. `tc_052_float_rounding_domain_bounds_a_float`, run for float32 and float64, checks three things: the float requested alone requires a bound; the domain requested lowers; the default profile lowers the bare float. Both tests fail if the `requires_bound` arm change is reverted, and the second also fails if the `bounded_types` exemption is removed. They pass on the head.

There is no test for the `composite_type` `map` form. `requires_bound` returns `true` for `map` (`lower.rs:955`, since IR-1356), but AC-8's list of forms leaves map out and no test exercises its lowering under `require_bounds`. This is recorded as FND-001.

## Verdict

CONDITIONAL. One medium finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `requires_bound` returns true for `composite_type`/`map` (admitted since IR-1356), but FR-038-AC-8's rewritten ten-form set leaves map out and says every other declared form of the two families does not raise. No TC-052 test covers map, so the ten-forms test still passes while the AC's 'each other declared form does not' claim is false for map. | crates/quire-contract-model/src/checked_package/v2/lower.rs:955, tests/it/checked_package_v2_lowering.rs:720-806 |

## Coverage

- FR-038-AC-8 (examined): A closure holding both an out-of-profile tag and an unbounded type returns `unsupported`; a zero work limit returns `failed` for an absent key rather than `invalid_input`; each named offending key is the least in ascending order rather than the first visited; each of the ten unbounded forms (`integer`, `rational`, `decimal`, `float32`, `float64`, `text`, `sequence`, `set`, `bag`, `ordered_set`) raises `requires_bound` and each other declared form of those two families, carrying no `recursion_group`, does not; a `float32` or `float64` whose closure holds a `float_rounding` domain typed at it lowers under a bounds-required profile, and lowers under the default profile unbounded; and a lowered record's `dependencies` contains every key in its `bounds` and `claims`.
- TC-052 (examined): Exactly the six scalar (including `float32` and `float64`) and four composite unbounded forms return `requires_bound`, and every other declared form of those families lowers. A float whose closure holds a `float_rounding` domain over it lowers under a bounds-required profile, and an unbounded float lowers under the default profile.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-8 now lists map as the eleventh unbounded form. The new traced test tc_052_map_is_the_eleventh_unbounded_form lowers a Map<boolean, uuid> under a bounds-required profile and asserts RequiresBound naming the map itself, then asserts the default profile lowers it. It would fail if map's requires_bound arm became false. The arm is unchanged at lower.rs:955. |

Current verdict after disposition pass 1 (head fe7ca84a): **PASS**. No new findings in the fix diff.
