---
id: SR-1292
title: IR-721 reader spec-review/scope-boundary
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/model/functional/FR-019-rust-library-interface.md
review_set: subset
---

## Summary

Ticket: IR-721. Reader implementation activation and the default model API inventory preserve a clear allocation to QSpec, IR, QSL and runtime consumers.

## Verdict

**PASS** — Reader implementation activation and the default model API inventory preserve a clear allocation to QSpec, IR, QSL and runtime consumers.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

IR strict reader (core): guarantees closed shape/kinds/signatures/premises, selected model joins, borrowed access and canonical package roundtrip. QSpec is an external normative dependency consumed by reference and checked via reader contract tests. QSL producer semantics/capacity derivation and real producer closure are external assumptions not qualified here. RT modular deltas/execution and driver independently selected context/identity binding are downstream responsibilities, not reader guarantees. RT ProgramParts -> Program::try_new -> RcProgram remains the runtime construction seam; no executor-facing replacement or copied execution semantics is added. FR-038 and FR-019 each retain one owning IR component and core responsibility. No new external actor or dependency is created by status/inventory edits.

The immutable candidate was verified against local HEAD and GitHub PR head. Source review is read-only; artifacts are outside the branch. No applicable AssuranceProfile exists in spec/. QSpec FR-487/FR-322 define the wire; IR implements reader admission only. Root and all four entry structural keys bind the full program content; downstream consumers must independently bind their selected package/clause/subject/target identity. No PlanId, execution, lowering or producer qualification credit is claimed.

Recorded focused evidence was inspected rather than rerunning Cargo, as dispatched: twelve monitor groups exercise all 60 reader recipes plus MP-32; one real default-feature consumer reads and encodes/re-admits the package; three default reader controls and 26 owner controls pass; nine negative doctests fail for E0515/E0451/E0277 as intended. fmt and workspace/all-targets plus default-model Clippy pass. These are focused records, not an aggregate gate. Two population Text NFC failures reproduce on the supplied base with the same selected QSpec input; they remain explicit baseline qualification failures. Final aggregate/hosted gates remain the parent's work.

Examined requirement excerpts and roles:

