---
id: SR-612131
title: IR-1083 PR 653 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; PR 653; Makefile, crates/quire-contract-model/src/checked_package/v2/model_members.rs,
  crates/quire-contract-model/src/checked_package/v2/operations.rs, crates/quire-contract-model/src/checked_package/v2/timed_model.rs,
  tests/conformance_qspec/timed_model.rs
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-038
  type: reviews
---

## Summary

Ticket: IR-1083. Planless PR-diff-scoped matrix, test and source analysis. The published vector helpers read QSpec data but do not run complete packages through the public reader.

## Verdict

FAIL: partial tests make the computed matrix report full AC coverage while the spec and upstream dependency still identify it as incomplete. Quire matrix returned tagged for all six reviewed ACs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Quire marks FR-038-AC-335..340 tagged by partial helper and control tests even though the complete public-reader and vector obligations remain PLANNED/UNRUN. AC-340 is bound only by the one-operand Option control; no test replays all SL/SR/GD/PR vectors through public read, exact outcomes, and stage. | crates/quire-contract-model/src/checked_package/v2/operations.rs:10391; crates/quire-contract-model/src/checked_package/v2/timed_model.rs:1266 |
| FND-002 | medium | The changed slot test compares parsed vector fields through selector_fits only. It does not assert the selected owner join, canonical node and package keys, or public-reader refusal stage; its AC-335 and AC-339 tags therefore credit unverified clauses. STD-541 still blocks authoritative full package vectors. | crates/quire-contract-model/src/checked_package/v2/timed_model.rs:1266 |

## Coverage

- FR-038-AC-335, role: examined. Excerpt: PLANNED/UNRUN (IR-1083). One `clock_place` template admits `ts.0.x`, `ts.1.x` and `ts.2.x` over `Sequence<Timer>[0, 3]` with path steps `field(ts)`, `slot(sequence, Integer type node, 0, 3)`, `field(x)` and one node key; Set, Bag and OrderedSet paths admit with their own collection literal and key, two nested sequences admit two Integer-typed slot steps, and `Map<Text, Clock>` admits with the exact Text key type node. No place node stores a concrete index or state element count. A slot selector given as an integer literal, an expression node, the Text type node under Sequence, or the Integer t
- FR-038-AC-336, role: examined. Excerpt: PLANNED/UNRUN (IR-1083). A `clock_place` and `timed_model` root with empty `guards` admit when their selected model has no guard. The same path with one guard has a distinct place key but a second place of that path with different guards refuses `invalid_package`/`duplicate-clock-place`. A `clock_guard` omitted from the root's guard aggregate refuses `invalid_package`/`timed-node-reference`; the reader does not interpret the omission as an absent guard.
- FR-038-AC-337, role: examined. Excerpt: PLANNED/UNRUN (IR-1083). The public reader admits the Boolean `presence_read` place form of `quire.op.option.present` for `present(other.v)`, `present(self.q[i])` and `present(e)` in a quantifier over `q`, with arguments exactly the place template, owner, and each slot term in path order. The `q` slot terms `i` and `e` are typed at the Integer selector domain, including the quantifier binder `e`, rather than at the element value type. Each application retains its own rederived key while every instance of one path shares its place template key; no model, type, parameter or place node gains a pr
- FR-038-AC-338, role: examined. Excerpt: PLANNED/UNRUN (IR-1083). A place-form `presence_read` over `q` lacking its slot term refuses `ill_typed`/`operator-ineligible` at the application's `/body/arguments`; a first argument that is not a `clock_place` refuses that pair at `/body/arguments/0`; an Integer-typed owner instead of the place's object owner and a wrong-typed slot term each refuse `ill_typed`/`type-mismatch` at the offending argument. No refusal returns an admitted package.
- FR-038-AC-339, role: examined. Excerpt: PLANNED/UNRUN (IR-1083). For `v: Option<Clock>`, the place has only `field(v)` and one resolved-path row retaining `Option<Clock>`; the reader joins that row to the selected owner's declared `v`, optional presence and `Clock` inner type, then accepts terminal Clock after unwrapping Option without adding a path step. For `q: Sequence<Option<Clock>>`, it admits `field(q)` and a symbolically selected `slot` whose row retains Option type. A missing selected member, wrong declared presence or wrong inner type refuses the existing malformed-clock-place path join; the owner object-type key is unchang
- FR-038-AC-340, role: examined. Excerpt: PLANNED/UNRUN (IR-1083). TC-048 reads QSpec's published `timed-slot-selector-vectors.json` through the selected checkout, with no copy in IR, and checks SL-01 through SL-08, SR-01 through SR-04, GD-01 through GD-03 and PR-01 through PR-06 against exact admission or each vector's expected code and cause. For SR-01..04 and PR-04..06, it also checks the vector's RFC 6901 locus suffix. For GD-02, it checks the later root `clock_places` reference locus fixed by QSpec FR-322-AC-120. GD-03 has no vector locus or exact locus in FR-322-AC-150: its exact refusal locus remains PLANNED pending STD-540, an
- Reverse trace: source changes are owned by FR-038; no new unowned feature found.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Head f1d31d0b repairs the recorded defect. |
| FND-002 | fixed | Head f1d31d0b repairs the recorded defect. |

## Disposition verdict

Round 1 at f1d31d0bb6262b635b6ccf442de69570b7d6cb46: all recorded findings fixed. Full published public-reader vectors remain pending STD-541, and GD-03 exact locus remains pending STD-540. Aggregate gate belongs to the lead.
