---
id: SR-2026102743
title: IR-1109 PR 682 spec-review/integrity review
type: SpecReview
analysis: integrity
scope: 'agent-ix/quire-contract-ir; PR #682 head 146f5f36; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md'
review_set: subset
---

## Summary

Independent spec-review/integrity review of IR-1109 PR #682 at exact head 146f5f36b6ec5edc4c65cae032fd283a158f53d7 against QSpec efc22316. Ticket: IR-1109. The diff is spec-only; implementation evidence remains Planned/Unrun.

## Verdict

**CONDITIONAL** — findings require correction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-342 and TC-048 drop the separate two-member controls for root resets, guards, invariants and constraints, and the equal-minimum-key control for non-place aggregates. The generic instruction to test all six aggregates does not require an admitted original, a reversed identity-current member, the exact second-member locus, or a non-place equal-key refusal for each branch. Restore branch-specific controls while removing the invalid place.guards branch. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md |

## Examined units

- **FR-038-AC-336** (`examined`, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): "PLANNED/UNRUN (IR-1083; amended by IR-1109). A `timed_model` root with empty `guards` admits when its selected model has no guard. Adding one root-listed guard changes the guard and root identities while leaving the same `clock_place` path template unchanged; a `guards` binding on the place refuses at strict wire validation. A second distinct place node of the same path in otherwise canonical order refuses `invalid_package`/`duplicate-clock-place`. A `clock_guard` omitted from the root's guard aggregate refuses `invalid_package`/`timed-node-reference` at the unlisted guard's own `body`; omissi"
- **FR-038-AC-342** (`examined`, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): "PLANNED/UNRUN (IR-1096; amended by IR-1109). The public V2 reader admits QSpec's selected `AO-01` timed-slot package and `AO-05` tie-on-start package: all six root reference aggregates ascend by each node's minimum source-region triple, followed on a tie by FR-322-AC-265's structural member key. It refuses identity-current `AO-02`, `AO-03`, `AO-04`, `AO-06` and `AO-07` as `invalid_semantic_graph` with no cause at the first out-of-order member locus; digest order cannot admit `AO-03`, tied starts use ends in `AO-06`, and `AO-07` uses the earlier region of the additional source-map entry. `AO-08"
- **FR-038-AC-359** (`examined`, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): "PLANNED/UNRUN (IR-1109). Root aggregates, each guard part's constraint list and each invariant's bounds use the region-first then structural key of QSpec FR-322-AC-260/265. `CL-16`, `CL-30`, `CL-32` and each reversed member-key case `CL-34`, `CL-36`, `CL-38`, `CL-40`, `CL-42`, `CL-44`, `CL-46`, `CL-49`, `CL-51`, `CL-53` refuse `invalid_semantic_graph` with null cause at the first nonascending member; their paired ordered cases admit. Numeric rational bounds, comparison rank, owner and slot terms, operation, condition, value, binder and every invariant bound/predicate distinguish ties. `CL-47` "
- **TC-048** (`examined`, `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md`): "For AC-342 and AC-359, test all six root aggregates, guard-part constraint\nlists and invariant bound lists with source-region minimum first and QSpec\nFR-322-AC-265 structural tie-break second. `AO-02`..`AO-07` distinguish\nreversed references, digest order, tied starts and an earlier extra region.\n`AO-08`..`AO-10` distinguish ordered, reversed and equal place-path keys on\none region. `CL-29`..`CL-53` distinguish constraint, reset and invariant\nkeys, including comparison rank, rational numeric bound, owner and slot\nterms, operation, condition, value, binder order, all invariant bounds and\npredic"

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-342 and TC-048 now require branch-specific admitted/reversed controls for resets, guards, invariants and constraints, plus tied-region structural order and equal-full-key refusals; the deleted place.guards branch remains absent. |

## Current verdict

**PASS** after disposition pass 1 on PR #682 head d99601ea8d5c37bcfb02037c100da4d040ef3df2.