```yaml
scope:
- id: FR-038-AC-421
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. The strict reader admits QSpec FR-487 MP-01
    and MP-02 as distinct temporal monitor-plan roots with their exact seventeen ordered
    bindings, State, Inputs, Scope, Results, four ordinary pure-function signatures,
    six-case outcome and exact integer/rational/text/unit premises. The anonymous
    ownerless root has generated source occurrences and semantic_type equal to State;
    generic owner/source-map checks retain precedence. A capacity edit remains admitted;
    the entries' expression bodies are checked without execution and flatten/sequence
    extension MP-32 adds no operation
- id: FR-038-AC-422
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Missing, reordered and extra bindings, wrong
    premise literal kinds and wrong reference kinds (MP-03..MP-09, MP-28, MP-35, MP-37,
    MP-39..MP-42) refuse invalid_semantic_graph at the first breaking binding. Missing
    bindings use the expected index and an extra uses the index after the seventeenth.
    Each refusal retains the root node locus and exact primary pointer; no admitted
    view escapes.
- id: FR-038-AC-423
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Entry arity, parameter order/type and result-type
    defects, and missing/extra outcome cases or wrong Failed payload (MP-10..MP-17,
    MP-29), refuse ill_typed/type-mismatch at the applicable entry binding or results
    binding. Ordinary function body typing remains enforced before any admitted entry
    view is returned.
- id: FR-038-AC-424
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Out-of-range width, gap, event rate and capacity
    (MP-18..MP-22), negative uncertainty, empty counter, negative rate and noncanonical
    rational spellings (MP-34, MP-36, MP-38, MP-46, MP-47) refuse invalid_package/invalid-value
    at the literal's value; descending/repeated/unreached buffer subformulas and missing,
    mistyped, extra or misordered state fields (MP-23..MP-27, MP-30, MP-31) refuse
    invalid_semantic_graph at buffers or state. State is exactly integer now, integer
    history_start, one buffer_<n> sequence per ordered buffer, then activations with
    the specified ord
- id: FR-038-AC-425
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Paired defects select shape before reference
    kind, signatures/outcome before premise range, and range before buffers/state;
    a wrong dependencies set selects the earlier graph-shape invalid_semantic_graph
    at /dependencies. Two defective roots stored opposite structural order choose
    the structurally first root at the post-temporal plan step. The exact code, optional
    cause, primary pointer and root node locus are asserted for each control.
- id: FR-038-AC-426
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. A default-feature consumer iterates admitted
    plans and reads the specified typed root/type/clause keys, resolved root and entry
    structural content keys through structural_key, exact entry signatures and bodies,
    width/gap/counter/tick-unit/activation-capacity and exact rational event-rate/uncertainty
    plus ordered buffers/capacities through private-field borrowed views. Two plans
    sharing labels but differing selected keys remain distinct; empty plans yield
    an empty iterator. An entry or root content change is observed through the corresponding
    structural keys withou
- id: FR-038-AC-427
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. TC-048 reads QSpec proposals/checked-package-v2/monitor-plan-vectors.json
    MP-01..MP-73 by reference from the selected QSpec checkout, constructs complete
    selected packages around the MP-01..MP-31 and MP-33..MP-53 and MP-58..MP-65 root
    recipes and separately exercises MP-32's flatten/sequence expression, and checks
    exact admission or refusal code, cause, primary RFC 6901 pointer and structural
    locus. No upstream vector, schema or fixture is copied into IR. MP-54..MP-57 and
    MP-66..MP-73 remain by-reference QSL/runtime execution controls under AC-429/AC-430.
    These co
- id: FR-038-AC-428
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. MP-33 and MP-43..MP-45 admit zero uncertainty,
    changed counter, fractional rational rate and changed tick unit with distinct
    resolved root content keys; a capacity edit MP-02 also changes that key. MP-40
    and MP-41 refuse invalid_semantic_graph at tick_unit for a wrong node kind or
    non-Time unit by the exact base-Time identity test, never invalid-time-quantity.
    Exact rate and uncertainty are borrowed CheckedRational values with no floating
    conversion or narrowing. Tick intervals remain derivable from the selected clause
    and clock/tick-unit scales with no extra bind
- id: FR-038-AC-429
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. By-reference MP-48 and MP-49 refuse a missing
    on_gap or wrong reference kind as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/15;
    MP-50..MP-52 refuse reordered or noninteger gap parameters and a wrong return
    type as ill_typed/type-mismatch at the same binding, retaining the root structural
    locus. MP-53 admits a changed on_gap node with a distinct resolved root content
    key. The mandatory borrowed on_gap entry view exposes exactly (State, integer
    gap) and (State, Results), its whole checked body and resolved entry key; no Option/defa
- id: FR-038-AC-430
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. By-reference MP-58 admits activation_capacity
    0 with a distinct root content key and the borrowed canonical nonnegative decimal
    getter returns "0" without narrowing. MP-59 refuses a negative value as invalid_package/invalid-value
    at /semantic_graph/nodes/{n}/body/members/16/value; MP-60 and MP-61 refuse absent
    or noninteger activation capacity as invalid_semantic_graph with no cause at member
    16. MP-62..MP-65 refuse missing history_start or activations, missing origin in
    an activation record and a noninteger history_start as invalid_semantic_graph
    with no cause at
- id: FR-019-AC-23
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: 'Implemented; evidence: `default_feature_consumer_reads_an_actual_admitted_monitor_plan`
    and the monitor-view compile-fail doctests. A default-feature external consumer
    imports CheckedMonitorPlanRef, CheckedMonitorEntryRef and CheckedMonitorBufferRef
    and uses CheckedPackageV2::monitor_plans and the exact typed accessor signatures
    of FR-038, including the nonnegative decimal activation_capacity getter. Compile
    controls show no public view constructor, mutable field, unchecked raw-to-admitted
    conversion or borrow escaping the selected package lifetime. The implemented inventory
    row participates i'
```

