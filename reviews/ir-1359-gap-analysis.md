---
id: SR-2026110602354622
title: IR-1359 PR 933 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; IR-1359; spec/checked_package/functional/FR-359-bound-native-projection-refusals.md;
  spec/checked_package/matrix/TC-468-bounded-native-projection-refusals.md; tests/conformance_qspec/monitor_plan.rs;
  tests/it/exact_projection_limits.rs; tests/it/checked_package_v2_model_fields.rs
review_set: subset
---

## Summary

Computed Test Matrix and changed tests were checked against all five FR-359 criteria. AC5 is untagged, and AC4 has a partial tagged test. Plan completion: not assessed.

## Coverage

- `FR-359-AC-1` (examined, `spec/checked_package/functional/FR-359-bound-native-projection-refusals.md`): Selected clock and Exact projection require the same caller-owned `&mut NativeJoinBudget` used by stage-row acquisition and source joins. A prior affordable operation can leave either resource one unit short of a read that succeeds under a fresh ample budget; the shared read refuses at its first unpaid step. The runtime projection public signature also requires this budget, with no numeric-limit overload; its full accounting is deferred to IR-1366.
- `FR-359-AC-2` (examined, `spec/checked_package/functional/FR-359-bound-native-projection-refusals.md`): For a real admitted success in each qualified read family, measuring work and storage with ample limits and rerunning at both exact limits returns the same complete value. One unit below either measured requirement returns the typed exhausted kind, first unpaid charge and no partial value; cumulative counters stay within the limits.
- `FR-359-AC-3` (examined, `spec/checked_package/functional/FR-359-bound-native-projection-refusals.md`): A semantic refusal in each qualified family whose owned diagnostic includes a dynamic node path or form retains its stable category and first locus at ample limits. Zero storage returns fixed-size storage exhaustion; exact diagnostic storage returns the full semantic refusal, and one below it returns storage exhaustion. No refusal exposes a partial owned payload or successful delta.
- `FR-359-AC-4` (examined, `spec/checked_package/functional/FR-359-bound-native-projection-refusals.md`): Nested Exact projection and its error propagation charge each newly owned payload once. A payable semantic refusal retains its category and locus; an unpaid conversion returns fixed-size storage exhaustion. Repeated inputs and limits, including an admitted package clone, give equal values or first stops and equal counters.
- `FR-359-AC-5` (examined, `spec/checked_package/functional/FR-359-bound-native-projection-refusals.md`): Existing selected clock and Exact success and non-budget refusal decisions remain unchanged at sufficient limits. Neither qualified read has uncharged owned result, scratch or refusal allocation, a second accounting scheme, or a QSL or runtime dependency in the IR model. Runtime full-precharge evidence is not credited by this criterion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-359-AC-5 has no tagged test | spec/checked_package/functional/FR-359-bound-native-projection-refusals.md:90 |
| FND-002 | medium | The AC4 tagged nested-refusal test omits repeated clone and counter agreement | tests/it/checked_package_v2_model_fields.rs:7177 |

## Verdict

**FAIL** for this method.

Plan completion: not assessed. Optional semantic review was not selected; only changed test bindings were examined.

## Detail

FND-001: Computed Quire matrix reports FR-359-AC-5 untagged. Its Test verification includes unchanged clock/Exact decisions and the absence of uncharged allocations and a second accounting scheme; no test symbol binds this criterion.

FND-002: The only AC4-tagged test checks one nested semantic refusal, exact storage, and one-below storage. It never reruns on an admitted package clone or asserts equal cumulative counters and first stops for repeated inputs and limits, despite those explicit AC4 clauses.


## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The clock and Exact value/refusal tests now carry FR-359-AC-5 trace tags; computed matrix verification is pending. |
| FND-002 | fixed | The nested source-collection refusal test now repeats on an admitted package clone and compares semantic result, consumed counters, compact one-below refusal, and first unpaid charge. |
