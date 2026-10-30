---
id: SR-2026102994
title: "IR-1141 PR #723 spec-review/base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/TC-226-checked-package-v2-anonymous-structural-node-bodies.md, spec/model/functional/FR-046-derive-candidate-application-node-identity.md, spec/model/functional/FR-048-derive-structural-and-group-candidates.md"
review_set: subset
---

## Summary

Independent spec-review/base review of quire-contract-ir PR #723 at dacd564d against QSpec #536 at 198b5b1a. AC-145 contradicts its owner-free example and QSpec zero-declaration path rule.

## Verdict

**FAIL** — AC-145 is inconsistent and cannot be used as the owner-free path oracle.

## Scope

- `FR-038-AC-21` examined: A node whose declared `qualified_name` differs from its declared structural path refuses as `invalid_package` with cause `declaration-nominal-mismatch
- `FR-038-AC-145` examined: PLANNED/UNRUN (IR-1141). `List { next?: List; }`, `Tree { kids: Sequence<Tree>[0, 3]; }` and the owner-free self-option group admit with retained prod
- `FR-038-AC-146` examined: PLANNED/UNRUN (IR-1141). Boolean, Integer, integer-range and reference nodes retain their closed-shape checks inside or outside a labelled group. For 
- `FR-038-AC-147` examined: PLANNED/UNRUN (IR-1141). Every node carrying one shared non-empty group label participates in that group, including an off-cycle `Set<Int[0,1000]>` me
- `FR-038-AC-150` examined: PLANNED/UNRUN (IR-1141). The reader validates every structural ID for shape and uniqueness, then joins owners and graph references in QSpec FR-322 ord
- `FR-038-AC-364` examined: PLANNED/UNRUN (IR-1141; QSpec FR-322-AC-272/273/274). The reader admits declared, derived, model-owned and imported nodes with QSpec FR-322 structural
- `FR-046-AC-3` examined: A group containing declared application and nonapplication nodes admits with each declared member's own declaration path and one shared non-empty `rec
- `FR-046-AC-4` examined: A missing exactly referenced member, repeated structural id, malformed ordinal, mismatched cycle label, unresolved target, nonapplication body in an a
- `FR-048-AC-3` examined: PLANNED/UNRUN (IR-1141). A group with declared structural and application members, including TC-946 K14's three-declared-member cycle and an off-cycle
- `FR-048-AC-4` examined: Repeated ids, unknown tag/form, malformed closed body, invalid owner shape, invalid or mismatched group label or out-of-range structural ordinal retur
- `FR-038`, `FR-046`, `FR-048`, `TC-048`, `TC-226` edited prose and controls examined.
- QSpec proposed `FR-322-AC-274`, existing `FR-322-AC-29`, and `TC-946` K14 checked as cross-repo context.
- No applicable AssuranceProfile found under the changed module.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-145 calls the owner-free self-option case a one-declared-member group and requires its anonymous members to use a nonexistent declared path; QSpec FR-322-AC-274 instead admits a producer-chosen path when there are zero declared members. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4582 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-145 now separates the List/Tree one-declared-member path from the owner-free zero-declaration producer-chosen path; TC-226 G1 names the same producer-chosen path. |