## Review carry — pass 1

The review carries to frozen PR head `12ddb85e860e8804fc804f73919c8b565b163700`. Local HEAD and GitHub PR head agree. The merge combines review-artifact parent `d1281f487348c38a60e97ab1695c79df2c1774a5` with main `dcfe1b25da9dbe8e7d84b51ee57198a23843a39f`. No Rust, Cargo manifest, lockfile or Makefile difference exists from the original reviewed candidate, so the source/test analysis and its focused evidence remain applicable.

The single FR-019 resolution preserves implemented AC-23 verbatim from the reader branch and planned AC-24 verbatim from main. The operation-effect inventory row and complete TC-058 document also match main verbatim. AC-24 describes IR-1267's separate planned raw operation-effect-row read and does not assert that reader PR #824 implements it. Scoped Quire validation and merge diff whitespace checks pass. No new finding was found; the original verdict carries, including the whole-repository gap limitation.

The prior aggregate strict-coverage failure contains 98 unbacked rows and zero contradicted statuses. A structural git-show comparison confirms all 98 exact table rows exist unchanged on base `2aba7d2f4f04e624f57cfbf2c38948b8ddaff628`; the reader diff removes no evidence declarations and changes no module configuration. This is structural evidence of inheritance, not a baseline coverage-engine rerun. No coverage-engine or aggregate result is claimed for this merged head. Parent-owned final aggregate/hosted qualification remains required. Actual QSL producer, downstream lowering/execution and overall IR-721 completion remain open.

Additional context examined:

```yaml
scope:
- id: FR-019-AC-24
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: context_only
  excerpt: planned until IR-1267 lands. A default-feature external consumer calls
    `CheckedPackageV2::operation_effect_rows(&self) -> Option<&[CheckedOperationEffectRow]>`
    only after strict package admission. It obtains `None` when the optional carrier
    was omitted and a borrowed, nonempty slice of the exact admitted producer rows
    when present, retaining row order, selected document and declaring identities,
    original operation members, all four ordered effect collections and provenance;
    an explicitly empty effect remains a present row. It passes that slice directly
    as `CheckedPackageSidecarsRef::operation_
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: context_only
  excerpt: Existing public-interface test case; the complete merged document matches
    incoming main verbatim.
