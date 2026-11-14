---
id: SR-2026111317273300000006
title: "spec-review of IR-69 PR #1022"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-69 PR #1022; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (Behavior unbounded-form paragraph, FR-038-AC-8), spec/checked_package/matrix/TC-052-checked-package-v2-lowering-vocabulary.md"
review_set: subset
---

## Summary

Ticket: IR-69. Spec review of the three edited spec units: the FR-038 Behavior paragraph that defines the unbounded-form set, FR-038-AC-8 and the TC-052 prose. The float ruling (option A, both floats unbounded unless a `float_rounding` domain applies) is transcribed in the same terms as the code. A repo-wide grep found no leftover reference to "eight unbounded forms" or "four scalar" for this set; the remaining "eight" hits are about unrelated sets (integer members, interval operators). The edited set statement still disagrees with the code on `map`, and the new AC clause about the float's closure is worded so that it contradicts its own test.

## Verdict

CONDITIONAL. Two medium findings and one low finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The rewritten unbounded-form paragraph says the set is exactly six scalar plus sequence, set, bag and ordered_set, and that no other form of those two families is unbounded by form. `lower.rs:955` makes `composite_type`/`map` (admitted since IR-1356, FR-038-AC-488) unbounded, so the stated set should have eleven members, or map's arm should change with a stated reason. AC-8's 'ten unbounded forms' list and TC-052's 'four composite' wording share the error. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5016-5028 |
| FND-002 | medium | AC-8 (and TC-052) say 'a float32 or float64 whose closure holds a float_rounding domain typed at it lowers'. A float's own closure never contains a domain typed at it, because the domain depends on the float and not the other way round. The traced test asserts the float requested alone raises `requires_bound`; what lowers is the requested domain (or a node whose closure holds both the float and the domain). Read literally, the clause is false for its own test. It should name the request, e.g. 'a request whose closure holds a float32 or float64 and a float_rounding domain typed at it lowers'. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:6116 |
| FND-003 | low | The Behavior clause 'which for a float is the float_rounding form, the only bounded-domain form declared over floats' says more than the lowering predicate checks: lowering accepts any reachable bounded_domain whose semantic_type is the float, and admission does not check a float_rounding domain's base type. The rule sentence ('a bounded_domain in the closure whose semantic_type is that float') matches the code, so the trailing clause should be stated as a vocabulary fact or dropped. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5022-5025 |

## Examined scope

- FR-038 Behavior, unbounded-form paragraph (examined): A reachable type is unbounded by form exactly when its family and semantic form are a `scalar_type` of form `integer`, `rational`, `decimal`, `float32`, `float64` or `text`, or a `composite_type` of form `sequence`, `set`, `bag` or `ordered_set`. No other family and no other form of those two families is unbounded by form ... A `float32` or `float64` is bounded exactly as the other numeric forms are: by a `bounded_domain` in the closure whose `semantic_type` is that float ...
- FR-038-AC-8 (examined): ... each of the ten unbounded forms (...) raises `requires_bound` and each other declared form of those two families, carrying no `recursion_group`, does not; a `float32` or `float64` whose closure holds a `float_rounding` domain typed at it lowers under a bounds-required profile, and lowers under the default profile unbounded; ...
- TC-052 (examined): Exactly the six scalar (including `float32` and `float64`) and four composite unbounded forms return `requires_bound` ... A float whose closure holds a `float_rounding` domain over it lowers under a bounds-required profile ...
