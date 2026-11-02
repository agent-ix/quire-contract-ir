---
id: SR-2026110101286
title: IR-721 activation-capacity ears-conformance review
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/model/functional/FR-019-rust-library-interface.md
review_set: all
---

## Summary

Ticket: IR-721. Independent spec-only review of PR quire-contract-ir#822 at ee53b9f6e8fc3b23de139d4c4c0d0efe0177f917 against merged #820 base f93d7277 and published QSpec 0e89e9c930f956529bc906a2c8e485178525486b. The new requirement-bearing paragraphs use named reader/State subjects, concrete bounds, exact field order and explicit return representations. No ambiguous event-versus-state trigger is introduced. The two target files validate with only the pre-existing AC-364 noncanonical-shape warning at current line 5469; no new grammar finding is attributable to this diff.

## Verdict

**PASS** — no findings within this method. No implementation, producer qualification or runtime execution is claimed.

## Assurance Context

No active AssuranceProfile exists under this repository spec tree, so no enforced profile review run applies. The dispatch explicitly selected applicable analyses. Historical #820 review records remain unchanged and do not claim this seventeen-binding review. No source edit, build, Cargo or Kani command was performed.

## Scope examined

```yaml
scope:
- id: FR-038-AC-421
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. The strict reader admits QSpec FR-487 MP-01
    and MP-02 as distinct temporal monitor-plan roots with their exact seventeen ordered
    bindings, State, Inputs, Scope, Results, four ordinary pure-function signatures,
    six-case outcome and exact integer/rational/text/unit premises. The anonymous
    ownerless root has generated source occurrences and semantic_type equal to State;
    generic owner/source-map checks retain precedence. A capacity edit remains admitted;
    the entries' expression bodies are checked without execution and flatten/sequence
    extension MP-32 adds no operation i
- id: FR-038-AC-422
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. Missing, reordered and extra bindings, wrong
    premise literal kinds and wrong reference kinds (MP-03..MP-09, MP-28, MP-35, MP-37,
    MP-39..MP-42) refuse invalid_semantic_graph at the first breaking binding. Missing
    bindings use the expected index and an extra uses the index after the seventeenth.
    Each refusal retains the root node locus and exact primary pointer; no admitted
    view escapes.
- id: FR-038-AC-423
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. Entry arity, parameter order/type and result-type
    defects, and missing/extra outcome cases or wrong Failed payload (MP-10..MP-17,
    MP-29), refuse ill_typed/type-mismatch at the applicable entry binding or results
    binding. Ordinary function body typing remains enforced before any admitted entry
    view is returned.
- id: FR-038-AC-424
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. Out-of-range width, gap, event rate and capacity
    (MP-18..MP-22), negative uncertainty, empty counter, negative rate and noncanonical
    rational spellings (MP-34, MP-36, MP-38, MP-46, MP-47) refuse invalid_package/invalid-value
    at the literal's value; descending/repeated/unreached buffer subformulas and missing,
    mistyped, extra or misordered state fields (MP-23..MP-27, MP-30, MP-31) refuse
    invalid_semantic_graph at buffers or state. State is exactly integer now, integer
    history_start, one buffer_<n> sequence per ordered buffer, then activations with
    the specified order
- id: FR-038-AC-425
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. Paired defects select shape before reference
    kind, signatures/outcome before premise range, and range before buffers/state;
    a wrong dependencies set selects the earlier graph-shape invalid_semantic_graph
    at /dependencies. Two defective roots stored opposite structural order choose
    the structurally first root at the post-temporal plan step. The exact code, optional
    cause, primary pointer and root node locus are asserted for each control.
- id: FR-038-AC-426
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'planned until IR-721 lands. A default-feature consumer iterates admitted
    plans and reads the specified typed root/type/clause keys, resolved root and entry
    structural content keys through structural_key, exact entry signatures and bodies,
    width/gap/counter/tick-unit/activation-capacity and exact rational event-rate/uncertainty
    plus ordered buffers/capacities through private-field borrowed views. Two plans
    sharing labels but differing selected keys remain distinct; empty plans yield
    an empty iterator. An entry or root content change is observed through the corresponding
    structural keys without '
- id: FR-038-AC-427
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. TC-048 reads QSpec proposals/checked-package-v2/monitor-plan-vectors.json
    MP-01..MP-73 by reference from the selected QSpec checkout, constructs complete
    selected packages around the MP-01..MP-31 and MP-33..MP-53 and MP-58..MP-65 root
    recipes and separately exercises MP-32's flatten/sequence expression, and checks
    exact admission or refusal code, cause, primary RFC 6901 pointer and structural
    locus. No upstream vector, schema or fixture is copied into IR. MP-54..MP-57 and
    MP-66..MP-73 remain by-reference QSL/runtime execution controls under AC-429/AC-430.
    These cont
- id: FR-038-AC-428
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. MP-33 and MP-43..MP-45 admit zero uncertainty,
    changed counter, fractional rational rate and changed tick unit with distinct
    resolved root content keys; a capacity edit MP-02 also changes that key. MP-40
    and MP-41 refuse invalid_semantic_graph at tick_unit for a wrong node kind or
    non-Time unit by the exact base-Time identity test, never invalid-time-quantity.
    Exact rate and uncertainty are borrowed CheckedRational values with no floating
    conversion or narrowing. Tick intervals remain derivable from the selected clause
    and clock/tick-unit scales with no extra bindin
- id: FR-038-AC-429
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. By-reference MP-48 and MP-49 refuse a missing
    on_gap or wrong reference kind as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/15;
    MP-50..MP-52 refuse reordered or noninteger gap parameters and a wrong return
    type as ill_typed/type-mismatch at the same binding, retaining the root structural
    locus. MP-53 admits a changed on_gap node with a distinct resolved root content
    key. The mandatory borrowed on_gap entry view exposes exactly (State, integer
    gap) and (State, Results), its whole checked body and resolved entry key; no Option/defaul
- id: FR-038-AC-430
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. By-reference MP-58 admits activation_capacity
    0 with a distinct root content key and the borrowed canonical nonnegative decimal
    getter returns "0" without narrowing. MP-59 refuses a negative value as invalid_package/invalid-value
    at /semantic_graph/nodes/{n}/body/members/16/value; MP-60 and MP-61 refuse absent
    or noninteger activation capacity as invalid_semantic_graph with no cause at member
    16. MP-62..MP-65 refuse missing history_start or activations, missing origin in
    an activation record and a noninteger history_start as invalid_semantic_graph
    with no cause at t
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The strict reader SHALL admit `temporal`/`monitor_plan` roots according
    to

    [QSpec FR-487](ix://agent-ix/quire-specification/FR-487) and

    [QSpec FR-322-AC-308](ix://agent-ix/quire-specification/FR-322-AC-308).

    These obligations are planned until IR-721 lands. The root is a distinct

    closed temporal form, with no declaration or owner, generated occurrences,

    and its `semantic_type` equal to the state node. Its four entries remain

    ordinary anonymous `function`/`pure_function` nodes. The reader SHALL retain

    and validate their ordinary expression bodies without executing them.

    QSpec owns the producer '
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'For the published seventeen-binding form, the root SHALL have exactly
    the ordered

    bindings `clause`, `state`, `inputs`, `scope`, `results`, `init`, `on_event`,

    `on_watermark`, `counter_width_bits`, `max_reading_gap`, `max_event_rate`,

    `buffers`, `counter`, `tick_unit`, `uncertainty`, `on_gap`, `activation_capacity`.
    The clause references a `temporal`/`temporal_clause`; state, inputs

    and scope reference record types; results references a sequence type; the

    four entry references resolve to anonymous pure functions. `tick_unit` SHALL

    reference a `scalar_type`/`unit` whose `dimension_node_id` is e'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'After the temporal step, the plan step SHALL visit roots in ascending
    structural

    node-id order. Within one root it SHALL check shape, reference kinds, signatures

    and outcome payloads, premise ranges, then buffers and state, in that order.

    The first defect SHALL determine the refusal. Shape and reference-kind failures

    are `invalid_semantic_graph` at the breaking root binding; missing bindings use

    their expected index, extra bindings the index after the seventeenth. Entry parameter

    or result mismatches are `ill_typed`/`type-mismatch` at the entry binding;

    outcome union or payload mismatches use '
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`init` SHALL have no parameters and return State; `on_event` SHALL take

    (State, integer delta, Inputs), and `on_watermark` SHALL take

    (State, integer boundary, Scope); both SHALL return the exact (State, Results)

    tuple type. `on_gap` SHALL take exactly (State, integer checked gap), in that

    order, and return the same exact (State, Results) tuple type. Its required

    reference is member 15, an anonymous `function`/`pure_function`; a missing

    member or wrong reference kind refuses `invalid_semantic_graph` there, while

    a parameter order/type/arity or result mismatch refuses

    `ill_typed`/`type-mismatch'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The counter name is a separate admitted premise from the clause''s clock
    name.

    The tick unit''s exact root scale supplies the tick scale; the reader SHALL NOT

    invent a repeated scale or a default counter, rate or uncertainty. Tick intervals

    SHALL remain derived from the clause interval bounds multiplied by the clause

    clock unit scale and divided by the tick-unit scale, as exact rationals. They

    SHALL NOT become extra bound metadata or rounded wire values. Runtime sound

    rounding uses the uncertainty-widened cover `[k-u, k+1+u)` under QSpec FR-487;

    the reader exposes the premises without performing'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The root content key SHALL include, in published body order, referenced
    node

    index segments, width and gap decimal text, canonical rational rate text, each

    buffer''s subformula index and capacity decimal text, counter text, tick-unit

    index, canonical rational uncertainty text, the `on_gap` node index and activation

    capacity canonical decimal text. Every premise change SHALL be

    observable through the resolved root key, including a fractional event rate,

    zero uncertainty, a different counter or tick unit. Entry body changes SHALL

    remain observable through the entry key and consequently the root k'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '`CheckedPackageV2::monitor_plans(&self)` SHALL return an

    `impl ExactSizeIterator<Item = CheckedMonitorPlanRef<''_>>` over admitted roots

    in ascending structural node-id order. No roots yields an empty iterator.

    `CheckedMonitorPlanRef<''a>`, `CheckedMonitorEntryRef<''a>` and

    `CheckedMonitorBufferRef<''a>` SHALL have private fields, no public constructor,

    and no unchecked conversion from wire or raw body values. Their borrows SHALL

    not outlive the selected admitted package.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The plan view SHALL expose `node_id`, `clause`, `state`, `inputs`, `scope`
    and

    `results` as `&''a CheckedNodeId`, `body` as `&''a CheckedTerm`, and `init`,

    `on_event`, `on_watermark`, `on_gap` as `CheckedMonitorEntryRef<''a>`. Each plan
    and entry view SHALL additionally expose `structural_key() ->

    &''a CheckedNodeId`, borrowing the resolved content key returned by its admitted

    package''s existing `structural_key`/`structural_key_at` lookup, not merely an

    indexed wire selector. Each entry SHALL

    expose `node_id` and `result_type` as `&''a CheckedNodeId`, `body` as

    `&''a CheckedTerm`, and `parameter_typ'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The admitted package, root and four entry keys SHALL retain their existing

    structural content identity. No independent persistent plan hash or `PlanId`

    SHALL be introduced; a source `PlanId` remains metadata. A consumer SHALL bind

    these reads to its independently selected checked package, clause, subject,

    target and identity; equal labels, source spans, clock names or metadata SHALL

    NOT substitute for that selection. Reads SHALL expose the selected package''s

    actual keys and referenced values, without merging equal-named plans or

    copying QSL execution semantics into IR.'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: ''
- id: FR-019-AC-23
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: planned until IR-721 lands. A default-feature external consumer imports
    CheckedMonitorPlanRef, CheckedMonitorEntryRef and CheckedMonitorBufferRef and
    uses CheckedPackageV2::monitor_plans and the exact typed accessor signatures of
    FR-038, including the nonnegative decimal activation_capacity getter. Compile
    controls show no public view constructor, mutable field, unchecked raw-to-admitted
    conversion or borrow escaping the selected package lifetime. The planned inventory
    row remains excluded until implementation lands; no PlanId or downstream executor
    type is added to the model API.
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'The borrowed monitor-plan surface of FR-038-AC-421 through FR-038-AC-430
    is

    planned until IR-721 lands. It exposes admitted plan and ordinary entry keys,

    typed bodies and premises without executing a body or minting another identity.

    The associated `CheckedPackageV2::monitor_plans` and immutable reads use the

    exact signatures in FR-038, including borrowed `CheckedRational` rate and

    uncertainty, counter text and tick-unit identity. The required `on_gap` read

    returns the same private borrowed entry-view type as the other three entries

    with its exact two-parameter signature and resolved content k'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