```

## Normative amendment review — pass 2

Boundary allocation: QSpec specifies the wire and producer path rule; IR validates the closed State and existing identity admission; QSL/runtime own event-position transitions. No new downstream identity or executor is introduced.

Frozen normative review at PR #824 head `32926d915c4356aa0e79f8a00cca4735f39ba382`, verified against local and GitHub heads. The author-owned normative commit changes only FR-038 relative to parent `c889608dc02bb22d74d3932b726fec64a4e2418c`; incoming main's source/TC-058 work is distinct upstream material and receives no new source-review credit here. This is specification-first acceptance before reader implementation changes, not a final implementation carry or merge approval.

The exact State is now, history_start, ordered buffer_<n>, activations, then integer event_position. MP-74..76 select invalid_semantic_graph, no cause, at root member 1 with the root structural locus. There remain exactly seventeen root bindings and no new getter. Status stays opaque. MP-77's event increments (baseline included), MP-57's gap behavior and MP-69's poll behavior remain QSL/runtime program controls; the reader only checks shape/type and executes no entry. Updated QSpec contains 63 reader recipes; the new execution control is not a 64th reader recipe.

FR-322-AC-309 gives the producer's declaration-less entry derivation: parameter count, ordered parameter indices, body root and optional measure. EK-01..06 concern supplied-key equality/change; EK-07 is the existing duplicate package/path/ordinal refusal at the second node_id; EK-08 preserves a model accessor's owner-selected label. The reader resolves supplied keys, never recomputes the producer derivation or adds a content-path mismatch refusal. Root plus four entry keys remain the identity, with independent downstream context selection still required and no PlanId.

AC-431 and AC-432 are explicitly planned. Existing implementation claims distinguish those pending obligations. The source and prior 60-recipe logs do not qualify this amended contract: current matrix has 960 obligations, 805 tagged, 132 untagged, 1 ignored-only and 22 method-without-symbol. Both new criteria are untagged until the coding phase adds real controls. The gap-analysis overall verdict remains FAIL; the two planned additions must be implemented and tested before final qualification. Historical 98-row strict-coverage structural proof remains limited to its prior reviewed head; no new strict coverage, baseline engine replay, Cargo or aggregate run is claimed.

Scoped Quire validation passes, with one inherited AC grammar warning outside the changed text. quoin advise reports both new criteria authored Test with no mismatch, inconclusive or uncatalogued result. Reviewer judgment: complete-package reader tests and by-reference identity controls fit these admission obligations; runtime recommendations do not assign execution semantics to IR. No introduced normative defect was found. Actual producer/lowering/runtime differential qualification and overall IR-721 remain open.

```yaml
scope:
- id: FR-038-AC-424
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. Out-of-range width, gap, event rate and capacity
    (MP-18..MP-22), negative uncertainty, empty counter, negative rate and noncanonical
    rational spellings (MP-34, MP-36, MP-38, MP-46, MP-47) refuse invalid_package/invalid-value
    at the literal's value; descending/repeated/unreached buffer subformulas and missing,
    mistyped, extra or misordered state fields (MP-23..MP-27, MP-30, MP-31) refuse
    invalid_semantic_graph at buffers or state. State is exactly integer now, integer
    history_start, one buffer_<n> sequence per ordered buffer, then activations with
    the specified ord
- id: FR-038-AC-427
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Implemented reader admission. TC-048 reads QSpec proposals/checked-package-v2/monitor-plan-vectors.json
    MP-01..MP-77 by reference from the selected QSpec checkout, constructs complete
    selected packages around the MP-01..MP-31 and MP-33..MP-53 and MP-58..MP-65 root
    recipes (MP-74..MP-76 planned under AC-431) and separately exercises MP-32's flatten/sequence
    expression, and checks exact admission or refusal code, cause, primary RFC 6901
    pointer and structural locus. No upstream vector, schema or fixture is copied
    into IR. MP-54..MP-57, MP-66..MP-73 and MP-77 remain by-reference QSL/runtime
    execu
- id: FR-038-AC-431
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: planned until IR-721 lands. The exact monitor State is now, history_start,
    the ordered buffer_<n> fields, activations, then integer event_position. By-reference
    QSpec MP-74..MP-76 refuse a missing event_position, text-typed event_position
    or placement before activations as invalid_semantic_graph with no cause at /semantic_graph/nodes/{n}/body/members/1,
    retaining the root structural locus. Updated positive State contexts admit with
    no added root binding or public getter. MP-77, MP-57 and MP-69 remain by-reference
    QSL/runtime controls for init zero, one increment per admitted event including
    ba
