---
id: SR-202610310138
title: IR-1083 PR776 scoped specification base review
type: SpecReview
analysis: base
review_set: base
scope: "agent-ix/quire-contract-ir; PR776; FR-038-AC-335 through FR-038-AC-340; corresponding TC-048 narrative and new trace claims"
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1083. One user-authorized scoped base review. Normative alignment is correct: the owner FR-322 text explicitly excludes indexed presence, ruling #485 withdrew PR-02, and GD-01 supplies the empty-root case. No evidence that QSpec is wrong was found. Historical ruling citation is explicitly authorized by the user and is not a new package pin. No applicable AssuranceProfile found. IDs remain stable; no duplicate criterion introduced. No additional analysis lenses or code/gap review were requested.

## Verdict

CONDITIONAL: normative correction passes; full AC335..340 completion and ticket closure are blocked by the four incomplete assertion groups below. Trace tags bind relevant tests to these ACs, but those bindings alone do not verify every clause. Preserve the acceptance text and add the minimal controls. Full IR integration or CI Test must pass before merge.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The guard addition/removal with a retained clock-place template is not established by the newly tagged guard test: GD-01 admits a tick root with no places. The existing timed_root_without_guards_keeps_the_place_template test fails in the supplied baseline before admission. Add a passing public-reader control with a real place, comparing the unchanged place and changed guard/root bodies. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4867 | correct-requirement-no-evidence |
| FND-002 | medium | The newly tagged public-reader test discards admitted packages and does not assert retained PR-01/PR-03 Boolean types, ordered place/owner/slot arguments, selector-domain binder type, independent application ids, shared template, or absence of added presence fields. The unchanged one-operand Option form is checked only through a private operation helper, not an admitted public-reader package. Add retained graph assertions and a public-reader Option control. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4868 | correct-requirement-no-evidence |
| FND-003 | medium | The newly tagged test has no wrong-typed slot-term mutation or exact argument-2 refusal assertion. PR-06 changes the first argument to a non-place and checks arguments/0; it cannot establish the distinct wrong-slot type-mismatch clause. Add a schema/graph-valid wrong-slot public-reader control with its exact code, cause and locus. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4869 | correct-requirement-no-evidence |
| FND-004 | medium | The newly tagged test only admits the selected package; it does not independently exercise the missing selected member, wrong declared presence, or wrong inner type path joins required by this criterion. The older selected-package test checks positive g/q field types only. Add separate public-reader adverse controls reaching each malformed-clock-place join, and inspect the positive v/q resolved-path facts. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:4870 | correct-requirement-no-evidence |

## Coverage

Examined the entire PR diff, all six criteria and corresponding TC-048 procedure, the two newly tagged tests, older timed-model public-reader controls and operations private Option helper. All 19 published cases and 14 wire/graph precedence probes remain real evidence for the parts they assert. AC335 and the current published-case replay portion of AC340 are clean. Reviewed the exact owner FR-322 text selected by the lead and the historical ruling locally. The supplied baseline log specifically reports the guard-free-place test failing at Audit-node dependencies, so it cannot count as executed successful evidence for the new completion claim. No Cargo/Kani run or source edit performed. Diff whitespace passes; lead-provided document validation passes. Findings require assertions rather than normative narrowing. Other pre-existing repository criteria and assurance completeness are outside this explicitly scoped review.

## Examined scope

```yaml
scope:
- id: FR-038-AC-335
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: One `clock_place` template admits `ts.0.x`, `ts.1.x` and `ts.2.x` over
    `Sequence<Timer>[0, 3]` with path steps `field(ts)`, `slot(sequence, Integer type
    node, 0, 3)`, `field(x)` and one node key; Set, Bag and OrderedSet paths admit
    with their own collection literal and key, two nested sequences admit two Integer-typed
    slot steps, and `Map<Text, Clock>` admits with the exact Text key type node. No
    place node stores a concrete index or state element count. A slot selector given
    as an integer literal, an expression node, the Text type node under Sequence,
    or the Integer type node under `Map<Text,
- id: FR-038-AC-336
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `timed_model` root with empty `guards` admits when its selected model
    has no guard. Adding one root-listed guard changes the complete guard and root
    bodies while leaving the same `clock_place` path template unchanged; a `guards`
    binding on the place refuses at strict wire validation. A second distinct place
    node of the same model and path in otherwise canonical order has a forged `resolved_path`
    row and refuses `invalid_package`/`malformed-clock-place` at the first differing
    row; row join runs after aggregate order and before the retained `duplicate-clock-place`
    check. A conforming second pl
