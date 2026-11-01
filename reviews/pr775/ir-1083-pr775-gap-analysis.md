---
id: SR-202610310137
title: IR-1083 PR775 gap-analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR775; tests/conformance_qspec/main.rs, tests/conformance_qspec/timed_model.rs, tests/conformance_qspec/timed_slots.rs; FR-038-AC-335 through FR-038-AC-340"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
---

## Summary

Ticket: IR-1083. Independent review of the three-file test-only diff. Code-review includes the Rust lane and rust-style guides. No applicable AssuranceProfile found. No spec diff: spec-review subanalyses do not apply.

## Verdict

FAIL for repository completeness: existing untagged criteria remain. This is not a new PR regression; the qualification-only PR may land after its aggregate gate while IR-1083 remains open. No AC narrowing or withdrawn vectors are requested.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Existing FR-038-AC-335 remains untagged in the computed matrix; the new TC-048-only tests do not establish this complete criterion. IR-1083 must remain open. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4866 | correct-requirement-no-evidence |
| FND-002 | medium | Existing FR-038-AC-336 remains untagged in the computed matrix; the new TC-048-only tests do not establish this complete criterion. IR-1083 must remain open. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4867 | correct-requirement-no-evidence |
| FND-003 | medium | Existing FR-038-AC-337 remains untagged in the computed matrix; the new TC-048-only tests do not establish this complete criterion. IR-1083 must remain open. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4868 | correct-requirement-no-evidence |
| FND-004 | medium | Existing FR-038-AC-338 remains untagged in the computed matrix; the new TC-048-only tests do not establish this complete criterion. IR-1083 must remain open. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4869 | correct-requirement-no-evidence |
| FND-005 | medium | Existing FR-038-AC-339 remains untagged in the computed matrix; the new TC-048-only tests do not establish this complete criterion. IR-1083 must remain open. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4870 | correct-requirement-no-evidence |
| FND-006 | medium | Existing FR-038-AC-340 remains untagged in the computed matrix; the new TC-048-only tests do not establish this complete criterion. IR-1083 must remain open. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4871 | correct-requirement-no-evidence |

## Coverage

Plan completion: not assessed

Inspected every changed line, the production reader entry seam, existing patch and node-reference helpers, all 19 owner-published SL/SR/PR/GD cases and their oracles. No fixture/spec copy, second patch engine, gate narrowing, production source edit, unsafe code or new dependency occurs. Seven semantic negatives each independently assert wire and graph precedence. GD-01 inspects all six admitted empty aggregates. Test-only panics are deterministic fixture assertions. Existing helper visibility is limited to its parent test module and has a direct sibling caller.

Provided logs inspected: /tmp/ir1083-final-vectors.log reports 2 passing tests, 19 cases and 14 precedence probes in 1.80 seconds; /tmp/ir1083-clippy.log reports workspace/all-target success in 29.28 seconds. Diff whitespace passes independently. Cargo/Kani were not rerun per dispatch. Full integration/CI Test is required before merge. Existing timed_model failures were reported by the lead and are not caused by this diff.

Computed full matrix: 726 tagged, 134 untagged, 22 method-without-symbol, 1 tagged-by-ignored-test. The entire inventory is retained in matrix.json; this subset review does not assert repository-wide completeness. No new reverse ownership gap or stub found in the changed test surface. Optional gap semantic fanout was not requested and was skipped; ordinary code-test alignment was reviewed. AC337 requires indexed presence excluded by current owner language, while AC340 names withdrawn SL08/PR02. Those unresolved obligations remain with IR-1083.

## Examined scope

```yaml
scope:
- id: tests/conformance_qspec/main.rs
  path: tests/conformance_qspec/main.rs
  role: examined
  excerpt: mod timed_slots;
- id: tests/conformance_qspec/timed_model.rs
  path: tests/conformance_qspec/timed_model.rs
  role: examined
  excerpt: 'pub(super) fn apply_published_timed_patch(package: &mut Value, patch:
    &Value) {'
- id: tests/conformance_qspec/timed_slots.rs
  path: tests/conformance_qspec/timed_slots.rs
  role: examined
  excerpt: '#[trace("TC-048")]'
- id: FR-038-AC-335
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1083). One `clock_place` template admits `ts.0.x`, `ts.1.x`
    and `ts.2.x` over `Sequence<Timer>[0, 3]` with path steps `field(ts)`, `slot(sequence,
    Integer type node, 0, 3)`, `field(x)` and one node key; Set, Bag and OrderedSet
    paths admit with their own collection literal and key, two nested sequences admit
    two Integer-typed slot steps, and `Map<Text, Clock>` admits with the exact Text
    key type node. No place node stores a concrete index or state element count. A
    slot selector given as an integer literal, an expression node, the Text type node
    under Sequence, or the Integer t
- id: FR-038-AC-336
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1083; amended by IR-1109). A `timed_model` root with
    empty `guards` admits when its selected model has no guard. Adding one root-listed
    guard changes the complete guard and root bodies while leaving the same `clock_place`
    path template unchanged; a `guards` binding on the place refuses at strict wire
    validation. A second distinct place node of the same model and path in otherwise
    canonical order has a forged `resolved_path` row and refuses `invalid_package`/`malformed-clock-place`
    at the first differing row; row join runs after aggregate order and before the
    retained `duplica
- id: FR-038-AC-337
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1083). The public reader admits the Boolean `presence_read`
    place form of `quire.op.option.present` for `present(other.v)`, `present(self.q[i])`
    and `present(e)` in a quantifier over `q`, with arguments exactly the place template,
    owner, and each slot term in path order. The `q` slot terms `i` and `e` are typed
    at the Integer selector domain, including the quantifier binder `e`, rather than
    at the element value type. Each application retains its structural id while every
    instance of one path names its place template id; no model, type, parameter or
    place node gains a presence
- id: FR-038-AC-338
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1083). A place-form `presence_read` over `q` lacking
    its slot term refuses `ill_typed`/`operator-ineligible` at the application's `/body/arguments`;
    a first argument that is not a `clock_place` refuses that pair at `/body/arguments/0`;
    an Integer-typed owner instead of the place's object owner and a wrong-typed slot
    term each refuse `ill_typed`/`type-mismatch` at the offending argument. No refusal
    returns an admitted package.
- id: FR-038-AC-339
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'PLANNED/UNRUN (IR-1083). For `v: Option<Clock>`, the place has only `field(v)`
    and one resolved-path row retaining `Option<Clock>`; the reader joins that row
    to the selected owner''s declared `v`, optional presence and `Clock` inner type,
    then accepts terminal Clock after unwrapping Option without adding a path step.
    For `q: Sequence<Option<Clock>>`, it admits `field(q)` and a symbolically selected
    `slot` whose row retains Option type. A missing selected member, wrong declared
    presence or wrong inner type refuses the existing malformed-clock-place path join;
    the owner object-type key is unchang'
- id: FR-038-AC-340
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-1083; IR-1141). TC-048 reads QSpec's published `timed-slot-selector-vectors.json`
    through the selected checkout, with no copy in IR, and checks SL-01 through SL-08,
    SR-01 through SR-04, GD-01 through GD-03 and PR-01 through PR-06 against exact
    admission or each vector's expected code and cause. For SR-01..04 and PR-04..06,
    it also checks the vector's RFC 6901 locus suffix. For GD-02, it checks `invalid_package`/`malformed-clock-place`
    at the twin place's forged `resolved_path` row fixed by QSpec FR-322-AC-120, after
    aggregate order passes. GD-03 checks `invalid_package`/`time
```