- id: FR-038-AC-432
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'planned until IR-721 lands. TC-048 reads QSpec entry-key-vectors.json
    EK-01..EK-08 by reference: the published declaration-less function path has parameter
    count, ordered parameter indices, body-root index and optional measure index;
    equal content gives equal paths and a changed body, reordered parameter or measure
    gives a different path. Complete package controls resolve those supplied keys,
    refuse EK-07''s duplicate package/path/ordinal as invalid_semantic_graph at the
    second node_id, and admit EK-08''s owner-chosen model accessor label unchanged.
    The reader recomputes no producer path and add'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The state SHALL have exactly integer `now`, integer `history_start`, one

    sequence field `buffer_<n>` per buffer in order, then `activations`, a sequence

    of records with exactly ordered fields `subformula` (integer), `origin`

    (integer), and `status`; finally integer `event_position` (this final field is

    planned until IR-721 lands). The published plan contract leaves the type of

    `status` unspecified; the reader SHALL preserve its ordinary admitted field

    type without inventing an additional plan-specific type constraint.

    Planned until IR-721 lands: the exact closed State SHALL end with

    `event_pos'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Planned until IR-721 lands: the exact closed State SHALL end with

    `event_position`, an ordinary integer field after `activations`; it SHALL

    refuse absence, text typing or earlier placement as `invalid_semantic_graph`

    with no cause at the root `state` binding. The seventeen root bindings remain

    unchanged. QSpec FR-487-AC-11 owns its meaning as the unwrapped monotone count

    of admitted events: init 0, plus one for each admitted event including the

    baseline on_event(0), unchanged by watermark or gap, with polling calling no

    entry. These are QSL/runtime program responsibilities; reader admission ch'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'The reader SHALL NOT recompute this derivation or refuse a path merely
    for

    differing from it; it resolves the supplied identities and refuses duplicate

    package/path/ordinal identities under the existing graph rule. The new

    by-reference EK-01..EK-08 qualification is planned under AC-432, not a claim

    that the reader implements a new content-path checker.


    #### Borrowed monitor-plan reads


    `CheckedPackageV2::monitor_plans(&self)` SHALL return an

    `impl ExactSizeIterator<Item = CheckedMonitorPlanRef<''_>>` over admitted roots

    in ascending structural node-id order. No roots yields an empty iterator.

    '
- id: FR-487-AC-11
  path: /home/peter/dev/qualification-inputs/IR-721-qspec/spec/objects/interfaces/FR-487-checked-package-monitor-plan-body.md
  role: context_only
  excerpt: The state ends with `event_position`, an `integer` that `init` sets to
    0, `on_event` increases by 1 per admitted event (the baseline reading included),
    and `on_watermark`, `on_gap` and a clock poll leave unchanged (`MP-77`, `MP-57`,
    `MP-69`); a state lacking `event_position`, with it typed `text` or placed before
    `activations` refuses `invalid_semantic_graph` at the `state` binding (`MP-74`
    to `MP-76`).
- id: FR-322-AC-309
  path: /home/peter/dev/qualification-inputs/IR-721-qspec/spec/objects/interfaces/FR-322-checked-package-artifact.md
  role: context_only
  excerpt: 'A declaration-less `function` node has the derived path `["function",
    <semantic_form>, <decimal parameter count>, <parameter index segments in order>,
    <body root index segment>, <measure index segment if any>]`: equal parameters,
    body root and measure give an equal path, and a changed body root, a changed or
    reordered parameter or an added measure gives a different one; two nodes with
    equal `package`, `path` and `ordinal` are refused as a duplicate `node_id`, and
    a model-owned accessor with an owner-chosen label is admitted unchanged (`proposals/checked-package-v2/entry-key-vectors.json`,
    `EK-'
```

## Normative optional-measure follow-up

The optional-measure check identified FND-002 in SR-1290 (integrity). The earlier clean wording in this normative-pass narrative is superseded for the overall amendment: specification acceptance and the coding handoff await its fix. Other methods report no separate defect; no implementation or merge approval is given.

## Normative fix verification — pass 3

Frozen fix head `958c1f44ba1dd43c75591bc25e950f61d47509df` matches local and GitHub PR heads; the fix changes only FR-038. The owning paragraph and AC-432 now explicitly require exact ordered parameters/body/optional-final-measure admission, including a genuine complete EK-06 package with typed references. Malformed measure, duplicate, extra, reordered and clause-only controls retain existing ordinary validation precedence without measure execution, semantic recheck or path recomputation. This resolves SR-1290 FND-002; its original finding text is unchanged. No new normative defect was found. Scoped Quire and whitespace validation pass, with only the inherited unrelated AC grammar advisory.

**Normative verdict: PASS.** The coder may implement AC-431/432 against this reviewed contract. This is not a final code/aggregate pass: current code and tests still require the event_position/optional-measure changes and fresh by-reference qualification. Previous whole-repository gaps and real-producer/lowering/runtime limitations remain.