- id: FR-038-AC-337
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The public reader admits the Boolean `presence_read` place form of `quire.op.option.present`
    for `present(other.v)` and `present(e)` in a quantifier over `q`, with arguments
    exactly the place template, owner, and each slot term in path order. The `q` slot
    term `e` is typed at the Integer selector domain, including the quantifier binder
    `e`, rather than at the element value type. Each application retains its structural
    id while every instance of one path names its place template id; no model, type,
    parameter or place node gains a presence member or `Option<Boolean>`. The indexed
    read `present(s
- id: FR-038-AC-338
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A place-form `presence_read` over `q` lacking its slot term refuses `ill_typed`/`operator-ineligible`
    at the application's `/body/arguments`; a first argument that is not a `clock_place`
    refuses that pair at `/body/arguments/0`; an Integer-typed owner instead of the
    place's object owner and a wrong-typed slot term each refuse `ill_typed`/`type-mismatch`
    at the offending argument. No refusal returns an admitted package.
- id: FR-038-AC-339
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'For `v: Option<Clock>`, the place has only `field(v)` and one resolved-path
    row retaining `Option<Clock>`; the reader joins that row to the selected owner''s
    declared `v`, optional presence and `Clock` inner type, then accepts terminal
    Clock after unwrapping Option without adding a path step. For `q: Sequence<Option<Clock>>`,
    it admits `field(q)` and a symbolically selected `slot` whose row retains Option
    type. A missing selected member, wrong declared presence or wrong inner type refuses
    the existing malformed-clock-place path join; the owner object-type key is unchanged
    by the field set.'
- id: FR-038-AC-340
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: TC-048 reads QSpec's published `timed-slot-selector-vectors.json` through
    the selected checkout, with no copy in IR, and checks SL-01 through SL-07, SR-01
    through SR-04, GD-01 through GD-03 and PR-01 and PR-03 through PR-06 against exact
    admission or each vector's expected code and cause. For SR-01..04 and PR-04..06,
    it also checks the vector's RFC 6901 locus suffix. For GD-02, it checks `invalid_package`/`malformed-clock-place`
    at the twin place's forged `resolved_path` row fixed by QSpec FR-322-AC-120, after
    aggregate order passes. GD-03 checks `invalid_package`/`timed-node-reference`
    at `/s
```

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Real selected-slot package admits with and without its guard; typed place ids and bodies compare equal, and guard/root bodies differ. Focused control passes. |
| FND-002 | still-open | Retained presence arguments/types/ids and the public-reader Option control now pass; an explicit absence/preservation assertion for model/parameter presence fields and retained Option application id is still missing. |
| FND-003 | fixed | Wrong-type owner-reference substituted into slot argument 2 yields exact IllTyped/TypeMismatch at arguments/2 through the public reader. |
| FND-004 | fixed | Separate selected-document missing-member, required-presence and Integer-inner-type mutations reach exact MalformedClockPlace pointers; admitted v/q resolved rows compare against owner records. |

## Disposition verification

Fix audit at bc342189: supplied focused log reports all seven tests passing in 2.27 seconds. Three findings resolved; FND-002 awaits the remaining retained graph assertion. No new findings. Aggregate integration/CI Test remains the lead gate.

## Dispositions round 2

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-002 | fixed | The retained graph now compares node count and each semantic form/type/body against authored typed wire; the one-operand Option application id is compared with its authored id. These assertions close the remaining preservation and absence gap. |

## Final scoped verdict

PASS for normative alignment and assertion completeness after the round-2 additions. No new findings. Final focused execution and full integration/CI Test remain external merge gates; this review does not claim those pending executions passed. The previous seven-control run passed before these final assertions.

## Audit correction

The round-2 body-equality assertion was removed because direct public CheckedTerm deserialization is not valid for all wire bodies. Retained node count, semantic form and semantic type assertions pass in the final focused seven-test run (2.25 seconds), and the Option application id assertion remains. Those checks detect extra nodes and type rewrites, but do not detect an added presence binding inside an existing model or parameter body. FND-002 therefore remains open pending the narrow absence-of-presence-binding assertion. This correction supersedes the earlier round-2 completeness verdict; the historical disposition and posted comment remain unchanged.

## Dispositions round 3

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-002 | still-open | The removed body comparison no longer verifies absence of an added presence binding inside existing model/parameter bodies; add a typed or serialized-body binding-name assertion. |

## Dispositions round 4

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-002 | fixed | Every admitted node body is serialized and recursively inspected: any binding named presence fails. Retained graph count/form/type equality detects added nodes and Option<Boolean> rewrites, while Option application id is compared with its authored id. No direct wire-body deserialization or whole-body equality is claimed. |

## Latest scoped verdict

PASS by inspection: all four findings resolved. The actual final absence assertion is assert_no_added_presence_member over serialized admitted bodies, not body equality. Final focused run and full integration/CI Test remain execution gates for the lead.
