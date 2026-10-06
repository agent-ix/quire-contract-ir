---
id: SR-2133
title: "IR-662 spec-review/failure-domain of i128 model requirements"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@c5d7cca9fa3ed577c3578af2ea5d9a7c99ae4b75; spec/model/functional/FR-013-type-system.md, spec/model/functional/FR-014-expression-semantics.md, spec/model/functional/FR-015-definedness.md, spec/model/functional/FR-019-rust-library-interface.md; ticket IR-662"
review_set: subset
---

## Summary

Ticket IR-662, frozen PR #310 at c5d7cca9fa3ed577c3578af2ea5d9a7c99ae4b75. The changed wire and arithmetic boundary cases name typed refusals, panic safety, and no partial validated value. The conflicting rational cross-product outcome is recorded by integrity analysis.

## Verdict

**PASS** for this method. Review of the changed spec diff only; no source or test changes were made.

## Scope Examined

- `FR-013-AC-5` (examined), `spec/model/functional/FR-013-type-system.md:109`: Integer bounds and literals at `i128::MIN` and `i128::MAX`, an unsigned type `[0, u64::MAX]` with its maximum literal, rational numerator bounds and literals at both i128 endpoints, and positive maximum denominator and literal denominator at `i128::MAX` decode and serialize back to the same decimal strings. Each of the eight members given as a JSON number (`0`, `1`, `1.0`, `9223372036854775807` and `100000000000000000001`) or a string outside the minimal-decimal grammar (`""`, `"+1"`, `"01"`, `"-0"`, `"1.0"`, `"1e3"`, `" 1"`) refuses `invalid_wire_format`. Each of the eight given `"170141183460469231731687303715884105728"` or `"-170141183460469231731687303715884105729"` refuses `invalid_numeric_bounds`; `"0"` and negative strings for positive denominator members refuse `invalid_numeric_bounds`.
- `FR-013-AC-6` (examined), `spec/model/functional/FR-013-type-system.md:110`: Public `IntegerType` bounds/accessors, `RationalType` numerator bounds and maximum-denominator/accessors, and integer/rational literal payloads retain `i64::MAX + 1`, `u64::MAX`, and the permitted i128 endpoints without narrowing; one-past-endpoint wire values return `invalid_numeric_bounds` without a panic or partial validated value.
- `FR-014-AC-7` (examined), `spec/model/functional/FR-014-expression-semantics.md:141`: A guarded product of 120 leaves checks successfully for integer and rational operands because range intervals merge; a sum of more than 64 guarded leaves checks successfully because the range set is widened to a superset with the same minimum and maximum; a divisor whose resulting set contains zero reports `non_zero_divisor`; and every case returns a typed result or diagnostic without panic.
- `FR-014-AC-8` (examined), `spec/model/functional/FR-014-expression-semantics.md:142`: With matching declared types, integer literal nodes at `i64::MAX + 1`, `u64::MAX`, `i128::MIN` and `i128::MAX` and rational literal nodes with `i128::MIN/1`, `i128::MAX/1` and `1/i128::MAX` retain those exact values in typed output; the `i128::MIN` node has no numeric-negate child, while negation of `i128::MAX` remains a distinct node. A literal outside its declared type reports `invalid_numeric_bounds` at that literal's span.
- `FR-015-AC-8` (examined), `spec/model/functional/FR-015-definedness.md:105`: At an `i128::MIN` endpoint, integer `reject` negation and division by `-1` report `potentially_undefined` with the checked-range obligation at the operator span; `saturate` negation returns the declared maximum when its mathematical result exceeds that bound. Rational `i128::MIN/1` is admitted and normalizes without panic, while an unrepresentable rational cross-product or checked negation reports `potentially_undefined` rather than wrapping, panicking, or admitting an unproved result.
- `FR-019-AC-7` (examined), `spec/model/functional/FR-019-rust-library-interface.md:179`: Public compile-time/API fixtures construct and read `IntegerType` bounds, `RationalType` numerator and denominator bounds, and integer/rational literal payloads at `i64::MAX + 1`, `u64::MAX` and the permitted i128 endpoints through signed 128-bit signatures without lossy casts; exact and one-past numeric wire boundary requests through `catch_unwind` return the specified result with no public panic or partial validated value.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
