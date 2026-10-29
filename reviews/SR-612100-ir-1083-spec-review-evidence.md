---
id: SR-612100
title: "spec-review/evidence review of IR-1083 PR 649"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

IR-1083 spec-only PR #649. Examined FR-038 AC-335..340 and TC-048 against QSpec: FR-322-AC-148..150,155..157; TC-367 CK-26/27; timed-slot-selector-vectors SL-01..08, SR-01..04, GD-01..03, PR-01..06. No applicable AssuranceProfile was identified in the checked_package spec tree.

## Verdict

**PASS** — No findings in this lens.

## Scope

- FR-038-AC-335 (examined): PLANNED/UNRUN (IR-1083). One `clock_place` template admits `ts.0.x`, `ts.1.x` and `ts.2.x` over `Sequence<Timer>[0, 3]` with path steps `field(ts)`, `slot(sequence, Integer type node, 0, 3)`, `field(x)` and one node key; Set, Bag and OrderedSet paths admit with their own collection literal and key, two nested sequences admit two Integer-typed slot steps, and `Map<Text, Clock>` admits with the exact Text key type node. No place node stores a concrete index or state element count. A slot selector given as an integer literal, an expression node, the Text type node under Sequence, or the Integer t
- FR-038-AC-336 (examined): PLANNED/UNRUN (IR-1083). A `clock_place` and `timed_model` root with empty `guards` admit when their selected model has no guard. The same path with one guard has a distinct place key but a second place of that path with different guards refuses `invalid_package`/`duplicate-clock-place`. A `clock_guard` omitted from the root's guard aggregate refuses `invalid_package`/`timed-node-reference`; the reader does not interpret the omission as an absent guard.
- FR-038-AC-337 (examined): PLANNED/UNRUN (IR-1083). The public reader admits the Boolean `presence_read` place form of `quire.op.option.present` for `present(other.v)`, `present(self.q[i])` and `present(e)` in a quantifier over `q`, with arguments exactly the place template, owner, and each slot term in path order. The `q` slot terms `i` and `e` are typed at the Integer selector domain, including the quantifier binder `e`, rather than at the element value type. Each application retains its own rederived key while every instance of one path shares its place template key; no model, type, parameter or place node gains a pr
- FR-038-AC-338 (examined): PLANNED/UNRUN (IR-1083). A place-form `presence_read` over `q` lacking its slot term refuses `ill_typed`/`operator-ineligible` at the application's `/body/arguments`; a first argument that is not a `clock_place` refuses that pair at `/body/arguments/0`; an Integer-typed owner instead of the place's object owner and a wrong-typed slot term each refuse `ill_typed`/`type-mismatch` at the offending argument. No refusal returns an admitted package.
- FR-038-AC-339 (examined): PLANNED/UNRUN (IR-1083). For `v: Option<Clock>`, the place has only `field(v)` and one resolved-path row retaining `Option<Clock>`; the reader joins that row to the selected owner's declared `v`, optional presence and `Clock` inner type, then accepts terminal Clock after unwrapping Option without adding a path step. For `q: Sequence<Option<Clock>>`, it admits `field(q)` and a symbolically selected `slot` whose row retains Option type. A missing selected member, wrong declared presence or wrong inner type refuses the existing malformed-clock-place path join; the owner object-type key is unchang
- FR-038-AC-340 (examined): PLANNED/UNRUN (IR-1083). TC-048 reads QSpec's published `timed-slot-selector-vectors.json` through the selected checkout, with no copy in IR, and checks SL-01 through SL-08, SR-01 through SR-04, GD-01 through GD-03 and PR-01 through PR-06 against exact admission or the vector's expected code, cause and RFC 6901 locus suffix. The test checks reader stage against FR-322's stated order, separately from the vector data. Rebuilt node and package keys are checked before timed-body or operation refusals, and the existing Option-operand form remains an admitted control.
- TC-048 (examined): `proposals/checked-package-v2/timed-slot-selector-vectors.json` from the
selected QSpec checkout. Do not store a local copy. Apply its complete
SL-01..SL-08, SR-01..SR-04, GD-01..GD-03 and PR-01..PR-06 cases through the
public V2 reader, rebuilding dependent identities for a semantic mutation.
Compare exact admission or the code, cause and RFC 6901 locus suffix recorded
by the vector. Check the reader stage against FR-322's stated order. Keep strict-wire, graph-shape and stale-key checks ahead of the
timed-model and operation-body checks.

For AC-335, inspect the admitted template path and sel

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
