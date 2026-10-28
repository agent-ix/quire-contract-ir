---
id: SR-2026102716
title: IR-1096 PR 664 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/{mod.rs,model_members.rs,timed_model.rs}, tests/conformance_qspec/timed_model.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1096. Reviewed frozen PR #664 head 2b39685de747bc3b9566a4fd9adf0bf11e76dae6 against base 3e74887e and QSpec f8d636724af4e9e6b35fe5267b91c5815e8782cf. All four changed files were inspected.

## Examined units

- **FR-038-AC-335** (`examined`, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): PLANNED/UNRUN (IR-1083). One `clock_place` template admits `ts.0.x`, `ts.1.x` and `ts.2.x` over `Sequence<Timer>[0, 3]` with path steps `field(ts)`, `slot(sequence, Integer type node, 0, 3)`, `field(x)` and one node key; Set, Bag and OrderedSet paths admit with their own collection literal and key, two nested sequences admit two Integer-typed slot steps, and `Map<Text, Clock>` admits with the exact Text key type node. No place node stores a concrete index or state element count. A slot selector given as an integer literal, an expression node, the Text type node under Sequence, or the Integer t
- **FR-038-AC-336** (`context_only`, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): PLANNED/UNRUN (IR-1083). A `clock_place` and `timed_model` root with empty `guards` admit when their selected model has no guard. The same path with one guard has a distinct place key but a second place of that path with different guards refuses `invalid_package`/`duplicate-clock-place`. A `clock_guard` omitted from the root's guard aggregate refuses `invalid_package`/`timed-node-reference`; the reader does not interpret the omission as an absent guard.
- **FR-038-AC-339** (`examined`, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): PLANNED/UNRUN (IR-1083). For `v: Option<Clock>`, the place has only `field(v)` and one resolved-path row retaining `Option<Clock>`; the reader joins that row to the selected owner's declared `v`, optional presence and `Clock` inner type, then accepts terminal Clock after unwrapping Option without adding a path step. For `q: Sequence<Option<Clock>>`, it admits `field(q)` and a symbolically selected `slot` whose row retains Option type. A missing selected member, wrong declared presence or wrong inner type refuses the existing malformed-clock-place path join; the owner object-type key is unchang
- **FR-038-AC-340** (`examined`, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): PLANNED/UNRUN (IR-1083). TC-048 reads QSpec's published `timed-slot-selector-vectors.json` through the selected checkout, with no copy in IR, and checks SL-01 through SL-08, SR-01 through SR-04, GD-01 through GD-03 and PR-01 through PR-06 against exact admission or each vector's expected code and cause. For SR-01..04 and PR-04..06, it also checks the vector's RFC 6901 locus suffix. For GD-02, it checks the later root `clock_places` reference locus fixed by QSpec FR-322-AC-120. GD-03 has no vector locus or exact locus in FR-322-AC-150: its exact refusal locus remains PLANNED pending STD-540, an
- **FR-038-AC-342** (`examined`, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): PLANNED/UNRUN (IR-1096). The public V2 reader admits QSpec's selected `AO-01` timed-slot package and `AO-05` tie-on-start package, including six root reference aggregates and a place's `guards` in strictly increasing minimum source-region order. It refuses each identity-current `AO-02`, `AO-03`, `AO-04`, `AO-06` and `AO-07` patch as `invalid_semantic_graph` with no cause at the vector's exact first out-of-order member locus; `AO-03` does not admit merely because references are in node-id digest order, `AO-06` compares ends when starts tie, and `AO-07` considers the earlier region in an additio
- **TC-048** (`examined`, `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md`): CheckedPackage V2 strict reader re-derives package and nominal identities.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new AC-335 trace is bound only to selected-package admission and g/q field-type assertions. It does not test the required path instances, Set/Bag/OrderedSet and map selectors, or wrong-selector refusals, so the computed matrix reports coverage for unverified obligations. | tests/conformance_qspec/timed_model.rs:542-612 |
| FND-002 | high | The new AC-339 trace is bound only to the q field-type assertion. It does not assert v Option<Clock> path admission or the selected-member, declared-presence, and inner-type refusal controls, so its tagged evidence cannot discharge AC-339. | tests/conformance_qspec/timed_model.rs:542-612 |
| FND-003 | high | The only AC-340 binder reads the selected package; it never iterates the SL-01..08, SR-01..04, GD-01..03, or PR-01..06 vectors or checks their outcomes, loci, and stage. The trace turns this planned vector obligation into falsely tagged coverage. | tests/conformance_qspec/timed_model.rs:542-612 |
| FND-004 | high | The AC-342 public tests exercise AO outcomes, equal keys, all aggregates, duplicate place, and unlisted guard, but none combines an AO order defect with strict-wire, graph-shape, or stale-key defects to prove those earlier refusals win. The criterion explicitly requires all three precedence controls. | tests/conformance_qspec/timed_model.rs:148-540 |

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format json` reports AC-335, AC-339, AC-340 and AC-342 tagged; AC-336 remains untagged. The public AO cases provide real admission, refusal, and locus evidence, while the bindings listed in Findings overclaim broader criteria. Reverse trace of the changed source found FR-038 ownership; no stub or copied fixture was introduced.

## Verdict

FAIL: AC-335, AC-339, AC-340 and AC-342 have incomplete tagged evidence.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at head 193971a1: the selected-package test no longer claims planned AC-335, AC-339 or AC-340. |
| FND-002 | fixed | Verified at head 193971a1: the selected-package test no longer claims planned AC-335, AC-339 or AC-340. |
| FND-003 | fixed | Verified at head 193971a1: the selected-package test no longer claims planned AC-335, AC-339 or AC-340. |
| FND-004 | fixed | Verified at head 193971a1: the new public-reader test confirms each AO order defect and independently reaches strict-wire, graph-shape and stale-key refusals at their earlier loci. |

## Disposition verdict

All recorded findings are fixed at 193971a1781d4e9735f778c84d4c85812a5dba67. AC-335, AC-339 and AC-340 remain planned and untagged; AC-342 has the required public-reader precedence controls.
