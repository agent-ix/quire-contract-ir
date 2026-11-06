---
id: SR-2026110602354621
title: IR-1359 PR 933 code-review review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1359; spec/checked_package/functional/FR-359-bound-native-projection-refusals.md;
  spec/checked_package/matrix/TC-468-bounded-native-projection-refusals.md; crates/quire-contract-model/src/checked_package/v2/monitor_plan.rs;
  crates/quire-contract-model/src/checked_package/v2/exact_projection.rs; crates/quire-contract-model/src/checked_package/v2/native_join_budget.rs;
  tests/conformance_qspec/monitor_plan.rs; tests/it/exact_projection_limits.rs
review_set: subset
---

## Summary

Reviewed the clock and Exact budget paths, their public error shapes, focused tests and callers against FR-359 and repository Rust conventions. A clock resource-refusal path returns the wrong exhausted kind at zero storage.

## Coverage

- `FR-359-AC-1` (examined, `spec/checked_package/functional/FR-359-bound-native-projection-refusals.md`): Selected clock and Exact projection require the same caller-owned `&mut NativeJoinBudget` used by stage-row acquisition and source joins. A prior affordable operation can leave either resource one unit short of a read that succeeds under a fresh ample budget; the shared read refuses at its first unpaid step. The runtime projection public signature also requires this budget, with no numeric-limit overload; its full accounting is deferred to IR-1366.
- `FR-359-AC-2` (examined, `spec/checked_package/functional/FR-359-bound-native-projection-refusals.md`): For a real admitted success in each qualified read family, measuring work and storage with ample limits and rerunning at both exact limits returns the same complete value. One unit below either measured requirement returns the typed exhausted kind, first unpaid charge and no partial value; cumulative counters stay within the limits.
- `FR-359-AC-3` (examined, `spec/checked_package/functional/FR-359-bound-native-projection-refusals.md`): A semantic refusal in each qualified family whose owned diagnostic includes a dynamic node path or form retains its stable category and first locus at ample limits. Zero storage returns fixed-size storage exhaustion; exact diagnostic storage returns the full semantic refusal, and one below it returns storage exhaustion. No refusal exposes a partial owned payload or successful delta.
- `FR-359-AC-4` (examined, `spec/checked_package/functional/FR-359-bound-native-projection-refusals.md`): Nested Exact projection and its error propagation charge each newly owned payload once. A payable semantic refusal retains its category and locus; an unpaid conversion returns fixed-size storage exhaustion. Repeated inputs and limits, including an admitted package clone, give equal values or first stops and equal counters.
- `FR-359-AC-5` (examined, `spec/checked_package/functional/FR-359-bound-native-projection-refusals.md`): Existing selected clock and Exact success and non-budget refusal decisions remain unchanged at sufficient limits. Neither qualified read has uncharged owned result, scratch or refusal allocation, a second accounting scheme, or a QSL or runtime dependency in the IR model. Runtime full-precharge evidence is not credited by this criterion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Clock work exhaustion can report storage exhaustion and allocate an owned resource diagnostic | crates/quire-contract-model/src/checked_package/v2/monitor_plan.rs:347 |

## Verdict

**CONDITIONAL** for this method.

## Detail

FND-001: When the next Work charge is denied, ClockBudget::charge calls error(), which attempts to charge and clone an owned node path. With zero storage, the returned code and denied charge say StorageLimit while budget.first_unpaid() remains Work; with ample storage, a WorkLimit refusal still owns a dynamic node path, contrary to the fixed-size resource-refusal contract.

